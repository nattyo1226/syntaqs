//! Full real adjoint commutants and normalized-HS center splittings.

use ndarray::{Array1, Array2};
use ndarray_linalg::{Lapack, Scalar};
use num_traits::{Float, Zero};

use crate::{
    AdjointRepresentation, FamilyExpression, LieError, NumericalOptions, OperatorFamily,
    ToleranceCheck,
    numerical::{RankAnalysis, finite, rank_nullspace},
};

/// Effective SVD rank and residual diagnostics for a real linear constraint map.
#[derive(Debug, Clone)]
pub struct ConstraintDiagnostics<R> {
    /// Number of scalar constraints.
    pub rows: usize,
    /// Number of real unknowns.
    pub columns: usize,
    /// Number of singular values strictly above the recorded threshold.
    pub rank: usize,
    /// Singular values in descending order, including numerical zeros.
    pub singular_values: Array1<R>,
    /// Effective relative singular-value cutoff.
    pub relative_cutoff: R,
    /// Absolute cutoff, relative cutoff times the largest singular value.
    pub threshold: R,
    /// One check per nullspace vector, at scale `||L||_F ||v||_2`.
    pub residuals: Vec<ToleranceCheck<R>>,
}

/// The full real matrix space commuting with every retained adjoint matrix.
///
/// Matrices act on the original family's real coordinates. This is neither a
/// physical-operator centralizer nor merely the symmetric commuting subspace.
/// Its basis is Frobenius-orthonormal in these coordinates; the subspace, rather
/// than individual basis matrices, is invariant under changes of family basis.
pub struct AdjointCommutant<'a, E: FamilyExpression> {
    family: &'a OperatorFamily<E>,
    matrices: Vec<Array2<E::Real>>,
    diagnostics: ConstraintDiagnostics<E::Real>,
    options: NumericalOptions<E::Real>,
}

impl<'a, E: FamilyExpression> AdjointCommutant<'a, E> {
    /// Exact source family defining matrix row and column coordinates.
    pub fn family(&self) -> &'a OperatorFamily<E> {
        self.family
    }
    /// A basis of the full real adjoint commutant, in original coordinates.
    pub fn matrices(&self) -> &[Array2<E::Real>] {
        &self.matrices
    }
    /// Dimension of the full real commuting matrix space.
    pub fn dimension(&self) -> usize {
        self.matrices.len()
    }
    /// SVD diagnostics for row-major matrix unknowns `C[row,column]`.
    pub fn diagnostics(&self) -> &ConstraintDiagnostics<E::Real> {
        &self.diagnostics
    }
    /// Settings retained from the source adjoint representation.
    pub fn options(&self) -> &NumericalOptions<E::Real> {
        &self.options
    }
}

/// Full center and its HS-orthogonal complement, in the original family context.
///
/// Coordinate columns reconstruct the returned HS-orthonormal operator families.
/// Either component may be absent. Center extraction uses internal orthonormal
/// coordinates; no empty `OperatorFamily` is ever constructed.
pub struct CenterSplit<'a, E: FamilyExpression> {
    family: &'a OperatorFamily<E>,
    center: Option<OperatorFamily<E>>,
    complement: Option<OperatorFamily<E>>,
    center_coordinates: Array2<E::Real>,
    complement_coordinates: Array2<E::Real>,
    orthonormal: OrthonormalCoordinates<E::Real>,
    diagnostics: ConstraintDiagnostics<E::Real>,
    centrality: Vec<ToleranceCheck<E::Real>>,
    options: NumericalOptions<E::Real>,
}

impl<'a, E: FamilyExpression> CenterSplit<'a, E> {
    /// Exact original family and common expression context.
    pub fn family(&self) -> &'a OperatorFamily<E> {
        self.family
    }
    /// Full center, or `None` for a centerless algebra.
    pub fn center(&self) -> Option<&OperatorFamily<E>> {
        self.center.as_ref()
    }
    /// HS-orthogonal complement, or `None` for a purely abelian algebra.
    pub fn complement(&self) -> Option<&OperatorFamily<E>> {
        self.complement.as_ref()
    }
    /// HS-orthonormal center columns in original family coordinates, shape `(n,z)`.
    pub fn center_coordinates(&self) -> &Array2<E::Real> {
        &self.center_coordinates
    }
    /// HS-orthonormal complementary columns in original coordinates, shape `(n,n-z)`.
    pub fn complement_coordinates(&self) -> &Array2<E::Real> {
        &self.complement_coordinates
    }
    /// `U` taking internal orthonormal coordinates to original coordinates.
    pub fn orthonormal_transform(&self) -> &Array2<E::Real> {
        &self.orthonormal.forward
    }
    /// `U^-1=U^T G`, taking original coordinates to internal orthonormal coordinates.
    pub fn inverse_orthonormal_transform(&self) -> &Array2<E::Real> {
        &self.orthonormal.inverse
    }
    /// Common-kernel SVD diagnostics in the internal orthonormal family.
    pub fn diagnostics(&self) -> &ConstraintDiagnostics<E::Real> {
        &self.diagnostics
    }
    /// Common-kernel checks of center columns against the original adjoint matrices.
    pub fn centrality(&self) -> &[ToleranceCheck<E::Real>] {
        &self.centrality
    }
    /// Settings retained from the source adjoint representation.
    pub fn options(&self) -> &NumericalOptions<E::Real> {
        &self.options
    }
}

fn vector_norm<R: Float>(values: impl IntoIterator<Item = R>) -> R {
    values
        .into_iter()
        .fold(R::zero(), |norm, value| norm.hypot(value))
}

fn constraint_checks<R>(
    constraints: &Array2<R>,
    basis: &Array2<R>,
    options: &NumericalOptions<R>,
    operation: &'static str,
) -> Result<Vec<ToleranceCheck<R>>, LieError<R>>
where
    R: Float + Scalar<Real = R> + Lapack,
    NumericalOptions<R>: Default,
{
    finite(constraints.iter().copied().chain(basis.iter().copied()))?;
    let constraint_norm = vector_norm(constraints.iter().copied());
    finite([constraint_norm])?;
    let residual = constraints.dot(basis);
    finite(residual.iter().copied())?;
    let mut checks = Vec::new();
    for (index, vector) in basis.columns().into_iter().enumerate() {
        let measured = vector_norm(residual.column(index).iter().copied());
        let scale = constraint_norm * vector_norm(vector.iter().copied());
        let check = options.residual.check(measured, scale)?;
        if !check.passed() {
            return Err(LieError::ConstraintResidual {
                operation,
                index,
                check,
            });
        }
        checks.push(check);
    }
    Ok(checks)
}

fn diagnostics<R>(
    constraints: &Array2<R>,
    analysis: &RankAnalysis<R>,
    options: &NumericalOptions<R>,
    operation: &'static str,
) -> Result<ConstraintDiagnostics<R>, LieError<R>>
where
    R: Float + Scalar<Real = R> + Lapack,
    NumericalOptions<R>: Default,
{
    Ok(ConstraintDiagnostics {
        rows: constraints.nrows(),
        columns: constraints.ncols(),
        rank: analysis.rank,
        singular_values: analysis.singular_values.clone(),
        relative_cutoff: analysis.relative_cutoff,
        threshold: analysis.threshold,
        residuals: constraint_checks(constraints, &analysis.nullspace, options, operation)?,
    })
}

fn adjoint_kernel<R: Float>(matrices: &[Array2<R>], n: usize) -> Array2<R> {
    Array2::from_shape_fn((matrices.len() * n, n), |(row, col)| {
        matrices[row / n][(row % n, col)]
    })
}

fn commutant_constraints<R: Float>(matrices: &[Array2<R>], n: usize) -> Array2<R> {
    // Rows (generator,k,l), unknowns C[a,b] in row-major order.
    // (CA-AC)[k,l] = sum_b C[k,b] A[b,l] - sum_a A[k,a] C[a,l].
    let mut result = Array2::zeros((matrices.len() * n * n, n * n));
    for (i, matrix) in matrices.iter().enumerate() {
        for k in 0..n {
            for l in 0..n {
                for j in 0..n {
                    let row = (i * n + k) * n + l;
                    result[(row, k * n + j)] = result[(row, k * n + j)] + matrix[(j, l)];
                    result[(row, j * n + l)] = result[(row, j * n + l)] - matrix[(k, j)];
                }
            }
        }
    }
    result
}

pub(crate) struct OrthonormalCoordinates<R> {
    pub forward: Array2<R>,
    pub inverse: Array2<R>,
    pub adjoints: Vec<Array2<R>>,
}

pub(crate) fn orthonormal_coordinates<E: FamilyExpression>(
    adjoint: &AdjointRepresentation<'_, E>,
) -> Result<OrthonormalCoordinates<E::Real>, LieError<E::Real>>
where
    NumericalOptions<E::Real>: Default,
{
    let family = adjoint.family();
    let transform = family.orthonormalize(adjoint.options())?;
    let forward = transform.matrix().clone();
    let gram = family.gram(adjoint.options())?.values().mapv(|z| z.re);
    let inverse = forward.t().dot(&gram);
    let n = family.len();
    let mut adjoints = Vec::new();
    for j in 0..n {
        let combination = Array2::from_shape_fn((n, n), |(k, l)| {
            (0..n).fold(E::Real::zero(), |sum, i| {
                sum + forward[(i, j)] * adjoint.matrices()[i][(k, l)]
            })
        });
        let matrix = inverse.dot(&combination).dot(&forward);
        finite(matrix.iter().copied())?;
        adjoints.push(matrix);
    }
    finite(forward.iter().copied().chain(inverse.iter().copied()))?;
    Ok(OrthonormalCoordinates {
        forward,
        inverse,
        adjoints,
    })
}

fn reconstruct_component<E: FamilyExpression>(
    family: &OperatorFamily<E>,
    coordinates: &Array2<E::Real>,
) -> Result<Option<OperatorFamily<E>>, LieError<E::Real>> {
    if coordinates.ncols() == 0 {
        return Ok(None);
    }
    let operators = coordinates
        .columns()
        .into_iter()
        .map(|v| family.reconstruct(v))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(OperatorFamily::new_in(family.context(), operators)?))
}

impl<'a, E: FamilyExpression> AdjointRepresentation<'a, E>
where
    NumericalOptions<E::Real>: Default,
{
    /// Finds the full real adjoint commutant in the original family coordinates.
    /// Rank classification uses `rank_relative`; candidates must also pass the
    /// residual tolerance. A valid nullspace is not treated as a singular basis.
    pub fn commutant(&self) -> Result<AdjointCommutant<'a, E>, LieError<E::Real>> {
        let n = self.family().len();
        let constraints = commutant_constraints(self.matrices(), n);
        let analysis = rank_nullspace(&constraints, self.options())?;
        let diagnostics =
            diagnostics(&constraints, &analysis, self.options(), "adjoint commutant")?;
        let matrices = analysis
            .nullspace
            .columns()
            .into_iter()
            .map(|v| Array2::from_shape_fn((n, n), |(i, j)| v[i * n + j]))
            .collect();
        Ok(AdjointCommutant {
            family: self.family(),
            matrices,
            diagnostics,
            options: *self.options(),
        })
    }

    /// Extracts the full center and its HS-orthogonal complement.
    /// The common-kernel SVD runs in internal HS-orthonormal coordinates and
    /// retains its complementary right singular vectors for the direct sum.
    pub fn center(&self) -> Result<CenterSplit<'a, E>, LieError<E::Real>> {
        let n = self.family().len();
        let orthonormal = orthonormal_coordinates(self)?;
        let constraints = adjoint_kernel(&orthonormal.adjoints, n);
        let analysis = rank_nullspace(&constraints, self.options())?;
        let diagnostics = diagnostics(
            &constraints,
            &analysis,
            self.options(),
            "orthonormal adjoint center",
        )?;
        let center_coordinates = orthonormal.forward.dot(&analysis.nullspace);
        let complement_coordinates = orthonormal.forward.dot(&analysis.complement);
        finite(
            center_coordinates
                .iter()
                .copied()
                .chain(complement_coordinates.iter().copied()),
        )?;
        let centrality = constraint_checks(
            &adjoint_kernel(self.matrices(), n),
            &center_coordinates,
            self.options(),
            "original adjoint center",
        )?;
        let center = reconstruct_component(self.family(), &center_coordinates)?;
        let complement = reconstruct_component(self.family(), &complement_coordinates)?;
        Ok(CenterSplit {
            family: self.family(),
            center,
            complement,
            center_coordinates,
            complement_coordinates,
            orthonormal,
            diagnostics,
            centrality,
            options: *self.options(),
        })
    }
}

impl<E: FamilyExpression> OperatorFamily<E>
where
    NumericalOptions<E::Real>: Default,
{
    /// Validates the Lie preconditions and finds the full real adjoint commutant.
    pub fn commutant(
        &self,
        options: &NumericalOptions<E::Real>,
    ) -> Result<AdjointCommutant<'_, E>, LieError<E::Real>> {
        self.structure_constants(options)?.adjoint().commutant()
    }
    /// Validates the Lie preconditions and extracts the full center and complement.
    pub fn center(
        &self,
        options: &NumericalOptions<E::Real>,
    ) -> Result<CenterSplit<'_, E>, LieError<E::Real>> {
        self.structure_constants(options)?.adjoint().center()
    }
}

// Staged for Step 5's spectral ideal splitting; intentionally distinct from the
// public full real commutant. Unit tests exercise it before decomposition lands.
#[allow(dead_code)]
pub(crate) struct SymmetricCommutant<R> {
    pub matrices: Vec<Array2<R>>,
    pub diagnostics: ConstraintDiagnostics<R>,
}

#[allow(dead_code)]
pub(crate) fn symmetric_commutant<R>(
    orthonormal_adjoints: &[Array2<R>],
    n: usize,
    options: &NumericalOptions<R>,
) -> Result<SymmetricCommutant<R>, LieError<R>>
where
    R: Float + Scalar<Real = R> + Lapack,
    NumericalOptions<R>: Default,
{
    // Frobenius-orthonormal symmetric parameter basis: E_aa and
    // (E_ab+E_ba)/sqrt(2). It is meaningful only in HS-orthonormal coordinates.
    let pairs: Vec<_> = (0..n).flat_map(|a| (a..n).map(move |b| (a, b))).collect();
    let weight = R::one() / Float::sqrt(R::one() + R::one());
    let full = commutant_constraints(orthonormal_adjoints, n);
    let constraints = Array2::from_shape_fn((full.nrows(), pairs.len()), |(row, col)| {
        let (a, b) = pairs[col];
        if a == b {
            full[(row, a * n + b)]
        } else {
            (full[(row, a * n + b)] + full[(row, b * n + a)]) * weight
        }
    });
    let analysis = rank_nullspace(&constraints, options)?;
    let diagnostics = diagnostics(
        &constraints,
        &analysis,
        options,
        "symmetric adjoint commutant",
    )?;
    let matrices = analysis
        .nullspace
        .columns()
        .into_iter()
        .map(|column| {
            let mut matrix = Array2::zeros((n, n));
            for (i, &(a, b)) in pairs.iter().enumerate() {
                let value = if a == b {
                    column[i]
                } else {
                    column[i] * weight
                };
                matrix[(a, b)] = value;
                matrix[(b, a)] = value;
            }
            matrix
        })
        .collect();
    Ok(SymmetricCommutant {
        matrices,
        diagnostics,
    })
}

#[cfg(test)]
mod tests;
