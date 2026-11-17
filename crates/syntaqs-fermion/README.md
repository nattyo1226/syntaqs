# syntaqs-fermion

Construct single-mode fermion states and operators, combine modes in graded
tensor spaces, and build standard or symmetric Hubbard chains. `Fermion<C>`
selects the primitive and coefficient types for core expressions.

Use the [tagged installation guide](../../docs/installation.md) and read the
chosen tag's documentation. The paths below describe the current checkout.
Core supplies expression storage, graded evaluation, spaces, and coordinates;
geometry supplies the chain relations used by models. This crate uses
`no_std + alloc`, with numeric requirements requested per operation.

## Creating one particle

The local creation operator maps the empty state to the occupied state:

```rust
use num_complex::Complex;
use syntaqs_fermion::{Fermion, operator::FermionOperator as Op, state::FermionState as Ket};

type Mode = Fermion<Complex<f64>>;

fn main() {
    let empty = Mode::state(Ket::Empty);
    let created = empty.act(&Mode::operator(Op::Create)).evaluate();
    assert_eq!(
        created.inner_product(&Mode::state(Ket::Occupied)),
        Complex::new(1.0, 0.0)
    );
}
```

Multiple modes use graded tensor spaces so core can supply exchange signs from
factor order and primitive parity. The [Hubbard example](../../examples/src/bin/hubbard.rs)
shows the full many-body construction.

## Operators, bases, and Hubbard models

Primitives live under `state` and `operator`. For annihilation operator `a`, the
Majoranas are `Mx = a + a†` and `My = (a - a†)/i`. The even bilinear
`Mz = I - 2n` is the parity operator; Mx and My are odd. The other operator
primitives are I, Annihilate, Create, and Number.

`basis` provides state `OccupationBasis` in Empty/Occupied order and operator
`LadderBasis` and `MajoranaBasis`. Majorana-basis elements are orthonormal under
`Tr(A†B)/2`. Local primitives carry neither mode indices nor Jordan–Wigner strings.

`models::FermionModels` uses caller-supplied sites with two graded leaves in
`(up, down)` order. Standard Hubbard has interaction `U n_up n_down`; symmetric
Hubbard uses `U (n_up - I/2)(n_down - I/2)`, retaining its constant shift. The
prelude provides the system alias, primitives, and model helper.

## Integration and references

See the [API overview](src/lib.rs), [local fermion algebra](../../docs/design/local-structure.md#8-fermion-local-algebra),
and [Hubbard contracts](../../docs/design/models.md#3-fermionic-hubbard-models).
