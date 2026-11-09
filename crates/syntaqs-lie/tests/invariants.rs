//! Real adjoint constraints checked against independent su(2) block matrices.
extern crate blas_provider as _;

use ndarray::Array2;
use ndarray_linalg::Inverse;
use num_complex::Complex;
use syntaqs_core::{HilbertSpace, LocalOperator, Operator};
use syntaqs_lie::{LieError, NumericalError, NumericalOptions, OperatorFamily, Tolerance};
use syntaqs_spin_half::SpinHalfOperator as Pauli;

macro_rules! precision_tests {
    ($module:ident,$real:ty) => {
        mod $module {
            use super::*;
            type R = $real;
            type C = Complex<R>;
            type Local = LocalOperator<Pauli, C>;
            type Placed = Operator<u8, Pauli, C>;
            fn options() -> NumericalOptions<R> {
                NumericalOptions::default()
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
            fn matrix_close(a: &Array2<R>, b: &Array2<R>) {
                assert_eq!(a.dim(), b.dim());
                for (&x, &y) in a.iter().zip(b) {
                    close(x, y);
                }
            }
            fn frobenius(a: &Array2<R>, b: &Array2<R>) -> R {
                a.iter().zip(b).map(|(&x, &y)| x * y).sum()
            }
            fn project_matrix(basis: &[Array2<R>], target: &Array2<R>) -> Array2<R> {
                basis.iter().fold(Array2::zeros(target.dim()), |sum, c| {
                    sum + c.mapv(|x| x * frobenius(c, target))
                })
            }
            // Each block is the standard three-dimensional real adjoint of Pauli su(2).
            fn reference(n: usize, blocks: &[[usize; 3]]) -> Vec<Array2<R>> {
                let mut result = vec![Array2::zeros((n, n)); n];
                for &[x, y, z] in blocks {
                    for (a, b, c) in [(x, y, z), (y, z, x), (z, x, y)] {
                        result[a][(c, b)] = 2.;
                        result[b][(c, a)] = -2.;
                    }
                }
                result
            }
            fn case(kind: u8) -> (OperatorFamily<Placed>, Vec<[usize; 3]>, Vec<usize>, usize) {
                let a = HilbertSpace::factor(0u8);
                let b = HilbertSpace::factor(1u8);
                let space = HilbertSpace::tensor_all([a.clone(), b.clone()]).unwrap();
                let first: Vec<_> = [Pauli::Sx, Pauli::Sy, Pauli::Sz]
                    .into_iter()
                    .map(|p| Local::primitive(p).at(&a).unwrap())
                    .collect();
                let second: Vec<_> = [Pauli::Sx, Pauli::Sy, Pauli::Sz]
                    .into_iter()
                    .map(|p| Local::primitive(p).at(&b).unwrap())
                    .collect();
                let (operators, blocks, center, dimension) = match kind {
                    0 => (first, vec![[0, 1, 2]], vec![], 1),
                    1 => (
                        first.into_iter().chain([Placed::identity()]).collect(),
                        vec![[0, 1, 2]],
                        vec![3],
                        2,
                    ),
                    2 => (
                        vec![Placed::identity(), first[2].clone(), second[2].clone()],
                        vec![],
                        vec![0, 1, 2],
                        9,
                    ),
                    3 => (
                        first.into_iter().chain(second).collect(),
                        vec![[0, 1, 2], [3, 4, 5]],
                        vec![],
                        2,
                    ),
                    4 => (
                        first
                            .into_iter()
                            .chain([Placed::identity(), second[2].clone()])
                            .collect(),
                        vec![[0, 1, 2]],
                        vec![3, 4],
                        5,
                    ),
                    5 => (vec![Placed::identity()], vec![], vec![0], 1),
                    _ => unreachable!(),
                };
                (
                    OperatorFamily::new_in(&space, operators).unwrap(),
                    blocks,
                    center,
                    dimension,
                )
            }
            fn expected_projector(n: usize, indices: &[usize]) -> Array2<R> {
                Array2::from_shape_fn((n, n), |(i, j)| {
                    if i == j && indices.contains(&i) {
                        1.
                    } else {
                        0.
                    }
                })
            }

            #[test]
            fn full_commutants_match_independent_constraints_and_expected_spans() {
                for kind in 0..6 {
                    let (f, blocks, center, dimension) = case(kind);
                    let n = f.len();
                    let settings = options();
                    let adj = f.structure_constants(&settings).unwrap().adjoint();
                    let result = adj.commutant().unwrap();
                    assert_eq!(result.dimension(), dimension, "case {kind}");
                    assert!(std::ptr::eq(result.family(), &f));
                    assert_eq!(result.options(), &settings);
                    let diagnostics = result.diagnostics();
                    assert_eq!((diagnostics.rows, diagnostics.columns), (n * n * n, n * n));
                    assert_eq!(diagnostics.rank + dimension, n * n);
                    assert_eq!(
                        diagnostics.relative_cutoff,
                        settings.rank_cutoff(n * n * n, n * n).unwrap()
                    );
                    close(
                        diagnostics.threshold,
                        diagnostics.relative_cutoff * diagnostics.singular_values[0],
                    );
                    assert_eq!(diagnostics.residuals.len(), dimension);
                    assert!(diagnostics.residuals.iter().all(|check| check.passed()));
                    for (i, c) in result.matrices().iter().enumerate() {
                        for a in reference(n, &blocks) {
                            matrix_close(&c.dot(&a), &a.dot(c));
                        }
                        for (j, d) in result.matrices().iter().enumerate() {
                            close(frobenius(c, d), if i == j { 1. } else { 0. });
                        }
                    }
                    // Scalar identity on each simple block plus arbitrary real endomorphisms of the center.
                    for block in blocks {
                        let expected = expected_projector(n, &block);
                        matrix_close(&project_matrix(result.matrices(), &expected), &expected);
                    }
                    for &i in &center {
                        for &j in &center {
                            let mut expected = Array2::zeros((n, n));
                            expected[(i, j)] = 1.;
                            matrix_close(&project_matrix(result.matrices(), &expected), &expected);
                        }
                    }
                }
            }

            #[test]
            fn centers_and_complements_are_complete_hs_orthogonal_and_reconstruct() {
                for kind in 0..6 {
                    let (f, blocks, center, _) = case(kind);
                    let n = f.len();
                    let settings = options();
                    let result = f.center(&settings).unwrap();
                    assert!(std::ptr::eq(result.family(), &f));
                    assert_eq!(result.options(), &settings);
                    let z = result.center_coordinates();
                    let s = result.complement_coordinates();
                    assert_eq!(z.dim(), (n, center.len()));
                    assert_eq!(s.dim(), (n, n - center.len()));
                    assert_eq!(result.center().is_none(), center.is_empty());
                    assert_eq!(result.complement().is_none(), center.len() == n);
                    matrix_close(&z.dot(&z.t()), &expected_projector(n, &center));
                    matrix_close(&(z.dot(&z.t()) + s.dot(&s.t())), &Array2::eye(n));
                    matrix_close(
                        &z.t().dot(s),
                        &Array2::zeros((center.len(), n - center.len())),
                    );
                    assert_eq!(result.centrality().len(), center.len());
                    assert!(result.centrality().iter().all(|check| check.passed()));
                    assert_eq!(result.diagnostics().rank, n - center.len());
                    assert_eq!(
                        (result.diagnostics().rows, result.diagnostics().columns),
                        (n * n, n)
                    );
                    for a in reference(n, &blocks) {
                        matrix_close(&a.dot(z), &Array2::zeros((n, center.len())));
                    }
                    for (component, coordinates) in [(result.center(), z), (result.complement(), s)]
                    {
                        if let Some(component) = component {
                            assert!(component.context().is_compatible_with(f.context()));
                            assert!(component.orthonormality(&settings).unwrap().passed());
                            for (j, operator) in component.iter().enumerate() {
                                let reconstructed = f.coefficients(operator, &settings).unwrap();
                                for (&actual, &expected) in
                                    reconstructed.values().iter().zip(coordinates.column(j))
                                {
                                    close(actual, expected);
                                }
                            }
                        }
                    }
                }
            }

            #[test]
            fn nonorthonormal_basis_changes_preserve_commutant_and_center_subspaces() {
                for kind in [0, 1, 2, 3, 4] {
                    let (f, blocks, center, dimension) = case(kind);
                    let n = f.len();
                    let settings = options();
                    let u = Array2::from_shape_fn((n, n), |(i, j)| {
                        if i == j {
                            1. + (i as R) * 0.25
                        } else if j == (i + 1) % n {
                            0.25
                        } else {
                            0.
                        }
                    });
                    let inverse = u.inv().unwrap();
                    let transformed = f.transform(u.view()).unwrap();
                    let g = transformed.family();
                    let adj = g.structure_constants(&settings).unwrap().adjoint();
                    let original_commutant = f.commutant(&settings).unwrap();
                    let commutant = adj.commutant().unwrap();
                    assert_eq!(commutant.dimension(), dimension);
                    for c in original_commutant.matrices() {
                        let expected = inverse.dot(c).dot(&u);
                        matrix_close(&project_matrix(commutant.matrices(), &expected), &expected);
                    }
                    let original_ad = reference(n, &blocks);
                    for j in 0..n {
                        let combination = original_ad
                            .iter()
                            .enumerate()
                            .fold(Array2::zeros((n, n)), |sum, (i, a)| {
                                sum + a.mapv(|v| v * u[(i, j)])
                            });
                        let expected = inverse.dot(&combination).dot(&u);
                        matrix_close(&expected, &adj.matrices()[j]);
                        for c in commutant.matrices() {
                            matrix_close(&c.dot(&expected), &expected.dot(c));
                        }
                    }
                    let split = adj.center().unwrap();
                    let z = u.dot(split.center_coordinates());
                    let s = u.dot(split.complement_coordinates());
                    matrix_close(&z.dot(&z.t()), &expected_projector(n, &center));
                    matrix_close(&(z.dot(&z.t()) + s.dot(&s.t())), &Array2::eye(n));
                    let gram = u.t().dot(&u);
                    matrix_close(
                        &split
                            .center_coordinates()
                            .t()
                            .dot(&gram)
                            .dot(split.center_coordinates()),
                        &Array2::eye(center.len()),
                    );
                    matrix_close(
                        &split
                            .orthonormal_transform()
                            .t()
                            .dot(&gram)
                            .dot(split.orthonormal_transform()),
                        &Array2::eye(n),
                    );
                    matrix_close(
                        &split
                            .inverse_orthonormal_transform()
                            .dot(split.orthonormal_transform()),
                        &Array2::eye(n),
                    );
                }
            }

            #[test]
            fn nullspaces_are_valid_for_abelian_algebras_and_overrides_reach_constraints() {
                let (abelian, _, _, _) = case(2);
                let mut settings = options();
                settings.rank_relative = Some(2.);
                let full = abelian.commutant(&settings).unwrap();
                assert_eq!(full.dimension(), 9);
                assert_eq!(full.diagnostics().relative_cutoff, 2.);
                assert_eq!(full.diagnostics().threshold, 0.);
                let split = abelian.center(&settings).unwrap();
                assert_eq!(split.center().unwrap().len(), 3);
                assert!(split.complement().is_none());
                let (simple, _, _, _) = case(0);
                assert!(matches!(
                    simple.commutant(&settings),
                    Err(LieError::ConstraintResidual { .. })
                ));
                assert!(matches!(
                    simple.center(&settings),
                    Err(LieError::ConstraintResidual { .. })
                ));
                settings.residual = Tolerance {
                    absolute: 100.,
                    relative: 0.,
                };
                let relaxed = simple.commutant(&settings).unwrap();
                assert_eq!(relaxed.dimension(), 9);
                assert_eq!(relaxed.options(), &settings);
                let split = simple.center(&settings).unwrap();
                assert_eq!(split.center().unwrap().len(), 3);
                assert!(split.complement().is_none());
                settings = options();
                settings.rank_relative = Some(0.);
                assert_eq!(
                    abelian
                        .commutant(&settings)
                        .unwrap()
                        .diagnostics()
                        .relative_cutoff,
                    0.
                );
                assert_eq!(
                    abelian.center(&settings).unwrap().center().unwrap().len(),
                    3
                );
            }

            #[test]
            fn invalid_candidates_and_settings_fail_before_invariant_analysis() {
                let x = Local::primitive(Pauli::Sx);
                let z = Local::primitive(Pauli::Sz);
                let settings = options();
                let nonclosed = OperatorFamily::new([x.clone(), z]).unwrap();
                assert!(matches!(
                    nonclosed.commutant(&settings),
                    Err(LieError::NonClosedPair { .. })
                ));
                assert!(matches!(
                    nonclosed.center(&settings),
                    Err(LieError::NonClosedPair { .. })
                ));
                let dependent = OperatorFamily::new([x.clone(), x]).unwrap();
                assert!(matches!(
                    dependent.commutant(&settings),
                    Err(LieError::Numerical(NumericalError::SingularBasis { .. }))
                ));
                assert!(dependent.center(&settings).is_err());
                let nonhermitian = OperatorFamily::new([Local::primitive(Pauli::Sp)]).unwrap();
                assert!(matches!(
                    nonhermitian.center(&settings),
                    Err(LieError::NonHermitianElement { .. })
                ));
                assert!(nonhermitian.commutant(&settings).is_err());
                let (f, _, _, _) = case(0);
                let mut invalid = settings;
                invalid.rank_relative = Some(R::NAN);
                assert!(f.commutant(&invalid).is_err());
                assert!(f.center(&invalid).is_err());
                invalid = settings;
                invalid.gram_relative = Some(2.);
                assert!(f.commutant(&invalid).is_err());
                assert!(f.center(&invalid).is_err());
                let nonfinite =
                    OperatorFamily::new([Local::primitive(Pauli::Sx).scaled(R::INFINITY)]).unwrap();
                assert!(nonfinite.center(&settings).is_err());
                assert!(nonfinite.commutant(&settings).is_err());
            }

            #[test]
            fn graded_odd_generator_algebra_has_scalar_commutant_and_no_center() {
                use syntaqs_fermion::FermionOperator as F;
                use syntaqs_lie::lie_bracket;
                let a = HilbertSpace::factor(0u8);
                let b = HilbertSpace::factor(1u8);
                let space = HilbertSpace::tensor_all_g([a.clone(), b.clone()]).unwrap();
                let x = LocalOperator::<F, C>::primitive(F::Mx)
                    .at(&a)
                    .unwrap()
                    .bind(&space)
                    .unwrap();
                let y = LocalOperator::<F, C>::primitive(F::Mx)
                    .at(&b)
                    .unwrap()
                    .bind(&space)
                    .unwrap();
                let z = lie_bracket(&x, &y).unwrap().scaled(0.5);
                let f = OperatorFamily::new([x, y, z]).unwrap();
                let settings = options();
                assert_eq!(f.commutant(&settings).unwrap().dimension(), 1);
                let split = f.center(&settings).unwrap();
                assert!(split.center().is_none());
                assert_eq!(split.complement().unwrap().len(), 3);
            }
        }
    };
}
precision_tests!(single, f32);
precision_tests!(double, f64);
