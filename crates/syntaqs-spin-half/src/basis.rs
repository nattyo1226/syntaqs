//! Ordered state, Cartesian-spin, and ladder-operator bases.

use core::ops::{Add, Div, Neg};

use num_traits::One;
use syntaqs_core::{
    basis::Basis, coefficient::ImaginaryUnit,
    expression::parts::linear_combination::LinearCombination,
};

use crate::{
    SpinHalfOperator, SpinHalfState,
    coefficient::{half, imaginary},
};

/// The orthonormal `Sz` eigenbasis in the order `Up, Down`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SzBasis;

/// The operator basis in the order `I, Sx, Sy, Sz`.
///
/// Its spin elements are Pauli matrices, orthonormal under `Tr(A†B)/2`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CartesianBasis;

/// The operator basis in the order `I, Sp, Sm, Sz`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LadderBasis;

impl<C: One> Basis<SpinHalfState, C> for SzBasis {
    fn expand_indices(&self, primitive: &SpinHalfState) -> LinearCombination<usize, C> {
        LinearCombination::primitive(
            Basis::<_, C>::index(self, primitive).expect("basis primitive"),
        )
    }

    fn index(&self, primitive: &SpinHalfState) -> Option<usize> {
        Some(match primitive {
            SpinHalfState::Up => 0,
            SpinHalfState::Down => 1,
        })
    }

    fn element(&self, index: usize) -> Option<SpinHalfState> {
        [SpinHalfState::Up, SpinHalfState::Down].get(index).copied()
    }
}

impl<C: One + Add<Output = C> + Div<Output = C> + Neg<Output = C> + ImaginaryUnit>
    Basis<SpinHalfOperator, C> for CartesianBasis
{
    fn expand_indices(&self, primitive: &SpinHalfOperator) -> LinearCombination<usize, C> {
        use SpinHalfOperator::*;
        match primitive {
            Sp => {
                LinearCombination::term(half(), 1) + LinearCombination::term(imaginary(half()), 2)
            }
            Sm => {
                LinearCombination::term(half(), 1)
                    + LinearCombination::term(imaginary(-half::<C>()), 2)
            }
            _ => LinearCombination::primitive(
                Basis::<_, C>::index(self, primitive).expect("basis primitive"),
            ),
        }
    }

    fn index(&self, primitive: &SpinHalfOperator) -> Option<usize> {
        use SpinHalfOperator::*;
        match primitive {
            I => Some(0),
            Sx => Some(1),
            Sy => Some(2),
            Sz => Some(3),
            Sp | Sm => None,
        }
    }

    fn element(&self, index: usize) -> Option<SpinHalfOperator> {
        use SpinHalfOperator::*;
        [I, Sx, Sy, Sz].get(index).copied()
    }
}

impl<C> Basis<SpinHalfOperator, C> for LadderBasis
where
    C: One + Neg<Output = C> + ImaginaryUnit,
{
    fn expand_indices(&self, primitive: &SpinHalfOperator) -> LinearCombination<usize, C> {
        use SpinHalfOperator::*;
        match primitive {
            Sx => LinearCombination::primitive(1) + LinearCombination::primitive(2),
            Sy => {
                LinearCombination::term(imaginary(-C::one()), 1)
                    + LinearCombination::term(imaginary(C::one()), 2)
            }
            _ => LinearCombination::primitive(
                Basis::<_, C>::index(self, primitive).expect("basis primitive"),
            ),
        }
    }

    fn index(&self, primitive: &SpinHalfOperator) -> Option<usize> {
        use SpinHalfOperator::*;
        match primitive {
            I => Some(0),
            Sp => Some(1),
            Sm => Some(2),
            Sz => Some(3),
            Sx | Sy => None,
        }
    }

    fn element(&self, index: usize) -> Option<SpinHalfOperator> {
        use SpinHalfOperator::*;
        [I, Sp, Sm, Sz].get(index).copied()
    }
}
