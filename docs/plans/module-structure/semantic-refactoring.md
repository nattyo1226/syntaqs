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
- [ ] **Core algebra and expression.** Rename `ops.rs` to `algebra.rs`, and `expr.rs` plus `expr/` to `expression.rs` plus `expression/`. Move `Dimension` and `DimensionOverflow` definitions to `space`; leave algebraic and grading contracts together in `algebra`. Update internal imports, tests, examples, doc links and doctests. Preserve the existing root exports and old public `ops`/`expr` module paths through temporary aliases, including the existing `ops::Dimension` and `error::DimensionOverflow` paths. Check that aliasing preserves type identity and covers all formerly reachable descendants; record the new and old paths. Run core tests and doctests, affected consumer checks, and core no-default-features checks.
- [ ] **Geometry.** Move `Boundary` into `geometry/boundary.rs`, `SiteGroup` and `SiteGroupError` into `relation/group.rs`, and rename private `connection.rs` to `mapping.rs`. Update declarations and imports while keeping the existing crate-root public exports reachable. Test geometry, mapping and dependent examples; verify path and privacy changes do not accidentally broaden public modules.
- [ ] **Export.** Move the existing matrix and vector conversion implementations to their respective `export.rs` children, split `MatrixArgs` and the two metadata types into the agreed files, and leave thin module entries. Place `Ambient` under matrix with the minimum visibility required by conversion code. Keep existing crate-root exports and feature-gated external crate re-exports; do not expose the implementation modules merely to ease imports. Test export without default features, with `ndarray` and `sprs` separately, and with both; test affected shared integration and example consumers.
- [ ] **Lie family and numerical.** Move `transform.rs` to `family/transform.rs`, preserving root exports. Split existing `tolerance.rs` into `numerical/tolerance.rs` and `numerical/options.rs`; move `NumericalError` out of `error.rs` to `numerical/error.rs`; move the current numerical implementation and its tests to `numerical/kernels.rs` and `numerical/kernels/tests.rs`. Retain `LieError` outside numerical. Use restricted visibility for kernels and shared validation helpers, preserve existing public root exports and error source/conversion behavior, and update internal references. Run Lie unit, integration and documentation tests, the independent Lie example and shared workflow, and the downstream BLAS fixture.
- [ ] **Cross-workspace verification.** Review the entire diff, old and new paths, unintended changes in visibility or public capabilities, feature gates and test ownership. Compile targeted external-consumer checks for old paths, feature-dependent exports and type identity; verify existing tests and doctests remain discovered. Update affected current documentation and examples to accurately describe the intermediate structure without presenting deferred API decisions as implemented. Run the repository-required build, tests, formatting, Clippy and rustdoc checks, including feature combinations and supported Rust versions specified in [verification](../../design/verification.md). Record command results and limitations.
- [ ] **Handoff.** Update this plan with actual moves, temporary compatibility aliases, any unavoidable breakage and outstanding API decisions. Leave the full public API review, removal of temporary aliases and prelude selection for a later separately requested task.

Each stage should leave the workspace coherent and receive its relevant checks before proceeding. If a path cannot be preserved without broadening exposure or breaking an invariant, record the specific trade-off and resolve it in context rather than silently changing the contract.

## Completion criteria

The agreed files and responsibility boundaries are in place, existing public items remain usable through their prior paths wherever feasible, no duplicate type definitions or unintended visibility expansion are introduced, and affected test and documentation behavior remains correct. Required checks have passed or their limitations are recorded. The eventual canonical API and preludes remain explicitly unfinished.
