# syntaqs-normalize

Norm and normalization extensions for [syntaqs](../../README.md) state expressions.

## Responsibilities

| Extension trait | Operation |
| --- | --- |
| `Norm` | Compute `sqrt(Re(<psi | psi>))`. |
| `Normalize` | Return a state scaled to unit norm while preserving its phase. |
| `NormalizedExpectation` | Compute `<psi | A | psi> / <psi | psi>`. |
| `NormalizedExpectationIn` | Compute a normalized expectation with an explicit ambient space. |

Import the corresponding trait to enable its methods. Operations that divide by the state's self-inner-product reject an exactly zero denominator with `NormalizationError`. Norm computation itself permits zero.

## Integration

This crate builds on the public state and operator APIs of [syntaqs-core](../syntaqs-core/README.md). Squared norms, inner products, unnormalized expectations, and scalar division remain in the core. Normalization shares and scales the original expression.

The crate uses the standard library. Square-root-based norms and normalization require `Float` on the associated real type. Normalized expectations use scalar division and need neither square roots nor real-part extraction.

See the [crate API overview](src/lib.rs), [normalization design](../../docs/design/primitive-expression.md#41-norm正規化の-api-と-feature), and [Heisenberg example](../../examples/src/bin/heisenberg.rs). Installation is covered by the [project README](../../README.md).
