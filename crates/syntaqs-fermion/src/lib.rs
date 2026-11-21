//! Single-mode fermions, standard Majorana operators, and Hubbard chains.
//!
//! ```compile_fail
//! use syntaqs_fermion::FermionState;
//! ```
//! ```compile_fail
//! use syntaqs_fermion::FermionOperator;
//! ```
//! ```compile_fail
//! use syntaqs_fermion::OccupationBasis;
//! ```
//!
//! States are ordered `Empty, Occupied`. With `a = Annihilate`, the Hermitian
//! operators are `Mx = a + a†`, `My = (a - a†)/i`, and
//! `Mz = -i Mx My = I - 2n`. Thus `[Mx,My] = 2iMz` and `Mx² = My² = Mz² = I`.
//! Mx and My are odd Majoranas; Mz is the even parity operator.
//! Seven primitives span a four-dimensional operator space. MajoranaBasis
//! is orthonormal under the dimension-normalized inner product `Tr(A†B)/2`.
//!
//! Primitive types have no coefficient precision. [`Fermion<C>`] selects it
//! once, while [`syntaqs_core::system::ExpressionFactory`] constructs shared core expressions.
//! Numeric bounds belong to individual operations, not expression construction.
//! Complete operator bases are [`basis::LadderBasis`] and [`basis::MajoranaBasis`].
//!
//! ```
//! use num_complex::Complex;
//! use syntaqs_fermion::{Fermion, operator::FermionOperator as O, state::FermionState as S};
//! type F = Fermion<Complex<f64>>;
//! let occupied = F::state(S::Occupied);
//! let a = F::operator(O::Annihilate);
//! assert_eq!(F::state(S::Empty).inner_product(&occupied.act(&a)), Complex::new(1., 0.));
//! assert_eq!(a.anticommutator(&a.adjoint()).trace(), Complex::new(2., 0.));
//! ```
//!
//! Multiple modes use **graded** spaces and the core's common evaluator. No
//! mode indices or Jordan–Wigner strings are stored in the local primitives.
//!
//! ```
//! use num_complex::Complex;
//! use syntaqs_core::space::HilbertSpace;
//! use syntaqs_fermion::{Fermion, operator::FermionOperator as O, state::FermionState as S};
//! type F = Fermion<Complex<f32>>;
//! let up = HilbertSpace::factor("up");
//! let down = HilbertSpace::factor("down");
//! let space = HilbertSpace::tensor_all_g([up.clone(), down.clone()])?;
//! let double = F::state_tensor_from([S::Occupied, S::Occupied], &space)?;
//! let up_only = F::state_tensor_from([S::Occupied, S::Empty], &space)?;
//! let removed = double.act(&F::operator(O::Annihilate).at(&down)?)?;
//! assert_eq!(up_only.inner_product(&removed)?, Complex::new(-1., 0.));
//! # Ok::<(), syntaqs_core::expression::error::ExpressionError>(())
//! ```
//!
//! Different precisions cannot mix implicitly:
//!
//! ```compile_fail
//! use num_complex::Complex;
//! use syntaqs_fermion::{Fermion, operator::FermionOperator as O, state::FermionState as S};
//! let ket = Fermion::<Complex<f32>>::state(S::Empty);
//! let op = Fermion::<Complex<f64>>::operator(O::Create);
//! let _ = ket.act(&op);
//! ```

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;

pub mod basis;
mod coefficient;
pub mod models;
pub mod operator;
pub mod state;

pub(crate) use operator::FermionOperator;
pub(crate) use state::FermionState;

/// Selects fermion primitives and a coefficient type, without storing values.
///
/// `state` and `operator` require no trait import. No default
/// precision, numeric capability, or cloning capability is required here.
pub type Fermion<C> = syntaqs_core::system::ExpressionFactory<FermionState, FermionOperator, C>;

/// Common system, primitive, and model types.
pub mod prelude {
    pub use crate::{
        Fermion, models::FermionModels, operator::FermionOperator, state::FermionState,
    };
}
