use super::*;
use ndarray::array;
use num_complex::Complex;
use syntaqs_core::expression::local::LocalOperator;
use syntaqs_spin_half::operator::SpinHalfOperator as Pauli;

macro_rules! precision_tests {
    ($module:ident,$real:ty) => {
        mod $module {
            use super::*;
            type R = $real;
            type Op = LocalOperator<Pauli, Complex<R>>;
            fn check_matrix(a: &Array2<R>, b: &Array2<R>) {
                assert_eq!(a.dim(), b.dim());
                for (&x, &y) in a.iter().zip(b) {
                    assert!(
                        (x - y).abs() <= NumericalOptions::<R>::default().residual.absolute * 10.,
                        "{x} != {y}"
                    );
                }
            }

            #[test]
            fn zero_constraints_keep_per_column_residual_diagnostics() {
                let mut options = NumericalOptions::<R>::default();
                options.residual.absolute = 0.25;
                options.residual.relative = 0.5;
                let expected = options.residual.check(0., 0.).unwrap();
                for (rows, columns) in [(2, 3), (0, 3), (3, 0), (0, 0)] {
                    let constraints = Array2::<R>::zeros((rows, columns));
                    let analysis = rank_nullspace(&constraints, &options).unwrap();
                    let report =
                        diagnostics(&constraints, &analysis, &options, "test constraints").unwrap();
                    assert_eq!((report.rows, report.columns), (rows, columns));
                    assert_eq!(report.rank, 0);
                    assert_eq!(report.residuals, vec![expected; columns]);
                }
            }

            #[test]
            fn nonzero_constraints_with_rank_zero_still_check_residuals() {
                let options = NumericalOptions::<R> {
                    rank_relative: Some(1.),
                    ..NumericalOptions::default()
                };
                let constraints = array![[2., 0.], [0., 0.]];
                let analysis = rank_nullspace(&constraints, &options).unwrap();
                assert_eq!(analysis.rank, 0);
                assert!(matches!(
                    diagnostics(&constraints, &analysis, &options, "test constraints"),
                    Err(LieError::ConstraintResidual { check, .. }) if check.measured > 0.
                ));
            }

            #[test]
            fn orthonormal_adjoint_coordinates_match_independent_transformation() {
                let original = OperatorFamily::new([
                    Op::primitive(Pauli::Sx),
                    Op::primitive(Pauli::Sy),
                    Op::primitive(Pauli::Sz),
                    Op::identity(),
                ])
                .unwrap();
                let u = array![
                    [2., 1., 0., 1.],
                    [0., 1., 1., 0.],
                    [1., 0., 2., 0.],
                    [0., 1., 0., 2.]
                ];
                let transformed = original.transform(u.view()).unwrap();
                let settings = NumericalOptions::<R>::default();
                let adj = transformed
                    .family()
                    .structure_constants(&settings)
                    .unwrap()
                    .adjoint();
                let internal = orthonormal_coordinates(&adj).unwrap();
                check_matrix(&internal.inverse.dot(&internal.forward), &Array2::eye(4));
                check_matrix(&internal.forward.dot(&internal.inverse), &Array2::eye(4));
                let physical = u.dot(&internal.forward);
                check_matrix(&physical.t().dot(&physical), &Array2::eye(4));
                let expected = original.transform(physical.view()).unwrap();
                let expected = expected
                    .family()
                    .structure_constants(&settings)
                    .unwrap()
                    .adjoint();
                for (a, b) in internal.adjoints.iter().zip(expected.matrices()) {
                    check_matrix(a, b);
                    check_matrix(&(a + &a.t()), &Array2::zeros((4, 4)));
                }
                let symmetric = symmetric_commutant(&internal.adjoints, 4, &settings).unwrap();
                assert_eq!(symmetric.matrices.len(), 2);
                assert_eq!(symmetric.diagnostics.columns, 10);
                for c in &symmetric.matrices {
                    check_matrix(c, &c.t().to_owned());
                    for a in &internal.adjoints {
                        check_matrix(&c.dot(a), &a.dot(c));
                    }
                }
            }
            #[test]
            fn symmetric_commutant_is_distinct_from_full_real_commutant() {
                let settings = NumericalOptions::<R>::default();
                let zero = vec![Array2::<R>::zeros((3, 3)); 3];
                let symmetric = symmetric_commutant(&zero, 3, &settings).unwrap();
                assert_eq!(symmetric.matrices.len(), 6); // Full real space has dimension nine.
                assert_eq!(symmetric.diagnostics.rank, 0);
                for (i, a) in symmetric.matrices.iter().enumerate() {
                    check_matrix(a, &a.t().to_owned());
                    for (j, b) in symmetric.matrices.iter().enumerate() {
                        let hs: R = a.iter().zip(b).map(|(&x, &y)| x * y).sum();
                        assert!(
                            (hs - if i == j { 1. } else { 0. }).abs() < settings.residual.absolute
                        );
                    }
                }
                let ad_x = array![
                    [0., 0., 0., 0., 0.],
                    [0., 0., -2., 0., 0.],
                    [0., 2., 0., 0., 0.],
                    [0., 0., 0., 0., 0.],
                    [0., 0., 0., 0., 0.]
                ];
                let ad_y = array![
                    [0., 0., 2., 0., 0.],
                    [0., 0., 0., 0., 0.],
                    [-2., 0., 0., 0., 0.],
                    [0., 0., 0., 0., 0.],
                    [0., 0., 0., 0., 0.]
                ];
                let ad_z = array![
                    [0., -2., 0., 0., 0.],
                    [2., 0., 0., 0., 0.],
                    [0., 0., 0., 0., 0.],
                    [0., 0., 0., 0., 0.],
                    [0., 0., 0., 0., 0.]
                ];
                let matrices = vec![
                    ad_x,
                    ad_y,
                    ad_z,
                    Array2::zeros((5, 5)),
                    Array2::zeros((5, 5)),
                ];
                let symmetric = symmetric_commutant(&matrices, 5, &settings).unwrap();
                assert_eq!(symmetric.matrices.len(), 4); // Scalar on su(2), symmetric 2x2 on the center.
                assert_eq!(
                    rank_nullspace(&commutant_constraints(&matrices, 5), &settings)
                        .unwrap()
                        .nullspace
                        .ncols(),
                    5
                );
            }
        }
    };
}
precision_tests!(single, f32);
precision_tests!(double, f64);
