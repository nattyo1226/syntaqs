# Updates with version 0.6.0

## Expression and family construction

When migrating from `v0.5.0`, pass input sequences before the space or family
context. For operator products from primitives, pass each assignment as
`(primitive, &factor)`.

| Operation | v0.6.0 call |
| --- | --- |
| Tensor placed states through the factory | `ExpressionFactory::state_tensor(states, &space)` / `ExpressionFactory::try_state_tensor(states, &space)` |
| Tensor state primitives in leaf order | `ExpressionFactory::state_tensor_from(primitives, &space)` |
| Multiply placed operator primitives | `ExpressionFactory::operator_product_from([(primitive, &factor)])` |
| Sum states through the factory | `ExpressionFactory::state_sum(states, &space)` / `ExpressionFactory::try_state_sum(states, &space)` |
| Sum states directly | `State::sum(states, &space)` / `State::try_sum(states, &space)` |
| Tensor states directly | `State::tensor_in(states, &space)` / `State::try_tensor_in(states, &space)` |
| Construct a family in an explicit context | `OperatorFamily::new_in(operators, &context)` |

The factory changes also apply to aliases such as `SpinHalf`, `SpinOne`, and
`Fermion`. [Expression construction](../../design/expression-construction.md),
[Lie family contexts](../../design/lie.md#1-scope-and-contexts).

## Local basis traits

Import `BasisElements` for `element`, `index`, and `elements`. Built-in
bases such as `SzBasis` support these calls without selecting a coefficient
type. The same trait suffices for `HilbertSpace::basis_states` and
`HilbertSpace::basis_operators`. Select the coefficient type of the returned
expressions through their type context or generic arguments.

For a custom basis, implement zero-based `element` and `index` in
`BasisElements<P>`. Put any `elements` override on that implementation. For
coordinate queries and expression expansion, implement `Basis<P, C>` by
returning basis indices with their coefficients from `expand_indices`. Use
`Basis::expand` to resolve these indices into primitives and handle its
`Result`. In `v0.5.0`, custom bases supplied `expand` directly and returned a
primitive combination. [Basis design](../../design/basis.md),
[custom basis implementation](../../../crates/syntaqs-core/tests/basis.rs).

## Local operator trace

`LocalOperator::trace()` accepts operator primitives with `Product`, `Adjoint`,
and `Trace`. Normalized Hilbert–Schmidt inner products use `Identity` to obtain
the normalization factor `Tr(I)`. [Trace and normalization](../../design/primitive-expression.md#41-dimension-normalized-hilbertschmidt-inner-products).

## Core expression performance

Space binding reuses factor positions stored on each Hilbert space. State
actions skip unaffected tensor children, and operator evaluation keeps
independent blocks factored to reduce work in contractions and basis queries.
The reference workloads and measured trade-offs are recorded in the
[performance baseline](../../decisions/core-evaluation-performance.md#follow-up-performance-baseline).

Tensor construction and composite evaluation require ordered factor IDs
(`I: Ord`). `HilbertSpace::flatten()` and `tensor_flatten()` promote one level
of grouping.

## Lie analysis performance

Closure reuses evaluated Hilbert–Schmidt operands and skips disjoint commuting
term pairs in eligible brackets. Center extraction reuses the Gram matrix.
Exact-zero constraints skip residual matrix multiplication, and decomposition
builds symmetric adjoint constraints directly to reduce peak memory.
[Reference measurements](../../decisions/lie-analysis.md#evaluation-performance).

Scale the input basis coefficients before checking closure or decomposing a
family to limit roundoff from large intermediate products.
[Basis scaling guidance](../../design/lie.md#3-projection-and-numerical-scales).

## Lie decomposition generators

Center and simple-ideal generators use collected expressions with neutral
identity factors and exact unit scales omitted. Scalar addends, ambient
bindings, and coordinates in the source family are preserved.
[Lie decomposition](../../design/lie.md#7-complete-center-and-simple-ideal-decomposition).

## Release verification

- [ ] Run the [Rust 1.85.0 checks](../../design/verification.md#2-common-checks)
  for the release candidate. The local Rust 1.85.0 run for the public API changes
  was blocked while building toolchain dependencies.
