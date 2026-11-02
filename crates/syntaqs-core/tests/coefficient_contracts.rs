//! Coefficient capabilities are independent of storage and of each other.
use core::ops::{Add, Mul};
use num_complex::Complex;
use num_traits::{One, Zero};
use syntaqs_core::{
    Act, Adjoint, Conjugate, Divide, HilbertSpace, InnerProduct, LinearCombination, LocalOperator,
    LocalState,
};

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

#[test]
fn opaque_whole_coefficients_move_through_map_and_shared_local_and_composite_syntax() {
    struct Opaque(u8);
    let terms = LinearCombination::term(Opaque(7), Primitive).map(|_| 9);
    let term = terms.into_iter().next().unwrap();
    assert_eq!((term.coefficient.0, term.operand), (7, 9));
    let state = LocalState::<Primitive, Primitive, Opaque>::primitive(Primitive).scaled(Opaque(2));
    let operator = LocalOperator::primitive(Primitive).scaled(Opaque(3));
    let _ = state.clone().act(&operator.adjoint());
    let space = HilbertSpace::factor(0u8);
    let state = state.at(&space).unwrap();
    let operator = operator.at(&space).unwrap();
    let _ = state.clone().scaled(Opaque(4)).act(&operator).unwrap();
}

// No Num, Float, Copy, PartialEq, Neg, Div or RealPart. Real-part extraction is
// deliberately absent: scalar quotients do not need a real projection.
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
fn normalized_hs_needs_no_dimension_float_cast_or_general_div_trait() {
    use syntaqs_core::{Identity, Product, Trace};
    struct IdentityOnly;
    impl Identity for IdentityOnly {
        fn identity() -> Self {
            Self
        }
    }
    impl Adjoint<Weight> for IdentityOnly {
        type Term = Self;
        fn adjoint(&self) -> LinearCombination<Self, Weight> {
            LinearCombination::primitive(Self)
        }
    }
    impl Product<Weight> for IdentityOnly {
        type Term = Self;
        fn product(&self, _: &Self) -> LinearCombination<Self, Weight> {
            LinearCombination::primitive(Self)
        }
    }
    impl Trace<Weight> for IdentityOnly {
        fn trace(&self) -> Weight {
            Weight(3.0)
        }
    }
    impl syntaqs_core::HomogeneousParity for IdentityOnly {
        fn parity(&self) -> syntaqs_core::Parity {
            syntaqs_core::Parity::Even
        }
    }
    impl syntaqs_core::ParityOperator for IdentityOnly {
        fn parity_operator() -> Self {
            Self
        }
    }
    let local = LocalOperator::<IdentityOnly, Weight>::identity().scaled(Weight(4.0));
    assert_eq!(local.trace().0, 12.0);
    assert_eq!(local.hs_inner_product(&local).0, 16.0);
    let space = HilbertSpace::tensor_all((0..80).map(HilbertSpace::factor)).unwrap();
    let operator = local
        .at(space.leaves().next().unwrap())
        .unwrap()
        .bind(&space)
        .unwrap();
    assert_eq!(operator.hs_inner_product(&operator).unwrap().0, 16.0);
}

#[test]
fn expectation_quotients_require_neither_real_part_nor_float_or_general_comparison() {
    let state =
        LocalState::<Primitive, Primitive, Weight>::primitive(Primitive).scaled(Weight(2.0));
    let op = LocalOperator::primitive(Primitive).scaled(Weight(3.0));
    assert_eq!(
        state.expectation(&op).divide(state.inner_product(&state)).0,
        3.0
    );
    let space = HilbertSpace::factor(0u8);
    let state = state.at(&space).unwrap();
    let op = op.at(&space).unwrap();
    assert_eq!(
        state
            .expectation(&op)
            .unwrap()
            .divide(state.inner_product(&state).unwrap())
            .0,
        3.0
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
                let value = state.expectation(&op).divide(state.inner_product(&state));
                assert!((value.re - 1.0).abs() < $tol && value.im == 0.0);
                let space = HilbertSpace::factor(0u8);
                let state = state.at(&space).unwrap();
                let value = state
                    .expectation(&op.at(&space).unwrap())
                    .unwrap()
                    .divide(state.inner_product(&state).unwrap());
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

macro_rules! scaled_complex_division {
    ($name:ident, $real:ty, $large:expr, $small:expr, $tol:expr) => {
        #[test]
        fn $name() {
            type C = Complex<$real>;
            let close = |actual: C, expected: C| {
                assert!(
                    (actual.re - expected.re).abs() <= $tol * expected.re.abs().max(1.0),
                    "{actual:?} != {expected:?}"
                );
                assert!(
                    (actual.im - expected.im).abs() <= $tol * expected.im.abs().max(1.0),
                    "{actual:?} != {expected:?}"
                );
            };
            for scale in [
                $large,
                $small,
                <$real>::MAX,
                <$real>::MIN_POSITIVE,
                <$real>::from_bits(1),
            ] {
                for sign in [-1.0, 1.0] {
                    let divisor = C::new(scale, sign * scale);
                    close(divisor.divide(divisor), C::new(1.0, 0.0));
                    close(
                        C::new(-sign * scale, scale).divide(divisor),
                        C::new(0.0, 1.0),
                    );
                }
            }
            close(
                C::new(<$real>::MAX, <$real>::MAX).divide(C::new(1.0, 1.0)),
                C::new(<$real>::MAX, 0.0),
            );
            close(
                C::new(<$real>::MAX, 0.0).divide(C::new(0.75, 0.75)),
                C::new(<$real>::MAX / 1.5, -(<$real>::MAX / 1.5)),
            );
            let tiny = <$real>::from_bits(1);
            assert_eq!(
                C::new(tiny, tiny).divide(C::new(1.0, 1.0)),
                C::new(tiny, 0.0)
            );
            close(
                C::new(tiny * 3.0, 0.0).divide(C::new(tiny * 2.0, tiny)),
                C::new(1.2, -0.6),
            );
            for divisor in [
                C::new(1.0, 2.0),
                C::new(2.0, 1.0),
                C::new(-2.0, 1.0),
                C::new(0.0, -2.0),
            ] {
                let expected = C::new(3.0, 4.0) / divisor;
                close(C::new(3.0, 4.0).divide(divisor), expected);
            }
        }
    };
}

scaled_complex_division!(complex_division_scales_f32, f32, 1e20_f32, 1e-20_f32, 2e-6);
scaled_complex_division!(
    complex_division_scales_f64,
    f64,
    1e200_f64,
    1e-200_f64,
    2e-15
);

#[test]
fn complex_division_keeps_truncating_integer_arithmetic() {
    assert_eq!(
        Complex::new(3i64, 4).divide(Complex::new(1, 2)),
        Complex::new(2, 0),
    );
}

#[test]
fn complex_division_needs_no_float_copy_one_or_negation() {
    #[derive(Clone, PartialEq, PartialOrd)]
    struct Ordered(f64);
    macro_rules! arithmetic {
        ($trait:ident, $method:ident, $op:tt) => {
            impl core::ops::$trait for Ordered {
                type Output = Self;
                fn $method(self, rhs: Self) -> Self { Self(self.0 $op rhs.0) }
            }
        };
    }
    arithmetic!(Add, add, +);
    arithmetic!(Sub, sub, -);
    arithmetic!(Mul, mul, *);
    arithmetic!(Div, div, /);
    impl Zero for Ordered {
        fn zero() -> Self {
            Self(0.0)
        }
        fn is_zero(&self) -> bool {
            self.0 == 0.0
        }
    }
    let quotient =
        Complex::new(Ordered(3.0), Ordered(4.0)).divide(Complex::new(Ordered(1.0), Ordered(2.0)));
    assert!((quotient.re.0 - 2.2).abs() < 1e-15);
    assert!((quotient.im.0 + 0.4).abs() < 1e-15);
}

#[test]
fn every_scaled_entry_converts_a_non_clone_source_exactly_once() {
    use core::cell::Cell;
    type C = Complex<f64>;
    struct Input<'a>(&'a Cell<usize>);
    // Deliberately implement Into directly: the API must not require From.
    #[allow(clippy::from_over_into)]
    impl Into<C> for Input<'_> {
        fn into(self) -> C {
            self.0.set(self.0.get() + 1);
            C::new(2.0, 0.0)
        }
    }
    let calls = Cell::new(0);
    let s = LocalState::<Primitive, Primitive, C>::primitive(Primitive);
    let op = LocalOperator::<Primitive, C>::primitive(Primitive);
    let h = HilbertSpace::factor(0u8);
    let expected = C::new(4.0, 0.0);
    let scaled = s.scaled(Input(&calls));
    assert_eq!(calls.get(), 1);
    assert_eq!(scaled.inner_product(&scaled), expected);
    let scaled_op = op.scaled(Input(&calls));
    assert_eq!(calls.get(), 2);
    assert_eq!(s.expectation(&scaled_op), C::new(2.0, 0.0));
    let placed = s.at(&h).unwrap().scaled(Input(&calls));
    assert_eq!(calls.get(), 3);
    assert_eq!(placed.inner_product(&placed).unwrap(), expected);
    let placed_op = op.at(&h).unwrap().scaled(Input(&calls));
    assert_eq!(calls.get(), 4);
    assert_eq!(
        s.at(&h).unwrap().expectation(&placed_op).unwrap(),
        C::new(2.0, 0.0)
    );
    // Conversion occurs once per call, including empty and many-term inputs.
    for count in 0..=3 {
        let before = calls.get();
        let terms = LinearCombination::<_, C>::from_terms((0..count).map(|i| syntaqs_core::Term {
            coefficient: C::new(f64::from(i), 0.0),
            operand: i,
        }));
        let scaled = terms.clone().scaled(Input(&calls));
        assert_eq!(calls.get(), before + 1);
        assert_eq!(scaled, terms.scaled(C::new(2.0, 0.0)));
    }
    // Numeric operations and Clone remain unnecessary for expression storage.
    struct Opaque;
    struct OpaqueInput;
    impl From<OpaqueInput> for Opaque {
        fn from(_: OpaqueInput) -> Self {
            Self
        }
    }
    let state =
        LocalState::<Primitive, Primitive, Opaque>::primitive(Primitive).scaled(OpaqueInput);
    let op = LocalOperator::<Primitive, Opaque>::primitive(Primitive).scaled(OpaqueInput);
    let _ = state.scaled(Opaque).act(&op);
    let _ = state.at(&h).unwrap().scaled(OpaqueInput);
    let _ = op.at(&h).unwrap().scaled(OpaqueInput);
}
