//! Public Lie APIs checked against independently assembled dense matrices.
extern crate blas_provider as _;

use ndarray::{Array2, array};
use num_complex::Complex;
use syntaqs_core::{HilbertSpace, LocalOperator};
use syntaqs_lie::{
    LieError, NumericalError, NumericalOptions, OperatorFamily, Tolerance, lie_bracket,
};
use syntaqs_spin_half::SpinHalfOperator as Pauli;

macro_rules! precision_tests {
    ($module:ident, $real:ty) => {
        mod $module {
            use super::*;
            type R = $real;
            type C = Complex<R>;
            type Op = LocalOperator<Pauli, C>;
            fn op(p: Pauli) -> Op {
                Op::primitive(p)
            }
            fn options() -> NumericalOptions<R> {
                NumericalOptions::default()
            }
            fn family() -> OperatorFamily<Op> {
                OperatorFamily::new([op(Pauli::Sx), op(Pauli::Sy), op(Pauli::Sz)]).unwrap()
            }
            fn close(a: R, b: R) {
                assert!(
                    options()
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
            fn matrix([i, x, y, z]: [R; 4]) -> Array2<C> {
                array![
                    [C::new(i + z, 0.), C::new(x, -y)],
                    [C::new(x, y), C::new(i - z, 0.)]
                ]
            }
            fn dense_bracket(a: &Array2<C>, b: &Array2<C>) -> Array2<C> {
                (a.dot(b) - b.dot(a)).mapv(|x| x * C::new(0., -1.))
            }
            fn dense_hs(a: &Array2<C>, b: &Array2<C>) -> C {
                a.iter().zip(b).map(|(&x, &y)| x.conj() * y).sum::<C>() / (a.nrows() as R)
            }
            fn dense_local(a: &Op) -> Array2<C> {
                let mut coefficients = [0.; 4];
                for (i, p) in [Pauli::I, Pauli::Sx, Pauli::Sy, Pauli::Sz]
                    .into_iter()
                    .enumerate()
                {
                    let v = op(p).hs_inner_product(a);
                    close(v.im, 0.);
                    coefficients[i] = v.re;
                }
                matrix(coefficients)
            }
            fn matrices_close(a: &Array2<C>, b: &Array2<C>) {
                assert_eq!(a.dim(), b.dim());
                for (&x, &y) in a.iter().zip(b) {
                    cclose(x, y);
                }
            }
            fn real_matrices_close(a: &Array2<R>, b: &Array2<R>) {
                assert_eq!(a.dim(), b.dim());
                for (&x, &y) in a.iter().zip(b) {
                    close(x, y);
                }
            }

            #[test]
            fn pauli_structure_constants_adjoint_columns_and_killing() {
                let f = family();
                let settings = options();
                let factor = f.gram(&settings).unwrap().factorize().unwrap();
                let constants = factor.structure_constants().unwrap();
                assert!(std::ptr::eq(constants.family(), &f));
                assert_eq!(constants.options(), &settings);
                assert_eq!(constants.closure().checks().len(), 9);
                let adj = constants.adjoint();
                assert!(std::ptr::eq(adj.family(), &f));
                assert_eq!(adj.options(), &settings);
                for i in 0..3 {
                    for j in 0..3 {
                        for k in 0..3 {
                            let expected = if (i, j, k) == (0, 1, 2)
                                || (i, j, k) == (1, 2, 0)
                                || (i, j, k) == (2, 0, 1)
                            {
                                2.
                            } else if (j, i, k) == (0, 1, 2)
                                || (j, i, k) == (1, 2, 0)
                                || (j, i, k) == (2, 0, 1)
                            {
                                -2.
                            } else {
                                0.
                            };
                            close(constants.values()[(k, i, j)], expected);
                            close(adj.matrices()[i][(k, j)], expected);
                        }
                    }
                }
                let tiny = settings.residual.absolute / 1000.;
                let bracket = lie_bracket(&op(Pauli::Sx).scaled(tiny), &op(Pauli::Sy)).unwrap();
                assert_eq!(op(Pauli::Sz).hs_inner_product(&bracket).re, 2. * tiny);
                assert_eq!(
                    f.bracket_coefficients(0, 1, &settings)
                        .unwrap()
                        .family()
                        .len(),
                    3
                );
                let killing = adj.killing_form().unwrap();
                assert!(std::ptr::eq(killing.family(), &f));
                assert_eq!(killing.options(), &settings);
                real_matrices_close(killing.values(), &(Array2::<R>::eye(3) * -8.));
            }

            #[test]
            fn nonorthonormal_dense_brackets_jacobi_and_coordinate_actions() {
                let f = family();
                let u = array![[2., 1., 0.], [0., 1., 1.], [1., 0., 2.]];
                let changed = f.transform(u.view()).unwrap();
                let g = changed.family();
                let settings = options();
                let constants = g.structure_constants(&settings).unwrap();
                let adj = constants.adjoint();
                for i in 0..3 {
                    for j in 0..3 {
                        let direct = g.bracket(i, j).unwrap();
                        matrices_close(
                            &dense_local(&direct),
                            &dense_bracket(
                                &matrix([0., u[(0, i)], u[(1, i)], u[(2, i)]]),
                                &matrix([0., u[(0, j)], u[(1, j)], u[(2, j)]]),
                            ),
                        );
                        let coefficients = array![
                            constants.values()[(0, i, j)],
                            constants.values()[(1, i, j)],
                            constants.values()[(2, i, j)]
                        ];
                        matrices_close(
                            &dense_local(&g.reconstruct(coefficients.view()).unwrap()),
                            &dense_local(&direct),
                        );
                        for k in 0..3 {
                            close(
                                constants.values()[(k, i, j)],
                                -constants.values()[(k, j, i)],
                            );
                        }
                    }
                }
                let v = array![0.25, -0.5, 1.];
                let target = g.reconstruct(v.view()).unwrap();
                for i in 0..3 {
                    let action = adj.matrices()[i].dot(&v);
                    matrices_close(
                        &dense_local(&g.reconstruct(action.view()).unwrap()),
                        &dense_local(&lie_bracket(&g.operators()[i], &target).unwrap()),
                    );
                    for j in 0..3 {
                        for k in 0..3 {
                            for output in 0..3 {
                                let mut jacobi = 0.;
                                for l in 0..3 {
                                    jacobi += constants.values()[(l, j, k)]
                                        * constants.values()[(output, i, l)]
                                        + constants.values()[(l, k, i)]
                                            * constants.values()[(output, j, l)]
                                        + constants.values()[(l, i, j)]
                                            * constants.values()[(output, k, l)];
                                }
                                close(jacobi, 0.);
                            }
                        }
                    }
                }
                let k = adj.killing_form().unwrap();
                real_matrices_close(k.values(), &u.t().dot(&(Array2::<R>::eye(3) * -8.)).dot(&u));
            }

            #[test]
            fn nonclosed_projections_are_reported_and_never_structure_constants() {
                let f = OperatorFamily::new([op(Pauli::Sx), op(Pauli::Sz)]).unwrap();
                let settings = options();
                let factor = f.gram(&settings).unwrap().factorize().unwrap();
                let projection = f.bracket_projection(0, 1, &settings).unwrap();
                assert!(!projection.in_span());
                close(projection.residual_norm(), 2.);
                assert!(
                    f.bracket_coefficients(0, 1, &settings)
                        .unwrap()
                        .values()
                        .iter()
                        .all(|&x| x == 0.)
                );
                assert!(
                    factor
                        .bracket_coefficients(0, 1)
                        .unwrap()
                        .values()
                        .iter()
                        .all(|&x| x == 0.)
                );
                let closure = factor.closure().unwrap();
                assert!(!closure.passed());
                assert_eq!(closure.options(), &settings);
                assert!(std::ptr::eq(closure.family(), &f));
                let pair = &closure.checks()[1];
                assert_eq!((pair.left, pair.right), (0, 1));
                close(pair.check.measured, 2.);
                close(pair.check.scale, 2.);
                assert!(matches!(
                    closure.structure_constants(),
                    Err(LieError::NonClosedPair {
                        left: 0,
                        right: 1,
                        ..
                    })
                ));
                assert!(matches!(
                    f.structure_constants(&settings),
                    Err(LieError::NonClosedPair { .. })
                ));
                let mut relaxed = settings;
                relaxed.residual = Tolerance {
                    absolute: 3.,
                    relative: 0.,
                };
                assert!(f.closure(&relaxed).unwrap().passed());
                assert_eq!(f.structure_constants(&relaxed).unwrap().options(), &relaxed);
            }

            #[test]
            fn abelian_family_has_zero_brackets_and_killing() {
                let f = OperatorFamily::new([Op::identity(), op(Pauli::Sz)]).unwrap();
                let constants = f.structure_constants(&options()).unwrap();
                assert!(constants.values().iter().all(|&x| x == 0.));
                assert!(
                    constants
                        .adjoint()
                        .killing_form()
                        .unwrap()
                        .values()
                        .iter()
                        .all(|&x| x == 0.)
                );
            }

            #[test]
            fn transform_selection_and_orthogonalization_retain_coordinates() {
                let f = family();
                let u = array![[2., 1., 0.], [0., 1., 1.], [1., 0., 2.]];
                let changed = f.transform(u.view()).unwrap();
                assert!(std::ptr::eq(changed.source(), &f));
                assert!(changed.options().is_none());
                assert_eq!(changed.matrix(), &u);
                for (j, new) in changed.family().iter().enumerate() {
                    matrices_close(
                        &dense_local(new),
                        &matrix([0., u[(0, j)], u[(1, j)], u[(2, j)]]),
                    );
                }
                let selected = changed.family().select(&[2, 0]).unwrap();
                assert_eq!(selected.matrix(), &array![[0., 1.], [0., 0.], [1., 0.]]);
                matrices_close(
                    &dense_local(&selected.family().operators()[0]),
                    &dense_local(&changed.family().operators()[2]),
                );
                let settings = options();
                assert!(f.orthonormality(&settings).unwrap().passed());
                assert!(!changed.family().orthonormality(&settings).unwrap().passed());
                let orthogonal = changed.family().orthogonalize(&settings).unwrap();
                let gram = orthogonal.family().gram(&settings).unwrap();
                for i in 0..3 {
                    for j in 0..3 {
                        if i != j {
                            cclose(gram.values()[(i, j)], C::new(0., 0.));
                        }
                    }
                }
                let normalized = changed.family().orthonormalize(&settings).unwrap();
                assert_eq!(normalized.options(), Some(&settings));
                assert!(
                    normalized
                        .family()
                        .orthonormality(&settings)
                        .unwrap()
                        .passed()
                );
                let old_gram = changed
                    .family()
                    .gram(&settings)
                    .unwrap()
                    .values()
                    .mapv(|z| z.re);
                real_matrices_close(
                    &normalized
                        .matrix()
                        .t()
                        .dot(&old_gram)
                        .dot(normalized.matrix()),
                    &Array2::eye(3),
                );
                for (j, new) in normalized.family().iter().enumerate() {
                    let coordinates = u.dot(&normalized.matrix().column(j));
                    matrices_close(
                        &dense_local(new),
                        &matrix([0., coordinates[0], coordinates[1], coordinates[2]]),
                    );
                }
            }

            #[test]
            fn candidate_and_operation_preconditions_and_bad_inputs() {
                let f = family();
                let settings = options();
                let dependent = f.select(&[0, 0]).unwrap();
                assert!(matches!(
                    dependent.family().closure(&settings),
                    Err(LieError::Numerical(NumericalError::SingularBasis { .. }))
                ));
                assert!(dependent.family().orthonormalize(&settings).is_err());
                let invalid = OperatorFamily::new([op(Pauli::Sp)]).unwrap();
                assert!(matches!(
                    invalid.closure(&settings),
                    Err(LieError::NonHermitianElement { .. })
                ));
                assert!(invalid.orthogonalize(&settings).is_err());
                assert!(!invalid.orthonormality(&settings).unwrap().passed());
                assert!(matches!(
                    f.transform(Array2::zeros((3, 0)).view()),
                    Err(LieError::EmptyFamily)
                ));
                assert!(matches!(f.select(&[]), Err(LieError::EmptyFamily)));
                assert!(f.select(&[3]).is_err());
                assert!(f.bracket(3, 0).is_err());
                assert!(f.bracket(0, 3).is_err());
                assert!(f.transform(Array2::zeros((2, 1)).view()).is_err());
                assert!(f.transform(array![[R::NAN], [0.], [0.]].view()).is_err());
                let mut invalid_settings = settings;
                invalid_settings.rank_relative = Some(R::NAN);
                assert!(f.closure(&invalid_settings).is_err());
                assert!(f.orthonormalize(&invalid_settings).is_err());
                let mut strict_gram = settings;
                strict_gram.gram_relative = Some(2.);
                strict_gram.rank_relative = Some(0.);
                assert!(matches!(
                    f.structure_constants(&strict_gram),
                    Err(LieError::Numerical(NumericalError::SingularBasis { .. }))
                ));
                assert!(matches!(
                    f.orthonormalize(&strict_gram),
                    Err(LieError::Numerical(NumericalError::SingularBasis { .. }))
                ));
                let nonfinite = OperatorFamily::new([op(Pauli::Sx).scaled(R::INFINITY)]).unwrap();
                assert!(nonfinite.structure_constants(&settings).is_err());
            }

            #[test]
            fn placed_transforms_and_brackets_preserve_ambient_context_and_check_conflicts() {
                let a = HilbertSpace::factor(0u8);
                let b = HilbertSpace::factor(1u8);
                let space = HilbertSpace::tensor_all([a.clone(), b.clone()]).unwrap();
                let f = OperatorFamily::new_in(&space, family().iter().map(|x| x.at(&a).unwrap()))
                    .unwrap();
                let bracket = f.bracket(0, 1).unwrap();
                assert!(bracket.space().unwrap().is_compatible_with(&space));
                let transformed = f.orthonormalize(&options()).unwrap();
                assert!(transformed.family().context().is_compatible_with(&space));
                assert!(
                    f.structure_constants(&options())
                        .unwrap()
                        .closure()
                        .passed()
                );
                let left = op(Pauli::Sx).at(&a).unwrap().bind(&a).unwrap();
                let right = op(Pauli::Sy).at(&b).unwrap().bind(&b).unwrap();
                assert!(lie_bracket(&left, &right).is_err());
            }

            #[test]
            fn graded_odd_majoranas_use_ordinary_brackets_and_match_jordan_wigner() {
                use syntaqs_fermion::FermionOperator as F;
                type Local = LocalOperator<F, C>;
                let a = HilbertSpace::factor(0u8);
                let b = HilbertSpace::factor(1u8);
                let space = HilbertSpace::tensor_all_g([a.clone(), b.clone()]).unwrap();
                let x = Local::primitive(F::Mx)
                    .at(&a)
                    .unwrap()
                    .bind(&space)
                    .unwrap();
                let y = Local::primitive(F::Mx)
                    .at(&b)
                    .unwrap()
                    .bind(&space)
                    .unwrap();
                let z = lie_bracket(&x, &y).unwrap().scaled(0.5);
                let f = OperatorFamily::new([x.clone(), y.clone(), z]).unwrap();
                let constants = f.structure_constants(&options()).unwrap();
                close(constants.values()[(2, 0, 1)], 2.);
                real_matrices_close(
                    constants.adjoint().killing_form().unwrap().values(),
                    &(Array2::<R>::eye(3) * -8.),
                );
                fn kron(a: &Array2<C>, b: &Array2<C>) -> Array2<C> {
                    Array2::from_shape_fn((4, 4), |(i, j)| a[(i / 2, j / 2)] * b[(i % 2, j % 2)])
                }
                let id = matrix([1., 0., 0., 0.]);
                let mx = matrix([0., 1., 0., 0.]);
                let my = matrix([0., 0., 1., 0.]);
                let parity = matrix([0., 0., 0., 1.]);
                let dx = kron(&mx, &id);
                let dy = kron(&parity, &mx);
                let reference = dense_bracket(&dx, &dy);
                let direct = lie_bracket(&x, &y).unwrap();
                let primitives = [F::I, F::Mx, F::My, F::Mz];
                let dense = [id.clone(), mx, my, parity.clone()];
                for i in 0..4 {
                    for j in 0..4 {
                        let probe = (Local::primitive(primitives[i]).at(&a).unwrap()
                            * Local::primitive(primitives[j]).at(&b).unwrap())
                        .unwrap()
                        .bind(&space)
                        .unwrap();
                        let da = kron(&dense[i], &id);
                        let db = kron(if j == 1 || j == 2 { &parity } else { &id }, &dense[j]);
                        cclose(
                            probe.hs_inner_product(&direct).unwrap(),
                            dense_hs(&da.dot(&db), &reference),
                        );
                    }
                }
                // Odd operators on different modes anticommute; their ordinary bracket is nonzero.
                close(dense_hs(&reference, &reference).re, 4.);
            }
        }
    };
}
precision_tests!(single, f32);
precision_tests!(double, f64);
