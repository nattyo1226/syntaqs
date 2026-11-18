//! Release benchmark for closed physical operator families.

extern crate blas_provider as _;

use std::{env, error::Error, hint::black_box, ops::Range, time::Instant};

use num_complex::Complex64;
use syntaqs_core::{expression::composite::Operator, space::HilbertSpace};
use syntaqs_fermion::{Fermion, operator::FermionOperator as Majorana};
use syntaqs_geometry::{geometry::Boundary, geometry::chain::Chain};
use syntaqs_lie::{
    FamilyExpression,
    decomposition::DecompositionOptions,
    family::OperatorFamily,
    numerical::{options::NumericalOptions, tolerance::Tolerance},
};
use syntaqs_spin_half::{SpinHalf, models::SpinHalfModels, operator::SpinHalfOperator as Pauli};
use syntaqs_spin_one::{SpinOne, operator::SpinOneOperator as SpinOneOp};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Space = HilbertSpace<usize>;
type Half = SpinHalf<Complex64>;
type SpinOperator = Operator<usize, Pauli, Complex64>;
const AXES: [Pauli; 3] = [Pauli::Sx, Pauli::Sy, Pauli::Sz];

#[derive(Clone, Copy)]
enum Run {
    All,
    Closure,
    Commutant,
    Decompose,
}

struct Expected<'a> {
    hilbert: u128,
    center: usize,
    ideals: &'a [usize],
}

fn timed<T>(stage: &str, f: impl FnOnce() -> Result<T>) -> Result<T> {
    let start = Instant::now();
    let result = f()?;
    let ms = start.elapsed().as_secs_f64() * 1_000.0;
    println!("{stage},{ms:.6}");
    Ok(black_box(result))
}

fn measure<E: FamilyExpression<Real = f64>>(
    family: OperatorFamily<E>,
    expected: Expected<'_>,
    run: Run,
) -> Result<()> {
    let n = family.len();
    assert_eq!(n, expected.center + expected.ideals.iter().sum::<usize>());
    println!("# D={}, n={n}", expected.hilbert);
    let numerical = NumericalOptions {
        residual: Tolerance {
            absolute: 1e-10,
            relative: 1e-10,
        },
        ..NumericalOptions::default()
    };
    let sampling = DecompositionOptions {
        seed: 816,
        max_attempts: 16,
    };

    let gram = timed("gram", || Ok(family.gram(&numerical)?))?;
    let factor = timed("factorize", || Ok(gram.factorize()?))?;
    let closure = timed("closure", || Ok(factor.closure()?))?;
    assert_eq!(closure.checks().len(), n * n);
    assert!(
        closure.passed(),
        "first failed pair: {:?}",
        closure
            .checks()
            .iter()
            .find(|pair| !pair.check.passed())
            .map(|pair| (pair.left, pair.right, pair.check))
    );
    if matches!(run, Run::Closure) {
        return Ok(());
    }
    let adjoint = timed("adjoint", || Ok(closure.structure_constants()?.adjoint()))?;
    if matches!(run, Run::All) {
        let killing = timed("killing", || Ok(adjoint.killing_form()?))?;
        assert_eq!(killing.values().dim(), (n, n));
    }
    if matches!(run, Run::All | Run::Commutant) {
        let commutant = timed("commutant", || Ok(adjoint.commutant()?))?;
        assert_eq!(
            commutant.dimension(),
            expected.center.pow(2) + expected.ideals.len()
        );
        let constraints = commutant.diagnostics();
        assert_eq!(
            (constraints.rows, constraints.columns),
            (n.pow(3), n.pow(2))
        );
    }
    if matches!(run, Run::All) {
        let split = timed("center", || Ok(adjoint.center()?))?;
        assert_eq!(
            split.center().map_or(0, OperatorFamily::len),
            expected.center
        );
        assert_eq!(
            split.complement().map_or(0, OperatorFamily::len),
            n - expected.center
        );
    }
    if matches!(run, Run::All | Run::Decompose) {
        let split = timed("decompose", || Ok(adjoint.decompose(&sampling)?))?;
        assert!(split.diagnostics().validation.passed());
        assert_eq!(
            split.center().map_or(0, |part| part.family().len()),
            expected.center
        );
        let mut ideals: Vec<_> = split
            .simple_ideals()
            .iter()
            .map(|part| part.family().len())
            .collect();
        ideals.sort_unstable();
        assert_eq!(ideals, expected.ideals);
        println!("# attempts={}", split.diagnostics().attempts.len());
        let mut nonzero = 0;
        let mut tiny = 0;
        for part in split.center().into_iter().chain(split.simple_ideals()) {
            for &coefficient in part.coordinates() {
                if coefficient != 0.0 {
                    nonzero += 1;
                    if coefficient.abs() <= 1e-12 {
                        tiny += 1;
                    }
                }
            }
        }
        println!("# coordinate_nonzero={nonzero}, coordinate_tiny={tiny}");
    }
    Ok(())
}

fn local_paulis(sites: usize, active: usize, run: Run) -> Result<()> {
    let factors: Vec<_> = (0..sites).map(Space::factor).collect();
    let space = Space::tensor_all(factors.iter().cloned())?;
    let mut operators = Vec::new();
    for factor in &factors[..active] {
        for axis in AXES {
            operators.push(Half::operator(axis).at(factor)?);
        }
    }
    measure(
        OperatorFamily::new_in(&space, operators)?,
        Expected {
            hilbert: 1 << sites,
            center: 0,
            ideals: &vec![3; active],
        },
        run,
    )
}

fn two_site_paulis(mixed: bool, run: Run) -> Result<()> {
    let factors = [Space::factor(0), Space::factor(1)];
    let space = Space::tensor_all(factors.iter().cloned())?;
    let mut operators: Vec<_> = [Pauli::I, Pauli::Sx, Pauli::Sy, Pauli::Sz]
        .into_iter()
        .flat_map(|left| {
            [Pauli::I, Pauli::Sx, Pauli::Sy, Pauli::Sz]
                .into_iter()
                .map(move |right| (left, right))
        })
        .skip(1)
        .map(|(left, right)| Half::operator_product_at([(&factors[0], left), (&factors[1], right)]))
        .collect::<std::result::Result<_, _>>()?;
    if mixed {
        for pair in operators.chunks_exact_mut(2) {
            let sum = (&pair[0] + &pair[1])?;
            let difference = (&pair[0] - &pair[1])?;
            pair[0] = sum;
            pair[1] = difference;
        }
    }
    measure(
        OperatorFamily::new_in(&space, operators)?,
        Expected {
            hilbert: 4,
            center: 0,
            ideals: &[15],
        },
        run,
    )
}

fn spin_one(cartesian: bool, run: Run) -> Result<()> {
    use SpinOneOp::{Dxy, Q0, Qxy, Qyz, Qzx, Sx, Sy, Sz};
    let basis = [Sx, Sy, Sz, Dxy, Q0, Qxy, Qyz, Qzx];
    let n = if cartesian { 8 } else { 3 };
    let operators = basis[..n]
        .iter()
        .copied()
        .map(SpinOne::<Complex64>::operator);
    measure(
        OperatorFamily::new(operators)?,
        Expected {
            hilbert: 3,
            center: 0,
            ideals: &[n],
        },
        run,
    )
}

fn heisenberg(sites: usize, active: usize, run: Run) -> Result<()> {
    let factors: Vec<_> = (0..sites).map(Space::factor).collect();
    let space = Space::tensor_all(factors.iter().cloned())?;
    let active_factors = &factors[..active];
    let h = SpinHalfModels::<Half>::heisenberg(
        &Chain::new(active, Boundary::Open),
        active_factors,
        1.0,
    )?
    .bind(&space)?;
    let mut operators = Vec::new();
    for axis in AXES {
        let terms = active_factors
            .iter()
            .map(|factor| Half::operator(axis).at(factor))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        operators.push(SpinOperator::sum(terms)?);
    }
    operators.extend([h, SpinOperator::identity()]);
    measure(
        OperatorFamily::new_in(&space, operators)?,
        Expected {
            hilbert: 1 << sites,
            center: 2,
            ideals: &[3],
        },
        run,
    )
}

fn pauli_z(run: Run) -> Result<()> {
    let factors: Vec<_> = (0..4).map(Space::factor).collect();
    let space = Space::tensor_all(factors.iter().cloned())?;
    let operators = (1..16)
        .map(|mask| {
            Half::operator_product_at(factors.iter().enumerate().filter_map(|(site, factor)| {
                (mask & (1 << site) != 0).then_some((factor, Pauli::Sz))
            }))
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    measure(
        OperatorFamily::new_in(&space, operators)?,
        Expected {
            hilbert: 16,
            center: 15,
            ideals: &[],
        },
        run,
    )
}

fn fermion_quadratics(sites: usize, active: Range<usize>, run: Run) -> Result<()> {
    type F = Fermion<Complex64>;
    let factors: Vec<_> = (0..sites).map(Space::factor).collect();
    let space = Space::tensor_all_g(factors.iter().cloned())?;
    let active = &factors[active];
    let modes = active.len();
    let mut operators = Vec::new();
    for a in 0..2 * modes {
        for b in a + 1..2 * modes {
            if a / 2 == b / 2 {
                operators.push(F::operator(Majorana::Mz).at(&active[a / 2])?);
            } else {
                let axis = [Majorana::Mx, Majorana::My];
                let left = F::operator(axis[a % 2]).at(&active[a / 2])?;
                let right = F::operator(axis[b % 2]).at(&active[b / 2])?;
                operators.push((left * right)?.scaled(Complex64::new(0.0, -1.0)));
            }
        }
    }
    let n = modes * (2 * modes - 1);
    let ideals: &[usize] = if modes == 2 { &[3, 3] } else { &[n] };
    measure(
        OperatorFamily::new_in(&space, operators)?,
        Expected {
            hilbert: 1 << sites,
            center: 0,
            ideals,
        },
        run,
    )
}

fn site_scaling_case(case: &str, run: Run) -> Result<()> {
    let (family, size) = case
        .rsplit_once('-')
        .ok_or("expected a case with a site count")?;
    let sites: usize = size.parse()?;
    if !(2..=64).contains(&sites) {
        return Err("expected a site count between 2 and 64".into());
    }
    match family {
        "heisenberg" => heisenberg(sites, sites, run),
        "heisenberg-2-in" => heisenberg(sites, 2, run),
        "fermion-front" => fermion_quadratics(sites, 0..2, run),
        "fermion-tail" => fermion_quadratics(sites, sites - 2..sites, run),
        _ => Err(format!("unknown benchmark case: {case}").into()),
    }
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let case = args.next().ok_or("expected a benchmark case")?;
    let run = match args.next().as_deref() {
        None | Some("all") => Run::All,
        Some("closure") => Run::Closure,
        Some("commutant") => Run::Commutant,
        Some("decompose") => Run::Decompose,
        Some(_) => return Err("expected all, closure, commutant, or decompose".into()),
    };
    match case.as_str() {
        "spin-half-1" => local_paulis(1, 1, run),
        "spin-half-2" => local_paulis(2, 2, run),
        "spin-half-4" => local_paulis(4, 4, run),
        "spin-half-2-in-8" => local_paulis(8, 2, run),
        "pauli-2" => two_site_paulis(false, run),
        "pauli-2-mixed" => two_site_paulis(true, run),
        "spin-one-dipole" => spin_one(false, run),
        "spin-one-cartesian" => spin_one(true, run),
        "pauli-z-4" => pauli_z(run),
        "fermion-2" => fermion_quadratics(2, 0..2, run),
        "fermion-3" => fermion_quadratics(3, 0..3, run),
        "fermion-4" => fermion_quadratics(4, 0..4, run),
        _ => site_scaling_case(&case, run),
    }
}
