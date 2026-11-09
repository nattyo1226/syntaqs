//! Errors at the numerical analysis boundary, independent of expression evaluation.

/// Invalid numerical input, singular basis, or failed factorization.
#[derive(Debug, thiserror::Error)]
pub enum NumericalError {
    /// A tolerance, measurement, or scale is negative or nonfinite.
    #[error("invalid numerical setting: {0}")]
    InvalidSetting(&'static str),
    /// Input or output at the numerical boundary is not finite.
    #[error("nonfinite numerical data: {0}")]
    NonFinite(&'static str),
    /// Matrix/vector dimensions disagree.
    #[error("incompatible numerical shape: {0}")]
    Shape(&'static str),
    /// An imaginary component exceeds its configured bound.
    #[error("imaginary component exceeds the configured tolerance")]
    NonReal,
    /// A negative squared norm exceeds the roundoff allowance.
    #[error("squared norm is negative beyond the roundoff allowance")]
    NegativeSquaredNorm,
    /// Matrix asymmetry exceeds the configured bound.
    #[error("matrix is not symmetric within the configured tolerance")]
    NotSymmetric,
    /// A Gram matrix has a materially negative eigenvalue.
    #[error("Gram matrix is not positive semidefinite within tolerance")]
    IndefiniteGram,
    /// Unique coordinates cannot be computed under the selected cutoff.
    #[error("numerically singular basis: rank {rank}, dimension {dimension}")]
    SingularBasis {
        /// Effective numerical rank.
        rank: usize,
        /// Number of basis elements.
        dimension: usize,
    },
    /// A requested factorization output is missing.
    #[error("factorization did not return requested singular vectors")]
    MissingSingularVectors,
    /// LAPACK or its ndarray adapter rejected a calculation.
    #[error("{0}")]
    Backend(#[from] ndarray_linalg::error::LinalgError),
}

/// Invalid family context or failed operator-analysis precondition.
#[derive(Debug, thiserror::Error)]
pub enum LieError<R> {
    /// A bracket lies outside the family span under the recorded tolerance.
    #[error("bracket of family elements {left} and {right} is outside the span")]
    NonClosedPair {
        /// Left input index.
        left: usize,
        /// Right input index.
        right: usize,
        /// Residual norm, bracket norm scale, and effective bound.
        check: crate::ToleranceCheck<R>,
    },
    /// A family must retain at least one input operator.
    #[error("operator family is empty")]
    EmptyFamily,
    /// Core rejected a binding, factor index, or expression context.
    #[error("{0}")]
    Expression(#[from] syntaqs_core::ExpressionError),
    /// Invalid numerical data, singularity, or a factorization failure.
    #[error("{0}")]
    Numerical(#[from] NumericalError),
    /// An operation requiring Hermitian family elements rejected one input.
    #[error("family element {index} is not Hermitian within tolerance")]
    NonHermitianElement {
        /// Index in the original family order.
        index: usize,
        /// Measured defect, scale, and effective bound.
        check: crate::ToleranceCheck<R>,
    },
    /// Real projection requires a Hermitian target.
    #[error("target is not Hermitian within tolerance")]
    NonHermitianTarget {
        /// Measured defect, scale, and effective bound.
        check: crate::ToleranceCheck<R>,
    },
}
