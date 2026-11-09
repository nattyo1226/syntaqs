//! Exact collection retains core grading and needs neither Clone nor Dimension on primitives.

use num_complex::Complex;
use syntaqs_core::{
    Adjoint, HilbertSpace, HomogeneousParity, Identity, LinearCombination, LocalOperator, Parity,
    ParityOperator, Product, TensorKind, Trace,
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
