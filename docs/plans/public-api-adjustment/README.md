# Public API adjustment

Use representative caller workflows to identify which public APIs need
adjustment and how each change affects existing callers. The initial source
baseline is `v6` at `db16c4d`. The `v0.5.0` tag is the comparison
point for migration notes. The first pass covers imports, construction, space
selection, evaluation, and numerical output. The [refactoring proposal](refactor.md)
records the agreed changes, and the [implementation plan](implementation.md)
defines their order and verification.
[API usability candidates](../../future-work/api-usability.md).

## Current call paths

The two-spin example selects primitives and coefficients through
`SpinHalf<Complex<f64>>`, constructs a `HilbertSpace`, and builds a product
state in leaf order with `state_tensor_from`. It places and multiplies local
operators through `operator_product_from`, binds the result to the full space,
and calls `State::expectation`. [Two-spin example](../../../examples/src/bin/two_spins.rs).

| Workflow | Current entry points and responsibilities |
| --- | --- |
| Imports and type selection | Each library provides a `prelude`, and semantic modules expose the defining paths. `ExpressionFactory<S, O, C>` supplies constructors, and `QuantumSystem` exposes the associated primitive and coefficient types. `SpinHalf<C>`, `SpinOne<C>`, and `Fermion<C>` are factory aliases. [Module conventions](../../architecture.md#5-module-structure), [factory implementation](../../../crates/syntaqs-core/src/system.rs). |
| Factors and expressions | `HilbertSpace` holds factor identity, order, grouping, and tensor kind. Local expressions are placed through `at`. `State` always has a space, and `Operator::bind` selects a full ambient space for an operator. `act` constructs a delayed state expression, while queries such as `expectation` use the evaluator internally. [Space API](../../design/hilbert-space.md#9-public-apis), [expression API](../../design/composite-expression.md#3-current-public-api). |
| Geometry and models | `SiteMapper` borrows a geometry and caller-supplied data in site order. Concrete `models` modules build operators using caller-supplied site spaces. [Geometry](../../design/geometry.md#3-spaces-and-local-factors), [model construction](../../design/models.md). |
| Bases and numerical output | `BasisElements` defines the ordered primitives. `Basis` expands primitives into coefficients in that order. `Export::export` selects the container through its result type. Operator matrix export uses `MatrixArgs` to select a state primitive and basis, with `in_space` available for an explicit ambient space. Norms and normalized expectations have separate extension traits. [Basis](../../design/basis.md), [export](../../design/export.md#3-public-api-and-ownership), [normalization](../../design/normalization.md). |
| Lie analysis | `OperatorFamily::new` and `new_in` establish a common expression context. Validation, projection, closure, and decomposition are explicit operations on the family and intermediate results. [Family implementation](../../../crates/syntaqs-lie/src/family.rs), [Lie analysis](../../design/lie.md). |

Compared with `v0.5.0`, the `v6` baseline requires `I: Ord` for tensor-space
construction and composite operations that resolve factor positions.
`HilbertSpace::flatten` and `tensor_flatten` promote one level of grouping.
Custom bases now supply `element` and `index` through `BasisElements` and
`expand_indices` through `Basis`. The default `expand` resolves indices and
returns a `Result`, and `elements` has a default implementation.
[Version notes](../v6/README.md), [basis trait](../../../crates/syntaqs-core/src/basis.rs).

## Stages and progress

- [x] Locate the main public entry points, follow the two-spin workflow, and
  record the starting revision and known API changes.
- [x] Review representative callers and record the agreed API, capability, and
  generator expression changes.
- [x] Organize the changes into implementation units and define their checks.
- [x] Implement and verify [unit 1](implementation.md#1-public-construction-calls),
  including caller migration and the [v0.6.0 migration table](../v6/README.md#expression-and-family-construction).
- [x] Implement and verify [unit 2](implementation.md#2-internal-expression-and-space-order).
- [x] Implement and verify [unit 3](implementation.md#3-basis-element-access-and-expansion).
- [ ] Implement units 4–5 and run the final [verification matrix](../../design/verification.md#2-common-checks).

For unit 1, workspace all-target checks with all and no default features passed,
as did all-feature tests and doctests, Clippy, rustdoc, and formatting. An
external crate compiled 11 current public call forms and verified compiler
errors for 15 previous forms. The `two_spins` and Lie-only `lie_algebra` examples
passed. The root quick start also ran against its declared `v0.5.0` source.

For unit 2, placement nodes, evaluator tasks, basis reconstruction, and test
and example helpers use expression-first order. Derived `Debug` shows the new
node field order. `Key::Local` in the state evaluator keeps the factor first
in its derived comparison. Sparse operator term keys keep the ambient factor
position first for position-based lookup and merging.
[State shape keys](../../../crates/syntaqs-core/src/evaluate/state_terms.rs),
[sparse operator terms](../../../crates/syntaqs-core/src/evaluate/operator.rs).

All 95 planned tests passed across Core's unit and formatting tests and the
shared `core_tensor_products` and `core_evaluation_pipeline` targets.
Workspace tests and doctests passed with all features. All-target checks passed
with all features and with default features disabled, as did Clippy and
formatting. The `pauli_strings` example produced the expected 37 strings.

For unit 3, the `SzBasis.element(0)` doctest passed without a coefficient type
annotation. The `basis_capabilities` tests passed for state and operator
enumeration with an element-only basis and `()` coefficients. The custom
basis test, `core_basis_expansion`, and export tests for Vec, ndarray, and sprs
passed. All-target checks passed with all features and with default features
disabled. Workspace all-feature tests and doctests passed, as did Clippy,
rustdoc, and formatting. The `basis_expansion` example produced state
coordinates 5 and −1 and one ladder-basis operator coordinate.
[Basis migration](../v6/README.md#local-basis-traits).

On `db16c4d`, 23 tests passed across the `syntaqs-core` integration targets
`public_api`, `systems`, `spaces`, `fallible_construction`, and
`basis_capabilities`. The `two_spins` example passed its assertions for
normalization, energies, eigenstate residuals, and orthogonality. The workspace
all-feature tests and doctests also passed during the API review.

## Completion criteria

- Each adopted change has a caller example that demonstrates the improvement
  and migration guidance for any affected existing usage.
- Affected workflows preserve factor identity and order, ordinary and graded
  evaluation rules, ambient-space selection, coefficient precision, and
  `try_` collection error precedence wherever those contracts still apply.
- Updated examples, doctests, and targeted contract tests cover the adopted
  APIs, including custom algebras and feature combinations when affected.
