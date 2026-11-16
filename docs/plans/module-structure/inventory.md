# Workspace inventory and proposed moves

Paths are relative to the repository root. This is a static inventory and a proposed implementation map, not a claim that the moves have been compiled. The [execution plan](README.md) defines the scope of this pass. The API assessment records a later direction; its broad API migrations are deferred. The tree below is a historical candidate, not a final API or a mandatory list of moves. Public paths may change; update workspace consumers and documentation along with each batch.

## Historical candidate crate and module structure

The tree shows Rust module ownership, not public visibility. Each top-level entry is a separate workspace crate; `tests` denotes test-only child modules where shown, while integration test files are independent crate roots. Braces group sibling modules. Feature gates and existing inline private modules remain as they are. Proposed children and the exact placement of mixed tests remain subject to the migration checks described below. Do not interpret this tree as the final public API.

```text
syntaqs-core
├── basis
│   ├── coordinates
│   ├── enumeration
│   └── expansion
│       ├── composite
│       ├── local
│       └── operator
│           └── tests
├── coefficient
├── error
├── evaluate
│   ├── collect
│   ├── composite
│   ├── inner_product
│   ├── local
│   ├── state_terms
│   ├── trace
│   └── tests
│       ├── state
│       └── stopping
├── expr
│   ├── arithmetic
│   ├── composite
│   │   └── tests
│   ├── format (inline tests)
│   ├── group
│   │   └── tests
│   ├── local
│   │   └── tests
│   ├── parts
│   │   ├── linear_combination
│   │   ├── operation
│   │   ├── scale
│   │   └── sum
│   └── traversal (inline tests)
├── ops
├── space
├── system
└── test_coefficient (test-only; references tests/common/mod.rs)

syntaqs-basic                   (no named source modules)
syntaqs-export
├── args
├── backend
│   ├── ndarray
│   ├── sprs
│   └── vec
├── coordinates
├── error
├── matrix
├── metadata
├── output
└── vector
syntaqs-fermion                {basis, coefficient, models, operator, state}
syntaqs-geometry
├── boundary
├── connection
├── geometry
│   └── chain
├── group
└── relation
    ├── consecutive (inline tests)
    └── nearest_neighbor
syntaqs-lie
├── decomposition
│   ├── spectral
│   └── tests
├── error
├── expression (inline private module)
├── family
│   ├── projection
│   └── validation
├── invariant
│   └── tests
├── numerical
│   └── tests
├── representation
├── tolerance
└── transform
syntaqs-normalize              {error, expectation, norm}
syntaqs-spin-half              {basis, coefficient, models, operator, state}
syntaqs-spin-one               {basis, coefficient, models, operator, state}
workspace tests library        {graded, three_level}
BLAS provider support library  (no named source modules)
examples                       (independent binaries; inline tests remain)
```

Integration tests remain separate crate roots and use `mod common;` for shared helpers where needed. The core coefficient fixture is shared by unit and integration tests from `tests/common/mod.rs` as described below. The excluded `tests/fixtures/lie-blas` fixture remains an independent crate with its existing `main.rs`. A cross-expression test identified during migration may instead require an `expr::tests` child; it is not assumed to exist in this proposed tree.

## Module entry files

Move each entry file below to the indicated path; its existing child directory stays in place. Shared integration-test helpers are the explicit `mod.rs` exception described below.

| Current | Destination |
| --- | --- |
| `crates/syntaqs-core/src/basis/expansion/mod.rs` | `crates/syntaqs-core/src/basis/expansion.rs` |
| `crates/syntaqs-core/src/evaluate/mod.rs` | `crates/syntaqs-core/src/evaluate.rs` |
| `crates/syntaqs-core/src/expr/mod.rs` | `crates/syntaqs-core/src/expr.rs` |
| `crates/syntaqs-export/src/backend/mod.rs` | `crates/syntaqs-export/src/backend.rs` |
| `crates/syntaqs-geometry/src/geometry/mod.rs` | `crates/syntaqs-geometry/src/geometry.rs` |
| `crates/syntaqs-geometry/src/relation/mod.rs` | `crates/syntaqs-geometry/src/relation.rs` |
| `crates/syntaqs-export/tests/support/mod.rs` | `crates/syntaqs-export/tests/common/mod.rs` |
| `crates/syntaqs-fermion/tests/support/mod.rs` | `crates/syntaqs-fermion/tests/common/mod.rs` |
| `crates/syntaqs-spin-one/tests/support/mod.rs` | `crates/syntaqs-spin-one/tests/common/mod.rs` |

### Shared integration-test fixtures

Across all crates, place shared integration-test helpers at `tests/common/mod.rs` as an explicit exception to the named-module file convention. For the current inventory, move each of the three `tests/support/mod.rs` files to `tests/common/mod.rs`. Each existing integration-test crate that uses the helper declares `mod common;` and updates its imports and references from `support` to `common`. This avoids automatic discovery of a separate helper test target while retaining the existing test targets and avoiding `#[path]`.

| Package | Integration-test crates using `common` |
| --- | --- |
| `syntaqs-export` | `exports.rs` |
| `syntaqs-fermion` | `contracts.rs`, `minimal_scalar.rs`, `models.rs`, `local_expressions.rs` |
| `syntaqs-spin-one` | `contracts.rs`, `minimal_scalar.rs`, `tensor_products.rs`, `models.rs`, `local_expressions.rs` |

Use the same location for any other shared integration-test helpers identified during the inventory or migration. Core unit tests may use `#[path]` to share the coefficient fixture from this location as described below. Other integration-test files and all existing crate entry points remain at their current paths. Verify Cargo target discovery and run all consuming tests after the moves.

## Unit tests

All paths in this section are under `crates/syntaqs-core/src/` unless stated otherwise. Change declarations and relative imports with the moves; a child test can access ancestor-private items but cannot assume a former sibling's private items are accessible.

| Current | Proposed destination and ownership |
| --- | --- |
| `expr/composite_tests.rs` | `expr/composite/tests.rs`: composite construction and operations. Contraction/evaluation cases cross implementation boundaries; keep them here initially as composite behavior and reassess individual cases during migration. |
| `expr/group_tests.rs` | `expr/group/tests.rs`: grouping behavior. |
| `expr/tests.rs` | `expr/local/tests.rs`: primarily local construction, action and evaluation. Review each test for a broader owner; move any genuinely cross-expression cases to `expr/tests/` rather than forcing them into local. |
| `evaluate/state_tests.rs` | `evaluate/tests/state.rs`: state collection through local and composite evaluation. |
| `evaluate/stopping_tests.rs` | `evaluate/tests/stopping.rs`: stopping across inner product and trace. Update direct `super::inner_product` access for the extra nesting. |
| `basis/expansion/operator_tests.rs` | `basis/expansion/operator/tests.rs`: already declared as `operator::tests`; remove its explicit `#[path]`. |

Introduce `evaluate/tests.rs` as the test-only parent declaring `state` and `stopping`. Existing inline tests in `expr/format.rs`, `expr/traversal.rs`, `geometry/relation/consecutive.rs`, and example binaries already belong to their containing modules; existing `syntaqs-lie` tests in `decomposition/tests.rs`, `invariant/tests.rs`, and `numerical/tests.rs` already follow the intended relationship. Keep integration tests as separate crate roots: their public-facing contracts do not require moving into implementation modules.

## Responsibility boundaries

- `syntaqs-core::expr::parts`: move `SumExpr` and `IntoOperands` with their iterator implementations to `parts/sum.rs`; move `ScaleExpr` to `parts/scale.rs`; group the small operation nodes `ProductExpr`, `ActExpr`, and `AdjointExpr` in `parts/operation.rs`. Keep `linear_combination.rs`. Make `parts.rs` the module entry, adjust visibility and imports as required by the structural split, and record public path changes for the later API phase. The sum iterator belongs with its storage because its representation and iteration guarantees are coupled.
- `syntaqs-lie::family`: separate family construction and shared identity from diagnostics and Gram/projection operations. Proposed children are `family/validation.rs` for independence and Hermiticity reports, and `family/projection.rs` for Gram matrix, factorization, coordinates and projection. Keep `OperatorFamily` and its basic construction in `family.rs`. Inspect private field access and shared numeric helpers before placing individual implementations; avoid widening visibility beyond the enclosing module as needed.
- `syntaqs-lie::decomposition`: retain its public decomposition types and entry methods in `decomposition.rs`; move candidate sampling, grouping and validation machinery into `decomposition/spectral.rs`, including private helper types as their dependencies permit. Keep `decomposition/tests.rs` under the parent. Verify shared access with invariant and numerical modules.
- `syntaqs-core::expr::composite` and `local`: state and operator representations have closely coupled node types, action and destruction. Keep their current boundaries for this pass; size alone does not justify introducing extra visibility or splitting their invariants. Reassess if test relocation exposes a coherent independent responsibility.
- `syntaqs-spin-one::operator`: the long product implementation is primarily one direct multiplication table over the same enum. Keep the table together; splitting solely by line count would obscure the rule. Other reviewed crates already organize primary features into separate named files; no additional split is proposed without evidence of distinct responsibilities.

## Path-sensitive exceptions and verification

Move `crates/syntaqs-core/tests/support/coefficient.rs` to `crates/syntaqs-core/tests/common/mod.rs`. In `tests/linear_combination.rs`, replace the path override and `mod coefficient` with `mod common` and import `common::MinimalComplex`. In `src/lib.rs`, retain a test-only `test_coefficient` module with `#[path = "../tests/common/mod.rs"]` so the three unit tests can share the same fixture. This is an explicitly permitted path override across the unit/integration-test crate boundary; no duplicate coefficient implementation is needed. The operator test override is removed as described above. Check the workspace for any other path overrides and justify each remaining use.

A workspace text search found no `include!` references; repeat the search and check manifests, scripts and documentation when applying moves. Inline modules need no file rename. Verify Rust module resolution, imports, privacy, feature-gated builds, public paths and doctests per affected batch, then run workspace-required checks. Check Cargo's discovered test targets after the helper moves.
