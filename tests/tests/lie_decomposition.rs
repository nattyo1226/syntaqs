//! Higher-dimensional simple ideals and unequal-dimensional direct sums.
#![cfg(feature = "lie")]

extern crate blas_provider as _;

use num_complex::Complex;
use syntaqs_core::{HilbertSpace, LocalOperator};
use syntaqs_lie::{DecompositionOptions, NumericalOptions, OperatorFamily};
use syntaqs_tests::three_level::{Axis, Operator as MatrixUnit};

macro_rules! precision_tests {
    ($module:ident, $real:ty) => {
        mod $module {
            use super::*;
            type R = $real;
            type C = Complex<R>;
            type Local = LocalOperator<MatrixUnit, C>;

            // Six off-diagonal Gell-Mann generators followed by two traceless
            // diagonals. Their normalized HS Gram matrix is diag(2/3,...,2/3,2).
            // These expected norms follow directly from matrix entries, without
            // using the production contraction or its Gram matrix as an oracle.
            fn su3() -> Vec<Local> {
                use Axis::{A, B, C};
                let e = |i, j| Local::primitive(MatrixUnit::Unit(i, j));
                let mut generators = Vec::new();
                for (i, j) in [(A, B), (A, C), (B, C)] {
                    generators.push(e(i, j) + e(j, i));
                    generators.push((e(i, j) - e(j, i)).scaled(Complex::new(0., -1.)));
                }
                generators.push(e(A, A) - e(B, B));
                generators.push(e(A, A) + e(B, B) - e(C, C).scaled(2.));
                generators
            }

            fn check(mixed: bool) {
                let a = HilbertSpace::factor(0u8);
                let b = HilbertSpace::factor(1u8);
                let space = if mixed {
                    a.tensor(&b).unwrap()
                } else {
                    a.clone()
                };
                let generators = su3();
                let mut operators: Vec<_> =
                    generators.iter().map(|op| op.at(&a).unwrap()).collect();
                if mixed {
                    // An su(2) subalgebra on the second three-level factor
                    // commutes with the full su(3) on the first factor.
                    for &i in &[0, 1, 6] {
                        operators.push(generators[i].at(&b).unwrap());
                    }
                }
                let n = operators.len();
                // Mix the two ideals so a split by contiguous input indices
                // cannot satisfy the reference projector checks below.
                if mixed {
                    let x = operators[0].clone();
                    let y = operators[8].clone();
                    operators[0] = (&x + &y).unwrap();
                    operators[8] = (&x - &y).unwrap();
                }
                let family = OperatorFamily::new_in(&space, operators).unwrap();
                let options = NumericalOptions::<R>::default();
                let result = family
                    .decompose(
                        &options,
                        &DecompositionOptions {
                            seed: 42,
                            max_attempts: 16,
                        },
                    )
                    .unwrap();
                assert!(result.center().is_none());
                assert!(result.diagnostics().validation.passed());
                let mut dimensions: Vec<_> = result
                    .simple_ideals()
                    .iter()
                    .map(|ideal| ideal.family().len())
                    .collect();
                dimensions.sort_unstable();
                assert_eq!(dimensions, if mixed { vec![3, 8] } else { vec![8] });
                for ideal in result.simple_ideals() {
                    assert!(ideal.family().orthonormality(&options).unwrap().passed());
                    let q = ideal.coordinates();
                    // Convert source coordinates back to the unmixed generators,
                    // then to Euclidean coordinates with the independent HS norms.
                    let physical: Vec<Vec<R>> = (0..n)
                        .map(|i| {
                            (0..q.ncols())
                                .map(|j| {
                                    let value = if mixed && i == 0 {
                                        q[(0, j)] + q[(8, j)]
                                    } else if mixed && i == 8 {
                                        q[(0, j)] - q[(8, j)]
                                    } else {
                                        q[(i, j)]
                                    };
                                    let norm_squared: R = if i == 7 { 2. } else { 2. / 3. };
                                    value * norm_squared.sqrt()
                                })
                                .collect()
                        })
                        .collect();
                    // Compare spans, independent of sampled basis signs/order.
                    // The eight-dimensional ideal occupies axes 0..8, and the
                    // three-dimensional ideal occupies axes 8..11.
                    for i in 0..n {
                        for j in 0..n {
                            let actual: R = physical[i]
                                .iter()
                                .zip(&physical[j])
                                .map(|(a, b)| a * b)
                                .sum();
                            let belongs = if ideal.family().len() == 8 {
                                i < 8
                            } else {
                                i >= 8
                            };
                            let expected = if i == j && belongs { 1. } else { 0. };
                            let tolerance: R = if R::EPSILON > 1e-10 { 2e-5 } else { 1e-12 };
                            assert!(
                                (actual - expected).abs() <= tolerance,
                                "projector[{i},{j}]: {actual} != {expected}"
                            );
                        }
                    }
                }
            }

            #[test]
            fn su3_remains_one_eight_dimensional_simple_ideal() {
                check(false);
            }

            #[test]
            fn mixed_su3_and_su2_recover_independent_ideal_projectors() {
                check(true);
            }
        }
    };
}
precision_tests!(single, f32);
precision_tests!(double, f64);
