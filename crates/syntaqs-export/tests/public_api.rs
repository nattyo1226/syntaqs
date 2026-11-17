use num_complex::Complex;
use syntaqs_core::expression::local::LocalOperator;
use syntaqs_export::prelude::*;
use syntaqs_spin_half::{SpinHalfOperator, SpinHalfState, SzBasis};

#[test]
fn prelude_supports_matrix_export() {
    let operator = LocalOperator::<_, Complex<f64>>::primitive(SpinHalfOperator::Sz);
    let matrix: Exported<Vec<Complex<f64>>, _> = (&operator)
        .export(MatrixArgs::<SpinHalfState, _>::new(&SzBasis))
        .unwrap();
    assert_eq!(matrix.metadata().shape(), [2, 2]);
    assert_eq!(matrix.data().len(), 4);
}
