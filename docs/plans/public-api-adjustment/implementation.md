# Public API adjustment implementation plan

Apply the [agreed signatures and behavior](refactor.md) in five units. Migrate
each signature together with its callers so the workspace can be checked after
each unit. The baseline is `db16c4d`, and the release migration comparison is
`v0.5.0`.

| Unit | Change | Verification focus |
| --- | --- | --- |
| 1 | Rename the factory methods and migrate public argument and assignment order | Workspace callers, placement order, explicit spaces, and error precedence |
| 2 | Put expressions before spaces in internal representations and helpers | Ordinary and graded evaluation, formatting, and remaining space-first pairs |
| 3 | Split basis element access from coefficient expansion | Unannotated element lookup, enumeration-only bases, and coordinate order |
| 4 | Relax the local trace bound and simplify matrix export examples | Custom primitive without `Identity`, and export inference on each used backend |
| 5 | Compact center and simple-ideal generator expressions | Exact unit and identity handling, reconstruction, and decomposition diagnostics |

## 1. Public construction calls

Rename `tensor_in`, `try_tensor_in`, `product_state`, and `operator_product_at`
on `ExpressionFactory` using the proposal's mapping. Move the space to the end
of the factory and direct `State` constructors that accept a sequence and an
explicit space. Update `State` delegation, including `tensor_collected`. Use
`(primitive, &factor)` for the items passed to `operator_product_from` and
`OperatorFamily::new_in(operators, &context)` for Lie family construction.
Update both assignment producers and their destructuring. Migrate aliases,
model builders, examples, tests, doctests, and API examples in documentation.
[Factory call mapping](refactor.md#expression-factory-names-and-argument-order).

Check all targets with `cargo check --workspace --all-targets --all-features`.
Use the existing `systems`, `explicit_state_space`, `fallible_construction`,
model, and `operator_family` tests to verify empty inputs, repeated factors,
rightmost-first action, retained space order, and first-error precedence.
Verify that `new_in` still rejects conflicting bindings and missing factors.

## 2. Internal expression and space order

Reorder placed `StateNode::Local`, `OpNode::Local`, evaluator task payloads,
`Terms::local`, and the pair returned by `Terms::local_parts`. Update the
construction and match sites together across grouping, traversal, evaluation,
basis reconstruction, and formatting. Inspect the rest of the workspace for
space-first arguments and tuples, including helpers in tests and examples.
Where positions encode a separate invariant, such as comparison priority in an
ordered key, check that invariant before changing the representation and record
the reason for any retained space-first order. [Ordering scope](refactor.md#expression-and-space-order).

Run Core's unit and formatting tests and the shared `core_tensor_products` and
`core_evaluation_pipeline` tests. Check factor order, graded signs, expression
sharing, and the existing deep-expression cases. Keep this unit's changes to
positional order so the later generator cleanup can be reviewed on its own.

## 3. Basis element access and expansion

Move `element`, `index`, and the default `elements` iterator to
`BasisElements<P>`. Make `Basis<P, C>` extend it and provide `expand_indices`
and `expand`. Expose both traits in the core prelude. Use `BasisElements<P>`
in `HilbertSpace::basis_states`, `basis_operators`, and the shared element
lookup and validation helpers. Keep the primitive dimension checks and the
coefficient type on returned expressions. Split existing implementations and
move their `elements` overrides with the lookup methods. Update the custom
basis examples and migration guidance. [Basis proposal](refactor.md#basis-element-access-and-coefficient-expansion).

Add coverage for `SzBasis.element(0)` without a coefficient annotation and
for state and operator enumeration with a basis that implements only
`BasisElements<P>`. Reuse `basis_capabilities`, `core_basis_expansion`, and
the export contract tests to verify element order, error behavior, and
coordinate interpretation, including nonorthonormal bases.

## 4. Local trace and matrix export examples

Remove `O: Identity` from `LocalOperator::trace()`. Add a case to the existing
primitive contract tests for an operator with `Product`, `Adjoint`, and `Trace`
whose local expression yields the expected trace. The normalized HS tests
continue to cover the use of `Tr(I)`.
[Trace proposal](refactor.md#local-operator-trace).

Use `MatrixArgs::new(&basis)` in standard-basis examples after the trait split
in unit 3. Keep the result annotation that selects the container. Document
`MatrixArgs::<S, _>` for ambiguous state types. Build and run the changed export
examples with their enabled backends, including the Heisenberg example with
`cargo run -p examples --features export --bin heisenberg`.
[Matrix export example proposal](refactor.md#matrix-export-examples).

## 5. Lie generator expressions

Keep term aggregation in Core and use it from the shared Lie component
constructor. When rebuilding placed expressions from collected terms, omit an
exact unit scale on local terms, product terms, and the outer product. For
local expressions, share the existing term aggregation and use a
comparison-capable Core entry point from the Lie adapter. Follow the existing
`#[doc(hidden)]` cross-crate evaluator hooks for this entry point so the generic
`LocalOperator::collect_terms()` retains its present bounds. Use the resulting
syntax with the existing structural Display implementation.
[Generator proposal](refactor.md#lie-decomposition-generator-expressions).

Extend the existing collection and decomposition tests to check both local
and placed generators. Verify an identity center, a redundant identity factor
in a product, a genuine additive identity term, and a coefficient near one.
Check simple ideals without assuming a particular eigenvector orientation or
order. Run the existing ordinary and graded tests at both precisions to verify
coordinates, binding, reconstruction, and diagnostics. Include the Lie-only
example and the shared Lie integration tests in the verification run.

## Workspace verification and migration

Run the [repository verification matrix](../../design/verification.md#2-common-checks)
for default, all, and no default features, including doctests, formatting,
Clippy, and rustdoc. Check Vec, ndarray, and sprs independently so workspace
feature unification does not hide missing imports or bounds. Use the configured
MSRV and stable checks where available and record any verification gaps.

Review the final diff for stale factory names, old call and tuple orders,
missing basis implementation migrations, and unrelated edits. Check the
affected design documents and [v6 migration notes](../v6/README.md) against the
implemented signatures, with `v0.5.0` as the release reference. Record verified
results and remaining limitations in the [workstream README](README.md).
