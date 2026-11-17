use num_complex::Complex;
use syntaqs_core::prelude::*;
use syntaqs_fermion::prelude::*;
use syntaqs_spin_half::prelude::*;
use syntaqs_spin_one::prelude::*;

#[test]
fn core_and_physical_system_preludes_work_together() {
    type Half = SpinHalf<Complex<f64>>;
    type One = SpinOne<Complex<f64>>;
    type Mode = Fermion<Complex<f64>>;
    let space = HilbertSpace::factor(0);
    let half = Half::state(SpinHalfState::Up).at(&space).unwrap();
    let one = One::state(SpinOneState::Plus).at(&space).unwrap();
    let fermion = Mode::state(FermionState::Occupied).at(&space).unwrap();
    assert_eq!(half.space().indices().count(), 1);
    assert_eq!(one.space().indices().count(), 1);
    assert_eq!(fermion.space().indices().count(), 1);
    let _ = Half::operator(SpinHalfOperator::Sz);
    let _ = One::operator(SpinOneOperator::Sz);
    let _ = Mode::operator(FermionOperator::Mz);
    let _ = core::mem::size_of::<SpinHalfModels<Half>>();
    let _ = core::mem::size_of::<SpinOneModels<One>>();
    let _ = core::mem::size_of::<FermionModels<Mode>>();
}
