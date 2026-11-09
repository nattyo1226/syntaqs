//! Complete ideal spans compared with independent su(2) block references.
extern crate blas_provider as _;

use ndarray::Array2;
use num_complex::Complex;
use syntaqs_core::{HilbertSpace, LocalOperator, Operator};
use syntaqs_lie::{
    DecompositionFailure, DecompositionOptions, LieDecomposition, LieError, NumericalError,
    NumericalOptions, OperatorFamily,
};
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
            fn projector(n: usize, indices: &[usize]) -> Array2<R> {
                Array2::from_shape_fn((n, n), |(i, j)| {
                    if i == j && indices.contains(&i) {
                        1.
                    } else {
                        0.
                    }
                })
            }
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
            fn case(kind: u8) -> (OperatorFamily<Placed>, Vec<[usize; 3]>, Vec<usize>) {
                let factors: Vec<_> = (0..3u8).map(HilbertSpace::factor).collect();
                let space = HilbertSpace::tensor_all(factors.iter().cloned()).unwrap();
                let a: Vec<_> = [Pauli::Sx, Pauli::Sy, Pauli::Sz]
                    .into_iter()
                    .map(|p| Local::primitive(p).at(&factors[0]).unwrap())
                    .collect();
                let b: Vec<_> = [Pauli::Sx, Pauli::Sy, Pauli::Sz]
                    .into_iter()
                    .map(|p| Local::primitive(p).at(&factors[1]).unwrap())
                    .collect();
                let c: Vec<_> = [Pauli::Sx, Pauli::Sy, Pauli::Sz]
                    .into_iter()
                    .map(|p| Local::primitive(p).at(&factors[2]).unwrap())
                    .collect();
                let (operators, blocks, center) = match kind {
                    0 => (a, vec![[0, 1, 2]], vec![]),
                    1 => (
                        a.into_iter().chain(b).collect(),
                        vec![[0, 1, 2], [3, 4, 5]],
                        vec![],
                    ),
                    2 => (
                        a.into_iter().chain([Placed::identity()]).collect(),
                        vec![[0, 1, 2]],
                        vec![3],
                    ),
                    3 => (
                        vec![Placed::identity(), a[2].clone(), b[2].clone()],
                        vec![],
                        vec![0, 1, 2],
                    ),
                    4 => (
                        a.into_iter()
                            .chain([Placed::identity(), b[2].clone()])
                            .collect(),
                        vec![[0, 1, 2]],
                        vec![3, 4],
                    ),
                    5 => (vec![Placed::identity()], vec![], vec![0]),
                    6 => (
                        a.into_iter()
                            .chain(b)
                            .chain([Placed::identity(), c[2].clone()])
                            .collect(),
                        vec![[0, 1, 2], [3, 4, 5]],
                        vec![6, 7],
                    ),
                    7 => (
                        a.into_iter().chain(b).chain(c).collect(),
                        vec![[0, 1, 2], [3, 4, 5], [6, 7, 8]],
                        vec![],
                    ),
                    _ => unreachable!(),
                };
                (
                    OperatorFamily::new_in(&space, operators).unwrap(),
                    blocks,
                    center,
                )
            }
            fn check_result(
                result: &LieDecomposition<'_, Placed>,
                u: &Array2<R>,
                blocks: &[[usize; 3]],
                center: &[usize],
            ) {
                let n = u.nrows();
                assert_eq!(result.simple_ideals().len(), blocks.len());
                assert_eq!(result.center().is_none(), center.is_empty());
                assert!(result.diagnostics().validation.passed());
                let mut total = Array2::zeros((n, n));
                if let Some(component) = result.center() {
                    assert_eq!(component.family().len(), center.len());
                    let physical = u.dot(component.coordinates());
                    let actual = physical.dot(&physical.t());
                    matrix_close(&actual, &projector(n, center));
                    total = total + actual;
                    for a in reference(n, blocks) {
                        matrix_close(&a.dot(&physical), &Array2::zeros((n, center.len())));
                    }
                }
                let mut matched = vec![false; blocks.len()];
                let mut ideal_axes = Vec::new();
                for component in result.simple_ideals() {
                    assert_eq!(component.family().len(), 3);
                    let physical = u.dot(component.coordinates());
                    let p = physical.dot(&physical.t());
                    let block = blocks
                        .iter()
                        .position(|block| {
                            p.iter()
                                .zip(projector(n, block))
                                .all(|(&a, b)| (a - b).abs() < options().residual.absolute * 10.)
                        })
                        .expect("unexpected ideal span");
                    assert!(!matched[block]);
                    matched[block] = true;
                    for a in reference(n, blocks) {
                        let action = a.dot(&physical);
                        matrix_close(&(&action - p.dot(&action)), &Array2::zeros((n, 3)));
                    }
                    total = total + p;
                    ideal_axes.push(physical);
                }
                matrix_close(&total, &Array2::eye(n));
                let ambient = reference(n, blocks);
                for (i, a) in ideal_axes.iter().enumerate() {
                    for b in &ideal_axes[i + 1..] {
                        for x in a.columns() {
                            let action = Array2::from_shape_fn((n, n), |(row, col)| {
                                (0..n).map(|k| x[k] * ambient[k][(row, col)]).sum()
                            });
                            matrix_close(&action.dot(b), &Array2::zeros((n, 3)));
                        }
                    }
                }
                for component in result.center().into_iter().chain(result.simple_ideals()) {
                    assert!(std::ptr::eq(component.source(), result.family()));
                    assert_eq!(component.options(), result.options());
                    assert!(
                        component
                            .family()
                            .context()
                            .is_compatible_with(result.family().context())
                    );
                    let report = component.family().orthonormality(result.options()).unwrap();
                    assert!(
                        report.passed(),
                        "orthonormality {:?}; Hermiticity {:?}; independence {:?}",
                        report.check(),
                        report.validation().hermiticity(),
                        report.validation().independence()
                    );
                    for (j, operator) in component.family().iter().enumerate() {
                        let coordinates = result
                            .family()
                            .coefficients(operator, result.options())
                            .unwrap();
                        for (&a, &b) in coordinates
                            .values()
                            .iter()
                            .zip(component.coordinates().column(j))
                        {
                            close(a, b);
                        }
                    }
                }
            }

            #[test]
            fn compact_examples_recover_full_center_and_all_simple_ideal_spans() {
                let settings = DecompositionOptions::default();
                assert_eq!(settings.seed, 0);
                assert_eq!(settings.max_attempts, 32);
                for kind in 0..8 {
                    let (f, blocks, center) = case(kind);
                    let numerical = options();
                    let result = f.decompose(&numerical, &settings).unwrap();
                    assert!(std::ptr::eq(result.family(), &f));
                    assert_eq!(result.options(), &numerical);
                    assert_eq!(result.decomposition_options(), &settings);
                    check_result(&result, &Array2::eye(f.len()), &blocks, &center);
                    assert_eq!(result.diagnostics().attempts.is_empty(), blocks.is_empty());
                    assert_eq!(
                        result.diagnostics().semisimple_commutant.is_none(),
                        blocks.is_empty()
                    );
                    if let Some(report) = &result.diagnostics().semisimple_commutant {
                        assert_eq!(report.columns - report.rank, blocks.len());
                    }
                    for report in &result.diagnostics().validation.simple_commutants {
                        assert_eq!(report.columns - report.rank, 1);
                    }
                    for report in &result.diagnostics().validation.ideal_centers {
                        assert_eq!(report.columns, report.rank);
                    }
                }
            }

            #[test]
            fn real_nonorthonormal_changes_preserve_projectors_and_invariants() {
                for kind in [0, 1, 2, 3, 4, 6, 7] {
                    let (f, blocks, center) = case(kind);
                    let n = f.len();
                    let u = Array2::from_shape_fn((n, n), |(i, j)| {
                        if i == j {
                            2.
                        } else if j == i + 1 {
                            0.25
                        } else {
                            0.
                        }
                    });
                    let transformed = f.transform(u.view()).unwrap();
                    let g = transformed.family();
                    let settings = DecompositionOptions {
                        seed: 42,
                        max_attempts: 16,
                    };
                    let result = g
                        .structure_constants(&options())
                        .unwrap()
                        .adjoint()
                        .decompose(&settings)
                        .unwrap();
                    check_result(&result, &u, &blocks, &center);
                }
            }

            #[test]
            fn cyclic_nonorthonormal_changes_use_explicit_residual_settings() {
                for kind in [1, 4, 6, 7] {
                    let (f, blocks, center) = case(kind);
                    let n = f.len();
                    let u = Array2::from_shape_fn((n, n), |(i, j)| {
                        if i == j {
                            1. + i as R * 0.125
                        } else if j == (i + 1) % n {
                            0.25
                        } else {
                            0.
                        }
                    });
                    let changed = f.transform(u.view()).unwrap();
                    let mut numerical = options();
                    numerical.residual.absolute = numerical.residual.absolute.max(1e-8);
                    numerical.residual.relative = numerical.residual.relative.max(1e-8);
                    let result = changed
                        .family()
                        .decompose(&numerical, &DecompositionOptions::default())
                        .unwrap();
                    assert_eq!(result.options(), &numerical);
                    check_result(&result, &u, &blocks, &center);
                }
            }

            #[test]
            fn seed_repeats_within_the_environment_and_different_seeds_preserve_spans() {
                let (f, blocks, center) = case(6);
                let numerical = options();
                for seed in [0, 1, u64::MAX] {
                    let settings = DecompositionOptions {
                        seed,
                        max_attempts: 16,
                    };
                    let first = f.decompose(&numerical, &settings).unwrap();
                    let second = f.decompose(&numerical, &settings).unwrap();
                    assert_eq!(
                        first.diagnostics().attempts.len(),
                        second.diagnostics().attempts.len()
                    );
                    for (a, b) in first
                        .diagnostics()
                        .attempts
                        .iter()
                        .zip(&second.diagnostics().attempts)
                    {
                        assert_eq!(a.eigenvalues, b.eigenvalues);
                        assert_eq!(a.groups, b.groups);
                        assert_eq!(a.failure, b.failure);
                    }
                    for (a, b) in first.simple_ideals().iter().zip(second.simple_ideals()) {
                        assert_eq!(a.coordinates(), b.coordinates());
                    }
                    check_result(&first, &Array2::eye(f.len()), &blocks, &center);
                }
            }

            #[test]
            fn insufficient_spectral_separation_exhausts_and_records_effective_gaps() {
                let (f, _, _) = case(1);
                let mut numerical = options();
                numerical.eigenvalue_relative = Some(100.);
                let settings = DecompositionOptions {
                    seed: 123,
                    max_attempts: 3,
                };
                let error = f.decompose(&numerical, &settings).err().unwrap();
                let LieError::DecompositionExhausted { seed, attempts } = error else {
                    panic!("expected exhaustion")
                };
                assert_eq!(seed, 123);
                assert_eq!(attempts.len(), 3);
                for attempt in attempts {
                    assert_eq!(attempt.relative_gap, 100.);
                    let scale = attempt
                        .eigenvalues
                        .iter()
                        .fold(0. as R, |m, &x| m.max(x.abs()));
                    close(attempt.gap_threshold, 100. * scale);
                    assert_eq!(attempt.groups, vec![0..6]);
                    assert_eq!(
                        attempt.failure,
                        Some(DecompositionFailure::ComponentCount {
                            expected: 2,
                            found: 1
                        })
                    );
                    assert!(attempt.validation.is_none());
                }
                let (simple, _, _) = case(0);
                assert!(
                    simple
                        .decompose(&numerical, &settings)
                        .unwrap()
                        .diagnostics()
                        .validation
                        .passed()
                );
                numerical = options();
                numerical.eigenvalue_relative = Some(0.01);
                let result = f
                    .decompose(
                        &numerical,
                        &DecompositionOptions {
                            max_attempts: 32,
                            ..settings
                        },
                    )
                    .unwrap();
                assert!(
                    result
                        .diagnostics()
                        .attempts
                        .iter()
                        .all(|a| a.relative_gap == 0.01)
                );
            }

            #[test]
            fn components_can_be_orthogonalized_without_losing_original_coordinates() {
                let (f, _, _) = case(6);
                let settings = DecompositionOptions::default();
                let numerical = options();
                let result = f.decompose(&numerical, &settings).unwrap();
                for component in result.center().into_iter().chain(result.simple_ideals()) {
                    for changed in [
                        component.orthogonalize(&numerical).unwrap(),
                        component.orthonormalize(&numerical).unwrap(),
                    ] {
                        assert!(std::ptr::eq(changed.source(), &f));
                        assert_eq!(changed.options(), &numerical);
                        matrix_close(
                            &changed.coordinates().dot(&changed.coordinates().t()),
                            &component.coordinates().dot(&component.coordinates().t()),
                        );
                        assert!(
                            changed
                                .family()
                                .orthonormality(&numerical)
                                .unwrap()
                                .passed()
                        );
                        for (j, op) in changed.family().iter().enumerate() {
                            let coords = f.coefficients(op, &numerical).unwrap();
                            for (&a, &b) in
                                coords.values().iter().zip(changed.coordinates().column(j))
                            {
                                close(a, b);
                            }
                        }
                    }
                }
            }

            #[test]
            fn invalid_settings_and_lie_preconditions_fail() {
                let numerical = options();
                let settings = DecompositionOptions::default();
                let (f, _, _) = case(0);
                let invalid = DecompositionOptions {
                    max_attempts: 0,
                    ..settings
                };
                assert!(matches!(
                    f.decompose(&numerical, &invalid),
                    Err(LieError::Numerical(NumericalError::InvalidSetting(_)))
                ));
                let (abelian, _, _) = case(3);
                assert!(abelian.decompose(&numerical, &invalid).is_err());
                let mut invalid_num = numerical;
                invalid_num.eigenvalue_relative = Some(R::NAN);
                assert!(f.decompose(&invalid_num, &settings).is_err());
                invalid_num = numerical;
                invalid_num.rank_relative = Some(R::NAN);
                assert!(f.decompose(&invalid_num, &settings).is_err());
                let x = Local::primitive(Pauli::Sx);
                let z = Local::primitive(Pauli::Sz);
                let nonclosed = OperatorFamily::new([x.clone(), z]).unwrap();
                assert!(matches!(
                    nonclosed.decompose(&numerical, &settings),
                    Err(LieError::NonClosedPair { .. })
                ));
                let dependent = OperatorFamily::new([x.clone(), x]).unwrap();
                assert!(matches!(
                    dependent.decompose(&numerical, &settings),
                    Err(LieError::Numerical(NumericalError::SingularBasis { .. }))
                ));
                let nonhermitian = OperatorFamily::new([Local::primitive(Pauli::Sp)]).unwrap();
                assert!(matches!(
                    nonhermitian.decompose(&numerical, &settings),
                    Err(LieError::NonHermitianElement { .. })
                ));
            }

            #[test]
            fn graded_ordinary_brackets_decompose_with_full_identity_center() {
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
                let f = OperatorFamily::new_in(&space, [x, y, z, Operator::identity()]).unwrap();
                let result = f
                    .decompose(&options(), &DecompositionOptions::default())
                    .unwrap();
                assert_eq!(result.simple_ideals().len(), 1);
                assert_eq!(result.center().unwrap().family().len(), 1);
                let c = result.center().unwrap().coordinates();
                matrix_close(&c.dot(&c.t()), &projector(4, &[3]));
                let q = result.simple_ideals()[0].coordinates();
                matrix_close(&q.dot(&q.t()), &projector(4, &[0, 1, 2]));
                assert!(result.diagnostics().validation.passed());
            }
        }
    };
}
precision_tests!(single, f32);
precision_tests!(double, f64);
