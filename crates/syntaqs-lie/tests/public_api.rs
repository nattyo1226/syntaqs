extern crate blas_provider as _;

use num_complex::Complex;
use syntaqs_core::expression::local::LocalOperator;
use syntaqs_lie::prelude::*;
use syntaqs_spin_half::operator::SpinHalfOperator as Pauli;

#[test]
fn prelude_constructs_a_family_and_its_bracket() {
    type Op = LocalOperator<Pauli, Complex<f64>>;
    let x = Op::primitive(Pauli::Sx);
    let y = Op::primitive(Pauli::Sy);
    let family = OperatorFamily::new([x.clone(), y.clone()]).unwrap();
    let options = NumericalOptions::<f64>::default();
    assert!(family.gram(&options).unwrap().validate().unwrap().passed());
    let bracket = lie_bracket(&x, &y).unwrap();
    assert_eq!(bracket.trace(), Complex::new(0.0, 0.0));
}
