//! Numerical configuration, diagnostics and internal kernels.

mod error;
pub(crate) mod kernels;
mod options;
mod tolerance;

pub use error::NumericalError;
pub use options::NumericalOptions;
pub use tolerance::{Tolerance, ToleranceCheck};
