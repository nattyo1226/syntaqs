//! Numerical foundations for Lie algebra analysis on syntaqs expressions.
//!
//! Coefficients retain their real precision. Native BLAS/LAPACK implementations
//! are selected by the application through `blas-src`; this crate selects none.
//! Ordered operator families support explicit validation and real HS projection.
//! Lie brackets, closure diagnostics, retained representations, and real basis
//! transformations, adjoint commutants, centers, and complete simple-ideal
//! decomposition are supported.
//!
//! ```
//! # extern crate blas_provider as _;
//! use num_complex::Complex;
//! use syntaqs_core::expression::local::LocalOperator;
//! use syntaqs_lie::{decomposition::DecompositionOptions, LieError, numerical::options::NumericalOptions, family::OperatorFamily};
//! use syntaqs_spin_half::operator::SpinHalfOperator as Pauli;
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
//! let settings = DecompositionOptions { seed: 7, max_attempts: 16 };
//! let decomposition = adjoint.decompose(&settings)?;
//! assert!(decomposition.center().is_none());
//! assert_eq!(decomposition.simple_ideals().len(), 1);
//! assert!(decomposition.diagnostics().validation.passed());
//! # Ok::<(), LieError<f64>>(())
//! ```
//!
//! The former root and parent-module facades are not public:
//!
//! ```compile_fail
//! use syntaqs_lie::OperatorFamily;
//! ```
//! ```compile_fail
//! use syntaqs_lie::NumericalOptions;
//! ```
//! ```compile_fail
//! use syntaqs_lie::FamilyTransform;
//! ```
//! ```compile_fail
//! use syntaqs_lie::family::GramMatrix;
//! ```
//! ```compile_fail
//! use syntaqs_lie::numerical::Tolerance;
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate blas_src as _;

#[cfg(test)]
extern crate blas_provider as _;

pub mod decomposition;
mod expression;
pub mod family;
pub mod invariant;
pub mod representation;
// Some kernels are reserved for subsequent Lie-representation operations.
#[allow(dead_code)]
pub mod numerical;

pub(crate) use decomposition::DecompositionAttempt;
pub(crate) use family::{
    FamilyCoordinates, FamilyValidation, GramFactorization, OperatorFamily, Projection,
};
pub(crate) use invariant::ConstraintDiagnostics;
pub(crate) use numerical::{NumericalError, NumericalOptions, ToleranceCheck};
pub(crate) use representation::AdjointRepresentation;

use ndarray_linalg::{Lapack, Scalar};
use num_complex::Complex;
use num_traits::Float;
use syntaqs_core::expression::error::ExpressionError;

mod private {
    pub trait Sealed {}
    impl<O, R> Sealed for syntaqs_core::expression::local::LocalOperator<O, num_complex::Complex<R>> {}
    impl<I, O, R> Sealed
        for syntaqs_core::expression::composite::Operator<I, O, num_complex::Complex<R>>
    {
    }
}

/// A supported local or placed core operator with complex f32/f64 coefficients.
///
/// This sealed trait fixes the normalized Hilbert–Schmidt contraction. Users
/// supply primitive algebra rules through core traits, not a custom metric.
/// Primitives also need `PartialEq` for exact term collection before contraction.
/// Local context is `()`; placed context is `HilbertSpace<I>`.
pub trait FamilyExpression: private::Sealed + Clone {
    /// Real precision of the expression's complex coefficients.
    type Real: Float + Scalar<Real = Self::Real> + Lapack;
    /// Common context retained by a family.
    type Context: Clone;

    #[doc(hidden)]
    fn inferred_context(operators: &[Self]) -> Result<Self::Context, ExpressionError>;
    #[doc(hidden)]
    fn in_context(&self, context: &Self::Context) -> Result<Self, ExpressionError>;
    #[doc(hidden)]
    fn sum_in(operators: Vec<Self>, context: &Self::Context) -> Result<Self, ExpressionError>;
    #[doc(hidden)]
    fn collected_in(&self, context: &Self::Context) -> Result<Self, ExpressionError>;
    #[doc(hidden)]
    fn scaled_real(&self, coefficient: Self::Real) -> Self;
    #[doc(hidden)]
    fn adjoint_expression(&self) -> Self;
    #[doc(hidden)]
    fn lie_bracket_expression(&self, rhs: &Self) -> Result<Self, ExpressionError>;
    #[doc(hidden)]
    fn hs_in(
        &self,
        rhs: &Self,
        context: &Self::Context,
    ) -> Result<Complex<Self::Real>, ExpressionError>;
}

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

/// Constructs `-i(AB-BA)`, including on graded spaces (an ordinary bracket).
///
/// This constructs an expression without numerical validation. Core resolves
/// physical grading signs and rejects incompatible placed contexts.
pub fn lie_bracket<E: FamilyExpression>(left: &E, right: &E) -> Result<E, LieError<E::Real>> {
    Ok(left.lie_bracket_expression(right)?)
}

/// Common entry points for operator-family analysis.
pub mod prelude {
    pub use crate::{family::OperatorFamily, lie_bracket, numerical::options::NumericalOptions};
}
