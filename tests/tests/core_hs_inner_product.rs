//! Normalized HS contractions preserve ambient-space and grading semantics.
use syntaqs_tests::graded as support;

use num_complex::Complex;
use support::{Op, Real, c, close};
use syntaqs_core::{
    algebra::Adjoint, algebra::HomogeneousParity, algebra::Identity, algebra::Parity,
    algebra::Product, algebra::Trace, expression::composite::Operator,
    expression::local::LocalOperator, expression::parts::linear_combination::LinearCombination,
    space::DimensionOverflow, space::HilbertSpace, space::TensorKind,
};

// No Dimension, state type, or basis implementation: HS gets d from Tr(I).
#[derive(Clone, Copy, PartialEq)]
struct Primitive(Op);
impl Identity for Primitive {
    fn identity() -> Self {
        Self(Op::I)
    }
}
impl HomogeneousParity for Primitive {
    fn parity(&self) -> Parity {
        self.0.parity()
    }
}
impl<T: Real> Product<Complex<T>> for Primitive {
    type Term = Self;
    fn product(&self, rhs: &Self) -> LinearCombination<Self, Complex<T>> {
        self.0.product(&rhs.0).map(Self)
    }
}
impl<T: Real> Adjoint<Complex<T>> for Primitive {
    type Term = Self;
    fn adjoint(&self) -> LinearCombination<Self, Complex<T>> {
        self.0.adjoint().map(Self)
    }
}
impl<T: Real> Trace<Complex<T>> for Primitive {
    fn trace(&self) -> Complex<T> {
        self.0.trace()
    }
}

fn contractions<T: Real>() {
    type Local<T> = LocalOperator<Primitive, Complex<T>>;
    for a in support::OPS {
        for b in support::OPS {
            let lhs = Local::<T>::primitive(Primitive(a)).scaled(c(0.3, -0.7));
            let rhs = Local::<T>::primitive(Primitive(b)).scaled(c(-0.2, 0.4));
            let reference = support::trace(&support::product(
                &support::adjoint(&support::local::<T>(a)),
                &support::local(b),
            )) * c::<T>(0.3, 0.7)
                * c::<T>(-0.2, 0.4)
                / c(2.0, 0.0);
            close(lhs.hs_inner_product(&rhs), reference);
            for kind in [TensorKind::Ordinary, TensorKind::Graded] {
                let factors = [HilbertSpace::factor(0u8), HilbertSpace::factor(1)];
                let space = HilbertSpace::tensor_all_with(kind, factors.iter().cloned()).unwrap();
                let left = lhs.at(&factors[1]).unwrap().bind(&space).unwrap();
                let right = rhs.at(&factors[1]).unwrap().bind(&space).unwrap();
                close(left.hs_inner_product(&right).unwrap(), reference);
                // Different placements exercise nontrivial parity strings.
                let other = rhs.at(&factors[0]).unwrap();
                let dense = support::trace(&support::product(
                    &support::adjoint(&support::embedding::<T>(a, 1, 2, kind)),
                    &support::embedding(b, 0, 2, kind),
                )) * c::<T>(0.3, 0.7)
                    * c::<T>(-0.2, 0.4)
                    / c(4.0, 0.0);
                close(left.hs_inner_product_in(&other, &space).unwrap(), dense);
            }
        }
    }
}

#[test]
fn local_and_graded_contractions_match_independent_matrices() {
    contractions::<f32>();
    contractions::<f64>();
}

fn interleaved_blocks<T: Real>() {
    type Local<T> = LocalOperator<Primitive, Complex<T>>;
    let factors = [5, 0, 3, 1].map(HilbertSpace::factor);
    for kind in [TensorKind::Ordinary, TensorKind::Graded] {
        let space = HilbertSpace::tensor_all_with(kind, factors.iter().cloned()).unwrap();
        let placed = |p, i: usize| Local::<T>::primitive(Primitive(p)).at(&factors[i]).unwrap();
        let embed = |p, i| support::embedding::<T>(p, i, 4, kind);
        let block = |i, j, odd, weight| {
            let even = Op::U(0, 0);
            // Each block has both an odd and an even summand and its support
            // interleaves the other block's support.
            let p = (&placed(even, i) * &placed(even, j)).unwrap();
            let q = (&placed(Op::P, i) * &placed(odd, j)).unwrap();
            let expression = (p + q.scaled(weight)).unwrap();
            let matrix: Vec<_> = support::product(&embed(even, i), &embed(even, j))
                .into_iter()
                .zip(support::product(&embed(Op::P, i), &embed(odd, j)))
                .map(|(p, q)| p + q * weight)
                .collect();
            (expression, matrix)
        };
        let (a, a_dense) = block(0, 2, support::A, c(0.5, -0.25));
        let (b, b_dense) = block(1, 3, support::CREATE, c(-0.5, 0.75));
        let lhs_scale = c::<T>(0.75, -0.5);
        let rhs_scale = c::<T>(-0.25, 0.5);
        let lhs = (&a * &b).unwrap().scaled(lhs_scale);
        let left = support::product(&a_dense, &b_dense);
        for reverse in [false, true] {
            let (rhs, right) = if reverse {
                // Odd block exchanges change signs in this product.
                ((&b * &a).unwrap(), support::product(&b_dense, &a_dense))
            } else {
                // Separately built, ordered blocks permit symmetry locally.
                let (other_a, _) = block(0, 2, support::A, c(0.5, -0.25));
                let (other_b, _) = block(1, 3, support::CREATE, c(-0.5, 0.75));
                ((&other_a * &other_b).unwrap(), left.clone())
            };
            let expected = support::trace(&support::product(&support::adjoint(&left), &right))
                * lhs_scale.conj()
                * rhs_scale
                / c(16.0, 0.0);
            let rhs = rhs.scaled(rhs_scale);
            close(lhs.hs_inner_product_in(&rhs, &space).unwrap(), expected);
            close(
                rhs.hs_inner_product_in(&lhs, &space).unwrap(),
                expected.conj(),
            );
            close(
                lhs.trace_in(&space).unwrap(),
                support::trace(&left) * lhs_scale,
            );
            close(
                rhs.trace_in(&space).unwrap(),
                support::trace(&right) * rhs_scale,
            );
        }
    }
}

#[test]
fn interleaved_mixed_parity_blocks_match_dense_contractions() {
    interleaved_blocks::<f32>();
    interleaved_blocks::<f64>();
}

#[test]
fn reversed_odd_blocks_stay_factored() {
    type Local = LocalOperator<Primitive, Complex<f64>>;
    let factors: Vec<_> = (0..31u8).map(HilbertSpace::factor).collect();
    let local =
        Local::primitive(Primitive(support::A)) + Local::primitive(Primitive(support::CREATE));
    let forward = Operator::try_product(factors.iter().map(|factor| local.at(factor))).unwrap();
    let reverse =
        Operator::try_product(factors.iter().rev().map(|factor| local.at(factor))).unwrap();
    let ordinary = HilbertSpace::tensor_all(factors.iter().cloned()).unwrap();
    let graded = HilbertSpace::tensor_all_g(factors.iter().cloned()).unwrap();
    close(
        forward.hs_inner_product_in(&reverse, &ordinary).unwrap(),
        c(1.0, 0.0),
    );
    // Each block is odd and reversing 31 factors exchanges 465 pairs.
    close(
        forward.hs_inner_product_in(&reverse, &graded).unwrap(),
        c(-1.0, 0.0),
    );
}

#[test]
fn trace_keeps_the_exchange_sign_of_even_local_products() {
    type Local = LocalOperator<Primitive, Complex<f64>>;
    let factors = [HilbertSpace::factor(9u8), HilbertSpace::factor(4)];
    for kind in [TensorKind::Ordinary, TensorKind::Graded] {
        let space = HilbertSpace::tensor_all_with(kind, factors.iter().cloned()).unwrap();
        let placements = [
            (support::A, 1),
            (support::A, 0),
            (support::CREATE, 1),
            (support::CREATE, 0),
        ];
        let expression = Operator::try_product(
            placements.map(|(p, i)| Local::primitive(Primitive(p)).at(&factors[i])),
        )
        .unwrap();
        let mut matrices = placements
            .into_iter()
            .map(|(p, i)| support::embedding::<f64>(p, i, 2, kind));
        let first = matrices.next().unwrap();
        let expected = support::trace(&matrices.fold(first, |a, b| support::product(&a, &b)));
        assert_eq!(
            expected,
            c(
                if kind == TensorKind::Graded {
                    -1.0
                } else {
                    1.0
                },
                0.0
            )
        );
        close(expression.trace_in(&space).unwrap(), expected);
    }
}

#[test]
fn explicit_space_overrides_bindings_and_keeps_spectators_normalized() {
    type Local = LocalOperator<Primitive, Complex<f64>>;
    let factors = [
        HilbertSpace::factor(0u8),
        HilbertSpace::factor(1),
        HilbertSpace::factor(2),
    ];
    for kind in [TensorKind::Ordinary, TensorKind::Graded] {
        let old = HilbertSpace::tensor_all_with(kind, factors[..2].iter().cloned()).unwrap();
        let full = HilbertSpace::tensor_all_with(kind, factors.iter().cloned()).unwrap();
        let op = Local::primitive(Primitive(support::A))
            .at(&factors[1])
            .unwrap()
            .bind(&old)
            .unwrap();
        close(op.hs_inner_product(&op).unwrap(), c(0.5, 0.0));
        close(op.hs_inner_product_in(&op, &full).unwrap(), c(0.5, 0.0));
        let number = Local::primitive(Primitive(Op::U(1, 1)))
            .at(&factors[1])
            .unwrap();
        close(number.trace_in(&old).unwrap(), c(2.0, 0.0));
        close(number.trace_in(&full).unwrap(), c(4.0, 0.0));
    }
}

#[test]
fn scalar_unit_spaces_and_dimensions_beyond_usize_and_float_ranges() {
    type Op = Operator<usize, Primitive, Complex<f64>>;
    for kind in [TensorKind::Ordinary, TensorKind::Graded] {
        let empty = HilbertSpace::<usize>::tensor_all_with(kind, []).unwrap();
        let scalar = Op::scalar(c(3.0, 4.0)).bind(&empty).unwrap();
        close(scalar.hs_inner_product(&scalar).unwrap(), c(25.0, 0.0));
        close(scalar.trace().unwrap(), c(3.0, 4.0));
        let large =
            HilbertSpace::tensor_all_with(kind, (0..2048).map(HilbertSpace::factor)).unwrap();
        assert_eq!(large.dim::<support::Ket>(), Err(DimensionOverflow));
        let identity = Op::identity().bind(&large).unwrap();
        close(identity.hs_inner_product(&identity).unwrap(), c(1.0, 0.0));
        let zero = Op::zero().bind(&large).unwrap();
        close(identity.hs_inner_product(&zero).unwrap(), c(0.0, 0.0));
    }
}

macro_rules! scaled_trace_regression {
    ($name:ident, $real:ty) => {
        #[test]
        fn $name() {
            type C = Complex<$real>;
            type Local = LocalOperator<Primitive, C>;
            let identity = Local::identity();
            for scale in [<$real>::MAX * 0.75, <$real>::from_bits(1)] {
                for coefficient in [C::new(scale, 0.0), C::new(0.0, scale)] {
                    let scaled = identity.scaled(coefficient);
                    assert_eq!(scaled.hs_inner_product(&identity), coefficient.conj());
                    assert_eq!(identity.hs_inner_product(&scaled), coefficient);
                    for kind in [TensorKind::Ordinary, TensorKind::Graded] {
                        let factors = [HilbertSpace::factor(0u8), HilbertSpace::factor(1)];
                        let space = HilbertSpace::tensor_all_with(kind, factors.clone()).unwrap();
                        let placed_identity =
                            identity.at(&factors[1]).unwrap().bind(&space).unwrap();
                        let local_scale = scaled.at(&factors[1]).unwrap().bind(&space).unwrap();
                        let composite_scale = placed_identity.scaled(coefficient);
                        for operator in [local_scale, composite_scale] {
                            assert_eq!(
                                operator.hs_inner_product(&placed_identity).unwrap(),
                                coefficient.conj(),
                            );
                            assert_eq!(
                                placed_identity.hs_inner_product(&operator).unwrap(),
                                coefficient,
                            );
                            assert_eq!(
                                operator
                                    .hs_inner_product_in(&placed_identity, &space)
                                    .unwrap(),
                                coefficient.conj(),
                            );
                        }
                    }
                }
            }
        }
    };
}

scaled_trace_regression!(normalized_local_traces_preserve_extreme_scales_f32, f32);
scaled_trace_regression!(normalized_local_traces_preserve_extreme_scales_f64, f64);
