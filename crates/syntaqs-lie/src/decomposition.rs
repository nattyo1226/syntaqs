//! Center-first spectral splitting of compact Hermitian Lie algebras.

use std::ops::Range;

use ndarray::{Array1, Array2, Axis, Slice};
use ndarray_linalg::{Lapack, Scalar};
use num_traits::{Float, Zero};

use crate::{
    AdjointRepresentation, ConstraintDiagnostics, FamilyExpression, LieError, NumericalError,
    NumericalOptions, OperatorFamily, ToleranceCheck,
    invariant::{adjoint_kernel, diagnostics, symmetric_commutant, vector_norm},
    numerical::{finite, rank_nullspace, symmetric_eigen},
};

/// Finite, reproducible controls for spectral ideal sampling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecompositionOptions {
    /// Seed of the deterministic sampling stream. Reproducibility is limited to
    /// the same numerical environment; eigenvectors/order are not canonical.
    pub seed: u64,
    /// Maximum number of sampled eigensystems; must be positive.
    pub max_attempts: usize,
}

impl Default for DecompositionOptions {
    fn default() -> Self {
        Self {
            seed: 0,
            max_attempts: 32,
        }
    }
}

impl DecompositionOptions {
    /// Rejects a zero attempt budget, even for a purely abelian input.
    pub fn validate(&self) -> Result<(), NumericalError> {
        if self.max_attempts == 0 {
            return Err(NumericalError::InvalidSetting(
                "decomposition max_attempts must be positive",
            ));
        }
        Ok(())
    }
}

/// A reconstructed subspace with column coordinates in its source family.
///
/// Decomposition initially returns HS-orthonormal components. These coordinate
/// handles support further orthogonalization without losing their source basis.
pub struct LieComponent<'a, E: FamilyExpression> {
    source: &'a OperatorFamily<E>,
    family: OperatorFamily<E>,
    coordinates: Array2<E::Real>,
    options: NumericalOptions<E::Real>,
}

impl<'a, E: FamilyExpression> LieComponent<'a, E> {
    /// Original family defining the coordinate rows and expression context.
    pub fn source(&self) -> &'a OperatorFamily<E> {
        self.source
    }
    /// Ordered reconstructed component operators.
    pub fn family(&self) -> &OperatorFamily<E> {
        &self.family
    }
    /// Component columns in original family coordinates, shape `(n,dimension)`.
    pub fn coordinates(&self) -> &Array2<E::Real> {
        &self.coordinates
    }
    /// Numerical settings used when constructing this component.
    pub fn options(&self) -> &NumericalOptions<E::Real> {
        &self.options
    }
}

impl<'a, E: FamilyExpression> LieComponent<'a, E>
where
    NumericalOptions<E::Real>: Default,
{
    /// Orthogonalizes this component and composes its coordinates with the source.
    pub fn orthogonalize(
        &self,
        options: &NumericalOptions<E::Real>,
    ) -> Result<Self, LieError<E::Real>> {
        let transform = self.family.orthogonalize(options)?;
        component(
            self.source,
            self.coordinates.dot(transform.matrix()),
            options,
        )
    }
    /// Orthonormalizes this component and retains coordinates in the original source.
    pub fn orthonormalize(
        &self,
        options: &NumericalOptions<E::Real>,
    ) -> Result<Self, LieError<E::Real>> {
        let transform = self.family.orthonormalize(options)?;
        component(
            self.source,
            self.coordinates.dot(transform.matrix()),
            options,
        )
    }
}

/// Independent invariant checks for a proposed complete center/simple-ideal split.
#[derive(Debug, Clone)]
pub struct DecompositionValidation<R> {
    /// Frobenius defect of the combined coordinate Gram matrix from identity.
    pub orthonormality: ToleranceCheck<R>,
    /// Frobenius defect of the combined projector from the full identity.
    pub span: ToleranceCheck<R>,
    /// Full-center common-kernel residual in internal orthonormal coordinates.
    pub centrality: ToleranceCheck<R>,
    /// Invariance under all ambient adjoints, in proposed ideal order.
    pub ideal_invariance: Vec<ToleranceCheck<R>>,
    /// Cross-ideal bracket checks for pairs `(0,1),(0,2),...,(1,2),...`.
    pub cross_commutation: Vec<ToleranceCheck<R>>,
    /// Checks against zero bracket action. A nonabelian ideal must FAIL this check.
    pub abelianity: Vec<ToleranceCheck<R>>,
    /// Common-kernel diagnostics for each ideal's own restricted adjoints.
    pub ideal_centers: Vec<ConstraintDiagnostics<R>>,
    /// Restricted symmetric commutants; a simple compact ideal has dimension one.
    pub simple_commutants: Vec<ConstraintDiagnostics<R>>,
}

impl<R: Float> DecompositionValidation<R> {
    /// Whether the geometric and Lie invariants establish a complete simple split.
    pub fn passed(&self) -> bool {
        self.orthonormality.passed()
            && self.span.passed()
            && self.centrality.passed()
            && self.ideal_invariance.iter().all(ToleranceCheck::passed)
            && self.cross_commutation.iter().all(ToleranceCheck::passed)
            && self.abelianity.iter().all(|check| !check.passed())
            && self
                .ideal_centers
                .iter()
                .all(|report| report.rank == report.columns)
            && self
                .simple_commutants
                .iter()
                .all(|report| report.columns - report.rank == 1)
    }
}

/// Reason a spectral grouping was not accepted as complete.
#[derive(Debug, Clone, PartialEq)]
pub enum DecompositionFailure<R> {
    /// The number of eigenspaces disagrees with the symmetric commutant dimension.
    ComponentCount {
        /// Required number of centerless simple components.
        expected: usize,
        /// Number of spectral groups produced under the configured gap.
        found: usize,
    },
    /// A numerical invariant or complete-splitting check failed.
    Validation,
    /// A candidate's restricted constraint analysis failed its residual check.
    ConstraintResidual {
        /// Restricted analysis whose constraints could not be satisfied.
        operation: &'static str,
        /// Failed constraint basis-vector index.
        index: usize,
        /// Residual measurement and effective tolerance bound.
        check: ToleranceCheck<R>,
    },
}

/// Effective eigensystem grouping and validation of one sampled candidate.
#[derive(Debug, Clone)]
pub struct DecompositionAttempt<R> {
    /// Ascending eigenvalues of the sampled real symmetric commutant matrix.
    pub eigenvalues: Array1<R>,
    /// Effective dimension-aware relative gap cutoff.
    pub relative_gap: R,
    /// Absolute gap threshold, relative gap times the largest eigenvalue magnitude.
    pub gap_threshold: R,
    /// Contiguous ranges of eigenvalue/vector indices, joining adjacent gaps
    /// less than or equal to the threshold.
    pub groups: Vec<Range<usize>>,
    /// Invariant checks, absent when component counts or restricted constraints reject the candidate.
    pub validation: Option<DecompositionValidation<R>>,
    /// Rejection reason, or `None` for the accepted final candidate.
    pub failure: Option<DecompositionFailure<R>>,
}

/// Retained center, commutant, sampling, and final invariant diagnostics.
#[derive(Debug, Clone)]
pub struct DecompositionDiagnostics<R> {
    /// Full-center common-kernel SVD from the center-first split.
    pub center: ConstraintDiagnostics<R>,
    /// Symmetric commutant of the semisimple complement, absent for abelian input.
    pub semisimple_commutant: Option<ConstraintDiagnostics<R>>,
    /// All sampled attempts, including the accepted final one; empty for abelian input.
    pub attempts: Vec<DecompositionAttempt<R>>,
    /// Final full-span, center, ideal, and complete-splitting checks.
    pub validation: DecompositionValidation<R>,
}

/// Full center plus nonabelian simple ideals in the original expression context.
///
/// The center is returned as one subspace, even if multidimensional. Centerless
/// input has `None`; purely abelian input has no simple ideals. Components are
/// HS-orthonormal, but basis vectors, signs, and component order are not canonical.
pub struct LieDecomposition<'a, E: FamilyExpression> {
    family: &'a OperatorFamily<E>,
    center: Option<LieComponent<'a, E>>,
    simple_ideals: Vec<LieComponent<'a, E>>,
    diagnostics: DecompositionDiagnostics<E::Real>,
    options: NumericalOptions<E::Real>,
    decomposition_options: DecompositionOptions,
}

impl<'a, E: FamilyExpression> LieDecomposition<'a, E> {
    /// Exact originating family and common expression context.
    pub fn family(&self) -> &'a OperatorFamily<E> {
        self.family
    }
    /// The full center as one component, or `None` for centerless input.
    pub fn center(&self) -> Option<&LieComponent<'a, E>> {
        self.center.as_ref()
    }
    /// Nonabelian simple ideals in sampled spectral order.
    pub fn simple_ideals(&self) -> &[LieComponent<'a, E>] {
        &self.simple_ideals
    }
    /// Effective constraints, spectral gaps, retry history, and final validation.
    pub fn diagnostics(&self) -> &DecompositionDiagnostics<E::Real> {
        &self.diagnostics
    }
    /// Numerical settings used throughout decomposition.
    pub fn options(&self) -> &NumericalOptions<E::Real> {
        &self.options
    }
    /// Seed and finite sampling budget used by this result.
    pub fn decomposition_options(&self) -> &DecompositionOptions {
        &self.decomposition_options
    }
}

mod spectral;

use spectral::{Sampler, component, decompose_with_sampler};
#[cfg(test)]
use spectral::{groups, validate_candidate};

impl<'a, E: FamilyExpression> AdjointRepresentation<'a, E>
where
    NumericalOptions<E::Real>: Default,
{
    /// Decomposes the full center and nonabelian simple ideals, with finite retries.
    /// Uses retained numerical settings; unresolved spectral groups never become
    /// a successful complete decomposition. Exhaustion retains all failed attempts.
    pub fn decompose(
        &self,
        settings: &DecompositionOptions,
    ) -> Result<LieDecomposition<'a, E>, LieError<E::Real>> {
        let mut sampler = Sampler(settings.seed);
        decompose_with_sampler(self, settings, |n, basis| sampler.sample(n, basis))
    }
}

impl<E: FamilyExpression> OperatorFamily<E>
where
    NumericalOptions<E::Real>: Default,
{
    /// Validates Hermiticity, independence, and closure before complete decomposition.
    pub fn decompose(
        &self,
        options: &NumericalOptions<E::Real>,
        settings: &DecompositionOptions,
    ) -> Result<LieDecomposition<'_, E>, LieError<E::Real>> {
        settings.validate()?;
        self.structure_constants(options)?
            .adjoint()
            .decompose(settings)
    }
}

#[cfg(test)]
mod tests;
