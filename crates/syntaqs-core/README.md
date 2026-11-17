# syntaqs-core

Build and evaluate quantum states and operators from local algebra rules. This
crate supplies the shared expressions, Hilbert spaces, tensor composition, and
basis coordinates used throughout [syntaqs](../../README.md).

Start with the [two-spin Quick start](../../README.md#quick-start). For release
installation, [select a version tag](../../docs/installation.md) and use that
tag's documentation. This README describes the current checkout.

## Building a system

Choose state/operator primitives and a coefficient type through
`ExpressionFactory<S, O, C>`, or an alias such as `SpinHalf<C>` from a concrete
algebra crate. Create factors with `HilbertSpace::factor(index)`, compose their
space in the required order, and place local expressions on those factors.
Actions, inner products, expectations, and traces use local rules; basis
expansion and numerical export are separate choices.

Use the built-in [spin-half](../syntaqs-spin-half/README.md),
[spin-one](../syntaqs-spin-one/README.md), or [fermion](../syntaqs-fermion/README.md)
algebra, or implement the public capability traits for your own primitives.
Custom algebras reuse the same expression trees and evaluators.

## Main API

| Area | Entry points |
| --- | --- |
| Local rules | `Act`, `Product`, `Adjoint`, `InnerProduct`, `Trace`, `Identity`, `Dimension` |
| Grading | `HomogeneousParity`, `ParityOperator`, `Parity`, `TensorKind` |
| Expressions | `LinearCombination`, `LocalState`, `LocalOperator`, `State`, `Operator` |
| Construction | `ExpressionFactory`, `QuantumSystem`, `StateOf`, `OperatorOf` |
| Spaces and bases | `HilbertSpace`, `Basis`, `Coordinates`, `dim_state`, `dim_op` |
| Coefficients | `Conjugate`, `Divide`, `ImaginaryUnit`, `RealPart` |

Each operation requests its own capabilities. Storage, sharing, and delayed
construction impose no uniform evaluation or coefficient-arithmetic bounds.
Coefficients can be `Complex<f32>`, `Complex<f64>`, or a custom compatible type.

## Integration and detailed contracts

Core uses `no_std + alloc` and has no default features. It is independent of
concrete algebras, geometry, identifier generation, normalization, and numerical
backends. Squared norms and ordinary expectations belong here;
[normalize](../syntaqs-normalize/README.md) adds square-root norms and normalized
expectations, and [export](../syntaqs-export/README.md) adds numerical containers.

See the [API overview](src/lib.rs), [expression contracts](../../docs/design/composite-expression.md),
[basis contracts](../../docs/design/basis.md), and [architecture](../../docs/architecture.md).
