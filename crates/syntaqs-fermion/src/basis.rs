//! Complete ordered occupation, ladder, and Majorana bases.

use crate::{
    FermionOperator, FermionState,
    coefficient::{half, imaginary},
};
use core::ops::{Add, Div, Neg};
use num_traits::One;
use syntaqs_core::{
    basis::{Basis, BasisElements},
    coefficient::ImaginaryUnit,
    expression::parts::linear_combination::LinearCombination,
};

/// The orthonormal state basis `[Empty, Occupied]`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct OccupationBasis;

/// The operator basis `[I, Annihilate, Create, Number]`.
///
/// Complete, but not orthogonal: `Tr(I† Number) = 1`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LadderBasis;

/// The Hermite operator basis `[I, Mx, My, Mz]`.
///
/// Orthonormal under the dimension-normalized inner product `Tr(A† B)/2`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MajoranaBasis;

impl BasisElements<FermionState> for OccupationBasis {
    fn index(&self, primitive: &FermionState) -> Option<usize> {
        Some(match primitive {
            FermionState::Empty => 0,
            FermionState::Occupied => 1,
        })
    }

    fn element(&self, index: usize) -> Option<FermionState> {
        [FermionState::Empty, FermionState::Occupied]
            .get(index)
            .copied()
    }
}

impl<C: One> Basis<FermionState, C> for OccupationBasis {
    fn expand_indices(&self, primitive: &FermionState) -> LinearCombination<usize, C> {
        LinearCombination::primitive(self.index(primitive).expect("basis primitive"))
    }
}

impl BasisElements<FermionOperator> for LadderBasis {
    fn index(&self, primitive: &FermionOperator) -> Option<usize> {
        use FermionOperator::*;
        match primitive {
            I => Some(0),
            Annihilate => Some(1),
            Create => Some(2),
            Number => Some(3),
            _ => None,
        }
    }

    fn element(&self, index: usize) -> Option<FermionOperator> {
        use FermionOperator::*;
        [I, Annihilate, Create, Number].get(index).copied()
    }
}

impl<C> Basis<FermionOperator, C> for LadderBasis
where
    C: One + Add<Output = C> + Neg<Output = C> + ImaginaryUnit,
{
    fn expand_indices(&self, primitive: &FermionOperator) -> LinearCombination<usize, C> {
        use FermionOperator::*;
        use LinearCombination as Lc;
        match primitive {
            Mx => Lc::primitive(1) + Lc::primitive(2),
            My => Lc::term(imaginary(-C::one()), 1) + Lc::term(imaginary(C::one()), 2),
            Mz => Lc::primitive(0) + Lc::term(-(C::one() + C::one()), 3),
            _ => Lc::primitive(self.index(primitive).expect("basis primitive")),
        }
    }
}

impl BasisElements<FermionOperator> for MajoranaBasis {
    fn index(&self, primitive: &FermionOperator) -> Option<usize> {
        use FermionOperator::*;
        match primitive {
            I => Some(0),
            Mx => Some(1),
            My => Some(2),
            Mz => Some(3),
            _ => None,
        }
    }

    fn element(&self, index: usize) -> Option<FermionOperator> {
        use FermionOperator::*;
        [I, Mx, My, Mz].get(index).copied()
    }
}

impl<C> Basis<FermionOperator, C> for MajoranaBasis
where
    C: One + Add<Output = C> + Div<Output = C> + Neg<Output = C> + ImaginaryUnit,
{
    fn expand_indices(&self, primitive: &FermionOperator) -> LinearCombination<usize, C> {
        use FermionOperator::*;
        use LinearCombination as Lc;
        match primitive {
            Annihilate => Lc::term(half(), 1) + Lc::term(imaginary(half()), 2),
            Create => Lc::term(half(), 1) + Lc::term(imaginary(-half::<C>()), 2),
            Number => Lc::term(half(), 0) + Lc::term(-half::<C>(), 3),
            _ => Lc::primitive(self.index(primitive).expect("basis primitive")),
        }
    }
}
