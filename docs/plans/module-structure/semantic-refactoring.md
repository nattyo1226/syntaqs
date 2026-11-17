# Semantic module refactoring plan

## Objective and boundaries

Implement the agreed semantic module names and file organization recorded in the [API assessment](api-assessment.md), after the completed structural pass. Preserve behavior, trait bounds, feature gates, error semantics, supported Rust versions, and the existing public surface as far as possible. This document plans the work; it does not authorize implementation.

The eventual direction is canonical semantic module paths and public re-exports only in preludes. In this intermediate phase, the user's request to preserve existing visibility and usable public paths takes precedence: retain existing re-exports and use narrowly scoped temporary aliases where a move would otherwise break an existing public path. Record every such alias and any unavoidable visibility or path change for the later API phase. Do not remove dependency re-exports, add preludes, systematically expose private modules, or undertake the complete public API inventory here. Do not retain duplicate definitions: old and new paths must refer to the same types and traits. Public paths used by callers should remain usable; compiler-generated type names, rustdoc definition locations, and module paths may change even when aliases preserve imports. Verify those effects and document them rather than promising identical textual output.

## Target implementation layout

Names below describe file and module ownership, not final public visibility. Child entry files are thin; restricted imports may be used to connect implementation modules. Exact canonical API paths and eventual removal of temporary re-exports remain for the subsequent API work.

```text
syntaqs-core/src/
├── algebra.rs                    # former ops.rs; algebra and parity traits
├── expression.rs                 # former expr.rs
├── expression/                   # former expr/; existing children retained
│   └── parts/{linear_combination,operation,scale,sum}.rs
├── space.rs                      # Dimension, DimensionOverflow, existing space types
└── error.rs                      # temporary compatibility entry if needed

syntaqs-geometry/src/
├── geometry.rs
├── geometry/{boundary,chain}.rs
├── relation.rs
├── relation/{group,consecutive,nearest_neighbor}.rs
└── mapping.rs                    # former connection.rs

syntaqs-export/src/
├── matrix.rs
├── matrix/{args,metadata,export}.rs
├── vector.rs
└── vector/{metadata,export}.rs

syntaqs-lie/src/
├── family.rs
├── family/{projection,validation,transform}.rs
├── numerical.rs
└── numerical/{options,tolerance,error,kernels}.rs
    └── kernels/tests.rs
```

The tree lists affected paths, not every existing module. `ExpressionError` and `LinearCombination` remain in their existing owners for this phase: the exact public API is deferred, and `LinearCombination`, `Term`, and `IntoTerms` remain in `expression/parts/linear_combination.rs`. The existing public reachability of expression parts must not be narrowed. `LieError` stays outside `numerical`; `NumericalError` moves under it. In export, `Export`, `Exported`, `ExportError`, and backend implementations are not part of this layout change. The agreed `models` names and previous Neel/error changes remain intact.

## Execution stages

- [x] Agree on semantic boundaries and candidate file layout; assess the existing modules and their responsibilities.
- [x] **Baseline and compatibility map.** Checked the clean worktree, source and destination paths, path-sensitive references, and scoped public reachability. Captured the [compatibility baseline](semantic-baseline.md), Cargo targets, and default, all-features and no-default-features test listings. Recheck the worktree and move destinations immediately before implementation.
- [x] **Core algebra and expression.** Renamed `ops`/`expr` to `algebra`/`expression`, moved `Dimension` and `DimensionOverflow` definitions to `space`, and updated internal references and relevant documentation. Temporary aliases preserve `ops`, `expr`, `ops::Dimension`, and `error::DimensionOverflow`; existing root exports remain. An integration test verifies old/new type identity and trait bounds. The 75 Cargo targets and all 448 baseline all-features tests remain; one compatibility test was added. Workspace all-features tests, all-target no-default-features check, Clippy, warning-denied rustdoc, formatting, and core tests on Rust 1.85 passed. Rustdoc now locates the dimension definitions in `space`.
- [x] **Geometry.** Moved `Boundary` to `geometry/boundary.rs`, `SiteGroup` and `SiteGroupError` to `relation/group.rs`, and renamed `connection.rs` to `mapping.rs`. The three files retain their contents, their enclosing modules remain private, and crate-root public exports remain available. All 14 geometry unit, integration and documentation test entries match the baseline; Cargo's 75 targets are unchanged. Geometry and example tests, dependent physical-model tests, the workspace all-target no-default-features check, targeted Clippy and warning-denied rustdoc passed; geometry tests also passed on Rust 1.85.
- [x] **Export.** Split matrix arguments, metadata and conversion implementations under `matrix/`, and vector metadata and conversion under `vector/`, with thin module entries. `Ambient` is restricted to matrix; existing root exports and feature-gated backend re-exports remain. All 16 export test/doctest entries and 75 Cargo targets match the baseline. Export tests passed with no default features, `ndarray`, `sprs`, and both; shared integration tests for each backend, the all-features chain example test, targeted Clippy and warning-denied rustdoc also passed. Export all-features tests passed on Rust 1.85; workspace all-target no-default-features checks passed.
- [x] **Lie family and numerical.** Moved family transformations to `family/transform.rs`; split numerical errors, options, tolerances and kernels under `numerical/`, including moving kernel tests to `numerical/kernels/tests.rs`. `LieError` remains outside numerical, kernel visibility remains restricted, and existing crate-root exports remain. An integration test confirms numerical-error conversion, source and message. All 104 baseline Lie test entries and all 75 Cargo targets remain; one error-source test was added. Lie tests and doctests, the Lie-only shared workflow and example, downstream BLAS fixture, targeted Clippy, warning-denied rustdoc and workspace no-default-features all-target checks passed. Lie tests also passed on Rust 1.85.
- [x] **Cross-workspace verification.** Reviewed changes from `7c3c003` through `27bb85b`; no unintended public capability, visibility or behavior change was identified. An external consumer compiled using old and new core paths, type-identity and trait-bound checks, geometry and Lie root exports, and both feature-gated export backends. All 75 Cargo targets and 448 baseline all-features test entries remain, with two compatibility tests added. The old and new `DimensionOverflow` paths are the same type; the observed `type_name` is `syntaqs_core::space::DimensionOverflow`, reflecting its new definition location.
  - Workspace default, all-features and no-default-features tests, all-features builds, and no-default-features all-target checks passed on Rust 1.85.0 and 1.99.1. Formatting, Clippy, and warning-denied rustdoc passed on 1.99.1.
  - Independent export and shared-test backend configurations, the Lie-only shared test and example, all-features Lie example, and downstream BLAS fixture passed. Existing code and documentation references to moved physical paths were checked; `git diff --check` passed.
- [ ] **Handoff.** Update this plan with actual moves, temporary compatibility aliases, any unavoidable breakage and outstanding API decisions. Leave the full public API review, removal of temporary aliases and prelude selection for a later separately requested task.

Each stage should leave the workspace coherent and receive its relevant checks before proceeding. If a path cannot be preserved without broadening exposure or breaking an invariant, record the specific trade-off and resolve it in context rather than silently changing the contract.

## Completion criteria

The agreed files and responsibility boundaries are in place, existing public items remain usable through their prior paths wherever feasible, no duplicate type definitions or unintended visibility expansion are introduced, and affected test and documentation behavior remains correct. Required checks have passed or their limitations are recorded. The eventual canonical API and preludes remain explicitly unfinished.
