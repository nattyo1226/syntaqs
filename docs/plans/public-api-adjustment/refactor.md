# Public API refactoring proposal

## Expression factory names and argument order

Rename four inherent `ExpressionFactory` methods to make the state and operator
construction APIs easier to read together. The factory also adopts the
[workspace-wide argument order](#expression-and-space-order) below. The renames,
argument reordering, and placement tuple reordering are source-breaking changes,
including for callers that use factory aliases such as `SpinHalf`, `SpinOne`,
and `Fermion`. Reuse the current construction logic with these call forms.
[Current factory implementation](../../../crates/syntaqs-core/src/system.rs).

| Current call | Planned call | Construction contract |
| --- | --- | --- |
| `tensor_in(&space, states)` | `state_tensor(states, &space)` | Tensor an ordered sequence of placed states that exactly covers the supplied space. |
| `try_tensor_in(&space, states)` | `try_state_tensor(states, &space)` | Collect a sequence of `Result` items. Return the first input error before validating the composition. |
| `product_state(&space, primitives)` | `state_tensor_from(primitives, &space)` | Place one state primitive per leaf in the supplied space's leaf order and form their tensor product. A count mismatch returns `ExpressionError::IncompatibleSpaces`. |
| `state_sum(&space, states)` | `state_sum(states, &space)` | Sum placed states on the supplied space. An empty sequence gives zero on that space. |
| `try_state_sum(&space, states)` | `try_state_sum(states, &space)` | Sum a sequence of `Result` items. Return the first input error before validating the composition. |
| `operator_product_at([(&factor, primitive)])` | `operator_product_from([(primitive, &factor)])` | Take a sequence of `(primitive, &factor)` pairs and form their product in input order. Repeated factors are allowed, the rightmost operator acts first, and the result is unbound. The first placement error stops iteration. Empty input gives unbound identity. |

`state_tensor` and `operator_product` name the operations on placed expressions.
The `_from` suffix identifies the corresponding constructors from primitives.
This suffix keeps the paired names short. The usual Rust conversion constructor
uses `from_<source>`, so the first sentence of each `_from` method's rustdoc
should state the input form explicitly. The `try_` prefix continues to identify
the variants that collect `Result` items and stop at the first input error.
[Rust constructor naming](https://rust-lang.github.io/api-guidelines/predictability.html#constructors-are-static-inherent-methods-c-ctor).

## Expression and space order

Use `(expression, space)` wherever an expression is paired with its space and
the current positional order has no separate purpose. Apply this throughout the
workspace, including public APIs, internal helpers, tuples, and tuple variants.
The expression may be a primitive or a sequence. For placement, the factor is
the space component. In `operator_product_from`, each assignment therefore has
item type `(O, &'a HilbertSpace<I>)`.

| Area | Planned order |
| --- | --- |
| Core state constructors | `State::sum(states, &space)`, `State::try_sum(states, &space)`, `State::tensor_in(states, &space)`, and `State::try_tensor_in(states, &space)`. Apply the same order to `tensor_collected` and the factory delegation. [State composition](../../../crates/syntaqs-core/src/expression/composite.rs). |
| Core expression nodes and evaluation | Use `StateNode::Local(local, factor)`, `OpNode::Local(local, factor)`, and `Terms::local(poly, factor)`. Move the local operator before its factor in evaluation tasks `Local` and `Placed`. Update constructors and patterns, including grouping, traversal, evaluation, and formatting. [Expression nodes](../../../crates/syntaqs-core/src/expression/composite.rs), [evaluation tasks](../../../crates/syntaqs-core/src/evaluate/composite.rs), [state term collection](../../../crates/syntaqs-core/src/evaluate/state_terms.rs). |
| Lie operator families | `OperatorFamily::new_in(operators, &context)`, with the context after the operator sequence. [Family construction](../../../crates/syntaqs-lie/src/family.rs). |

Review the remaining space-first forms across the workspace and update callers,
tuple construction, and destructuring in library code, model builders, examples,
and tests. Record the reason for any space-first order that an existing contract
requires, including comparison priority where field order determines it. Keep
the order within each input sequence and the existing validation, error
precedence, and evaluation behavior. Update doctests, documentation, and
migration notes for changed calls and tuple shapes.

## Basis element access and coefficient expansion

Split element access from coefficient expansion so callers can inspect a basis
without selecting a coefficient type. On `db16c4d`, with `Basis` in scope,
`SzBasis.element(0)` produces E0283 even with an explicit
`Option<SpinHalfState>` result type. The result fixes the primitive type, and
the current `Basis<P, C>` still needs an explicit `C` to select its implementation.
[Current basis trait](../../../crates/syntaqs-core/src/basis.rs),
[spin-half basis](../../../crates/syntaqs-spin-half/src/basis.rs).

- `BasisElements<P>` provides `element`, `index`, and `elements`, including
  the default iterator built from `element`. A given basis value and primitive
  type have one element order across supported coefficient types.
- `Basis<P, C>` extends `BasisElements<P>` and provides `expand_indices` and
  `expand`. Each implementation supplies its own coefficient bounds for
  expansion.
- `HilbertSpace::basis_states` and `basis_operators` use `BasisElements<P>`
  for lookup and enumeration. The coefficient type of the returned expression
  still comes from its type context. Expansion and coordinate queries use
  `Basis<P, C>`. [Enumeration](../../../crates/syntaqs-core/src/basis/enumeration.rs).

Migrate existing basis implementations by moving their lookup and enumeration
methods to `BasisElements<P>`. Expose both traits in the core prelude and update
imports, tests, and documentation examples. Verify `SzBasis.element(0)` with
`BasisElements` in scope and no coefficient annotation. Check state and operator
enumeration with an element-only basis, and rerun expansion and export checks
to confirm the basis order and coordinate interpretation.

## Local operator trace

Remove the `O: Identity` bound from `LocalOperator::trace()`. Its implementation
sums the weighted traces of evaluated primitives using `Product`, `Adjoint`,
and `Trace`. The normalized `hs_inner_product()` requires `Identity` to compute
the denominator `Tr(I)`. Verify that a custom primitive implementing only the
required operator rules can call `LocalOperator::trace()` and obtain the
expected trace. [Local trace implementation](../../../crates/syntaqs-core/src/evaluate/trace.rs).

## Matrix export examples

Use `MatrixArgs::new(&basis)` in examples where the basis uniquely determines
the state primitive type. Keep the result type annotation that selects the
output container, such as `Exported<Vec<_>, _>`. On `db16c4d`, this form
compiled and exported matrices with `SzBasis` for both local and placed
spin-half operators. The `Basis<S, C>` bound on export lets Rust infer `S`
from that basis once it knows the coefficient type `C` from the operator.
[Matrix export bounds](../../../crates/syntaqs-export/src/matrix/export.rs),
[Heisenberg example](../../../examples/src/bin/heisenberg.rs).

Document `MatrixArgs::<S, _>::new(&basis)` as the way to choose a state type
when multiple types satisfy the export bounds. Verify the examples with their
enabled output backends after simplifying the annotations.

## Lie decomposition generator expressions

Return compact generator expressions from `decompose` for the center and each
simple ideal. The shared component constructor already collects reconstructed
operators through `collected_in`, which calls `LocalOperator::collect_terms`
or `Operator::collect_terms_in`. Both collection paths can build scale nodes
with coefficient one when turning terms back into expressions.
[Component reconstruction](../../../crates/syntaqs-lie/src/decomposition/spectral.rs),
[collection adapters](../../../crates/syntaqs-lie/src/expression.rs),
[local collection](../../../crates/syntaqs-core/src/evaluate/collect.rs),
[placed expression reconstruction](../../../crates/syntaqs-core/src/evaluate/operator.rs).

A Pauli-family probe on `db16c4d` reproduced `(1+0i) * I` for the center of
both local and placed families and an outer `(1+0i) * ...` on placed simple
ideal generators. The placed collector already removed the identity factor
from `Sx[a] * I[b]`, leaving `(1+0i) * ((1+0i) * Sx)[a]`.

Build compact syntax when materializing collected terms. Omit scale nodes for
coefficients exactly equal to one and omit identities used as neutral product
factors. Keep genuine scalar terms, including a central generator equal to
identity, and preserve nonzero coefficients at their computed precision.
Retain the component coordinates, ambient binding, and reconstruction checks.
Use the exact comparison capabilities already available for Lie coefficients
while preserving the bounds of generic local collection. Display continues to
render the stored expression structurally.
[Formatting contract](../../design/expression-formatting.md#1-scope-and-type-bounds).

Verify the returned expressions for local and placed decompositions. The Pauli
center should display as `I`, and a placed product with an identity factor
should reduce to `Sx[a]`. Check a nonzero additive identity term and a
coefficient close to one to confirm that the cleanup preserves their values.
