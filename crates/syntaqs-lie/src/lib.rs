//! Numerical foundations for Lie algebra analysis on syntaqs expressions.
//!
//! Coefficients retain their real precision. Native BLAS/LAPACK implementations
//! are selected by the application through `blas-src`; this crate selects none.
//! Operator families and Lie decomposition are subsequent implementation steps.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate blas_src as _;

#[cfg(test)]
extern crate blas_provider as _;

mod error;
// These tested kernels are consumed by operator families in the next step.
#[allow(dead_code)]
mod numerical;
mod tolerance;

pub use error::NumericalError;
pub use tolerance::{NumericalOptions, Tolerance, ToleranceCheck};
