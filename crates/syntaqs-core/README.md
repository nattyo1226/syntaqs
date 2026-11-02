# syntaqs-core

The common algebra and expression layer of [syntaqs](../../README.md).

## Responsibilities

- Define capabilities for local primitives: dimension, identity, products, actions, adjoints, inner products, traces, and parity.
- Store finite linear combinations and shared local/composite expressions.
- Manage factor identity, tensor ordering, and ordinary/graded composition through `HilbertSpace<I>`.
- Validate placement and ambient-space bindings, and evaluate expressions using local rules without full matrices.
- Extend local basis rules to tensor spaces, expression expansion, and sparse coordinates.

## Main API

| Area | Entry points |
| --- | --- |
| Algebra contracts | `Act`, `Product`, `Adjoint`, `InnerProduct`, `Trace`, `Identity`, `Dimension` |
| Grading | `HomogeneousParity`, `ParityOperator`, `Parity`, `TensorKind` |
| Expressions | `LinearCombination`, `LocalState`, `LocalOperator`, `State`, `Operator` |
| Construction | `ExpressionFactory`, `QuantumSystem`, `StateOf`, `OperatorOf` |
| Spaces and bases | `HilbertSpace`, `Basis`, `Coordinates`, `dim_state`, `dim_op` |
| Coefficients | `Conjugate`, `Divide`, `ImaginaryUnit`, `RealPart` |

Numeric bounds are requested by individual operations. A coefficient type may be `Complex<f32>`, `Complex<f64>`, or a custom type implementing the required capabilities.

## Integration

This crate uses `no_std + alloc` and has no default features. Concrete primitive rules come from the [spin-half](../syntaqs-spin-half/README.md), [spin-one](../syntaqs-spin-one/README.md), and [fermion](../syntaqs-fermion/README.md) crates, or from your own implementations of the public traits. The core does not depend on those crates, geometry, index generation, normalization, or numerical backends.

Use [syntaqs-normalize](../syntaqs-normalize/README.md) for square-root-based norms and normalization, and [syntaqs-export](../syntaqs-export/README.md) for numerical containers. Squared norms, expectations, and full traces belong to the core.

See the [crate API overview](src/lib.rs), [architecture](../../docs/architecture.md), [expression design](../../docs/design/composite-expression.md), and [basis design](../../docs/design/basis.md). Installation and development commands are in the [project README](../../README.md).
