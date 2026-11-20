//! Ordered spin-1 state and dipole/quadrupole operator bases.

use crate::{
    SpinOneOperator as O, SpinOneState as S,
    coefficient::{half, imaginary},
};
use core::ops::{Add, Div, Neg};
use num_traits::One;
use syntaqs_core::{
    basis::Basis, coefficient::ImaginaryUnit,
    expression::parts::linear_combination::LinearCombination,
};

/// The orthonormal state basis in the order `Plus, Zero, Minus`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SzBasis;

/// The orthogonal Hermitian basis `I, Sx, Sy, Sz, Dxy, Q0, Qxy, Qyz, Qzx`.
///
/// Orthonormal under the dimension-normalized inner product `Tr(A† B)/3`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CartesianBasis;

/// The orthogonal basis `I, Sp, Sm, Sz, Q0, Q1p, Q1m, Q2p, Q2m`.
///
/// Dimension-normalized squared norms are 1 for I, Sz and Q0, and 2
/// for the six ladder elements. No spherical-tensor phase factors are applied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LadderBasis;

impl<C: One> Basis<S, C> for SzBasis {
    fn expand_indices(&self, primitive: &S) -> LinearCombination<usize, C> {
        LinearCombination::primitive(
            Basis::<_, C>::index(self, primitive).expect("basis primitive"),
        )
    }
    fn index(&self, primitive: &S) -> Option<usize> {
        Some(match primitive {
            S::Plus => 0,
            S::Zero => 1,
            S::Minus => 2,
        })
    }
    fn element(&self, index: usize) -> Option<S> {
        [S::Plus, S::Zero, S::Minus].get(index).copied()
    }
}

impl<C: One + Neg<Output = C> + ImaginaryUnit> Basis<O, C> for CartesianBasis {
    fn expand_indices(&self, primitive: &O) -> LinearCombination<usize, C> {
        use LinearCombination as Lc;
        use O::*;
        match primitive {
            Sp => Lc::primitive(1) + Lc::term(imaginary(C::one()), 2),
            Sm => Lc::primitive(1) + Lc::term(imaginary(-C::one()), 2),
            Q1p => Lc::primitive(8) + Lc::term(imaginary(C::one()), 7),
            Q1m => Lc::primitive(8) + Lc::term(imaginary(-C::one()), 7),
            Q2p => Lc::primitive(4) + Lc::term(imaginary(C::one()), 6),
            Q2m => Lc::primitive(4) + Lc::term(imaginary(-C::one()), 6),
            _ => Lc::primitive(Basis::<_, C>::index(self, primitive).expect("basis primitive")),
        }
    }
    fn index(&self, primitive: &O) -> Option<usize> {
        use O::*;
        match primitive {
            I => Some(0),
            Sx => Some(1),
            Sy => Some(2),
            Sz => Some(3),
            Dxy => Some(4),
            Q0 => Some(5),
            Qxy => Some(6),
            Qyz => Some(7),
            Qzx => Some(8),
            Sp | Sm | Q1p | Q1m | Q2p | Q2m => None,
        }
    }
    fn element(&self, index: usize) -> Option<O> {
        use O::*;
        [I, Sx, Sy, Sz, Dxy, Q0, Qxy, Qyz, Qzx].get(index).copied()
    }
}

impl<C> Basis<O, C> for LadderBasis
where
    C: One + Add<Output = C> + Div<Output = C> + Neg<Output = C> + ImaginaryUnit,
{
    fn expand_indices(&self, primitive: &O) -> LinearCombination<usize, C> {
        use LinearCombination as Lc;
        use O::*;
        match primitive {
            Sx => Lc::term(half(), 1) + Lc::term(half(), 2),
            Sy => Lc::term(imaginary(-half::<C>()), 1) + Lc::term(imaginary(half()), 2),
            Qzx => Lc::term(half(), 5) + Lc::term(half(), 6),
            Qyz => Lc::term(imaginary(-half::<C>()), 5) + Lc::term(imaginary(half()), 6),
            Dxy => Lc::term(half(), 7) + Lc::term(half(), 8),
            Qxy => Lc::term(imaginary(-half::<C>()), 7) + Lc::term(imaginary(half()), 8),
            _ => Lc::primitive(Basis::<_, C>::index(self, primitive).expect("basis primitive")),
        }
    }
    fn index(&self, primitive: &O) -> Option<usize> {
        use O::*;
        match primitive {
            I => Some(0),
            Sp => Some(1),
            Sm => Some(2),
            Sz => Some(3),
            Q0 => Some(4),
            Q1p => Some(5),
            Q1m => Some(6),
            Q2p => Some(7),
            Q2m => Some(8),
            Sx | Sy | Dxy | Qxy | Qyz | Qzx => None,
        }
    }
    fn element(&self, index: usize) -> Option<O> {
        use O::*;
        [I, Sp, Sm, Sz, Q0, Q1p, Q1m, Q2p, Q2m].get(index).copied()
    }
}
