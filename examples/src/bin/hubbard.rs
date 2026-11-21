//! Standard and symmetric Hubbard models on a two-site graded chain.
use num_complex::Complex;
use syntaqs_basic::index::Id128;
use syntaqs_core::space::HilbertSpace;
use syntaqs_fermion::{
    Fermion, basis::MajoranaBasis, operator::FermionOperator as O, state::FermionState as S,
};
use syntaqs_geometry::{geometry::Boundary, geometry::chain::Chain};
#[cfg(feature = "normalize")]
use syntaqs_normalize::{expectation::NormalizedExpectation, norm::Normalize};

type F = Fermion<Complex<f64>>;
type Models = syntaqs_fermion::models::FermionModels<F>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chain = Chain::new(2, Boundary::Open);
    // Site-major ordering: up_0, down_0, up_1, down_1.
    let modes: Vec<_> = (0..4).map(|_| HilbertSpace::factor(Id128::new())).collect();
    let sites: Vec<_> = modes
        .chunks_exact(2)
        .map(|pair| HilbertSpace::tensor_all_g(pair.iter().cloned()))
        .collect::<Result<_, _>>()?;
    let h = Models::hubbard(&chain, &sites, 1., 4.)?;
    let symmetric = Models::symmetric_hubbard(&chain, &sites, 1., 4.)?;
    let psi = F::state_tensor_from(
        [S::Occupied, S::Empty, S::Empty, S::Occupied],
        h.space().expect("model is bound"),
    )?;
    assert!((psi.norm_squared()? - 1.).abs() < 1e-12);
    let energy = psi.expectation(&h)?;
    let shifted_energy = psi.expectation(&symmetric)?;
    assert!(energy.norm_sqr() < 1e-24);
    assert!((shifted_energy - Complex::new(-2., 0.)).norm_sqr() < 1e-24);
    println!("|up, down>: Hubbard energy {energy}, symmetric energy {shifted_energy}");
    #[cfg(feature = "normalize")]
    {
        let normalized = psi.scaled(Complex::new(0., 2.)).normalized()?;
        assert!(
            (normalized.normalized_expectation(&symmetric)? - shifted_energy).norm_sqr() < 1e-24
        );
    }
    let p = F::operator(O::Mz).coordinates(&MajoranaBasis)?;
    for entry in p.flat_entries() {
        let (index, coefficient) = entry?;
        println!("Parity in MajoranaBasis: coordinate {index} = {coefficient}");
    }
    #[cfg(feature = "export")]
    {
        use syntaqs_export::{Export, Exported, matrix::args::MatrixArgs};
        use syntaqs_fermion::basis::OccupationBasis;
        let dense: Exported<Vec<Complex<f64>>, _> =
            (&h).export(MatrixArgs::new(&OccupationBasis))?;
        let sparse: Exported<sprs::CsMat<Complex<f64>>, _> =
            (&h).export(MatrixArgs::new(&OccupationBasis))?;
        assert_eq!(dense.metadata().shape(), [16, 16]);
        for (left, right) in dense.data().iter().zip(sparse.data().to_dense().iter()) {
            assert!((*left - *right).norm_sqr() < 1e-24);
        }
        println!(
            "Hubbard matrix: {:?}, {} nonzero entries",
            dense.metadata().shape(),
            sparse.data().nnz()
        );
    }
    Ok(())
}
