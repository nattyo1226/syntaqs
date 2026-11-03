use super::*;
use crate::Tolerance;
use ndarray::array;

macro_rules! precision_tests {
    ($module:ident, $real:ty) => {
        mod $module {
            use super::*;
            type R = $real;

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

            #[test]
            fn solves_known_system_and_rejects_nonunique_coordinates() {
                let options = NumericalOptions::<R>::default();
                let a: Array2<R> = array![[3., 2., -1.], [2., -2., 4.], [-2., 1., -2.]];
                let result = solve(&a, &array![1., -2., 0.], &options).unwrap();
                for (&x, expected) in result.iter().zip([1., -2., -2.]) {
                    close(x, expected);
                }
                let dependent = array![[1., 2.], [2., 4.]];
                assert!(matches!(
                    solve(&dependent, &array![1., 2.], &options),
                    Err(NumericalError::SingularBasis {
                        rank: 1,
                        dimension: 2
                    })
                ));
                assert!(matches!(
                    solve(&a, &array![1.], &options),
                    Err(NumericalError::Shape(_))
                ));
            }

            #[test]
            fn explicit_rank_override_reaches_the_actual_solve() {
                let mut options = NumericalOptions::<R>::default();
                let small = options.residual.relative / 10.;
                let a = array![[1., 0.], [0., small]];
                assert!(matches!(
                    solve(&a, &array![1., small], &options),
                    Err(NumericalError::SingularBasis { .. })
                ));
                options.rank_relative = Some(0.);
                let x = solve(&a, &array![1., small], &options).unwrap();
                close(x[0], 1.);
                close(x[1], 1.);
                assert_eq!(options.rank_cutoff(2, 2).unwrap(), 0.);
            }

            #[test]
            fn wide_and_tall_nullspaces_have_all_orthonormal_directions() {
                let options = NumericalOptions::<R>::default();
                let matrices: Vec<Array2<R>> = vec![
                    array![[1., 0., 0.], [0., 1., 0.]],
                    array![[1., 2.], [2., 4.], [3., 6.]],
                    array![[1., 2., 3.], [2., 4., 6.]],
                ];
                for (a, expected_rank) in matrices.into_iter().zip([2, 1, 1]) {
                    let result = rank_nullspace(&a, &options).unwrap();
                    assert_eq!(result.rank, expected_rank);
                    assert_eq!(result.nullspace.ncols(), a.ncols() - expected_rank);
                    close(
                        result.threshold,
                        result.relative_cutoff * result.singular_values[0],
                    );
                    for &x in &a.dot(&result.nullspace) {
                        close(x, 0.);
                    }
                    let gram = result.nullspace.t().dot(&result.nullspace);
                    for ((i, j), &x) in gram.indexed_iter() {
                        close(x, if i == j { 1. } else { 0. });
                    }
                }
            }

            #[test]
            fn zero_and_empty_matrices_are_legitimate_nullspace_problems() {
                let options = NumericalOptions::<R>::default();
                for (m, n) in [(2, 3), (0, 3), (3, 0), (0, 0)] {
                    let a = Array2::<R>::zeros((m, n));
                    let result = rank_nullspace(&a, &options).unwrap();
                    assert_eq!(result.rank, 0);
                    assert_eq!(result.nullspace, Array2::eye(n));
                }
                assert!(
                    solve(&Array2::<R>::zeros((0, 0)), &Array1::zeros(0), &options)
                        .unwrap()
                        .is_empty()
                );
                assert!(
                    symmetric_eigen(&Array2::<R>::zeros((0, 0)), &options)
                        .unwrap()
                        .values
                        .is_empty()
                );
            }

            #[test]
            fn symmetric_spectrum_matches_known_eigenvalues_and_equations() {
                let options = NumericalOptions::<R>::default();
                let a: Array2<R> = array![[2., 1.], [1., 2.]];
                for a in [a.clone(), a.reversed_axes()] {
                    let spectrum = symmetric_eigen(&a, &options).unwrap();
                    close(spectrum.values[0], 1.);
                    close(spectrum.values[1], 3.);
                    let residual = a.dot(&spectrum.vectors)
                        - spectrum.vectors.dot(&Array2::from_diag(&spectrum.values));
                    for &x in &residual {
                        close(x, 0.);
                    }
                    let orthogonal = spectrum.vectors.t().dot(&spectrum.vectors);
                    for ((i, j), &x) in orthogonal.indexed_iter() {
                        close(x, if i == j { 1. } else { 0. });
                    }
                }
                assert!(matches!(
                    symmetric_eigen(&array![[1., 1.], [0., 1.]], &options),
                    Err(NumericalError::NotSymmetric)
                ));
            }

            #[test]
            fn symmetry_override_is_checked_before_using_a_triangle() {
                let mut options = NumericalOptions::<R>::default();
                let delta = options.hermiticity.absolute / 10.;
                let a = array![[1., delta], [0., 2.]];
                assert!(symmetric_eigen(&a, &options).is_ok());
                options.hermiticity = Tolerance {
                    absolute: 0.,
                    relative: 0.,
                };
                assert!(matches!(
                    symmetric_eigen(&a, &options),
                    Err(NumericalError::NotSymmetric)
                ));
            }

            #[test]
            fn gram_eigenvalue_cutoff_is_separate_from_singular_value_cutoff() {
                let mut options = NumericalOptions::<R>::default();
                let small = options.residual.relative / 10.;
                let gram = array![[1., 0.], [0., small]];
                options.rank_relative = Some(0.);
                assert!(matches!(
                    gram_spectrum(&gram, &options),
                    Err(NumericalError::SingularBasis { .. })
                ));
                options.gram_relative = Some(0.);
                let (spectrum, threshold) = gram_spectrum(&gram, &options).unwrap();
                assert_eq!(threshold, 0.);
                close(spectrum.values[0], small);
                assert!(matches!(
                    gram_spectrum(&array![[1., 0.], [0., -1.]], &options),
                    Err(NumericalError::IndefiniteGram)
                ));
            }

            #[test]
            fn imaginary_and_squared_norm_roundoff_have_distinct_units() {
                let mut options = NumericalOptions::<R>::default();
                let allowance = options.imaginary.absolute;
                assert_eq!(
                    real_coordinate(Complex::new(3., allowance / 2.), 0., &options).unwrap(),
                    3.
                );
                assert!(matches!(
                    real_coordinate(Complex::new(3., allowance * 2.), 0., &options),
                    Err(NumericalError::NonReal)
                ));
                assert_eq!(
                    squared_norm(Complex::new(-R::EPSILON, 0.), 1., &options).unwrap(),
                    0.
                );
                assert!(matches!(
                    squared_norm(Complex::new(-1000. * R::EPSILON, 0.), 1., &options),
                    Err(NumericalError::NegativeSquaredNorm)
                ));
                // A tiny positive squared norm is not pruned by a residual tolerance.
                let tiny = R::MIN_POSITIVE;
                assert_eq!(
                    squared_norm(Complex::new(tiny, 0.), tiny, &options).unwrap(),
                    tiny
                );
                options.roundoff = Tolerance {
                    absolute: 0.,
                    relative: 0.,
                };
                assert!(matches!(
                    squared_norm(Complex::new(-R::EPSILON, 0.), 1., &options),
                    Err(NumericalError::NegativeSquaredNorm)
                ));
            }

            #[test]
            fn invalid_and_nonfinite_inputs_fail_before_lapack() {
                let mut options = NumericalOptions::<R>::default();
                for value in [R::NAN, R::INFINITY, -1.] {
                    options.rank_relative = Some(value);
                    assert!(options.validate().is_err());
                    assert!(rank_nullspace(&Array2::<R>::eye(2), &options).is_err());
                }
                let options = NumericalOptions::<R>::default();
                assert!(matches!(
                    rank_nullspace(&array![[R::NAN]], &options),
                    Err(NumericalError::NonFinite(_))
                ));
                assert!(matches!(
                    symmetric_eigen(&array![[R::INFINITY]], &options),
                    Err(NumericalError::NonFinite(_))
                ));
                assert!(matches!(
                    solve(&array![[1.]], &array![R::INFINITY], &options),
                    Err(NumericalError::NonFinite(_))
                ));
                assert!(options.residual.check(R::NAN, 1.).is_err());
                assert!(options.residual.check(0., -1.).is_err());
                assert!(
                    Tolerance {
                        absolute: 0.,
                        relative: R::MAX
                    }
                    .check(0., 2.)
                    .is_err()
                );
            }

            #[test]
            fn diagnostics_record_inclusive_bounds_and_dimension_scaled_defaults() {
                let options = NumericalOptions::<R>::default();
                let tolerance = Tolerance {
                    absolute: 1.,
                    relative: 0.5,
                };
                let check = tolerance.check(2., 2.).unwrap();
                assert_eq!(check.threshold, 2.);
                assert_eq!(check.scale, 2.);
                assert_eq!(check.tolerance, tolerance);
                assert!(check.passed());
                assert!(!tolerance.check(2.5, 2.).unwrap().passed());
                let d = 1_000_000;
                let expected = options.residual.relative.max(d as R * R::EPSILON);
                assert_eq!(options.rank_cutoff(2, d).unwrap(), expected);
                assert_eq!(options.eigenvalue_cutoff(d).unwrap(), expected);
                let mut changed = options;
                changed.residual = Tolerance {
                    absolute: 1.,
                    relative: 1.,
                };
                assert_eq!(
                    changed.rank_cutoff(2, 2).unwrap(),
                    options.rank_cutoff(2, 2).unwrap()
                );
            }

            #[test]
            fn stable_norm_and_noncontiguous_input_are_supported() {
                assert!(norm([R::MAX / 4., R::MAX / 4.]).is_finite());
                let a: Array2<R> = array![[1., 9., 0., 9.], [0., 9., 1., 9.]];
                let a = a.slice_axis_move(ndarray::Axis(1), ndarray::Slice::new(0, None, 2));
                assert!(!a.is_standard_layout());
                let result = rank_nullspace(&a, &NumericalOptions::<R>::default()).unwrap();
                assert_eq!(result.rank, 2);
                let tiny = R::from_bits(1);
                assert_eq!(
                    symmetric_eigen(&array![[tiny]], &NumericalOptions::<R>::default())
                        .unwrap()
                        .values[0],
                    tiny
                );
            }
        }
    };
}
precision_tests!(single, f32);
precision_tests!(double, f64);
