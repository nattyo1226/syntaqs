# Implementation references

## Source baseline

| Repository | Inspected code revision |
| --- | --- |
| syntaqs | `de686db19630a7a2fadcaab660fd12ea4fb5dcdc` |
| LieAlgebra.jl | `575029c12fcad44b3b41206ee564437a7ec5fa45` |

The reference checkout is `~/repos/github.com/nattyo1226/LieAlgebra`, declaring
`1.0.0-DEV` and Julia 1.11. Source/test inspection established the feature map
below; upstream freshness and full Julia test-suite success were not verified.
The workspace HEAD at document consolidation is
`6d6936a8dfe1ad798b88a33395c03a7c12e8a3f1`; its change from the syntaqs code
baseline only adds this project's earlier documents.

## Existing syntaqs extension points

| Source | Reuse / constraint |
| --- | --- |
| [Core operations](../../../crates/syntaqs-core/src/ops.rs), [coefficients](../../../crates/syntaqs-core/src/coefficient.rs) | Operation-specific algebra capabilities and precision; no new evaluator needed. |
| [Local expressions](../../../crates/syntaqs-core/src/expr/local.rs), [placed expressions](../../../crates/syntaqs-core/src/expr/composite.rs) | Shared immutable handles, local/ambient context, placement and binding. |
| [Arithmetic](../../../crates/syntaqs-core/src/expr/arithmetic.rs), [HS contraction](../../../crates/syntaqs-core/src/evaluate/trace.rs) | Existing commutator, scaling, trace, and normalized HS without dense physical matrices or an integer total dimension. |
| [Basis design](../../design/basis.md) | Existing Basis means a complete local primitive basis; do not repurpose it for arbitrary operator-expression families. |
| [Export](../../../crates/syntaqs-export/src/metadata.rs) | State-basis action matrices and operator coordinate vectors are distinct from Lie adjoint matrices. Existing metadata is not a general Lie-basis descriptor. |
| [Numeric contracts](../../design/numerics.md), [HS tests](../../../tests/tests/core_hs_inner_product.rs) | Preserve exact zeros and coefficient precision; independent references already cover ordinary/graded HS behavior. |

See [architecture](../../architecture.md) for crate responsibilities. The
[general map-representation proposal](../../future-work/representation-matrix.md)
remains future work; it is not a dependency of this implementation.

## Julia feature map

Paths below are relative to the pinned reference's
[src/](https://github.com/nattyo1226/LieAlgebra.jl/tree/575029c12fcad44b3b41206ee564437a7ec5fa45/src).
The [specification](spec.md) controls target behavior; this table is source navigation.

| Reference files / APIs | Target work |
| --- | --- |
| `type.jl`, `basis.jl`: MatrixBasis, access/enumeration, rank, independence | OperatorFamily and validation (Step 2). Julia's physical matrix-size/vectorization helpers inform context/coordinate access; do not require materializing full physical matrices. |
| `inner_products.jl`, `basis.jl`: gram_matrix, coefficients, reconstruct, project, residual, in_span | Fixed normalized HS, dense Gram solves, explicit projection semantics (Step 2). |
| `brackets.jl`, `basis.jl`: commutator, lie_bracket, bracket_coefficients, closure checks | Reuse core arithmetic and add Lie-specific bracket/closure APIs (Step 3). |
| `structure_constants.jl`, `adjoint.jl`, `killing.jl` | Retained coefficients, adjoint matrices, classical Killing form (Step 3). |
| `orthogonalization.jl`, `decomposition.jl`: orthogonalize, is_orthonormal, transform_basis, component_basis | Family preparation/transforms (Step 3) and result-component support (Step 5). |
| `commutant.jl` | Full real adjoint commutant and a separate symmetric subspace (Step 4). |
| `decomposition.jl`: generic_commutant_element, decompose, Decomposition | Julia-style sampling/eigenspaces with full-center separation, validation, retries, and seeds (Step 5). |

Intentional differences: rename the two main types; fix the inner product and
residual norm; reject singular basis solves; verify mathematical preconditions;
preserve f32/f64 instead of forcing Float64/ComplexF64; use classical Killing
form rather than the reference's normalized adjoint HS Gram matrix. General
complex/physical-centralizer APIs and configurable metrics are not required.

The reference [tests](https://github.com/nattyo1226/LieAlgebra.jl/tree/575029c12fcad44b3b41206ee564437a7ec5fa45/test)
cover Pauli su(2), two commuting spin triples, and seed 816 decomposition and
orthogonalization. They provide starting examples, not complete target coverage.
There is no dedicated reference Killing-form test.

## Numerical documentation

- [ndarray-linalg 0.18.1 dependencies and library guidance](https://docs.rs/crate/ndarray-linalg/0.18.1): ndarray `^0.17.1` and application-selected LAPACK.
- [Provider features](https://docs.rs/crate/ndarray-linalg/0.18.1/features), [Cargo feature unification](https://doc.rust-lang.org/cargo/reference/features.html#feature-unification).
- [NumPy rank threshold](https://numpy.org/doc/stable/reference/generated/numpy.linalg.matrix_rank.html), [SciPy nullspace threshold](https://docs.scipy.org/doc/scipy/reference/generated/scipy.linalg.null_space.html): dimension-times-epsilon scaling. The fixed floors in our specification are project recommendations, not those libraries' defaults.
