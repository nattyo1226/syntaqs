//! Analyze su(2) + su(2) + u(1), without physical matrices or a state basis.
//!
//! The `lie` feature selects the internal application provider through blas-src.
//! System OpenBLAS (including LAPACK) and pkg-config must be available.

extern crate blas_provider as _;

use num_complex::Complex;
use syntaqs_core::{HilbertSpace, LocalOperator, Operator};
use syntaqs_lie::{DecompositionOptions, NumericalOptions, OperatorFamily, Tolerance};
use syntaqs_spin_half::SpinHalfOperator as Pauli;

type Local = LocalOperator<Pauli, Complex<f64>>;
type Placed = Operator<&'static str, Pauli, Complex<f64>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let left = HilbertSpace::factor("left");
    let right = HilbertSpace::factor("right");
    let space = left.tensor(&right)?;
    let mut operators = Vec::new();
    for factor in [&left, &right] {
        for primitive in [Pauli::Sx, Pauli::Sy, Pauli::Sz] {
            operators.push(Local::primitive(primitive).at(factor)?);
        }
    }
    operators.push(Placed::identity());
    let family = OperatorFamily::new_in(&space, operators)?;

    // Overrides are explicit. They do not change independent rank/Gram settings.
    let options = NumericalOptions {
        residual: Tolerance {
            absolute: 1e-10,
            relative: 1e-10,
        },
        ..NumericalOptions::<f64>::default()
    };
    let sampling = DecompositionOptions {
        seed: 816,
        max_attempts: 16,
    };
    let gram = family.gram(&options)?;
    assert!(gram.validate()?.passed());
    let factor = gram.factorize()?;
    println!("family dimension: {}", family.len());

    let target = (&family.operators()[0] + &family.operators()[1])?;
    let coordinates = factor.coefficients(&target)?;
    let reconstructed = coordinates.reconstruct()?;
    assert!(family.in_span(&reconstructed, &options)?);
    println!("target coordinates: {:?}", coordinates.values().to_vec());

    // Project X_left + Y_left onto [X_left,Z_left]: coefficients [1,0], residual 1.
    let partial = family.select(&[0, 2])?;
    let projection = partial.family().project(&target, &options)?;
    assert_eq!(
        projection.coordinates().values().as_slice().unwrap(),
        &[1., 0.]
    );
    assert_eq!(projection.residual_norm(), 1.);
    assert!(!projection.in_span());
    println!(
        "partial projection: {:?}, residual: {}",
        projection.coordinates().values().to_vec(),
        projection.residual_norm()
    );

    // Reuse the same factorization for closure and retained structure constants.
    let closure = factor.closure()?;
    assert!(closure.passed());
    let constants = closure.structure_constants()?;
    assert_eq!(constants.values()[(2, 0, 1)], 2.);
    let adjoint = constants.adjoint();
    let killing = adjoint.killing_form()?;
    for i in 0..7 {
        assert_eq!(killing.values()[(i, i)], if i < 6 { -8. } else { 0. });
    }
    println!(
        "f[2,0,1]: {}, Killing diagonal: {:?}",
        constants.values()[(2, 0, 1)],
        killing.values().diag().to_vec()
    );
    let commutant = adjoint.commutant()?;
    assert_eq!(commutant.dimension(), 3);
    println!("adjoint commutant dimension: {}", commutant.dimension());

    let decomposition = adjoint.decompose(&sampling)?;
    assert!(decomposition.diagnostics().validation.passed());
    let center = decomposition.center().unwrap();
    assert_eq!(center.family().len(), 1);
    let dimensions: Vec<_> = decomposition
        .simple_ideals()
        .iter()
        .map(|ideal| ideal.family().len())
        .collect();
    assert_eq!(dimensions, [3, 3]);
    println!(
        "center dimension: {}, simple ideal dimensions: {:?}",
        center.family().len(),
        dimensions
    );
    println!(
        "seed: {}, spectral attempts: {}",
        sampling.seed,
        decomposition.diagnostics().attempts.len()
    );
    Ok(())
}
