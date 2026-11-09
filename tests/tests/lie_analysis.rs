//! End-to-end Lie analysis using public APIs and independent small matrices.
#![cfg(feature = "lie")]

extern crate blas_provider as _;

use num_complex::Complex;
use syntaqs_core::{
    Adjoint, HilbertSpace, HomogeneousParity, Identity, LinearCombination, LocalOperator, Operator,
    Parity, ParityOperator, Product, Trace,
};
use syntaqs_lie::{
    DecompositionOptions, FamilyExpression, NumericalOptions, OperatorFamily, lie_bracket,
};
use syntaqs_tests::{graded, three_level};

// These adapters deliberately have no Clone, Dimension, Basis, state, or action
// implementations. They forward only local operator algebra capabilities.
#[derive(PartialEq)]
struct ThreePrimitive(three_level::Operator);
impl Identity for ThreePrimitive {
    fn identity() -> Self {
        Self(three_level::Operator::Identity)
    }
}
impl<R: three_level::Real> Product<Complex<R>> for ThreePrimitive {
    type Term = Self;
    fn product(&self, rhs: &Self) -> LinearCombination<Self, Complex<R>> {
        self.0.product(&rhs.0).map(Self)
    }
}
impl<R: three_level::Real> Adjoint<Complex<R>> for ThreePrimitive {
    type Term = Self;
    fn adjoint(&self) -> LinearCombination<Self, Complex<R>> {
        self.0.adjoint().map(Self)
    }
}
impl<R: three_level::Real> Trace<Complex<R>> for ThreePrimitive {
    fn trace(&self) -> Complex<R> {
        self.0.trace()
    }
}

struct GradedPrimitive(graded::Op);
impl PartialEq for GradedPrimitive {
    fn eq(&self, rhs: &Self) -> bool {
        match (self.0, rhs.0) {
            (graded::Op::I, graded::Op::I) | (graded::Op::P, graded::Op::P) => true,
            (graded::Op::U(i, j), graded::Op::U(k, l)) => i == k && j == l,
            _ => false,
        }
    }
}
impl Identity for GradedPrimitive {
    fn identity() -> Self {
        Self(graded::Op::I)
    }
}
impl HomogeneousParity for GradedPrimitive {
    fn parity(&self) -> Parity {
        self.0.parity()
    }
}
impl ParityOperator for GradedPrimitive {
    fn parity_operator() -> Self {
        Self(graded::Op::P)
    }
}
impl<R: graded::Real> Product<Complex<R>> for GradedPrimitive {
    type Term = Self;
    fn product(&self, rhs: &Self) -> LinearCombination<Self, Complex<R>> {
        self.0.product(&rhs.0).map(Self)
    }
}
impl<R: graded::Real> Adjoint<Complex<R>> for GradedPrimitive {
    type Term = Self;
    fn adjoint(&self) -> LinearCombination<Self, Complex<R>> {
        self.0.adjoint().map(Self)
    }
}
impl<R: graded::Real> Trace<Complex<R>> for GradedPrimitive {
    fn trace(&self) -> Complex<R> {
        self.0.trace()
    }
}

#[derive(PartialEq, Eq)]
struct Factor(u8);

macro_rules! precision_tests {
    ($module:ident,$real:ty) => {
        mod $module {
            use super::*;
            type R = $real;
            type C = Complex<R>;
            fn close(a: R, b: R) {
                assert!(
                    NumericalOptions::<R>::default()
                        .residual
                        .check((a - b).abs(), b.abs())
                        .unwrap()
                        .passed(),
                    "{a} != {b}"
                );
            }
            fn cclose(a: C, b: C) {
                close(a.re, b.re);
                close(a.im, b.im);
            }

            // Ordinary state-space matrices assembled independently of Product,
            // Trace, core evaluation, and syntaqs-export.
            fn matrices(d: usize) -> Vec<Vec<C>> {
                let mut result = vec![vec![C::new(0., 0.); d * d]; 4];
                result[0][1] = C::new(1., 0.);
                result[0][d] = C::new(1., 0.);
                result[1][1] = C::new(0., -1.);
                result[1][d] = C::new(0., 1.);
                result[2][0] = C::new(1., 0.);
                result[2][d + 1] = C::new(-1., 0.);
                for i in 0..d {
                    result[3][i * d + i] = C::new(1., 0.);
                }
                result
            }
            fn bracket(d: usize, a: &[C], b: &[C]) -> Vec<C> {
                (0..d * d)
                    .map(|entry| {
                        let (i, j) = (entry / d, entry % d);
                        (0..d)
                            .map(|k| a[i * d + k] * b[k * d + j] - b[i * d + k] * a[k * d + j])
                            .sum::<C>()
                            * C::new(0., -1.)
                    })
                    .collect()
            }
            fn dense_reference<E: FamilyExpression<Real = R>>(
                family: &OperatorFamily<E>,
                d: usize,
                matrices: &[Vec<C>],
            ) {
                let options = NumericalOptions::<R>::default();
                let gram = family.gram(&options).unwrap();
                let constants = gram.factorize().unwrap().structure_constants().unwrap();
                for i in 0..4 {
                    for j in 0..4 {
                        let expected = matrices[i]
                            .iter()
                            .zip(&matrices[j])
                            .map(|(&x, &y)| x.conj() * y)
                            .sum::<C>()
                            / (d as R);
                        cclose(gram.values()[(i, j)], expected);
                        let dense = bracket(d, &matrices[i], &matrices[j]);
                        for (entry, &value) in dense.iter().enumerate() {
                            let reconstructed = (0..4)
                                .map(|k| matrices[k][entry] * constants.values()[(k, i, j)])
                                .sum();
                            cclose(reconstructed, value);
                        }
                    }
                }
            }
            fn workflow<E: FamilyExpression<Real = R>>(
                family: &OperatorFamily<E>,
                target: &E,
                metric: R,
            ) {
                let options = NumericalOptions::<R>::default();
                let gram = family.gram(&options).unwrap();
                assert!(gram.validate().unwrap().passed());
                for i in 0..4 {
                    for j in 0..4 {
                        cclose(
                            gram.values()[(i, j)],
                            C::new(
                                if i != j {
                                    0.
                                } else if i == 3 {
                                    1.
                                } else {
                                    metric
                                },
                                0.,
                            ),
                        );
                    }
                }
                let factor = gram.factorize().unwrap();
                let coordinates = factor.coefficients(target).unwrap();
                for (&value, expected) in coordinates.values().iter().zip([1., 1., 0., 0.]) {
                    close(value, expected);
                }
                let reconstructed = coordinates.reconstruct().unwrap();
                assert!(family.in_span(&reconstructed, &options).unwrap());
                let partial = family.select(&[0, 2]).unwrap();
                let projected = partial.family().project(target, &options).unwrap();
                close(projected.coordinates().values()[0], 1.);
                close(projected.coordinates().values()[1], 0.);
                close(projected.residual_norm(), metric.sqrt());
                assert!(!projected.in_span());
                let closure = factor.closure().unwrap();
                assert!(closure.passed());
                let constants = closure.structure_constants().unwrap();
                close(constants.values()[(2, 0, 1)], 2.);
                let adjoint = constants.adjoint();
                let killing = adjoint.killing_form().unwrap();
                for i in 0..4 {
                    for j in 0..4 {
                        close(
                            killing.values()[(i, j)],
                            if i == j && i < 3 { -8. } else { 0. },
                        );
                    }
                }
                assert_eq!(adjoint.commutant().unwrap().dimension(), 2);
                let center = adjoint.center().unwrap();
                assert_eq!(center.center().unwrap().len(), 1);
                assert_eq!(center.complement().unwrap().len(), 3);
                let settings = DecompositionOptions {
                    seed: 816,
                    max_attempts: 16,
                };
                let result = adjoint.decompose(&settings).unwrap();
                assert!(std::ptr::eq(result.family(), family));
                assert!(result.diagnostics().validation.passed());
                assert_eq!(result.simple_ideals().len(), 1);
                let simple = &result.simple_ideals()[0];
                assert_eq!(simple.family().len(), 3);
                let central = result.center().unwrap();
                assert_eq!(central.family().len(), 1);
                for i in 0..4 {
                    for j in 0..4 {
                        let a = central.coordinates()[(i, 0)] * central.coordinates()[(j, 0)];
                        close(a, if i == 3 && j == 3 { 1. } else { 0. });
                        let b = (0..3)
                            .map(|k| simple.coordinates()[(i, k)] * simple.coordinates()[(j, k)])
                            .sum::<R>()
                            * metric;
                        close(b, if i == j && i < 3 { 1. } else { 0. });
                    }
                }
                for component in result.center().into_iter().chain(result.simple_ideals()) {
                    assert!(std::ptr::eq(component.source(), family));
                    assert!(
                        component
                            .family()
                            .orthonormality(&options)
                            .unwrap()
                            .passed()
                    );
                    assert!(
                        component
                            .orthonormalize(&options)
                            .unwrap()
                            .family()
                            .orthonormality(&options)
                            .unwrap()
                            .passed()
                    );
                }
            }

            #[test]
            fn spin_workflow_agrees_with_pauli_matrices() {
                use syntaqs_spin_half::SpinHalfOperator as P;
                type Op = LocalOperator<P, C>;
                let x = Op::primitive(P::Sx);
                let y = Op::primitive(P::Sy);
                let target = &x + &y;
                let family =
                    OperatorFamily::new([x, y, Op::primitive(P::Sz), Op::identity()]).unwrap();
                workflow(&family, &target, 1.);
                dense_reference(&family, 2, &matrices(2));
            }
            #[test]
            fn fermion_workflow_agrees_with_majorana_matrices() {
                use syntaqs_fermion::FermionOperator as F;
                type Op = LocalOperator<F, C>;
                let x = Op::primitive(F::Mx);
                let y = Op::primitive(F::My);
                let target = &x + &y;
                let family =
                    OperatorFamily::new([x, y, Op::primitive(F::Mz), Op::identity()]).unwrap();
                workflow(&family, &target, 1.);
                dense_reference(&family, 2, &matrices(2));
            }
            #[test]
            fn custom_three_level_workflow_needs_no_state_basis_or_primitive_clone() {
                use three_level::{
                    Axis::{A, B},
                    Operator::Unit,
                };
                type Op = LocalOperator<ThreePrimitive, C>;
                let e = |i, j| Op::primitive(ThreePrimitive(Unit(i, j)));
                let x = e(A, B) + e(B, A);
                let y = (e(A, B) - e(B, A)).scaled(C::new(0., -1.));
                let z = e(A, A) - e(B, B);
                let target = &x + &y;
                let family = OperatorFamily::new([x, y, z, Op::identity()]).unwrap();
                workflow(&family, &target, 2. / 3.);
                dense_reference(&family, 3, &matrices(3));
            }
            #[test]
            fn graded_shared_fixture_uses_ordinary_brackets_without_states_or_dimension() {
                type Local = LocalOperator<GradedPrimitive, C>;
                let a = HilbertSpace::factor(Factor(0));
                let b = HilbertSpace::factor(Factor(1));
                let space = HilbertSpace::tensor_all_g([a.clone(), b.clone()]).unwrap();
                let local = Local::primitive(GradedPrimitive(graded::A))
                    + Local::primitive(GradedPrimitive(graded::CREATE));
                let x = local.at(&a).unwrap().bind(&space).unwrap();
                let y = local.at(&b).unwrap().bind(&space).unwrap();
                let z = lie_bracket(&x, &y).unwrap().scaled(0.5);
                let target = (&x + &y).unwrap();
                let family =
                    OperatorFamily::new_in(&space, [x, y, z, Operator::identity()]).unwrap();
                workflow(&family, &target, 1.);
                let embed = |at| {
                    graded::embedding::<R>(graded::A, at, 2, syntaqs_core::TensorKind::Graded)
                        .into_iter()
                        .zip(graded::embedding(
                            graded::CREATE,
                            at,
                            2,
                            syntaqs_core::TensorKind::Graded,
                        ))
                        .map(|(a, b)| a + b)
                        .collect::<Vec<_>>()
                };
                let dx = embed(0);
                let dy = embed(1);
                let dz = bracket(4, &dx, &dy).into_iter().map(|x| x * 0.5).collect();
                let identity = (0..16)
                    .map(|i| C::new(if i / 4 == i % 4 { 1. } else { 0. }, 0.))
                    .collect();
                dense_reference(&family, 4, &[dx, dy, dz, identity]);
            }
            #[test]
            fn eighty_spectators_do_not_require_a_physical_action_matrix() {
                use syntaqs_spin_half::SpinHalfOperator as P;
                type Local = LocalOperator<P, C>;
                let factors: Vec<_> = (0..80u8).map(|i| HilbertSpace::factor(Factor(i))).collect();
                let space = HilbertSpace::tensor_all(factors.iter().cloned()).unwrap();
                let x = Local::primitive(P::Sx).at(&factors[0]).unwrap();
                let y = Local::primitive(P::Sy).at(&factors[0]).unwrap();
                let z = Local::primitive(P::Sz).at(&factors[0]).unwrap();
                let target = (&x + &y).unwrap();
                let family =
                    OperatorFamily::new_in(&space, [x, y, z, Operator::identity()]).unwrap();
                workflow(&family, &target, 1.);
            }
        }
    };
}
precision_tests!(single, f32);
precision_tests!(double, f64);
