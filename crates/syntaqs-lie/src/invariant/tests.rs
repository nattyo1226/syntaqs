use super::*;
use ndarray::array;
use num_complex::Complex;
use syntaqs_core::LocalOperator;
use syntaqs_spin_half::SpinHalfOperator as Pauli;

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
