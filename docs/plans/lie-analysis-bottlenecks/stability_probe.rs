//! Check decomposition on the Pauli bases used in the Julia comparison.

extern crate blas_provider as _;

use std::error::Error;

use num_complex::Complex64;
use syntaqs_core::{expression::composite::Operator, space::HilbertSpace};
use syntaqs_lie::{
    LieError, decomposition::DecompositionOptions, family::OperatorFamily,
    numerical::options::NumericalOptions,
};
use syntaqs_spin_half::{SpinHalf, operator::SpinHalfOperator as Pauli};

fn main() -> Result<(), Box<dyn Error>> {
    let factors = [HilbertSpace::factor(0u8), HilbertSpace::factor(1u8)];
    let space = HilbertSpace::tensor_all(factors.iter().cloned())?;
    let operators: Vec<Operator<u8, Pauli, Complex64>> = factors
        .iter()
        .flat_map(|factor| {
            [Pauli::Sx, Pauli::Sy, Pauli::Sz]
                .map(|pauli| SpinHalf::<Complex64>::operator(pauli).at(factor))
        })
        .collect::<Result<_, _>>()?;
    let numerical = NumericalOptions::default();
    let sampling = DecompositionOptions {
        seed: 816,
        ..DecompositionOptions::default()
    };

    let mut mixed = operators.clone();
    mixed[3] = (&operators[3] + &operators[0])?;
    for (label, basis) in [
        ("orthonormal", operators.clone()),
        ("nonorthonormal", mixed),
    ] {
        let family = OperatorFamily::new_in(&space, basis)?;
        let split = family.decompose(&numerical, &sampling)?;
        let sizes: Vec<_> = split
            .simple_ideals()
            .iter()
            .map(|ideal| ideal.family().len())
            .collect();
        assert_eq!(sizes, [3, 3]);
        println!("{label} returned simple ideals of sizes {sizes:?}");
    }

    let family = OperatorFamily::new_in(&space, [operators[2].clone(), operators[5].clone()])?;
    let split = family.decompose(&numerical, &sampling)?;
    assert_eq!(split.center().unwrap().family().len(), 2);
    assert!(split.simple_ideals().is_empty());
    println!("abelian input returned a center of dimension 2");

    let perturbed = (&operators[2] + &operators[3])?.scaled(1.0 / 2.0_f64.sqrt());
    let family = OperatorFamily::new_in(
        &space,
        [operators[0].clone(), operators[1].clone(), perturbed],
    )?;
    let error = family
        .decompose(&numerical, &sampling)
        .err()
        .expect("nonclosed input should fail");
    assert!(matches!(error, LieError::NonClosedPair { .. }));
    println!("nonclosed input returned {error:?}");

    let almost_dependent = (&operators[0] + operators[1].clone().scaled(1e-7))?;
    let family = OperatorFamily::new_in(
        &space,
        [operators[0].clone(), almost_dependent, operators[2].clone()],
    )?;
    let report = family.validate(&numerical)?;
    let independence = report.independence();
    assert_eq!(independence.rank, 2);
    println!(
        "ill-conditioned basis classified as rank 2, Gram eigenvalues {:?}, cutoff {}",
        independence.eigenvalues, independence.threshold
    );
    Ok(())
}
