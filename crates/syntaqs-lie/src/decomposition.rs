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

fn component<'a, E: FamilyExpression>(
    source: &'a OperatorFamily<E>,
    coordinates: Array2<E::Real>,
    options: &NumericalOptions<E::Real>,
) -> Result<LieComponent<'a, E>, LieError<E::Real>> {
    finite(coordinates.iter().copied())?;
    let operators = coordinates
        .columns()
        .into_iter()
        .map(|v| Ok(source.reconstruct(v)?.collected_in(source.context())?))
        .collect::<Result<Vec<_>, LieError<E::Real>>>()?;
    Ok(LieComponent {
        source,
        family: OperatorFamily::new_in(source.context(), operators)?,
        coordinates,
        options: *options,
    })
}

fn check_reconstruction<E: FamilyExpression>(
    component: &LieComponent<'_, E>,
    index: usize,
) -> Result<(), LieError<E::Real>>
where
    NumericalOptions<E::Real>: Default,
{
    let report = component.family.orthonormality(&component.options)?;
    for (index, check) in report.validation().hermiticity().iter().enumerate() {
        if !check.passed() {
            return Err(LieError::NonHermitianElement {
                index,
                check: *check,
            });
        }
    }
    let independence = report.validation().independence();
    if !independence.is_independent() {
        return Err(NumericalError::SingularBasis {
            rank: independence.rank,
            dimension: independence.dimension,
        }
        .into());
    }
    if !report.check().passed() {
        return Err(LieError::ConstraintResidual {
            operation: "reconstructed component orthonormality",
            index,
            check: *report.check(),
        });
    }
    Ok(())
}

// SplitMix64 supplies a small deterministic sampling stream without introducing
// another production dependency. It is not a cryptographic generator.
struct Sampler(u64);
impl Sampler {
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
        value ^= value >> 31;
        ((value >> 11) as f64) * (2.0 / 9007199254740992.0) - 1.0
    }
    fn sample<R: Float + Scalar<Real = R> + Lapack>(
        &mut self,
        n: usize,
        basis: &[Array2<R>],
    ) -> Array2<R> {
        let mut result = Array2::zeros((n, n));
        for matrix in basis {
            // Conversion to either supported precision is finite.
            let coefficient = R::from_f64(self.next()).expect("finite f32/f64 sample");
            result = result + matrix.mapv(|x| x * coefficient);
        }
        result
    }
}

fn groups<R: Float>(eigenvalues: &Array1<R>, threshold: R) -> Vec<Range<usize>> {
    if eigenvalues.is_empty() {
        return Vec::new();
    }
    let mut start = 0;
    let mut groups = Vec::new();
    for i in 1..eigenvalues.len() {
        if eigenvalues[i] - eigenvalues[i - 1] > threshold {
            groups.push(start..i);
            start = i;
        }
    }
    groups.push(start..eigenvalues.len());
    groups
}

fn restricted_adjoints<R>(
    ambient: &[Array2<R>],
    axes: &Array2<R>,
) -> Result<Vec<Array2<R>>, NumericalError>
where
    R: Float + Scalar<Real = R> + Lapack,
{
    let n = axes.nrows();
    let d = axes.ncols();
    let mut result = Vec::new();
    for j in 0..d {
        let combined = Array2::from_shape_fn((n, n), |(a, b)| {
            (0..n).fold(R::zero(), |sum, i| sum + axes[(i, j)] * ambient[i][(a, b)])
        });
        let matrix = axes.t().dot(&combined).dot(axes);
        finite(matrix.iter().copied())?;
        result.push(matrix);
    }
    Ok(result)
}

fn validate_candidate<R>(
    ambient: &[Array2<R>],
    center: &Array2<R>,
    ideals: &[Array2<R>],
    options: &NumericalOptions<R>,
) -> Result<DecompositionValidation<R>, LieError<R>>
where
    R: Float + Scalar<Real = R> + Lapack,
    NumericalOptions<R>: Default,
{
    let n = center.nrows();
    let mut combined = Array2::zeros((n, n));
    let mut column = 0;
    for axes in std::iter::once(center).chain(ideals) {
        for vector in axes.columns() {
            combined.column_mut(column).assign(&vector);
            column += 1;
        }
    }
    let identity = Array2::eye(n);
    let identity_norm = vector_norm(identity.iter().copied());
    let orthonormality = options.residual.check(
        vector_norm((combined.t().dot(&combined) - &identity).iter().copied()),
        identity_norm,
    )?;
    let span = options.residual.check(
        vector_norm((combined.dot(&combined.t()) - &identity).iter().copied()),
        identity_norm,
    )?;
    let ambient_norm = vector_norm(ambient.iter().flat_map(|a| a.iter().copied()));
    let center_defect = vector_norm(ambient.iter().flat_map(|a| a.dot(center).into_iter()));
    let centrality = options.residual.check(
        center_defect,
        ambient_norm * vector_norm(center.iter().copied()),
    )?;
    let mut ideal_invariance = Vec::new();
    let mut cross_commutation = Vec::new();
    let mut abelianity = Vec::new();
    let mut ideal_centers = Vec::new();
    let mut simple_commutants = Vec::new();
    for axes in ideals {
        let projector = axes.dot(&axes.t());
        let defect = vector_norm(ambient.iter().flat_map(|a| {
            let action = a.dot(axes);
            (&action - projector.dot(&action)).into_iter()
        }));
        ideal_invariance.push(
            options
                .residual
                .check(defect, ambient_norm * vector_norm(axes.iter().copied()))?,
        );
        let restricted = restricted_adjoints(ambient, axes)?;
        let restricted_norm = vector_norm(restricted.iter().flat_map(|a| a.iter().copied()));
        abelianity.push(options.residual.check(restricted_norm, ambient_norm)?);
        let kernel = adjoint_kernel(&restricted, axes.ncols());
        let analysis = rank_nullspace(&kernel, options)?;
        ideal_centers.push(diagnostics(
            &kernel,
            &analysis,
            options,
            "restricted ideal center",
        )?);
        simple_commutants
            .push(symmetric_commutant(&restricted, axes.ncols(), options)?.diagnostics);
    }
    for (i, left) in ideals.iter().enumerate() {
        for right in &ideals[i + 1..] {
            let mut defect = R::zero();
            let mut scale = R::zero();
            for vector in left.columns() {
                let action = Array2::from_shape_fn((n, n), |(a, b)| {
                    (0..n).fold(R::zero(), |sum, k| sum + vector[k] * ambient[k][(a, b)])
                });
                defect = defect.hypot(vector_norm(action.dot(right).iter().copied()));
                scale = scale.hypot(
                    vector_norm(action.iter().copied()) * vector_norm(right.iter().copied()),
                );
            }
            cross_commutation.push(options.residual.check(defect, scale)?);
        }
    }
    Ok(DecompositionValidation {
        orthonormality,
        span,
        centrality,
        ideal_invariance,
        cross_commutation,
        abelianity,
        ideal_centers,
        simple_commutants,
    })
}

fn decompose_with_sampler<'a, E, F>(
    adjoint: &AdjointRepresentation<'a, E>,
    settings: &DecompositionOptions,
    mut sample: F,
) -> Result<LieDecomposition<'a, E>, LieError<E::Real>>
where
    E: FamilyExpression,
    NumericalOptions<E::Real>: Default,
    F: FnMut(usize, &[Array2<E::Real>]) -> Array2<E::Real>,
{
    settings.validate()?;
    let options = adjoint.options();
    options.validate()?;
    let split = adjoint.center()?;
    let ambient = &split.orthonormal.adjoints;
    let semisimple = &split.complement_axes;
    let dimension = semisimple.ncols();
    let mut attempts = Vec::new();
    let mut accepted = None;
    let mut commutant_diagnostics = None;
    if dimension == 0 {
        let validation = validate_candidate(ambient, &split.center_axes, &[], options)?;
        for (operation, check) in [
            (
                "abelian decomposition orthonormality",
                validation.orthonormality,
            ),
            ("abelian decomposition span", validation.span),
            ("abelian decomposition centrality", validation.centrality),
        ] {
            if !check.passed() {
                return Err(LieError::ConstraintResidual {
                    operation,
                    index: 0,
                    check,
                });
            }
        }
        accepted = Some((Vec::new(), validation));
    } else {
        let restricted = restricted_adjoints(ambient, semisimple)?;
        let commutant = symmetric_commutant(&restricted, dimension, options)?;
        let expected = commutant.matrices.len();
        let relative_gap = options.eigenvalue_cutoff(dimension)?;
        commutant_diagnostics = Some(commutant.diagnostics);
        for _ in 0..settings.max_attempts {
            let spectrum = symmetric_eigen(&sample(dimension, &commutant.matrices), options)?;
            let largest = spectrum
                .values
                .iter()
                .copied()
                .fold(E::Real::zero(), |m, x| m.max(Float::abs(x)));
            let gap_threshold = relative_gap * largest;
            finite([gap_threshold])?;
            let groups = groups(&spectrum.values, gap_threshold);
            let mut attempt = DecompositionAttempt {
                eigenvalues: spectrum.values,
                relative_gap,
                gap_threshold,
                groups,
                validation: None,
                failure: None,
            };
            if attempt.groups.len() != expected {
                attempt.failure = Some(DecompositionFailure::ComponentCount {
                    expected,
                    found: attempt.groups.len(),
                });
                attempts.push(attempt);
                continue;
            }
            let axes: Vec<_> = attempt
                .groups
                .iter()
                .map(|group| {
                    semisimple.dot(
                        &spectrum
                            .vectors
                            .slice_axis(Axis(1), Slice::from(group.clone())),
                    )
                })
                .collect();
            let validation = match validate_candidate(ambient, &split.center_axes, &axes, options) {
                Ok(validation) => validation,
                Err(LieError::ConstraintResidual {
                    operation,
                    index,
                    check,
                }) => {
                    attempt.failure = Some(DecompositionFailure::ConstraintResidual {
                        operation,
                        index,
                        check,
                    });
                    attempts.push(attempt);
                    continue;
                }
                Err(error) => return Err(error),
            };
            let passed = validation.passed();
            attempt.validation = Some(validation.clone());
            if !passed {
                attempt.failure = Some(DecompositionFailure::Validation);
            }
            attempts.push(attempt);
            if passed {
                accepted = Some((axes, validation));
                break;
            }
        }
    }
    let Some((axes, validation)) = accepted else {
        return Err(LieError::DecompositionExhausted {
            seed: settings.seed,
            attempts,
        });
    };
    let center = if split.center_axes.ncols() == 0 {
        None
    } else {
        Some(component(
            adjoint.family(),
            split.center_coordinates().clone(),
            options,
        )?)
    };
    let simple_ideals = axes
        .iter()
        .map(|q| component(adjoint.family(), split.orthonormal.forward.dot(q), options))
        .collect::<Result<Vec<_>, _>>()?;
    for (index, component) in center.iter().chain(&simple_ideals).enumerate() {
        check_reconstruction(component, index)?;
    }
    let diagnostics = DecompositionDiagnostics {
        center: split.diagnostics().clone(),
        semisimple_commutant: commutant_diagnostics,
        attempts,
        validation,
    };
    Ok(LieDecomposition {
        family: adjoint.family(),
        center,
        simple_ideals,
        diagnostics,
        options: *options,
        decomposition_options: *settings,
    })
}

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
