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

#[test]
fn all_enabled_preludes_can_be_used_together() {
    use syntaqs_basic::prelude::*;
    #[cfg(feature = "export")]
    use syntaqs_export::prelude::*;
    use syntaqs_geometry::prelude::*;
    #[cfg(feature = "lie")]
    use syntaqs_lie::prelude::*;
    #[cfg(feature = "normalize")]
    use syntaqs_normalize::prelude::*;

    type Half = SpinHalf<Complex<f64>>;
    let index = Tag::new("site");
    let _: Option<Id128> = None;
    let space = HilbertSpace::factor(index);
    let chain = Chain::new(1, Boundary::Open);
    let sites = [space.clone()];
    let mapper = SiteMapper::new(&chain, &sites).unwrap();
    assert_eq!(mapper.groups(&Consecutive::<1>).unwrap().count(), 1);
    assert_eq!(NearestNeighbors.groups(&chain).unwrap().count(), 0);
    let state = Half::state(SpinHalfState::Up);
    let x = Half::operator(SpinHalfOperator::Sx);
    #[cfg(feature = "normalize")]
    assert_eq!(state.norm(), 1.0);
    #[cfg(feature = "export")]
    {
        let matrix: Exported<Vec<Complex<f64>>, _> = (&x)
            .export(MatrixArgs::new(&syntaqs_spin_half::basis::SzBasis))
            .unwrap();
        assert_eq!(matrix.metadata().shape(), [2, 2]);
    }
    #[cfg(feature = "lie")]
    {
        let y = Half::operator(SpinHalfOperator::Sy);
        let family = OperatorFamily::new([x.clone(), y.clone()]).unwrap();
        assert_eq!(
            family
                .gram(&NumericalOptions::default())
                .unwrap()
                .values()
                .shape(),
            &[2, 2]
        );
        assert_eq!(lie_bracket(&x, &y).unwrap().trace(), Complex::new(0.0, 0.0));
    }
    let _ = (state, x);
}
