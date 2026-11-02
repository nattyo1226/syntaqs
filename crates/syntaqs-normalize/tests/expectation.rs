//! Checked expectations use only contraction and scalar division.
use core::ops::{Add, Mul};
use num_complex::Complex;
use num_traits::{One, Zero};
use syntaqs_core::{
    Act, Adjoint, Conjugate, Divide, HilbertSpace, InnerProduct, LinearCombination, LocalOperator,
    LocalState,
};

use syntaqs_normalize::{NormalizationError, NormalizedExpectation, NormalizedExpectationIn};

struct Primitive;
impl syntaqs_core::HomogeneousParity for Primitive {
    fn parity(&self) -> syntaqs_core::Parity {
        syntaqs_core::Parity::Even
    }
}
impl syntaqs_core::ParityOperator for Primitive {
    fn parity_operator() -> Self {
        Self
    }
}

impl<C: One> Act<C> for Primitive {
    type Operator = Self;
    type Term = Self;
    fn act(&self, _: &Self) -> LinearCombination<Self, C> {
        LinearCombination::primitive(Self)
    }
}
impl<C: One> Adjoint<C> for Primitive {
    type Term = Self;
    fn adjoint(&self) -> LinearCombination<Self, C> {
        LinearCombination::primitive(Self)
    }
}
impl<C: One> InnerProduct<C> for Primitive {
    fn inner_product(&self, _: &Self) -> C {
        C::one()
    }
}

// No Num, Float, Copy, PartialEq, Neg, Div or RealPart. Real-part extraction is
// deliberately absent: normalized expectations divide the whole coefficient.
#[derive(Clone)]
struct Weight(f64);
impl Add for Weight {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}
impl Mul for Weight {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}
impl Zero for Weight {
    fn zero() -> Self {
        Self(0.0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
}
impl One for Weight {
    fn one() -> Self {
        Self(1.0)
    }
}
impl Conjugate for Weight {
    type Output = Self;
    fn conjugate(&self) -> Self {
        self.clone()
    }
}
impl Divide for Weight {
    type Output = Self;
    fn divide(self, rhs: Self) -> Self {
        Self(self.0 / rhs.0)
    }
}

#[test]
fn normalized_expectations_require_neither_real_part_nor_float_or_general_comparison() {
    let state =
        LocalState::<Primitive, Primitive, Weight>::primitive(Primitive).scaled(Weight(2.0));
    let op = LocalOperator::primitive(Primitive).scaled(Weight(3.0));
    assert_eq!(state.normalized_expectation(&op).unwrap().0, 3.0);
    let zero = LocalState::<Primitive, Primitive, Weight>::zero();
    assert_eq!(
        zero.normalized_expectation(&op).err(),
        Some(NormalizationError::ZeroNorm)
    );
    let space = HilbertSpace::factor(0u8);
    let state = state.at(&space).unwrap();
    let op = op.at(&space).unwrap();
    assert_eq!(state.normalized_expectation(&op).unwrap().0, 3.0);
    assert_eq!(state.normalized_expectation_in(&op, &space).unwrap().0, 3.0);
    assert_eq!(
        zero.at(&space).unwrap().normalized_expectation(&op).err(),
        Some(NormalizationError::ZeroNorm)
    );
}

macro_rules! complex_regressions {
    ($name:ident, $real:ty, $large:expr, $small:expr, $tol:expr) => {
        #[test]
        fn $name() {
            // These self-inner-products are representable, but squaring them
            // during complex division would overflow or underflow.
            for scale in [$large, $small] {
                let state =
                    LocalState::<Primitive, Primitive, Complex<$real>>::primitive(Primitive)
                        .scaled(Complex::new(scale, 0.0));
                let op = LocalOperator::primitive(Primitive);
                let value = state.normalized_expectation(&op).unwrap();
                assert!((value.re - 1.0).abs() < $tol && value.im == 0.0);
                let space = HilbertSpace::factor(0u8);
                let state = state.at(&space).unwrap();
                let value = state
                    .normalized_expectation(&op.at(&space).unwrap())
                    .unwrap();
                assert!((value.re - 1.0).abs() < $tol && value.im == 0.0);
            }
            let quotient = Complex::<$real>::new(3.0, 4.0).divide(Complex::new(1.0, 2.0));
            assert!((quotient.re - 2.2).abs() < $tol);
            assert!((quotient.im + 0.4).abs() < $tol);
        }
    };
}
complex_regressions!(
    complex32_division_preserves_extreme_real_denominators,
    f32,
    1e15_f32,
    1e-15_f32,
    1e-5
);
complex_regressions!(
    complex64_division_preserves_extreme_real_denominators,
    f64,
    1e100_f64,
    1e-100_f64,
    1e-12
);

#[test]
fn large_superposition_with_a_roundoff_imaginary_norm_has_finite_expectations() {
    type C = Complex<f64>;
    let state = LocalState::sum([(5.0, 7.0), (5.0, 2.0), (7.0, 1.0)].map(|(re, im)| {
        LocalState::<Primitive, Primitive, C>::primitive(Primitive)
            .scaled(C::new(re * 1e100, im * 1e100))
    }));
    let norm = state.inner_product(&state);
    assert!(norm.re.is_finite() && norm.re > 0.0);
    assert_ne!(norm.im, 0.0); // Exercise complex division, not its real shortcut.
    let op = LocalOperator::primitive(Primitive);
    let close = |value: C| {
        assert!(
            (value.re - 1.0).abs() < 1e-12 && value.im.abs() < 1e-12,
            "{value:?}"
        );
    };
    close(state.normalized_expectation(&op).unwrap());
    let space = HilbertSpace::factor(0u8);
    let state = state.at(&space).unwrap();
    let op = op.at(&space).unwrap();
    close(state.normalized_expectation(&op).unwrap());
    close(state.normalized_expectation_in(&op, &space).unwrap());
}
