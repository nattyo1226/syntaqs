//! Display for product states, superpositions, delayed actions, and a Hamiltonian.

use num_complex::Complex;
use qmb_basic::{Id128, Tag};
use qmb_core::{HilbertSpace, LocalOperator, LocalState, State};
use qmb_geometry::{Boundary, Chain};
use qmb_spin_half::{SpinHalf, SpinHalfOperator as O, SpinHalfState as S, models::SpinHalfModels};

type Spin = SpinHalf<Complex<f64>>;
type Models = SpinHalfModels<Spin>;
type LocalKet = LocalState<S, O, Complex<f64>>;
type LocalOp = LocalOperator<O, Complex<f64>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let [a, b, c] = ["a", "b", "c"].map(|name| HilbertSpace::factor(Tag::new(name)));
    let h = HilbertSpace::tensor_all([a.clone(), b.clone(), c.clone()])?;
    let up = LocalKet::primitive(S::Up);
    let down = LocalKet::primitive(S::Down);
    let state = State::tensor_in(&h, [up.at(&a)?, down.at(&b)?, up.at(&c)?])?;
    assert_eq!(state.to_string(), "|up[a],down[b],up[c]>");
    println!("product: {state}");
    println!("multiline:\n{state:#}");

    // A local superposition keeps the index on each primitive ket.
    let local = LocalKet::sum([up.clone(), down]);
    let mixed = State::tensor_in(&h, [local.at(&a)?, up.at(&b)?, up.at(&c)?])?;
    assert_eq!(
        mixed.to_string(),
        "(|up[a]> + |down[a]>) ⊗ |up[b]> ⊗ |up[c]>"
    );
    println!("superposition: {mixed}");
    println!("superposition (multiline):\n{mixed:#}");
    println!("scaled: {}", local.scaled(Complex::new(2.0, 3.0)).at(&a)?);

    // Action and adjoint are displayed without evaluating the expression.
    let op = LocalOp::sum([LocalOp::primitive(O::Sx), LocalOp::primitive(O::Sy)]);
    let action = local.act(&op).at(&a)?;
    println!("local action: {action}");
    println!("adjoint: {}", op.adjoint().at(&a)?);

    // An entangled state, not a tensor product of three local states:
    // |GHZ> = (|up,up,up> + |down,down,down>) / sqrt(2).
    let ghz = Spin::state_sum(
        &h,
        [
            Spin::product_state(&h, [S::Up, S::Up, S::Up])?,
            Spin::product_state(&h, [S::Down, S::Down, S::Down])?,
        ],
    )?
    .scaled(1.0 / 2.0_f64.sqrt());
    println!("\nGHZ state: {ghz}");
    println!("GHZ state (multiline):\n{ghz:#}");

    // Keep the action unevaluated to expose nested operators and state sums.
    let acted_ghz = ghz.act(&op.at(&a)?)?;
    println!("(Sx + Sy)[a] acting on GHZ: {acted_ghz}");
    println!("(Sx + Sy)[a] acting on GHZ (multiline):\n{acted_ghz:#}");

    // Four-site open Heisenberg chain with a transverse field. Sx/Sy/Sz are
    // Pauli matrices: H = -(1/4) sum_<i,j> S_i . S_j - 0.2 sum_i Sx_i.
    let chain = Chain::new(4, Boundary::Open);
    let factors: Vec<_> = (0..chain.num_sites())
        .map(|site| HilbertSpace::factor(Tag::new(format!("s{site}"))))
        .collect();
    let space = HilbertSpace::tensor_all(factors.iter().cloned())?;
    let exchange = Models::heisenberg(&chain, &factors, 1.0)?;
    let field = Spin::try_operator_sum(
        factors
            .iter()
            .map(|factor| Spin::operator(O::Sx).at(factor)),
    )?
    .scaled(-0.2);
    let hamiltonian = Spin::operator_sum([exchange, field])?.bind(&space)?;
    println!("\n4-site open Heisenberg chain + transverse field:");
    println!("H = {hamiltonian}");
    println!("H (multiline):\n{hamiltonian:#}");

    // Standalone indices retain their full labels; expression Display uses 8 chars.
    let id = Id128::new();
    let factor = HilbertSpace::factor(id);
    let placed = up.at(&factor)?;
    assert_eq!(
        placed.to_string(),
        format!("|up[{}]>", &id.to_string()[..8])
    );
    println!("UUID: {id}; ket: {placed}");
    let tag = Tag::new("spin-left-long");
    println!(
        "Tag: {tag}; ket: {}",
        up.at(&HilbertSpace::factor(tag.clone()))?
    );
    Ok(())
}
