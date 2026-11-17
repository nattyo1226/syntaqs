//! Abstract algebra contracts, shared expressions and ordinary and graded tensor spaces.
//!
//! Expressions hold the whole coefficient `C`, including [`Complex<T>`](num_complex::Complex).
//! Numeric bounds belong to individual operations; primitive types need not be
//! tied to a numeric precision.
//! This crate uses `alloc` but does not require `std` or a math-function backend.
//! Local and composite expressions share immutable nodes and evaluate using
//! primitive rules. [`HilbertSpace`] holds ordered factors with generic indices;
//! index generation and concrete algebras belong to separate crates.
//!
//! Legacy module aliases and root facades are not part of the public API:
//!
//! ```compile_fail
//! use syntaqs_core::ops;
//! ```
//! ```compile_fail
//! use syntaqs_core::expr;
//! ```
//! ```compile_fail
//! use syntaqs_core::error::DimensionOverflow;
//! ```
//! ```compile_fail
//! use syntaqs_core::algebra::Dimension;
//! ```
//! ```compile_fail
//! use syntaqs_core::HilbertSpace;
//! ```
//! ```compile_fail
//! use syntaqs_core::expression::State;
//! ```
//! ```compile_fail
//! use syntaqs_core::expression::parts::LinearCombination;
//! ```
//!
//! ```
//! use num_complex::Complex;
//! use syntaqs_core::expression::parts::linear_combination::LinearCombination;
//!
//! let sum = LinearCombination::<_, Complex<f64>>::primitive("p")
//!     + LinearCombination::term(Complex::new(0.0, 0.0), "p");
//! assert_eq!(sum.len(), 2); // Duplicate primitives and zero coefficients remain.
//! let scaled = sum.scaled(Complex::new(0.0, 1.0));
//! assert_eq!(scaled.terms().next().unwrap().coefficient, Complex::new(0.0, 1.0));
//! ```
//!
//! Different coefficient precisions cannot be mixed implicitly:
//!
//! ```compile_fail
//! use num_complex::Complex;
//! use syntaqs_core::expression::parts::linear_combination::LinearCombination;
//! let single = LinearCombination::<u8, Complex<f32>>::primitive(0);
//! let double = LinearCombination::<u8, Complex<f64>>::primitive(0);
//! let _ = single + double;
//! ```
//!
//! Scaling also preserves the coefficient precision:
//!
//! ```compile_fail
//! use num_complex::Complex;
//! use syntaqs_core::expression::parts::linear_combination::LinearCombination;
//! let single = LinearCombination::<u8, Complex<f32>>::primitive(0);
//! let _ = single.scaled(Complex::<f64>::new(2.0, 0.0));
//! ```

//! Scaling accepts inputs convertible into the coefficient type selected at
//! construction. Numeric literals can be real while the stored C stays complex:
//!
//! ```
//! use num_complex::Complex;
//! use syntaqs_core::expression::local::LocalOperator;
//! let op = LocalOperator::<u8, Complex<f32>>::primitive(0).scaled(0.5);
//! let same = op.scaled(Complex::new(1.0, 0.0));
//! ```
//!
//! Choosing C explicitly (or through [`system::QuantumSystem`]) replaces inference from
//! the scalar argument. Remove redundant `.into()` calls at the call site and
//! specify inputs such as `Complex::<f32>::default()` when needed.
//!
//! Real inputs also preserve precision; f64 is not narrowed to `Complex<f32>`:
//!
//! ```compile_fail
//! use num_complex::Complex;
//! use syntaqs_core::expression::local::LocalState;
//! let state = LocalState::<u8, u8, Complex<f32>>::primitive(0);
//! let _ = state.scaled(0.5_f64);
//! ```
//!
//! ```compile_fail
//! use num_complex::Complex;
//! use syntaqs_core::{space::HilbertSpace, expression::local::LocalOperator};
//! let factor = HilbertSpace::factor(0);
//! let op = LocalOperator::<u8, Complex<f32>>::primitive(0).at(&factor).unwrap();
//! let _ = op.scaled(Complex::<f64>::new(0.5, 0.0));
//! ```

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;

#[cfg(test)]
extern crate self as syntaqs_core;
#[cfg(test)]
extern crate std;
#[cfg(test)]
#[path = "../tests/common/mod.rs"]
mod test_coefficient;

pub mod coefficient;
mod evaluate;

pub mod algebra;
pub mod basis;
pub mod expression;
pub mod space;
pub mod system;

pub(crate) use algebra::{
    Act, Adjoint, HomogeneousParity, Identity, InnerProduct, Parity, ParityOperator, Product, Trace,
};
pub(crate) use basis::{Basis, BasisError, Coordinates};
pub(crate) use coefficient::{Conjugate, Divide, RealPart};
pub(crate) use expression::parts::linear_combination::{LinearCombination, Term};
pub(crate) use expression::{
    composite::{Operator, State},
    error::ExpressionError,
    local::{LocalOperator, LocalState},
};
pub(crate) use space::{Dimension, DimensionOverflow, HilbertSpace, TensorKind};

/// Frequently used expression, algebra, basis and system types.
pub mod prelude {
    pub use crate::algebra::{Act, Adjoint, Identity, InnerProduct, Product, Trace};
    pub use crate::basis::Basis;
    pub use crate::expression::composite::{Operator, State};
    pub use crate::expression::local::{LocalOperator, LocalState};
    pub use crate::expression::parts::linear_combination::LinearCombination;
    pub use crate::space::HilbertSpace;
    pub use crate::system::ExpressionFactory;
}
