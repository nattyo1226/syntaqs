# syntaqs-normalize

Compute state norms, normalize states, and divide expectations by the state's
self-inner-product. These extension traits build on the local and composite
state APIs of [syntaqs-core](../syntaqs-core/README.md).

Use the [tagged installation guide](../../docs/installation.md), adding core,
normalize, and the concrete algebra used by your application. Read the selected
tag's documentation; the example below describes the current checkout.

## Normalizing a state

Import `Normalize` and `Norm` from `norm`. This example uses a local spin-half
state with norm 2 and returns a unit-norm expression:

```rust
use num_complex::Complex;
use syntaqs_normalize::norm::{Norm, Normalize};
use syntaqs_spin_half::{SpinHalf, state::SpinHalfState};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = SpinHalf::<Complex<f64>>::state(SpinHalfState::Up).scaled(2.0);
    let unit = state.normalized()?;
    assert_eq!(unit.norm(), 1.0);
    Ok(())
}
```

Normalization shares and scales the expression, preserves phase, and leaves the
input unchanged. An exactly zero self-inner-product produces
`NormalizationError::ZeroNorm`; computing the norm of zero is valid.

## Choosing an operation

| Extension trait | Operation | Import module |
| --- | --- | --- |
| `Norm` | `sqrt(Re(<psi | psi>))` | `norm` |
| `Normalize` | Unit-norm state | `norm` |
| `NormalizedExpectation` | `<psi | A | psi> / <psi | psi>` | `expectation` |
| `NormalizedExpectationIn` | The same quotient in an explicit ambient space | `expectation` |

The prelude also provides the extension traits. `NormalizationError` is defined
at the crate root. Squared norms, ordinary expectations, inner products, and
scalar division remain in core.

This crate uses std. Norms and normalization require `Float` on the associated
real type. Normalized expectations require division and need neither square
roots nor real-part extraction.

## Integration and references

See the [API overview](src/lib.rs), [normalization contract](../../docs/design/normalization.md),
and [Heisenberg example](../../examples/src/bin/heisenberg.rs).
