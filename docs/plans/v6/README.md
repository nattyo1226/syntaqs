# Updates with version 0.6.0

## Expression and family construction

Construction calls take expression or primitive sequences before the space.
The following calls migrate from `v0.5.0`.

| API | v0.5.0 call | v0.6.0 call |
| --- | --- | --- |
| `ExpressionFactory` | `tensor_in(&space, states)` | `state_tensor(states, &space)` |
| `ExpressionFactory` | `try_tensor_in(&space, states)` | `try_state_tensor(states, &space)` |
| `ExpressionFactory` | `product_state(&space, primitives)` | `state_tensor_from(primitives, &space)` |
| `ExpressionFactory` | `operator_product_at([(&factor, primitive)])` | `operator_product_from([(primitive, &factor)])` |
| `ExpressionFactory` | `state_sum(&space, states)` / `try_state_sum(&space, states)` | `state_sum(states, &space)` / `try_state_sum(states, &space)` |
| `State` | `sum(&space, states)` / `try_sum(&space, states)` | `sum(states, &space)` / `try_sum(states, &space)` |
| `State` | `tensor_in(&space, states)` / `try_tensor_in(&space, states)` | `tensor_in(states, &space)` / `try_tensor_in(states, &space)` |
| `OperatorFamily` | `new_in(&context, operators)` | `new_in(operators, &context)` |

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
