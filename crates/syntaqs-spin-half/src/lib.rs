//! Spin-1/2 primitives, local rules, and ordered bases.
//!
//! ```compile_fail
//! use syntaqs_spin_half::SpinHalfState;
//! ```
//! ```compile_fail
//! use syntaqs_spin_half::SpinHalfOperator;
//! ```
//! ```compile_fail
//! use syntaqs_spin_half::SzBasis;
//! ```
//!
//! Cartesian primitives are Pauli matrices: physical spins are `J = S/2`
//! with `hbar = 1`. `Sp = (Sx + i Sy)/2`, `Sm = (Sx - i Sy)/2`.
//! States are ordered `Up, Down`; CartesianBasis is orthonormal under `Tr(A†B)/2`.
//! Primitive types carry no numeric precision; the coefficient type is inferred
//! from the result, specified through a qualified trait call, or selected once
//! using [`SpinHalf<C>`] and [`syntaqs_core::system::ExpressionFactory`].
//!
//! ```
//! use num_complex::Complex;
//! use syntaqs_core::{algebra::Act, expression::parts::linear_combination::LinearCombination, algebra::Product};
//! use syntaqs_spin_half::{operator::SpinHalfOperator as Op, state::SpinHalfState as State};
//!
//! let raised: LinearCombination<State, Complex<f64>> = State::Down.act(&Op::Sp);
//! assert_eq!(raised, LinearCombination::primitive(State::Up));
//!
//! let squared = <Op as Product<Complex<f32>>>::product(&Op::Sx, &Op::Sx);
//! assert_eq!(squared, LinearCombination::primitive(Op::I));
//! ```
//!
//! Local products retain natural primitives; basis expansion is a separate step:
//!
//! ```
//! use num_complex::Complex;
//! use syntaqs_core::{basis::Basis, expression::parts::linear_combination::LinearCombination, algebra::Product};
//! use syntaqs_spin_half::{basis::CartesianBasis, operator::SpinHalfOperator as Op};
//!
//! let product: LinearCombination<Op, Complex<f64>> = Op::I.product(&Op::Sp);
//! assert_eq!(product, LinearCombination::primitive(Op::Sp));
//! let expanded = <CartesianBasis as Basis<Op, Complex<f64>>>::expand(&CartesianBasis, &Op::Sp).unwrap();
//! assert_eq!(expanded, LinearCombination::term(Complex::new(0.5, 0.0), Op::Sx)
//!     + LinearCombination::term(Complex::new(0.0, 0.5), Op::Sy));
//! let elements: Vec<_> = <CartesianBasis as Basis<Op, Complex<f64>>>::elements(&CartesianBasis).collect();
//! assert_eq!(elements, [Op::I, Op::Sx, Op::Sy, Op::Sz]);
//! ```
//!
//! Numeric bounds are operation-specific. The numeric type must represent the
//! required real fractions; trait bounds alone cannot guarantee their laws.
//! No `Float`, `Copy`, `Clone`, `Num`, casts, or math backend is required.
//! This crate uses `syntaqs-core`'s finite linear combinations, and does not implement
//! expression evaluation or operator syntax.

//!
//! Shared local expressions construct actions lazily and evaluate without a basis:
//!
//! ```
//! use num_complex::Complex;
//! use syntaqs_spin_half::{SpinHalf, state::SpinHalfState as State, operator::SpinHalfOperator as Op};
//! type Spin = SpinHalf<Complex<f64>>;
//! let up = Spin::state(State::Up);
//! let sx = Spin::operator(Op::Sx);
//! let down = Spin::state(State::Down);
//! let result = up.act(&sx).evaluate();
//! assert_eq!(down.inner_product(&result), Complex::new(1.0, 0.0));
//! assert_eq!((sx.clone() * &sx).trace(), Complex::new(2.0, 0.0));
//! ```
//!
//! Place local expressions on independently identified factors to form a system:
//!
//! ```
//! use num_complex::Complex;
//! use syntaqs_core::space::HilbertSpace;
//! use syntaqs_spin_half::{SpinHalf, operator::SpinHalfOperator as Op, state::SpinHalfState as Ket};
//! type Spin = SpinHalf<Complex<f64>>;
//!
//! let a = HilbertSpace::<_>::factor("a");
//! let b = HilbertSpace::factor("b");
//! let h = a.tensor(&b)?;
//! let up = Spin::state(Ket::Up).at(&a)?;
//! let down = Spin::state(Ket::Down).at(&b)?;
//! let state = syntaqs_core::expression::composite::State::tensor_in(&h, [up, down])?;
//! let sz = Spin::operator(Op::Sz);
//! let zz = (sz.at(&a)?.clone() * &sz.at(&b)?)?.bind(&h)?;
//! assert_eq!(state.expectation(&zz)?, Complex::new(-1.0, 0.0));
//! assert_eq!(zz.trace()?, Complex::new(0.0, 0.0));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;

pub mod basis;
pub mod models;
pub mod operator;
pub mod state;

mod coefficient;

pub(crate) use operator::SpinHalfOperator;
pub(crate) use state::SpinHalfState;

/// Selects spin-1/2 primitives and a coefficient type without storing any values.
///
/// `state` and `operator` are inherent constructors; no trait import is needed.
/// There is no default precision; select `C` once in an application type alias.
///
/// ```
/// use num_complex::Complex;
/// use syntaqs_spin_half::{SpinHalf, state::SpinHalfState as Ket, operator::SpinHalfOperator as Op};
/// type Spin = SpinHalf<Complex<f32>>;
/// let up = Spin::state(Ket::Up);
/// let sx = Spin::operator(Op::Sx);
/// assert_eq!(up.act(&sx).inner_product(&Spin::state(Ket::Down)), Complex::new(1.0, 0.0));
/// ```
///
/// Different coefficient precisions cannot be mixed implicitly:
///
/// ```compile_fail
/// use num_complex::Complex;
/// use syntaqs_spin_half::{SpinHalf, state::SpinHalfState as Ket, operator::SpinHalfOperator as Op};
/// let up = SpinHalf::<Complex<f32>>::state(Ket::Up);
/// let sx = SpinHalf::<Complex<f64>>::operator(Op::Sx);
/// let _ = up.act(&sx);
/// ```
pub type SpinHalf<C> = syntaqs_core::system::ExpressionFactory<SpinHalfState, SpinHalfOperator, C>;

/// Common system, primitive, and model types.
pub mod prelude {
    pub use crate::{
        SpinHalf, models::SpinHalfModels, operator::SpinHalfOperator, state::SpinHalfState,
    };
}
