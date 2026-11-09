//! Independent downstream binary: no ndarray-linalg provider features.
#[cfg(feature = "system-openblas")]
extern crate blas_provider as _;

use ndarray::array;
use ndarray_linalg::{Eigh, Solve, UPLO};
use syntaqs_lie::NumericalOptions;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    macro_rules! check {
        ($real:ty) => {{
            let options = NumericalOptions::<$real>::default();
            let a = array![[2.0 as $real, 1.0], [1.0, 2.0]];
            let b = array![3.0 as $real, 3.0];
            let x = a.solve(&b)?;
            let (eigenvalues, _) = a.eigh(UPLO::Lower)?;
            for residual in [
                x[0] - 1.0,
                x[1] - 1.0,
                eigenvalues[0] - 1.0,
                eigenvalues[1] - 3.0,
            ] {
                assert!(options.residual.check(residual.abs(), 3.0)?.passed());
            }
            assert!(
                options
                    .residual
                    .check((a.dot(&x) - &b)[0].abs(), 3.0)?
                    .passed()
            );
        }};
    }
    check!(f32);
    check!(f64);
    println!("downstream f32/f64 BLAS and LAPACK linkage passed");
    Ok(())
}
