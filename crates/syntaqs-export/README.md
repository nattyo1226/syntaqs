# syntaqs-export

Convert syntaqs states into vectors and operators into action matrices, keeping
the basis, space, shape, and layout needed to interpret the numbers.

Follow the [tagged installation and feature guide](../../docs/installation.md)
and use the selected tag's documentation. The example below uses current APIs.
This std crate depends on core for actions, basis enumeration, and coordinates.

## Exporting a state vector

`Export` selects the output container through the result type. Vec output is
always available; this example exports the spin-up state in the spin-z basis:

```rust
use num_complex::Complex;
use syntaqs_export::{Export, Exported};
use syntaqs_spin_half::{SpinHalf, basis::SzBasis, state::SpinHalfState};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let up = SpinHalf::<Complex<f64>>::state(SpinHalfState::Up);
    let vector: Exported<Vec<_>, _> = (&up).export(&SzBasis)?;
    assert_eq!(
        vector.data().as_slice(),
        &[Complex::new(1.0, 0.0), Complex::new(0.0, 0.0),]
    );
    Ok(())
}
```

`Exported` pairs the container with interpretation metadata. For an operator
matrix, pass `MatrixArgs::new(&basis)` from `matrix::args` and annotate the
result with the desired container. If the state type is ambiguous, choose it
with `MatrixArgs::<S, _>::new(&basis)`. A composite operator also needs a
binding or an explicit ambient space.

## Containers and features

| Feature | State/coordinate vectors | Operator matrices |
| --- | --- | --- |
| Always available | `Vec<C>` | Flat, row-major `Vec<C>` |
| `ndarray` | `ndarray::Array1<C>` | `ndarray::Array2<C>` |
| `sprs` | `sprs::CsVec<C>` | `sprs::CsMat<C>` in CSC format |

The default feature set is empty. Enable `ndarray` and `sprs` independently or
together, and declare direct dependencies on container crates whose types you
use. Owned Vec exports convert to ndarray without copying coefficient data;
sprs output comes from sparse coordinates without a dense intermediate.

Operator-basis coordinates are coefficient vectors. Action matrices instead
contain columns obtained by applying the operator to state-basis elements.
Both paths preserve coefficient type, basis choice, and factor order.

## Integration and references

`Export`, `Exported`, and `ExportError` are crate-root types; matrix arguments and
metadata live under `matrix`, and vector metadata under `vector`. The prelude
provides `Export`, `Exported`, and `MatrixArgs`.

See the [API overview](src/lib.rs), [export contracts](../../docs/design/export.md),
and [matrix export example](../../examples/src/bin/heisenberg.rs).
