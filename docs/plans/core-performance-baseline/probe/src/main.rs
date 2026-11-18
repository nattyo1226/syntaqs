mod fermion;
mod measure;
mod spin_half;
mod spin_one;

use measure::{Mode, Runner};
use num_complex::Complex64 as C;
use syntaqs_core::space::HilbertSpace;

type Space = HilbertSpace<usize>;

fn factors(n: usize) -> Vec<Space> {
    (0..n).map(HilbertSpace::factor).collect()
}

fn close(actual: C, expected: impl Into<C>) {
    let expected = expected.into();
    assert!(
        (actual - expected).norm() < 1e-10,
        "expected {expected}, got {actual}"
    );
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mode = match args.next().as_deref() {
        None | Some("time") if !cfg!(feature = "allocations") => Mode::Time,
        Some("allocations") if cfg!(feature = "allocations") => Mode::Allocations,
        Some("diagnostics") => Mode::Diagnostics,
        _ => {
            panic!("use time without the allocations feature, allocations with it, or diagnostics")
        }
    };
    let filter = args.next().unwrap_or_default();
    assert!(args.next().is_none(), "expected at most one case ID filter");
    let runner = Runner { mode, filter };
    runner.print_header();
    spin_half::run(&runner);
    spin_one::run(&runner);
    fermion::run(&runner);
}
