//! Norms, state normalization and normalized expectations.
//!
//! Import the corresponding extension traits to use these operations. Squared
//! norms, inner products, expectations and scalar division belong to `syntaqs-core`.
//! Only square-root-based operations need [`Float`](num_traits::Float) on the associated real value;
//! normalized expectations require neither square roots nor real-part extraction.
//!
//! Extension traits are available in their modules and the prelude:
//!
//! ```compile_fail
//! use syntaqs_normalize::Norm;
//! ```
//! ```compile_fail
//! use syntaqs_normalize::Normalize;
//! ```
//! ```compile_fail
//! use syntaqs_normalize::NormalizedExpectation;
//! ```
//! ```compile_fail
//! use syntaqs_normalize::NormalizedExpectationIn;
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
pub mod expectation;
pub mod norm;

use syntaqs_core::expression::error::ExpressionError;

/// Failure to normalize a state or an expectation value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NormalizationError {
    /// The computed self-inner-product is exactly zero, including cancellation.
    #[error("state has zero norm")]
    ZeroNorm,
    /// Placement, binding or contraction failed.
    #[error(transparent)]
    Expression(#[from] ExpressionError),
}

/// Common norm, normalization and expectation extension traits.
pub mod prelude {
    pub use crate::expectation::{NormalizedExpectation, NormalizedExpectationIn};
    pub use crate::norm::{Norm, Normalize};
}
