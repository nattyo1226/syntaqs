//! Mathematical Display, multiline Display, and structural Debug.

use num_complex::Complex;
use qmb_basic::{Id128, Tag};
use qmb_core::{HilbertSpace, LocalOperator, LocalState, State};
use qmb_spin_half::{SpinHalfOperator as O, SpinHalfState as S};

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
    println!("scaled: {}", local.scaled(Complex::new(2.0, 3.0)).at(&a)?);

    // Action and adjoint are displayed without evaluating the expression.
    let op = LocalOp::sum([LocalOp::primitive(O::Sx), LocalOp::primitive(O::Sy)]);
    let action = local.act(&op).at(&a)?;
    println!("local action: {action}");
    println!("adjoint: {}", op.adjoint().at(&a)?);
    println!("structure:\n{action:#?}");

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
