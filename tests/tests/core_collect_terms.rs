//! Exact collection retains core grading and needs neither Clone nor Dimension on primitives.

use num_complex::Complex;
use syntaqs_core::{
    algebra::Adjoint, algebra::HomogeneousParity, algebra::Identity, algebra::Parity,
    algebra::ParityOperator, algebra::Product, algebra::Trace, expression::local::LocalOperator,
    expression::parts::linear_combination::LinearCombination, space::HilbertSpace,
    space::TensorKind,
};
use syntaqs_tests::graded::{self as reference, Op, Real, c, close};

// No Clone, Dimension, Basis, or state action implementation.
struct Primitive(Op);
impl PartialEq for Primitive {
    fn eq(&self, other: &Self) -> bool {
        match (self.0, other.0) {
            (Op::I, Op::I) | (Op::P, Op::P) => true,
            (Op::U(i, j), Op::U(k, l)) => i == k && j == l,
            _ => false,
        }
    }
}
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
impl ParityOperator for Primitive {
    fn parity_operator() -> Self {
        Self(Op::P)
    }
}
impl<R: Real> Product<Complex<R>> for Primitive {
    type Term = Self;
    fn product(&self, rhs: &Self) -> LinearCombination<Self, Complex<R>> {
        self.0.product(&rhs.0).map(Self)
    }
}
impl<R: Real> Adjoint<Complex<R>> for Primitive {
    type Term = Self;
    fn adjoint(&self) -> LinearCombination<Self, Complex<R>> {
        self.0.adjoint().map(Self)
    }
}
impl<R: Real> Trace<Complex<R>> for Primitive {
    fn trace(&self) -> Complex<R> {
        self.0.trace()
    }
}

fn collection<R: Real>() {
    type Local<R> = LocalOperator<Primitive, Complex<R>>;
    for kind in [TensorKind::Ordinary, TensorKind::Graded] {
        let a = HilbertSpace::factor(0u8);
        let b = HilbertSpace::factor(1u8);
        let space = HilbertSpace::tensor_all_with(kind, [a.clone(), b.clone()]).unwrap();
        let left = Local::<R>::primitive(Primitive(reference::A))
            .at(&b)
            .unwrap();
        let right = Local::<R>::primitive(Primitive(reference::CREATE))
            .at(&a)
            .unwrap();
        let product = (&left * &right).unwrap();
        let coefficient = c::<R>(0.3, 0.7);
        let op =
            (product.scaled(coefficient) + product.adjoint().scaled(coefficient.conj())).unwrap();
        let dense = reference::product(
            &reference::embedding::<R>(reference::A, 1, 2, kind),
            &reference::embedding(reference::CREATE, 0, 2, kind),
        );
        let expected: Vec<_> = dense
            .iter()
            .zip(reference::adjoint(&dense))
            .map(|(&a, b)| a * coefficient + b * coefficient.conj())
            .collect();
        let collected = op.collect_terms_in(&space).unwrap();
        assert!(collected.space().unwrap().is_compatible_with(&space));
        let again = collected.collect_terms_in(&space).unwrap();
        for p in reference::OPS {
            for q in reference::OPS {
                let probe = (Local::<R>::primitive(Primitive(p)).at(&a).unwrap()
                    * Local::<R>::primitive(Primitive(q)).at(&b).unwrap())
                .unwrap();
                let probe_matrix = reference::product(
                    &reference::embedding::<R>(p, 0, 2, kind),
                    &reference::embedding(q, 1, 2, kind),
                );
                let hs = reference::trace(&reference::product(
                    &reference::adjoint(&probe_matrix),
                    &expected,
                )) / c(4., 0.);
                close(probe.hs_inner_product_in(&collected, &space).unwrap(), hs);
                close(probe.hs_inner_product_in(&again, &space).unwrap(), hs);
            }
        }
        let cancelled = (&op + op.scaled(c(-1., 0.)))
            .unwrap()
            .collect_terms_in(&space)
            .unwrap();
        assert!(cancelled.space().unwrap().is_compatible_with(&space));
        assert_eq!(cancelled.hs_inner_product(&cancelled).unwrap(), c(0., 0.));
        let missing = HilbertSpace::factor(9u8);
        let zero = Local::<R>::primitive(Primitive(Op::I))
            .at(&missing)
            .unwrap()
            .scaled(c(0., 0.));
        assert!(zero.collect_terms_in(&space).is_err());
    }
    let p = Local::<R>::primitive(Primitive(reference::A));
    let tiny = c::<R>(R::TOL * 0.001, 0.);
    let op = Local::sum([p.clone(), p.scaled(c(-1., 0.)), p.scaled(tiny)]);
    let collected = op.collect_terms();
    close(collected.hs_inner_product(&p), tiny / c(2., 0.));
    assert_ne!(collected.hs_inner_product(&p), c(0., 0.));
}

#[test]
fn exact_collection_f32() {
    collection::<f32>();
}
#[test]
fn exact_collection_f64() {
    collection::<f64>();
}

#[test]
fn shared_graded_blocks_keep_the_sign_of_each_addend() {
    let factors = [7, 3, 5, 0].map(HilbertSpace::factor);
    let kind = TensorKind::Graded;
    let space = HilbertSpace::tensor_all_with(kind, factors.clone()).unwrap();
    let local = |p, i: usize| {
        LocalOperator::<_, Complex<f64>>::primitive(p)
            .at(&factors[i])
            .unwrap()
    };
    let dense = |p, i| reference::embedding::<f64>(p, i, 4, kind);
    let add = |a: Vec<_>, b: Vec<_>| a.into_iter().zip(b).map(|(x, y)| x + y).collect::<Vec<_>>();
    let odd = |i| (local(reference::A, i) + local(reference::CREATE, i)).unwrap();
    let odd_dense = |i| add(dense(reference::A, i), dense(reference::CREATE, i));
    let a = odd(1);
    let b = odd(2);
    let forward = (&a * &b).unwrap();
    let reverse = (&b * &a).unwrap();
    let cancelled = (&forward + &reverse)
        .unwrap()
        .coordinates_in(&reference::Parities, &space);
    assert!(cancelled.unwrap().is_empty());

    let left = odd(0);
    let right = odd(3);
    let forward_dense = reference::product(&odd_dense(1), &odd_dense(2));
    let reverse_dense = reference::product(&odd_dense(2), &odd_dense(1));
    let mixed_left = (&left + local(Op::P, 0)).unwrap();
    let mixed_right = (&right + local(Op::P, 3)).unwrap();
    let mixed_left_dense = add(odd_dense(0), dense(Op::P, 0));
    let mixed_right_dense = add(odd_dense(3), dense(Op::P, 3));
    // Moving homogeneous odd blocks, extracting to the right of a mixed
    // block, and preserving a product when both directions are blocked.
    for (lhs, middle, other_middle, rhs, expected) in [
        (
            &left,
            &forward,
            &reverse,
            &right,
            add(
                reference::product(&odd_dense(0), &forward_dense),
                reference::product(&reverse_dense, &odd_dense(3)),
            ),
        ),
        (
            &mixed_left,
            &a,
            &a,
            &local(Op::P, 3),
            add(
                reference::product(&mixed_left_dense, &odd_dense(1)),
                reference::product(&odd_dense(1), &dense(Op::P, 3)),
            ),
        ),
        (
            &mixed_left,
            &a,
            &a,
            &mixed_right,
            add(
                reference::product(&mixed_left_dense, &odd_dense(1)),
                reference::product(&odd_dense(1), &mixed_right_dense),
            ),
        ),
    ] {
        let sum = ((lhs * middle).unwrap() + (other_middle * rhs).unwrap()).unwrap();
        let collected = sum.collect_terms_in(&space).unwrap();
        for column in 0..16 {
            let input = reference::ket::<f64>(column, &factors, kind);
            let actual = reference::vector(&input.act(&collected).unwrap());
            let expected_column: Vec<_> =
                expected.chunks_exact(16).map(|row| row[column]).collect();
            reference::values(&actual, &expected_column);
        }
    }
}
