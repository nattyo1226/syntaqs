# syntaqs-spin-half

Construct spin-1/2 states, Pauli operators, and XYZ or Heisenberg chain models
using the shared expressions and evaluator in [syntaqs-core](../syntaqs-core/README.md).
`SpinHalf<C>` selects the coefficient type once for the system.

Install core and spin-half with the [same version tag](../../docs/installation.md)
and use that tag's documentation. The API paths below describe the current
checkout. This crate uses `no_std + alloc`; local numeric capabilities are
requested per operation, and primitives carry no fixed coefficient precision.

## Applying a local operator

Pauli X maps spin up to spin down. This example evaluates that action without a
basis or matrix:

```rust
use num_complex::Complex;
use syntaqs_spin_half::{SpinHalf, operator::SpinHalfOperator as Op, state::SpinHalfState as Ket};

type Spin = SpinHalf<Complex<f64>>;

fn main() {
    let up = Spin::state(Ket::Up);
    let down = up.act(&Spin::operator(Op::Sx)).evaluate();
    assert_eq!(
        down.inner_product(&Spin::state(Ket::Down)),
        Complex::new(1.0, 0.0)
    );
}
```

For multiple factors, follow the [two-spin Quick start](../../README.md#quick-start).
The [Heisenberg example](../../examples/src/bin/heisenberg.rs) adds chain geometry,
normalization, and numerical export.

## Physical conventions and available APIs

With `hbar = 1`, `Sx`, `Sy`, and `Sz` are Pauli matrices; physical spins are
`J = S/2`. The ladder operators are `Sp = (Sx + i Sy)/2` and
`Sm = (Sx - i Sy)/2`. Model couplings use physical spins: Heisenberg is
`H = -(g/4) sum S_i · S_j`, with positive `g` ferromagnetic.

| Module | Provides |
| --- | --- |
| `state` | `SpinHalfState::{Up, Down}` |
| `operator` | `SpinHalfOperator::{I, Sx, Sy, Sz, Sp, Sm}` and local rules |
| `basis` | State `SzBasis` in Up/Down order; operator `CartesianBasis` and `LadderBasis` |
| `models` | `SpinHalfModels` for XYZ/Heisenberg and model errors |

Cartesian operator-basis elements are orthonormal under `Tr(A†B)/2`. All
primitives are even, with parity operator I. The caller supplies site spaces
and indices; model helpers use geometry, and product states use core expression
building blocks.

## Integration and references

See the [API overview](src/lib.rs), [local conventions](../../docs/design/local-structure.md#6-spin-12-conventions),
and [chain-model contracts](../../docs/design/models.md#2-spinhalf-model-construction).
