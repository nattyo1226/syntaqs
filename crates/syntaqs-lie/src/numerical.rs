//! Numerical configuration, diagnostics and internal kernels.

/// Numerical failures.
pub mod error;
pub(crate) mod kernels;
/// Numerical configuration.
pub mod options;
/// Tolerances and comparison results.
pub mod tolerance;

pub(crate) use error::NumericalError;
pub(crate) use options::NumericalOptions;
pub(crate) use tolerance::{Tolerance, ToleranceCheck};
