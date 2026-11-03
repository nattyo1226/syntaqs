# Current syntaqs implementation

Inspected: 2026-11-03, at the revision recorded in [the workspace index](README.md).
This document describes existing code; proposed additions belong in
[design questions](design-questions.md).

## Architecture and responsibilities

The workspace has eight library crates plus example and integration-test
packages. Dependency direction is documented in
[architecture](../../../docs/architecture.md).

| Crate | Existing responsibility relevant to this work |
| --- | --- |
| `syntaqs-core` | Primitive capability traits, finite linear combinations, shared expressions, spaces, algebraic evaluation, bases, and sparse coordinates; `no_std + alloc`. |
| `syntaqs-basic` | Named and UUID indices for identifying factors. |
| `syntaqs-geometry` | Chain geometry and connections to caller-owned spaces and data. |
| `syntaqs-normalize` | State norms, state normalization, and normalized expectations. It does not currently provide operator-basis orthogonalization. |
| `syntaqs-export` | State vectors and operator action matrices in Vec, optionally ndarray and sprs, with interpretation metadata. It does not supply linear solvers, nullspaces, or eigensolvers. |
| `syntaqs-spin-half` | Pauli-normalized spin primitives and Cartesian/ladder operator bases. |
| `syntaqs-spin-one` | Dipole/quadrupole primitives and two nine-element operator bases. |
| `syntaqs-fermion` | Single-mode primitives, ladder/Majorana operator bases, and graded many-body models. |

Core is independent of concrete algebras, geometry, normalization, and numerical
containers. Existing numerical dependencies include num-traits, num-complex,
ndarray, and sprs; Cargo.toml has no numerical factorization backend.

## Expressions and algebra

[`ops.rs`](../../../crates/syntaqs-core/src/ops.rs) separates capabilities such as
`Product`, `Act`, `Adjoint`, `Trace`, `Identity`, and `Dimension`. Each operation
requests its own coefficient capabilities. Primitive types do not select f32 or
f64; expressions store a chosen coefficient type `C`.

[`LocalOperator<O,C>`](../../../crates/syntaqs-core/src/expr/local.rs) and
[`Operator<I,O,C>`](../../../crates/syntaqs-core/src/expr/composite.rs) share
immutable Arc-backed expression nodes. Sums, ordered products, scaling, and
adjoints build delayed expressions. The composite operator handle has an
optional ambient-space binding. `ExpressionFactory` and `QuantumSystem` select
primitive/coefficient types without introducing another evaluator.

Both operator types already expose `commutator`, meaning `AB-BA`, in
[`arithmetic.rs`](../../../crates/syntaqs-core/src/expr/arithmetic.rs). Multiplying
that expression by `-i` can express the reference library's Lie bracket.
Composite arithmetic is fallible and checks space compatibility. Public nodes
are opaque; an extension should use public operations rather than inspect
private expression trees.

[`trace.rs`](../../../crates/syntaqs-core/src/evaluate/trace.rs) already provides
ordinary unnormalized trace and the dimension-normalized HS inner product
`Tr(A†B)/D`. The composite `hs_inner_product_in` selects an explicit ambient
space, overriding previous bindings. Local normalization occurs before tensor
contraction, avoiding the need to materialize an integer total dimension. These
operations require neither a state type nor a full state-space matrix.

[`HilbertSpace`](../../../crates/syntaqs-core/src/space.rs) stores factor identity,
order, grouping, and ordinary/graded composition, rather than a basis or local
dimension. Graded evaluation resolves signs and parity strings. The existing
commutator remains `AB-BA`; it is not a graded Lie superbracket.

## Three distinct coordinate spaces

1. `Basis<P,C>` describes a complete, independent basis of local primitives.
   Whole-expression expansion and coordinates extend it uniformly over factors.
2. A Lie algebra basis such as `[Sx,Sy,Sz]` is an ordered list of operators
   spanning a possibly proper subspace. Its elements can be arbitrary local or
   composite expressions. No such general subspace-basis abstraction exists.
3. An operator action matrix acts on a state basis. This differs from both its
   operator-basis coordinate vector and an adjoint-representation matrix acting
   on a Lie algebra basis.

The first is specified in [basis design](../../../docs/design/basis.md) and
implemented in [`basis.rs`](../../../crates/syntaqs-core/src/basis.rs) and its
submodules. `Basis` must expand every valid primitive exactly in mathematical
value; it is not a numerical projection interface. Consequently it cannot
directly represent the second case without changing its contract.

[`Coordinates`](../../../crates/syntaqs-core/src/basis/coordinates.rs) owns sparse
coefficients, borrows a basis, and retains the selected space. Entries aggregate
by factor-index digits; the rightmost digit changes fastest. Only exact zeros
are removed. For graded operators these are intrinsic placement coordinates,
whereas state-basis action matrices include parity strings.

## Reuse and boundaries

Existing expressions and HS contractions can build Gram entries, reconstruct
linear combinations, and compute bracket expressions without dense physical
matrices. Small matrices indexed by a chosen Lie algebra basis would still be
needed for numerical solves and decomposition. Their dimension is the number
of supplied basis elements, not the Hilbert-space dimension.

The adopted implementation direction is to retain this algebraic path and use
dense ndarray coefficient arrays for numerical work. This is a planned
extension, not existing factorization functionality; see
[the adopted boundary](design-questions.md#algebraic-and-numerical-boundary).
The new functionality is planned in the dedicated syntaqs-lie crate, which
will depend on existing core capabilities; see
[the placement decision](design-questions.md#crate-placement).

The [map-representation proposal](../../../docs/future-work/representation-matrix.md)
already discusses commutator maps, bilinear coefficients, distinct bases, and
retained representations. It is future work, not an implemented API. Coordinate
and metadata implementations for this iteration stay within syntaqs-lie;
commonality with that proposal will be considered later and is not a
prerequisite. See [the storage scope](design-questions.md#representation-storage-scope).

Preserve the current [numeric contracts](../../../docs/design/numerics.md): no
implicit precision promotion, operation-specific capabilities, and no implicit
approximate pruning. Numerical rank, projection, and eigenspace tolerances will
need explicit new contracts outside ordinary expression evaluation.

Existing [verification infrastructure](../../../docs/design/verification.md)
includes independent small matrices, both precisions, nonorthogonal bases,
graded signs, sparse export, and deep expression trees. In particular,
[`core_hs_inner_product.rs`](../../../tests/tests/core_hs_inner_product.rs)
provides direct matrix comparisons for the contraction to be reused here.
