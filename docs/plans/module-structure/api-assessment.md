# Semantic API decisions

## Policy and precedence

The module-structure refactoring is complete. The public API review is also complete; the agreed changes and preludes are planned in the [API implementation plan](api-implementation.md).

For the upcoming API reorganization, the user explicitly accepts breaking changes and does not require compatibility with the old module structure. The temporary `ops`, `expr`, and dimension compatibility re-exports are not constraints on the target API; remove obsolete compatibility paths as part of the planned API work. Preserve functionality and behavior unless their removal or change is explicitly decided.

The canonical public API follows the actual module hierarchy. Public re-exports are limited to `prelude` modules directly under each crate root; no other public facade or compatibility re-export is retained. Outside the root prelude, expose public items through their defining module paths, including any relevant child modules. Module and item visibility is specified below; this policy does not by itself designate every implementation detail as public. Root-defined items and semantic type aliases are definitions, not re-exports. Internal `use` and restricted-visibility imports are unaffected.

Crate roots are themselves meaningful namespaces: central crate-wide abstractions may be defined there without a redundant module. The user requested a prelude as a convenient entry point; provide one per public library crate, including small crates, with a deliberately limited selection. Prelude exports are public API too.

Within repository crates other than the `examples` crate, do not use another syntaqs crate's prelude, whether through a wildcard or an explicit import. Use the dependency's canonical public module paths instead. This applies to their library and binary targets, tests, examples, and documentation examples. The `examples` crate may use syntaqs preludes. Check this rule during the API migration.

These decisions describe the agreed target API, not yet implemented. Check the inventory, signatures, and practical compatibility of the decisions while implementing them. Preserve public functionality and observable behavior except for explicitly agreed changes to visibility or functionality.

## Decided canonical boundaries

- **core:** Rename `ops` to `algebra` and `expr` to `expression`. Define `Product`, `Act`, `Adjoint`, `Identity`, `Trace`, `InnerProduct`, and the current parity abstractions directly in `algebra`; keep grading there unless its scope later warrants a child module. Define `Dimension` and `DimensionOverflow` in `space`. `Dimension` is implemented by primitive types and describes the dimension of their spanned state or operator space, independently of a chosen basis. The `space` namespace therefore covers both state-space and operator-space dimensional concepts; basis enumeration and `HilbertSpace` queries both use them. Keep `system` and `coefficient` as independent concepts. The agreed expression visibility and canonical paths are detailed below. Keep evaluation, traversal, formatting, and grouping implementation private. Put `Basis`, `BasisError`, `Coordinates`, `dim_state`, and `dim_op` in `basis`, and keep enumeration/expansion machinery private: these are all basis-facing concepts, while `space::DimensionOverflow` captures the shared failure.
- **basic:** Define `Id128` and `Tag` in a public `index` module and include both in the crate-root prelude. This supersedes the earlier proposal to keep their definitions at the crate root.
- **geometry:** Use public `geometry`, `relation`, and `mapping` namespaces. Rename `connection` to `mapping`; it maps sites and groups to data rather than representing a differential-geometric connection. The agreed item paths and prelude are detailed below.
- **normalize:** Expose the `norm` and `expectation` domains and define their shared `NormalizationError` at the crate root. The agreed item paths and prelude are detailed below.
- **export:** Define the central `Export`, `Exported`, and shared `ExportError` at the crate root. Keep backend and coordinate extraction internals private. The agreed representation-specific paths, prelude, and external dependency policy are detailed below.
- **lie:** Expose `family`, `representation`, `invariant`, `decomposition`, and `numerical` as semantic domains. Keep numerical kernels and decomposition spectral machinery private. The agreed item paths and prelude are detailed below.
- **physical systems:** Keep the `SpinHalf`, `SpinOne`, and `Fermion` aliases at their respective crate roots; they identify the crate's system. Retain public `basis`, `operator`, `state`, and `models` consistently across all three crates, with private coefficient helpers. Keep `models` plural to match existing names. The agreed item paths and preludes are detailed below. The dedicated spin-half Neel constructor has been removed, and the Heisenberg example constructs its alternating product state from expression building blocks. The three physical-system errors are now `models::ModelError` without root re-exports; their variants and source behavior are retained.

`parts::sum`, `parts::scale`, and `parts::operation` remain the planned internal syntax-container split. Existing crate-root definitions such as system aliases are not re-exports and remain valid. Preserve feature gates, `no_std` requirements, and public signatures apart from agreed API changes.

The core algebra, grading, dimension placement, expression boundaries, and core prelude have been agreed, as have the basic, geometry, normalize, export, Lie, and physical-system APIs and preludes. Implementation-time verification may reveal details that require adjustment.

## Core compatibility aliases

Remove the temporary `syntaqs_core::ops` and `syntaqs_core::expr` module aliases, the `algebra::Dimension` re-export retained for `ops::Dimension`, and the `error::DimensionOverflow` compatibility re-export. The definitions of `Dimension` and `DimensionOverflow` remain in `space`. If the `error` module has no other responsibility after the re-export is removed, remove that empty module as well. Existing crate-root and parent-module re-exports are addressed by the general prelude-only policy and the agreed paths below.

## Core expression API

The expression tree is shared infrastructure. Users customize local algebra by implementing its primitive traits; user-defined tree nodes are not a supported extension point. Make the currently exposed `SumExpr`, `ScaleExpr`, `ProductExpr`, `ActExpr`, and `AdjointExpr` internal, with no public re-exports. The existing local and composite expression construction and evaluation capabilities remain available through their public types; this decision does not introduce a public tree traversal API.

Use these canonical defining paths, without parent-module re-exports outside the root prelude:

- `expression::local::{LocalState, LocalOperator}` and `expression::composite::{State, Operator}`; expose both modules.
- `expression::error::ExpressionError`; move the shared error out of `composite` and expose its module.
- `expression::parts::linear_combination::{LinearCombination, Term, IntoTerms}`; expose the module and retain the linear-combination construction and iteration capabilities used by local algebra implementations.

Make `Term` an independent public type rather than an alias for the internal `ScaleExpr`. Decouple the public `IntoTerms` iterator from the internal `IntoOperands` representation, retaining its iteration capabilities. The exact iterator representation is an implementation detail. Check public signatures, `from_terms` consumers, and iterator trait guarantees when implementing the change.

## Core prelude

The agreed core prelude exports these commonly used types and traits from their canonical defining modules:

- `space::HilbertSpace`.
- `expression::local::{LocalState, LocalOperator}` and `expression::composite::{State, Operator}`.
- `algebra::{Act, Adjoint, Product, Identity, InnerProduct, Trace}` and `basis::Basis`.
- `expression::parts::linear_combination::LinearCombination`.
- `system::ExpressionFactory`, as the expression-construction entry point for user-defined local algebras.

`QuantumSystem` and the system-specific expression aliases, dimension, parity and coefficient traits, `Term`, `IntoTerms`, errors, and other specialized types remain available through their canonical paths without inclusion in the core prelude. When selecting other crates' preludes, test common combinations for naming conflicts, particularly the generic names `State` and `Operator`.

## Other core APIs

Retain the following currently public items at their canonical defining paths. The prelude selection above specifies the convenience exports; all other access uses the paths below.

| Canonical module | Items |
| --- | --- |
| `algebra` | `Act`, `Adjoint`, `Product`, `Identity`, `InnerProduct`, `Trace`, `Parity`, `HomogeneousParity`, `ParityOperator` |
| `basis` | `Basis`, `BasisError`, `Coordinates`, `dim_state`, `dim_op` |
| `coefficient` | `Conjugate`, `RealPart`, `ImaginaryUnit`, `Divide` |
| `space` | `HilbertSpace`, `Dimension`, `DimensionOverflow`, `TensorKind`, `SpaceError`, `Indices`, `Leaves` |
| `system` | `QuantumSystem`, `ExpressionFactory`, `LocalStateOf`, `LocalOperatorOf`, `StateOf`, `OperatorOf` |

Move the definitions of `BasisError`, `Coordinates`, `dim_state`, and `dim_op` to `basis` rather than publicly re-exporting them from children; basis expansion and enumeration implementation can remain private. Keep the public space iterators accessible, including `Leaves`, which is used in a geometry API signature. Preserve existing public methods, including `HilbertSpace::shares_node_with`, and their behavior.

## Geometry API and prelude

Retain all currently public geometry items, defining them at these canonical paths. Remove the crate-root and intermediate public re-exports, except for the prelude exports listed below.

| Canonical path | In prelude |
| --- | --- |
| `geometry::Geometry` | Yes |
| `geometry::Boundary` | Yes |
| `geometry::chain::Chain` | Yes |
| `relation::Relation` | Yes |
| `relation::SiteGroup` | No |
| `relation::SiteGroupError` | No |
| `relation::consecutive::Consecutive` | Yes |
| `relation::consecutive::ConsecutiveError` | No |
| `relation::nearest_neighbor::NearestNeighbors` | Yes |
| `mapping::SiteMapper` | Yes |
| `mapping::MappedGroup` | No |
| `mapping::MappedGroupResult` | No |
| `mapping::MappingError` | No |

Expose the corresponding modules. Move the definitions of `Boundary`, `SiteGroup`, and `SiteGroupError` into their listed parent modules rather than re-exporting them from child modules. Preserve the existing behavior, including public error handling and support for custom geometries and relations.

## Normalize API and prelude

Retain all five currently public items at these canonical paths. Expose the `norm` and `expectation` modules and remove their crate-root trait re-exports outside the prelude.

| Canonical path | In prelude |
| --- | --- |
| `norm::Norm` | Yes |
| `norm::Normalize` | Yes |
| `expectation::NormalizedExpectation` | Yes |
| `expectation::NormalizedExpectationIn` | Yes |
| `NormalizationError` | No |

The four extension traits are commonly imported to call their methods; `NormalizedExpectationIn` provides the explicit-ambient-space variant. Define `NormalizationError` directly at the crate root because it is shared by normalization and expectation operations. Keep the `nonzero` helper private. Preserve the operations' mathematical behavior and error conditions.

## Export API and prelude

Retain all currently public export-owned items at these canonical paths. Expose the listed representation-specific child modules and remove the existing parent and crate-root re-exports outside the prelude.

| Canonical path | In prelude |
| --- | --- |
| `Export` | Yes |
| `Exported` | Yes |
| `ExportError` | No |
| `matrix::args::MatrixArgs` | Yes |
| `matrix::metadata::MatrixMetadata` | No |
| `matrix::metadata::Layout` | No |
| `vector::metadata::VectorMetadata` | No |

Define `Exported` and `ExportError` directly at the crate root, alongside the existing `Export` definition. The child paths supersede the earlier proposal to define representation-specific types directly in `matrix` and `vector`; they preserve the agreed thin module entry files without parent re-exports. Keep conversion implementations, backend machinery, and coordinate extraction private.

Remove the `ndarray` and `sprs` public re-exports. Preserve their feature-gated export functionality, update consumers to depend on those crates directly, and check that their dependency versions are compatible with the types used by `syntaqs-export`.

## Lie API and prelude

Retain all currently public Lie items at these canonical paths. Expose the listed domain and child modules and remove the existing parent and crate-root re-exports outside the prelude.

| Canonical module | Items | In prelude |
| --- | --- | --- |
| crate root | `FamilyExpression`, `LieError` | No |
| crate root | `lie_bracket` | Yes |
| `family` | `OperatorFamily` | Yes |
| `family::projection` | `FamilyCoordinates`, `GramFactorization`, `GramMatrix`, `Projection` | No |
| `family::validation` | `FamilyValidation`, `IndependenceReport` | No |
| `family::transform` | `FamilyTransform`, `OrthonormalityReport` | No |
| `representation` | `AdjointRepresentation`, `BracketClosureCheck`, `ClosureReport`, `KillingForm`, `StructureConstants` | No |
| `invariant` | `AdjointCommutant`, `CenterSplit`, `ConstraintDiagnostics` | No |
| `decomposition` | `DecompositionOptions`, `DecompositionAttempt`, `DecompositionDiagnostics`, `DecompositionFailure`, `DecompositionValidation`, `LieComponent`, `LieDecomposition` | No |
| `numerical::options` | `NumericalOptions` | Yes |
| `numerical::tolerance` | `Tolerance`, `ToleranceCheck` | No |
| `numerical::error` | `NumericalError` | No |

Define `FamilyExpression`, `LieError`, and `lie_bracket` directly at the crate root instead of re-exporting their definitions from children. `FamilyExpression` remains sealed. Keep `DecompositionOptions` in `decomposition` because it controls decomposition-specific sampling, and keep numerical kernels and decomposition spectral machinery private. Preserve numerical behavior and access to diagnostics.

## Physical-system APIs and preludes

Retain the currently public items in all three physical-system crates at their canonical paths below. Remove crate-root re-exports of child-module items outside the prelude; the system aliases are root definitions and remain there.

| Crate | Canonical path | In prelude |
| --- | --- | --- |
| spin-half | `SpinHalf` | Yes |
| spin-half | `state::SpinHalfState`, `operator::SpinHalfOperator` | Yes |
| spin-half | `models::SpinHalfModels` | Yes |
| spin-half | `basis::{SzBasis, CartesianBasis, LadderBasis}`, `models::ModelError` | No |
| spin-one | `SpinOne` | Yes |
| spin-one | `state::SpinOneState`, `operator::SpinOneOperator` | Yes |
| spin-one | `models::SpinOneModels` | Yes |
| spin-one | `basis::{SzBasis, CartesianBasis, LadderBasis}`, `models::ModelError` | No |
| fermion | `Fermion` | Yes |
| fermion | `state::FermionState`, `operator::FermionOperator` | Yes |
| fermion | `models::FermionModels` | Yes |
| fermion | `basis::{OccupationBasis, LadderBasis, MajoranaBasis}`, `models::ModelError` | No |

The prelude of each crate provides its factory, state and operator primitives, and model constructors. The distinct `*Models` names can coexist across preludes. Import basis types and model errors explicitly: several have identical names across crates. Check combined prelude usage when implementing the API migration.

## Expression, export and numerical file layout

The core, export, and numerical file layouts below are agreed. Their public visibility and canonical paths are specified above. A file layout alone does not authorize parent-module public re-exports.

- **core expression parts:** keep `expression/parts/linear_combination.rs` for `LinearCombination`, `Term`, and `IntoTerms`, `sum.rs` for `SumExpr` and its owning iterator, `scale.rs` for `ScaleExpr`, and `operation.rs` for product, action, and adjoint syntax. Rename the enclosing `expr` module to `expression` as agreed. The existing split already has these child files.
- **export:** keep `matrix.rs` and `vector.rs` as thin module entry points. Under `matrix/`, separate arguments (`args.rs`), matrix metadata and layout (`metadata.rs`), and conversion implementations (`export.rs`). Keep the matrix-specific internal `Ambient` trait with the arguments or conversion implementation according to its access needs. Under `vector/`, separate vector metadata (`metadata.rs`) and conversion implementations (`export.rs`).
- **lie numerical:** keep `numerical.rs` as a thin module entry point. Under `numerical/`, separate tolerances (`tolerance.rs`), numerical options (`options.rs`), numerical errors (`error.rs`), and internal calculation kernels (`kernels.rs`). Move the existing kernel tests under `numerical/kernels/tests.rs`. Keep `LieError` outside `numerical`, as it also covers non-numerical failures. Assess shared validation helper placement while implementing.

These layouts preserve existing behavior and test coverage. The file names alone do not determine child-module visibility, canonical paths, or prelude membership.

## Implementation-time checks

1. **Exact public item inventory and paths:** inspect every current public item, including iterator types, aliases, associated types and externally reachable signature types; assign any not listed above and check that no public capability becomes unreachable except where explicitly agreed. Verify the decided expression paths and signatures without exposing implementation solely for Rust visibility.
2. **Prelude validation:** selections for all public library crates are agreed above. Test common combinations for wildcard conflicts and verify the cross-crate prelude usage rule.
3. **Detailed implementation boundaries:** determine visibility and placement for shared helpers without exposing internals. Confirm the planned file splits through implementation and rustdoc checks.

## Neel removal and integration tests

The Heisenberg example constructs a Neel state from alternating primitive states placed on the existing factor spaces. Former constructor calls in `crates/syntaqs-spin-half/tests/systems.rs` and `examples/tests/chain.rs` have been replaced with expression building blocks.

Integration tests use simpler product states where alternation is incidental, and construct alternating basis states for analytic Neel energy and export checks. Hamiltonian construction, evaluation, empty and grouped spaces, boundaries, feature-gated exports and model error handling remain covered. Constructor-only assertions and capability checks were removed; no public Neel helper was added.

## Migration and validation

Update workspace consumers, examples, README snippets, doctests, feature-gated code and the excluded BLAS fixture. Inspect dependency declarations when replacing external-crate re-exports; preserve supported feature combinations. Build canonical-path and prelude usage examples, check cross-crate prelude use against the rule above, check rustdoc and reachable signature types, run relevant tests per batch followed by workspace-required checks, and compare public item inventories for unintended omissions. Test changes to trait bounds or visibility must not silently narrow existing behavior.
