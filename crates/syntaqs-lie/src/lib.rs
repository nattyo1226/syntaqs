//! Numerical foundations for Lie algebra analysis on syntaqs expressions.
//!
//! Coefficients retain their real precision. Native BLAS/LAPACK implementations
//! are selected by the application through `blas-src`; this crate selects none.
//! Ordered operator families support explicit validation and real HS projection.
//! Lie brackets, closure diagnostics, retained representations, and real basis
//! transformations, adjoint commutants, and centers are supported. Simple-ideal
//! decomposition is a subsequent step.
//!
//! ```
//! # extern crate blas_provider as _;
//! use num_complex::Complex;
//! use syntaqs_core::LocalOperator;
//! use syntaqs_lie::{LieError, NumericalOptions, OperatorFamily};
//! use syntaqs_spin_half::SpinHalfOperator as Pauli;
//!
//! type Op = LocalOperator<Pauli, Complex<f64>>;
//! let x = Op::primitive(Pauli::Sx);
//! let y = Op::primitive(Pauli::Sy);
//! let z = Op::primitive(Pauli::Sz);
//! let family = OperatorFamily::new([x.clone(), z])?;
//! let options = NumericalOptions::<f64>::default();
//! let gram = family.gram(&options)?;
//! assert!(gram.validate()?.passed()); // Hermiticity and independence, not closure.
//! let factor = gram.factorize()?;
//! let projection = factor.project(&(x + y))?;
//! assert_eq!(projection.coordinates().values().as_slice().unwrap(), &[1., 0.]);
//! assert_eq!(projection.residual_norm(), 1.);
//! assert!(!projection.in_span());
//! let full = OperatorFamily::new([
//!     Op::primitive(Pauli::Sx), Op::primitive(Pauli::Sy), Op::primitive(Pauli::Sz),
//! ])?;
//! let constants = full.structure_constants(&options)?;
//! assert_eq!(constants.values()[(2, 0, 1)], 2.);
//! let killing = constants.adjoint().killing_form()?;
//! assert_eq!(killing.values()[(0, 0)], -8.);
//! let adjoint = constants.adjoint();
//! assert_eq!(adjoint.commutant()?.dimension(), 1);
//! let split = adjoint.center()?;
//! assert!(split.center().is_none());
//! assert_eq!(split.complement().unwrap().len(), 3);
//! # Ok::<(), LieError<f64>>(())
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate blas_src as _;

#[cfg(test)]
extern crate blas_provider as _;

mod error;
mod expression;
mod family;
mod invariant;
mod representation;
mod transform;
// Some kernels are reserved for subsequent Lie-representation operations.
#[allow(dead_code)]
mod numerical;
mod tolerance;

pub use error::{LieError, NumericalError};
pub use expression::FamilyExpression;
pub use family::{
    FamilyCoordinates, FamilyValidation, GramFactorization, GramMatrix, IndependenceReport,
    OperatorFamily, Projection,
};
pub use invariant::{AdjointCommutant, CenterSplit, ConstraintDiagnostics};
pub use representation::{
    AdjointRepresentation, BracketClosureCheck, ClosureReport, KillingForm, StructureConstants,
    lie_bracket,
};
pub use tolerance::{NumericalOptions, Tolerance, ToleranceCheck};
pub use transform::{FamilyTransform, OrthonormalityReport};
