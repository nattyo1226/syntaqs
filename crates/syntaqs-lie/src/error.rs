//! Errors during operator-family analysis.

use crate::NumericalError;

/// Invalid family context or failed operator-analysis precondition.
#[derive(Debug, thiserror::Error)]
pub enum LieError<R> {
    /// Sampling failed to produce a validated complete simple-ideal split.
    #[error("decomposition exhausted its configured attempt budget (seed {seed})")]
    DecompositionExhausted {
        /// Seed used by the deterministic sampling stream.
        seed: u64,
        /// Spectral grouping and validation diagnostics for every failed attempt.
        attempts: Vec<crate::DecompositionAttempt<R>>,
    },
    /// A computed constraint-nullspace vector fails the residual tolerance.
    #[error("{operation} candidate {index} exceeds the constraint residual tolerance")]
    ConstraintResidual {
        /// Operation whose linear constraints were checked.
        operation: &'static str,
        /// Index of the returned candidate basis vector or matrix.
        index: usize,
        /// Aggregate residual norm, reference scale, and effective bound.
        check: crate::ToleranceCheck<R>,
    },
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
    Expression(#[from] syntaqs_core::expression::error::ExpressionError),
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
