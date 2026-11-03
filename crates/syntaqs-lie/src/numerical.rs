//! Internal real dense kernels. No physical Hilbert-space matrix is constructed here.

use ndarray::{Array1, Array2};
use ndarray_linalg::{Eigh, Lapack, SVD, Scalar, Solve, UPLO};
use num_complex::Complex;
use num_traits::{Float, FromPrimitive};

use crate::{NumericalError, NumericalOptions};

fn finite<R: Float>(values: impl IntoIterator<Item = R>) -> Result<(), NumericalError> {
    if values.into_iter().any(|value| !value.is_finite()) {
        return Err(NumericalError::NonFinite(
            "matrix/vector or factorization result",
        ));
    }
    Ok(())
}

/// Scaled sum-of-squares behavior without overflowing intermediate squares.
fn norm<R: Float>(values: impl IntoIterator<Item = R>) -> R {
    values
        .into_iter()
        .fold(R::zero(), |value, next| value.hypot(next))
}

pub(crate) fn real_coordinate<R: Float + FromPrimitive>(
    value: Complex<R>,
    scale: R,
    options: &NumericalOptions<R>,
) -> Result<R, NumericalError>
where
    NumericalOptions<R>: Default,
{
    options.validate()?;
    finite([value.re, value.im])?;
    if !options.imaginary.check(value.im.abs(), scale)?.passed() {
        return Err(NumericalError::NonReal);
    }
    Ok(value.re)
}

pub(crate) fn squared_norm<R: Float + FromPrimitive>(
    value: Complex<R>,
    scale_squared: R,
    options: &NumericalOptions<R>,
) -> Result<R, NumericalError>
where
    NumericalOptions<R>: Default,
{
    let real = real_coordinate(value, scale_squared, options)?;
    if real < R::zero() {
        if !options.roundoff.check(-real, scale_squared)?.passed() {
            return Err(NumericalError::NegativeSquaredNorm);
        }
        return Ok(R::zero());
    }
    Ok(real)
}

pub(crate) struct Spectrum<R> {
    pub values: Array1<R>,
    pub vectors: Array2<R>,
}

pub(crate) fn symmetric_eigen<R>(
    matrix: &Array2<R>,
    options: &NumericalOptions<R>,
) -> Result<Spectrum<R>, NumericalError>
where
    R: Float + Scalar<Real = R> + Lapack,
    NumericalOptions<R>: Default,
{
    options.validate()?;
    let n = matrix.nrows();
    if n != matrix.ncols() {
        return Err(NumericalError::Shape("eigensolve requires a square matrix"));
    }
    finite(matrix.iter().copied())?;
    let defect = norm(matrix.indexed_iter().map(|((i, j), &x)| x - matrix[(j, i)]));
    if !options
        .hermiticity
        .check(defect, norm(matrix.iter().copied()))?
        .passed()
    {
        return Err(NumericalError::NotSymmetric);
    }
    if n == 0 {
        return Ok(Spectrum {
            values: Array1::zeros(0),
            vectors: Array2::zeros((0, 0)),
        });
    }
    // Check the defect before averaging. Sum first to retain subnormals;
    // halve operands instead when the sum would overflow.
    let two = R::one() + R::one();
    let symmetric = Array2::from_shape_fn((n, n), |(i, j)| {
        let a = matrix[(i, j)];
        let b = matrix[(j, i)];
        if a == b {
            a
        } else {
            let sum = a + b;
            if sum.is_finite() {
                sum / two
            } else {
                a / two + b / two
            }
        }
    });
    let (values, vectors) = symmetric.eigh(UPLO::Lower)?;
    finite(values.iter().copied().chain(vectors.iter().copied()))?;
    Ok(Spectrum { values, vectors })
}

pub(crate) struct RankAnalysis<R> {
    pub rank: usize,
    pub singular_values: Array1<R>,
    pub relative_cutoff: R,
    pub threshold: R,
    /// Orthonormal right-nullspace vectors in columns, including wide-matrix nullity.
    pub nullspace: Array2<R>,
}

pub(crate) fn rank_nullspace<R>(
    matrix: &Array2<R>,
    options: &NumericalOptions<R>,
) -> Result<RankAnalysis<R>, NumericalError>
where
    R: Float + Scalar<Real = R> + Lapack,
    NumericalOptions<R>: Default,
{
    let (m, n) = matrix.dim();
    let relative_cutoff = options.rank_cutoff(m, n)?;
    finite(matrix.iter().copied())?;
    if m == 0 || n == 0 || matrix.iter().all(|&x| x == R::zero()) {
        return Ok(RankAnalysis {
            rank: 0,
            singular_values: Array1::zeros(m.min(n)),
            relative_cutoff,
            threshold: R::zero(),
            nullspace: Array2::eye(n),
        });
    }
    // gesvd returns the complete n-by-n Vt; an economy SVD would lose the
    // n-m structural nullspace directions when the matrix is wide.
    let (_, singular_values, vt) = matrix.svd(false, true)?;
    let vt = vt.ok_or(NumericalError::MissingSingularVectors)?;
    finite(singular_values.iter().copied().chain(vt.iter().copied()))?;
    let maximum = singular_values.iter().copied().fold(R::zero(), Float::max);
    let threshold = relative_cutoff * maximum;
    finite([threshold])?;
    let null_indices: Vec<_> = (0..n)
        .filter(|&i| i >= singular_values.len() || singular_values[i] <= threshold)
        .collect();
    let rank = n - null_indices.len();
    let nullspace =
        Array2::from_shape_fn((n, null_indices.len()), |(i, j)| vt[(null_indices[j], i)]);
    Ok(RankAnalysis {
        rank,
        singular_values,
        relative_cutoff,
        threshold,
        nullspace,
    })
}

pub(crate) fn solve<R>(
    matrix: &Array2<R>,
    rhs: &Array1<R>,
    options: &NumericalOptions<R>,
) -> Result<Array1<R>, NumericalError>
where
    R: Float + Scalar<Real = R> + Lapack,
    NumericalOptions<R>: Default,
{
    options.validate()?;
    if matrix.nrows() != matrix.ncols() || matrix.nrows() != rhs.len() {
        return Err(NumericalError::Shape(
            "solve requires a square matrix and matching vector",
        ));
    }
    finite(rhs.iter().copied())?;
    let analysis = rank_nullspace(matrix, options)?;
    if analysis.rank != matrix.ncols() {
        return Err(NumericalError::SingularBasis {
            rank: analysis.rank,
            dimension: matrix.ncols(),
        });
    }
    if rhs.is_empty() {
        return Ok(Array1::zeros(0));
    }
    let result = matrix.solve(rhs)?;
    finite(result.iter().copied())?;
    Ok(result)
}

/// Checks Gram eigenvalues directly, with a separately reported cutoff.
pub(crate) fn gram_spectrum<R>(
    matrix: &Array2<R>,
    options: &NumericalOptions<R>,
) -> Result<(Spectrum<R>, R), NumericalError>
where
    R: Float + Scalar<Real = R> + Lapack,
    NumericalOptions<R>: Default,
{
    let spectrum = symmetric_eigen(matrix, options)?;
    let largest = spectrum.values.iter().copied().fold(R::zero(), Float::max);
    let threshold = options.gram_cutoff(matrix.nrows())? * largest;
    finite([threshold])?;
    if spectrum.values.iter().any(|&value| value < -threshold) {
        return Err(NumericalError::IndefiniteGram);
    }
    let rank = spectrum
        .values
        .iter()
        .filter(|&&value| value > threshold)
        .count();
    if rank != matrix.nrows() {
        return Err(NumericalError::SingularBasis {
            rank,
            dimension: matrix.nrows(),
        });
    }
    Ok((spectrum, threshold))
}

#[cfg(test)]
mod tests;
