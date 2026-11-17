# syntaqs-spin-one

Construct spin-1 states, dipole and quadrupole operators, and XYZ, Heisenberg,
or bilinear-biquadratic (BLBQ) chain models. `SpinOne<C>` selects primitives and
coefficients for the shared core expression machinery.

Use the [tagged installation guide](../../docs/installation.md) and the chosen
tag's documentation; the example below describes the current checkout. Shared
expressions and evaluation use core, while model helpers use geometry.

## Working with the zero-magnetic-quantum-number state

`SpinOneState::Zero` is a normalized physical state. Applying the raising
operator gives squared norm 3 under this crate's operator normalization:

```rust
use num_complex::Complex;
use syntaqs_spin_one::{SpinOne, operator::SpinOneOperator as Op, state::SpinOneState as Ket};

type Spin = SpinOne<Complex<f64>>;

fn main() {
    let zero = Spin::state(Ket::Zero);
    assert_eq!(zero.norm_squared(), 1.0);
    let raised = zero.act(&Spin::operator(Op::Sp)).evaluate();
    assert!((raised.norm_squared() - 3.0).abs() < 1e-12);
}
```

The zero vector is an empty expression sum, distinct from this state. The
[spin-one example](../../examples/src/bin/spin_one.rs) covers quadrupoles,
basis changes, two-site models, and export.

## Normalization and models

In state order Plus/Zero/Minus, physical `Jz` has eigenvalues `1, 0, -1` with
`hbar = 1`. Cartesian primitives use `S = sqrt(3/2) J`; their operator basis is
orthonormal under `Tr(A†B)/3`. Fifteen operator primitives span the full
nine-dimensional operator space.

`basis` provides `SzBasis`, `CartesianBasis`, and `LadderBasis`. Primitives live
under `state` and `operator`; `models::SpinOneModels` constructs XYZ, Heisenberg,
and BLBQ expressions from caller-supplied site spaces. The prelude provides the
system alias, primitives, and model helper.

Model parameters retain physical-spin meaning. Positive bilinear coupling is
ferromagnetic, and BLBQ is `-sum [j (J_a · J_b) + k (J_a · J_b)^2]` per coupled
leaf pair, with constant terms retained.

## Integration and references

This crate uses std. Local products and actions require `Complex<R>` with
`R: num_traits::Float`; other capabilities have their own bounds. Model
construction is delayed and does not require local evaluation.

See the [API overview](src/lib.rs), [local algebra](../../docs/design/local-structure.md#9-spin-one-local-algebra),
and [model contracts](../../docs/design/models.md#4-spinone-model-construction).
