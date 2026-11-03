//! Public API checks against hand-assembled 2-by-2 matrices, without export or a state basis.

extern crate blas_provider as _;

use ndarray::{Array2, array};
use num_complex::Complex;
use syntaqs_core::{ExpressionError, HilbertSpace, LocalOperator, Operator, TensorKind};
use syntaqs_lie::{LieError, NumericalError, NumericalOptions, OperatorFamily, Tolerance};
use syntaqs_spin_half::SpinHalfOperator as Pauli;

macro_rules! precision_tests {
    ($module:ident, $real:ty) => {
        mod $module {
            use super::*;
            type R = $real;
            type C = Complex<R>;
            type Local = LocalOperator<Pauli, C>;
            type Placed = Operator<u8, Pauli, C>;

            fn local(op: Pauli) -> Local {
                Local::primitive(op)
            }

            fn close(actual: R, expected: R) {
                assert!(
                    NumericalOptions::<R>::default()
                        .residual
                        .check((actual - expected).abs(), expected.abs())
                        .unwrap()
                        .passed(),
                    "{actual} != {expected}"
                );
            }

            fn complex_close(actual: C, expected: C) {
                close(actual.re, expected.re);
                close(actual.im, expected.im);
            }

            // Explicit Pauli matrices, independent of the primitive's product/trace rules.
            fn matrix([i, x, y, z]: [R; 4]) -> Array2<C> {
                array![
                    [C::new(i + z, 0.), C::new(x, -y)],
                    [C::new(x, y), C::new(i - z, 0.)]
                ]
            }

            fn hs(left: &Array2<C>, right: &Array2<C>) -> C {
                left.iter().zip(right).map(|(a, b)| a.conj() * b).sum::<C>() / 2.
            }

            fn expression(values: [R; 4]) -> Local {
                Local::sum(
                    [Pauli::I, Pauli::Sx, Pauli::Sy, Pauli::Sz]
                        .into_iter()
                        .zip(values)
                        .map(|(op, value)| local(op).scaled(value)),
                )
            }

            fn compare_matrix(operator: &Local, expected: &Array2<C>) {
                for (op, weights) in [
                    (Pauli::I, [1., 0., 0., 0.]),
                    (Pauli::Sx, [0., 1., 0., 0.]),
                    (Pauli::Sy, [0., 0., 1., 0.]),
                    (Pauli::Sz, [0., 0., 0., 1.]),
                ] {
                    complex_close(
                        local(op).hs_inner_product(operator),
                        hs(&matrix(weights), expected),
                    );
                }
            }

            #[test]
            fn ordered_nonorthonormal_gram_projection_and_reconstruction_match_matrices() {
                let options = NumericalOptions::<R>::default();
                let family = OperatorFamily::new([
                    expression([0., 2., 0., 0.]),
                    expression([0., 1., 0., 3.]),
                ])
                .unwrap();
                assert_eq!(family.len(), 2);
                assert!(!family.is_empty());
                assert_eq!(family.iter().count(), 2);
                assert_eq!(family.context(), &());
                let dense = [matrix([0., 2., 0., 0.]), matrix([0., 1., 0., 3.])];
                let gram = family.gram(&options).unwrap();
                assert!(std::ptr::eq(gram.family(), &family));
                for i in 0..2 {
                    compare_matrix(&family.operators()[i], &dense[i]);
                    for j in 0..2 {
                        complex_close(gram.values()[(i, j)], hs(&dense[i], &dense[j]));
                    }
                }
                let validation = gram.validate().unwrap();
                assert!(validation.passed());
                assert!(std::ptr::eq(validation.family(), &family));
                assert_eq!(validation.options(), &options);
                let factor = gram.factorize().unwrap();
                assert!(std::ptr::eq(factor.family(), &family));
                assert_eq!(factor.options(), &options);
                assert!(factor.independence().is_independent());
                let target = expression([0., 5., 4., -6.]);
                let projection = factor.project(&target).unwrap();
                assert!(std::ptr::eq(projection.coordinates().family(), &family));
                close(projection.coordinates().values()[0], 3.5);
                close(projection.coordinates().values()[1], -2.);
                let expected = matrix([0., 5., 0., -6.]);
                compare_matrix(projection.operator(), &expected);
                compare_matrix(&projection.coordinates().reconstruct().unwrap(), &expected);
                compare_matrix(projection.target(), &matrix([0., 5., 4., -6.]));
                compare_matrix(projection.residual_operator(), &matrix([0., 0., 4., 0.]));
                close(projection.residual_norm(), 4.);
                close(projection.span_check().scale, (77. as R).sqrt());
                assert!(!projection.in_span());
                for operator in family.iter() {
                    complex_close(
                        operator.hs_inner_product(projection.residual_operator()),
                        C::new(0., 0.),
                    );
                }
                let in_span = expression([0., 5., 0., -6.]);
                assert!(factor.project(&in_span).unwrap().in_span());
                close(family.residual(&target, &options).unwrap(), 4.);
                assert!(!family.in_span(&target, &options).unwrap());
                assert!(family.in_span(&in_span, &options).unwrap());
                close(
                    family.coefficients(&target, &options).unwrap().values()[0],
                    3.5,
                );
            }

            #[test]
            fn out_of_span_target_keeps_its_basis_component_and_effective_tolerance() {
                let mut options = NumericalOptions::<R>::default();
                let family = OperatorFamily::new([local(Pauli::Sx), local(Pauli::Sz)]).unwrap();
                let target = local(Pauli::Sx) + local(Pauli::Sy);
                let projection = family.project(&target, &options).unwrap();
                close(projection.coordinates().values()[0], 1.);
                close(projection.coordinates().values()[1], 0.);
                close(projection.residual_norm(), 1.);
                assert!(!projection.in_span());
                options.residual = Tolerance {
                    absolute: 1.,
                    relative: 0.,
                };
                let check = family.project(&target, &options).unwrap();
                assert!(check.in_span());
                assert_eq!(check.span_check().threshold, 1.);
                assert_eq!(check.span_check().tolerance, options.residual);
            }

            #[test]
            fn dependent_zero_and_nonhermitian_candidates_are_constructible() {
                let options = NumericalOptions::<R>::default();
                for family in [
                    OperatorFamily::new([local(Pauli::Sx), local(Pauli::Sx).scaled(2.)]).unwrap(),
                    OperatorFamily::new([Local::zero(), local(Pauli::Sx)]).unwrap(),
                ] {
                    let report = family.validate(&options).unwrap();
                    assert!(!report.passed());
                    assert!(report.hermiticity().iter().all(|check| check.passed()));
                    assert_eq!(report.independence().rank, 1);
                    assert!(matches!(
                        family.coefficients(&local(Pauli::Sx), &options),
                        Err(LieError::Numerical(NumericalError::SingularBasis {
                            rank: 1,
                            dimension: 2
                        }))
                    ));
                    // Reconstruction does not need unique coordinates.
                    assert!(family.reconstruct(array![1., 0.].view()).is_ok());
                }
                let family = OperatorFamily::new([
                    local(Pauli::Sx),
                    local(Pauli::Sx).scaled(C::new(0., 1.)),
                ])
                .unwrap();
                let gram = family.gram(&options).unwrap();
                complex_close(gram.values()[(0, 1)], C::new(0., 1.));
                let report = gram.validate().unwrap();
                assert!(report.hermiticity()[0].passed());
                assert!(!report.hermiticity()[1].passed());
                close(report.hermiticity()[1].measured, 2.);
                assert!(report.independence().is_independent()); // Over R, not C.
                assert!(!report.passed());
                assert!(matches!(
                    gram.factorize(),
                    Err(LieError::NonHermitianElement { index: 1, .. })
                ));
            }

            #[test]
            fn hermitian_target_is_required_and_imaginary_policy_is_independent() {
                let family = OperatorFamily::new([local(Pauli::Sx)]).unwrap();
                let target = local(Pauli::Sp);
                let options = NumericalOptions::<R>::default();
                assert!(matches!(
                    family.project(&target, &options),
                    Err(LieError::NonHermitianTarget { .. })
                ));
                let mut relaxed = options;
                relaxed.hermiticity = Tolerance {
                    absolute: 10.,
                    relative: 0.,
                };
                relaxed.imaginary = Tolerance {
                    absolute: 0.,
                    relative: 0.,
                };
                let target = local(Pauli::Sx).scaled(C::new(1., 0.01));
                assert!(matches!(
                    family.coefficients(&target, &relaxed),
                    Err(LieError::Numerical(NumericalError::NonReal))
                ));
                let family = OperatorFamily::new([
                    local(Pauli::Sx),
                    local(Pauli::Sy) + local(Pauli::Sx).scaled(C::new(0., 0.01)),
                ])
                .unwrap();
                assert!(matches!(
                    family.gram(&relaxed).unwrap().factorize(),
                    Err(LieError::Numerical(NumericalError::NonReal))
                ));
            }

            #[test]
            fn gram_eigenvalue_override_controls_singular_basis_independently_of_rank() {
                let mut options = NumericalOptions::<R>::default();
                let small = (options.residual.relative / 10.).sqrt();
                let family =
                    OperatorFamily::new([local(Pauli::Sx), local(Pauli::Sz).scaled(small)])
                        .unwrap();
                options.rank_relative = Some(0.);
                let report = family.validate(&options).unwrap();
                assert_eq!(report.independence().rank, 1);
                close(
                    report.independence().threshold,
                    options.gram_cutoff(2).unwrap(),
                );
                assert!(family.project(&local(Pauli::Sz), &options).is_err());
                options.gram_relative = Some(0.);
                options.rank_relative = Some(1.); // Does not override the Gram decision.
                let gram = family.gram(&options).unwrap();
                let factor = gram.factorize().unwrap();
                assert_eq!(factor.independence().threshold, 0.);
                let projection = factor.project(&local(Pauli::Sz).scaled(small)).unwrap();
                close(projection.coordinates().values()[1], 1.);
                assert!(projection.in_span());
            }

            #[test]
            fn construction_checks_context_even_for_zero_and_preserves_original_handles() {
                assert!(matches!(
                    OperatorFamily::<Local>::new([]),
                    Err(LieError::EmptyFamily)
                ));
                let a = HilbertSpace::factor(0u8);
                let b = HilbertSpace::factor(1u8);
                let ab = a.tensor(&b).unwrap();
                let placed = local(Pauli::Sx).at(&a).unwrap();
                assert!(matches!(
                    OperatorFamily::new([placed.clone()]),
                    Err(LieError::Expression(ExpressionError::UnboundOperator))
                ));
                let family = OperatorFamily::new([
                    placed.clone(),
                    local(Pauli::Sz).at(&b).unwrap().bind(&ab).unwrap(),
                ])
                .unwrap();
                assert!(family.context().is_compatible_with(&ab));
                assert!(family.operators()[0].space().is_none());
                assert!(family.clone().context().shares_node_with(&ab));
                assert!(OperatorFamily::new_in(&ab, [placed]).is_ok());
                assert!(matches!(
                    OperatorFamily::new([
                        Placed::zero().bind(&a).unwrap(),
                        Placed::zero().bind(&b).unwrap(),
                    ]),
                    Err(LieError::Expression(ExpressionError::IncompatibleSpaces))
                ));
                assert!(matches!(
                    OperatorFamily::new_in(&ab, [Placed::zero().bind(&a).unwrap()]),
                    Err(LieError::Expression(ExpressionError::IncompatibleSpaces))
                ));
                let missing = HilbertSpace::factor(2u8);
                let zero = local(Pauli::Sx).at(&missing).unwrap().scaled(0.);
                assert!(matches!(
                    OperatorFamily::new_in(&ab, [zero]),
                    Err(LieError::Expression(ExpressionError::MissingIndex))
                ));
                let graded =
                    HilbertSpace::tensor_all_with(TensorKind::Graded, [a.clone(), b.clone()])
                        .unwrap();
                assert!(matches!(
                    OperatorFamily::new([
                        Placed::zero().bind(&ab).unwrap(),
                        Placed::zero().bind(&graded).unwrap(),
                    ]),
                    Err(LieError::Expression(ExpressionError::IncompatibleSpaces))
                ));
                let options = NumericalOptions::<R>::default();
                let factor = family.gram(&options).unwrap().factorize().unwrap();
                assert!(matches!(
                    factor.project(&Placed::zero().bind(&a).unwrap()),
                    Err(LieError::Expression(ExpressionError::IncompatibleSpaces))
                ));
                let zero_missing = local(Pauli::Sz).at(&missing).unwrap().scaled(0.);
                assert!(matches!(
                    factor.project(&zero_missing),
                    Err(LieError::Expression(ExpressionError::MissingIndex))
                ));
            }

            #[test]
            fn explicit_ordinary_and_graded_contexts_keep_hs_normalization() {
                let options = NumericalOptions::<R>::default();
                for kind in [TensorKind::Ordinary, TensorKind::Graded] {
                    let a = HilbertSpace::factor(0u8);
                    let b = HilbertSpace::factor(1u8);
                    let full = HilbertSpace::tensor_all_with(kind, [a.clone(), b]).unwrap();
                    let family =
                        OperatorFamily::new_in(&full, [local(Pauli::Sx).at(&a).unwrap()]).unwrap();
                    complex_close(
                        family.gram(&options).unwrap().values()[(0, 0)],
                        C::new(1., 0.),
                    );
                    let target = (local(Pauli::Sx) + local(Pauli::Sy)).at(&a).unwrap();
                    let projection = family.project(&target, &options).unwrap();
                    close(projection.coordinates().values()[0], 1.);
                    close(projection.residual_norm(), 1.);
                    assert!(
                        projection
                            .operator()
                            .space()
                            .unwrap()
                            .is_compatible_with(&full)
                    );
                    assert!(
                        projection
                            .residual_operator()
                            .space()
                            .unwrap()
                            .is_compatible_with(&full)
                    );
                }
                let unit = HilbertSpace::unit();
                let family = OperatorFamily::new_in(&unit, [Placed::identity()]).unwrap();
                let projection = family
                    .project(&Placed::scalar(C::new(3., 0.)), &options)
                    .unwrap();
                close(projection.coordinates().values()[0], 3.);
                assert!(projection.in_span());
            }

            #[test]
            fn tiny_positive_residuals_and_reconstruction_coefficients_are_not_pruned() {
                let options = NumericalOptions::<R>::default();
                let family = OperatorFamily::new([local(Pauli::Sx)]).unwrap();
                let tiny = options.residual.absolute / 1000.;
                let projection = family
                    .project(&local(Pauli::Sy).scaled(tiny), &options)
                    .unwrap();
                assert!(projection.residual_norm() > 0.);
                close(projection.residual_norm() / tiny, 1.);
                assert!(projection.in_span());
                let reconstructed = family.reconstruct(array![tiny].view()).unwrap();
                assert_eq!(local(Pauli::Sx).hs_inner_product(&reconstructed).re, tiny);
                assert_eq!(
                    family
                        .coefficients(&reconstructed, &options)
                        .unwrap()
                        .values()[0],
                    tiny
                );
            }

            #[test]
            fn invalid_settings_nonfinite_data_and_bad_coordinate_shapes_are_errors() {
                let family = OperatorFamily::new([local(Pauli::Sx)]).unwrap();
                let mut options = NumericalOptions::<R>::default();
                options.eigenvalue_relative = Some(R::NAN);
                assert!(matches!(
                    family.gram(&options),
                    Err(LieError::Numerical(NumericalError::InvalidSetting(_)))
                ));
                options = NumericalOptions::default();
                for value in [R::NAN, R::INFINITY] {
                    let invalid = local(Pauli::Sx).scaled(value);
                    let candidates = OperatorFamily::new([invalid.clone()]).unwrap();
                    assert!(matches!(
                        candidates.gram(&options),
                        Err(LieError::Numerical(NumericalError::NonFinite(_)))
                    ));
                    assert!(matches!(
                        family.project(&invalid, &options),
                        Err(LieError::Numerical(NumericalError::NonFinite(_)))
                    ));
                    assert!(matches!(
                        family.reconstruct(array![value].view()),
                        Err(LieError::Numerical(NumericalError::NonFinite(_)))
                    ));
                }
                assert!(matches!(
                    family.reconstruct(array![].view()),
                    Err(LieError::Numerical(NumericalError::Shape(_)))
                ));
                assert!(matches!(
                    family.reconstruct(array![1., 2.].view()),
                    Err(LieError::Numerical(NumericalError::Shape(_)))
                ));
            }

            #[test]
            fn real_basis_change_preserves_projection() {
                let options = NumericalOptions::<R>::default();
                let original = OperatorFamily::new([local(Pauli::Sx), local(Pauli::Sz)]).unwrap();
                let changed = OperatorFamily::new([
                    original.reconstruct(array![2., 1.].view()).unwrap(),
                    original.reconstruct(array![1., -1.].view()).unwrap(),
                ])
                .unwrap();
                let target = expression([0., 3., 2., 4.]);
                let p = original.project(&target, &options).unwrap();
                let q = changed.project(&target, &options).unwrap();
                compare_matrix(p.operator(), &matrix([0., 3., 0., 4.]));
                compare_matrix(q.operator(), &matrix([0., 3., 0., 4.]));
                close(p.residual_norm(), q.residual_norm());
            }

            #[test]
            fn complex_expression_coefficients_can_describe_a_hermitian_family() {
                let options = NumericalOptions::<R>::default();
                let y = (local(Pauli::Sp) - local(Pauli::Sm)).scaled(C::new(0., -1.));
                let family = OperatorFamily::new([local(Pauli::Sx), y, local(Pauli::Sz)]).unwrap();
                assert!(family.validate(&options).unwrap().passed());
                let target = expression([0., 0.3, 0.7, -0.2]);
                let projection = family.project(&target, &options).unwrap();
                for (&actual, expected) in projection
                    .coordinates()
                    .values()
                    .iter()
                    .zip([0.3, 0.7, -0.2])
                {
                    close(actual, expected);
                }
                assert!(projection.in_span());
                let a = HilbertSpace::factor(0u8);
                let placed =
                    OperatorFamily::new_in(&a, family.iter().map(|op| op.at(&a).unwrap())).unwrap();
                assert!(
                    placed
                        .project(&target.at(&a).unwrap(), &options)
                        .unwrap()
                        .in_span()
                );
            }

            #[test]
            fn separable_local_sums_do_not_require_global_primitive_expansion() {
                let options = NumericalOptions::<R>::default();
                let factors: Vec<_> = (0..32u8).map(HilbertSpace::factor).collect();
                let space = HilbertSpace::tensor_all(factors.iter().cloned()).unwrap();
                let polynomial = local(Pauli::Sx) + local(Pauli::Sz);
                let operator =
                    Placed::try_product(factors.iter().map(|factor| polynomial.at(factor)))
                        .unwrap();
                let family = OperatorFamily::new_in(&space, [operator.clone()]).unwrap();
                assert!(family.validate(&options).unwrap().passed());
                let projection = family.project(&operator.scaled(0.3), &options).unwrap();
                close(projection.coordinates().values()[0], 0.3);
                assert!(projection.in_span());
            }

            #[test]
            fn decimal_hermitian_combinations_and_nearly_hermitian_targets_are_resolved() {
                let options = NumericalOptions::<R>::default();
                let family = OperatorFamily::new([
                    expression([0., 0.3, 0.7, 0.2]),
                    expression([0., -0.1, 0.8, 0.4]),
                ])
                .unwrap();
                assert!(family.validate(&options).unwrap().passed());
                let reconstructed = family.reconstruct(array![0.4, -0.7].view()).unwrap();
                assert!(family.project(&reconstructed, &options).unwrap().in_span());
                let family = OperatorFamily::new([local(Pauli::Sx)]).unwrap();
                let delta = options.hermiticity.absolute * 100.;
                let target = local(Pauli::Sx) + local(Pauli::Sy).scaled(C::new(0., delta));
                assert!(matches!(
                    family.project(&target, &options),
                    Err(LieError::NonHermitianTarget { .. })
                ));
            }
        }
    };
}

precision_tests!(single, f32);
precision_tests!(double, f64);

#[test]
fn primitives_and_indices_need_neither_clone_nor_dimension_or_state_capabilities() {
    use syntaqs_core::{
        Adjoint, HomogeneousParity, Identity, LinearCombination, Parity, ParityOperator, Product,
        Trace,
    };

    #[derive(PartialEq)]
    struct Primitive(u8);
    #[derive(PartialEq, Eq)]
    struct Index(u8);
    type C = Complex<f64>;
    type Local = LocalOperator<Primitive, C>;
    impl Identity for Primitive {
        fn identity() -> Self {
            Self(0)
        }
    }
    impl Product<C> for Primitive {
        type Term = Self;
        fn product(&self, rhs: &Self) -> LinearCombination<Self, C> {
            LinearCombination::primitive(Self(self.0 ^ rhs.0))
        }
    }
    impl Adjoint<C> for Primitive {
        type Term = Self;
        fn adjoint(&self) -> LinearCombination<Self, C> {
            LinearCombination::primitive(Self(self.0))
        }
    }
    impl Trace<C> for Primitive {
        fn trace(&self) -> C {
            C::new(if self.0 == 0 { 2. } else { 0. }, 0.)
        }
    }
    impl HomogeneousParity for Primitive {
        fn parity(&self) -> Parity {
            Parity::Even
        }
    }
    impl ParityOperator for Primitive {
        fn parity_operator() -> Self {
            Self(0)
        }
    }
    let x = Local::primitive(Primitive(1));
    let options = NumericalOptions::<f64>::default();
    let local = OperatorFamily::new([x.clone(), Local::identity()]).unwrap();
    assert!(local.validate(&options).unwrap().passed());
    assert!(local.project(&x.scaled(0.3), &options).unwrap().in_span());
    let space = HilbertSpace::factor(Index(0));
    let placed =
        OperatorFamily::new_in(&space, local.iter().map(|op| op.at(&space).unwrap())).unwrap();
    assert!(placed.validate(&options).unwrap().passed());
    assert!(
        placed
            .project(&x.at(&space).unwrap().scaled(0.3), &options)
            .unwrap()
            .in_span()
    );
}
