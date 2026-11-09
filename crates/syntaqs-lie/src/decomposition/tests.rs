use super::*;
use ndarray::array;
use num_complex::Complex;
use syntaqs_core::{HilbertSpace, LocalOperator, Operator};
use syntaqs_spin_half::SpinHalfOperator as Pauli;

macro_rules! precision_tests {
    ($module:ident,$real:ty) => {
        mod $module {
            use super::*;
            type R = $real;
            type E = Operator<u8, Pauli, Complex<R>>;
            fn family() -> OperatorFamily<E> {
                let a = HilbertSpace::factor(0u8);
                let b = HilbertSpace::factor(1u8);
                let space = HilbertSpace::tensor_all([a.clone(), b.clone()]).unwrap();
                OperatorFamily::new_in(
                    &space,
                    [a, b].iter().flat_map(|factor| {
                        [Pauli::Sx, Pauli::Sy, Pauli::Sz]
                            .map(|p| LocalOperator::primitive(p).at(factor).unwrap())
                    }),
                )
                .unwrap()
            }
            #[test]
            fn an_accidental_collision_retries_and_exhaustion_retains_all_attempts() {
                let f = family();
                let numerical = NumericalOptions::<R>::default();
                let adj = f.structure_constants(&numerical).unwrap().adjoint();
                let settings = DecompositionOptions {
                    seed: 17,
                    max_attempts: 8,
                };
                let mut count = 0;
                let mut random = Sampler(settings.seed);
                let result = decompose_with_sampler(&adj, &settings, |n, basis| {
                    count += 1;
                    if count == 1 {
                        Array2::zeros((n, n))
                    } else {
                        random.sample(n, basis)
                    }
                })
                .unwrap();
                assert_eq!(result.simple_ideals().len(), 2);
                assert!(result.diagnostics().attempts.len() >= 2);
                assert_eq!(
                    result.diagnostics().attempts[0].failure,
                    Some(DecompositionFailure::ComponentCount {
                        expected: 2,
                        found: 1
                    })
                );
                assert!(
                    result
                        .diagnostics()
                        .attempts
                        .last()
                        .unwrap()
                        .failure
                        .is_none()
                );
                assert!(result.diagnostics().validation.passed());
                let settings = DecompositionOptions {
                    max_attempts: 3,
                    ..settings
                };
                let error = decompose_with_sampler(&adj, &settings, |n, _| Array2::zeros((n, n)))
                    .err()
                    .unwrap();
                let LieError::DecompositionExhausted { seed, attempts } = error else {
                    panic!("expected exhaustion")
                };
                assert_eq!(seed, settings.seed);
                assert_eq!(attempts.len(), 3);
                assert!(attempts.iter().all(|a| a.failure
                    == Some(DecompositionFailure::ComponentCount {
                        expected: 2,
                        found: 1
                    })));
            }
            #[test]
            fn incomplete_and_nonideal_splits_cannot_pass_invariant_validation() {
                let f = family();
                let options = NumericalOptions::<R>::default();
                let adj = f.structure_constants(&options).unwrap().adjoint();
                let center = Array2::zeros((6, 0));
                let merged =
                    validate_candidate(adj.matrices(), &center, &[Array2::eye(6)], &options)
                        .unwrap();
                assert!(merged.span.passed());
                assert!(merged.orthonormality.passed());
                assert!(merged.ideal_invariance[0].passed());
                assert_eq!(
                    merged.simple_commutants[0].columns - merged.simple_commutants[0].rank,
                    2
                );
                assert!(!merged.passed());
                let a = Array2::from_shape_fn((6, 2), |(i, j)| if i == j { 1. } else { 0. });
                let b = Array2::from_shape_fn((6, 4), |(i, j)| if i == j + 2 { 1. } else { 0. });
                let invalid =
                    validate_candidate(adj.matrices(), &center, &[a, b], &options).unwrap();
                assert!(invalid.span.passed());
                assert!(invalid.orthonormality.passed());
                assert!(invalid.ideal_invariance.iter().any(|x| !x.passed()));
                assert!(invalid.cross_commutation.iter().any(|x| !x.passed()));
                assert!(!invalid.passed());
            }
            #[test]
            fn eigenvalue_grouping_is_inclusive_and_sampler_accepts_empty_commutants() {
                let eigenvalues = array![-4., -3., 0., 1.];
                assert_eq!(groups(&eigenvalues, 1.), vec![0..2, 2..4]);
                assert_eq!(groups(&eigenvalues, 0.), vec![0..1, 1..2, 2..3, 3..4]);
                assert_eq!(
                    groups(&Array1::<R>::zeros(0), 0.),
                    Vec::<Range<usize>>::new()
                );
                let mut a = Sampler(0);
                let mut b = Sampler(0);
                for _ in 0..100 {
                    let x = a.next();
                    assert_eq!(x, b.next());
                    assert!((-1.0..1.0).contains(&x));
                }
                assert_eq!(a.sample::<R>(3, &[]), Array2::zeros((3, 3)));
            }
        }
    };
}
precision_tests!(single, f32);
precision_tests!(double, f64);
