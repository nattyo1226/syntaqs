//! Occupation states, their inner products, and local operator actions.

use crate::{FermionOperator, coefficient::imaginary};
use core::ops::Neg;
use num_traits::{One, Zero};
use syntaqs_core::{
    Act, Dimension, DimensionOverflow, HomogeneousParity, ImaginaryUnit, InnerProduct,
    LinearCombination, Parity,
};

/// Single-mode occupation states, in the order `Empty, Occupied`.
/// Display writes `0` or `1`; state expressions add ket markers and indices.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FermionState {
    /// Zero occupation, with even parity.
    Empty,
    /// One occupied mode, with odd parity.
    Occupied,
}

impl core::fmt::Display for FermionState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Empty => "0",
            Self::Occupied => "1",
        })
    }
}

impl Dimension for FermionState {
    fn dim() -> Result<core::num::NonZeroUsize, DimensionOverflow> {
        Ok(core::num::NonZeroUsize::new(2).unwrap())
    }
}

impl HomogeneousParity for FermionState {
    fn parity(&self) -> Parity {
        match self {
            Self::Empty => Parity::Even,
            Self::Occupied => Parity::Odd,
        }
    }
}

impl<C: Zero + One> InnerProduct<C> for FermionState {
    fn inner_product(&self, rhs: &Self) -> C {
        if self == rhs { C::one() } else { C::zero() }
    }
}

impl<C> Act<C> for FermionState
where
    C: One + Neg<Output = C> + ImaginaryUnit,
{
    type Operator = FermionOperator;
    type Term = Self;

    fn act(&self, operator: &FermionOperator) -> LinearCombination<Self, C> {
        use FermionOperator::*;
        use FermionState::*;
        use LinearCombination as Lc;
        match (*operator, *self) {
            (I, state) => Lc::primitive(state),
            (Annihilate, Empty) | (Create, Occupied) | (Number, Empty) => Lc::zero(),
            (Annihilate, Occupied) => Lc::primitive(Empty),
            (Create, Empty) | (Number, Occupied) => Lc::primitive(Occupied),
            (Mx, Empty) => Lc::term(C::one(), Occupied),
            (Mx, Occupied) => Lc::term(C::one(), Empty),
            (My, Empty) => Lc::term(imaginary(C::one()), Occupied),
            (My, Occupied) => Lc::term(imaginary(-C::one()), Empty),
            (Mz, Empty) => Lc::term(C::one(), Empty),
            (Mz, Occupied) => Lc::term(-C::one(), Occupied),
        }
    }
}
