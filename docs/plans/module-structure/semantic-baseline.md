# Semantic refactoring baseline

## Scope and captured state

This is a scoped compatibility map for the planned moves, not a complete public API inventory. It records the repository at commit `9171346` before semantic refactoring. The worktree was clean at the start of this stage. Existing root and nested paths below are current paths to preserve where feasible, not decisions about the eventual canonical API.

[Cargo target baseline](baseline-targets.tsv) records all 75 workspace targets from `cargo metadata --locked --no-deps`; source paths are workspace-relative. The excluded `tests/fixtures/lie-blas` package is not a workspace target and must be checked separately. The [all-features test listing](baseline-tests.txt) contains 448 listed test entries, including doctests, from `cargo test --locked --workspace --all-features -- --list`. The equivalent [default-feature listing](baseline-tests-default.txt) contains 431 entries and the [no-default-features listing](baseline-tests-no-default.txt) contains 430. Target hash suffixes are normalized. This is a discovery baseline, not a test execution result. Compare target names and kinds and each target's test names after a batch; test module names and doctest file paths/line numbers can legitimately move, so map those changes explicitly rather than dropping entries. Cargo workspace feature unification can affect which tests are listed; independent backend checks remain necessary.

## Existing public paths affected

### Core

- Root exports include `Dimension`, `DimensionOverflow`, `Act`, `Adjoint`, `Product`, `Identity`, `Trace`, `InnerProduct`, `Parity`, `HomogeneousParity`, `ParityOperator`, `LinearCombination`, `Term`, `IntoTerms`, `ExpressionError`, `State`, `Operator`, `LocalState`, and `LocalOperator`.
- `syntaqs_core::ops` is public and defines the listed operation and grading traits plus `Dimension`; `syntaqs_core::error::DimensionOverflow` is public. Both old module paths need to remain usable alongside their future owners. `Dimension::dim`, `HilbertSpace::{dim,dim_with}`, basis dimension queries and coordinate dimensions expose or use these contracts and error types.
- `syntaqs_core::expr` is public: `expr::{State,Operator,LocalState,LocalOperator,ExpressionError}` and `expr::composite::{State,Operator,ExpressionError}` are reachable. `expr::parts` exposes `LinearCombination`, `Term`, `IntoTerms`, `SumExpr`, `IntoOperands`, `ScaleExpr`, `ProductExpr`, `ActExpr`, and `AdjointExpr`. `expr::parts::linear_combination` is also public and exposes `LinearCombination`, `Term`, and `IntoTerms`. Preserve these nested paths and type identities through the module rename; `expr::local`, `traversal`, `format`, and `group` are not public modules.
- `syntaqs_core::space::{HilbertSpace,SpaceError,TensorKind,Indices,Leaves}` remains public. The crate is `no_std` outside tests; do not introduce a standard-library requirement in moved code.

### Geometry

`boundary`, `connection`, `geometry`, `group`, and `relation` are private modules in the current crate root. The public root exports are `Boundary`, `Geometry`, `Chain`, `SiteGroup`, `SiteGroupError`, `SiteMapper`, `MappedGroup`, `MappedGroupResult`, `MappingError`, `Relation`, `Consecutive`, `ConsecutiveError`, and `NearestNeighbors`. In particular, `Relation::Group` and mapping results carry group-related types; moving definitions must preserve their reachable types and signatures without making private module paths public by accident.

### Export

`args`, `metadata`, `matrix`, and `vector` are private modules. Root exports include `Export`, `ExportError`, `Exported`, `MatrixArgs`, `MatrixMetadata`, `VectorMetadata`, and `Layout`; `Export::Error`, output metadata methods, and matrix arguments expose these contracts. Keep their root imports and type identity when moving definitions. `syntaqs_export::ndarray` is exported only with the `ndarray` feature and `syntaqs_export::sprs` only with `sprs`; default features are empty. Their re-exports and feature gates are part of this intermediate compatibility surface.

### Lie

`family`, `transform`, `numerical`, `tolerance`, and `error` are private at the crate root. Preserve root exports of `FamilyTransform`, `OrthonormalityReport`, `NumericalOptions`, `Tolerance`, `ToleranceCheck`, `NumericalError`, and `LieError`. Family, representation, invariant and decomposition results and options are also root exports; their methods and diagnostic fields use the affected types. In particular preserve `LieError::Numerical` conversion/source behavior, tolerance validation, type identities and feature-independent access. Family projection and validation children already exist and are private implementation boundaries.

## File moves and references

All 12 direct source-to-destination moves and five new split-file destinations in the [plan](semantic-refactoring.md#target-implementation-layout) were checked: sources exist and destination files/directories do not currently exist. Existing parent entry files such as `matrix.rs`, `vector.rs`, and `numerical.rs` are intentionally retained or recreated; this is not a collision. Recheck immediately before each batch, as the worktree can change.

Workspace Rust source contains one explicit `#[path]`: core's test-only `../tests/common/mod.rs`. No `include!`, `include_str!`, or `include_bytes!` occurrence was found in the searched `crates`, `examples`, `tests`, and `support` trees. Relevant references include core implementation and unit tests using `crate::expr`, integration tests using `syntaqs_core::ops` and `expr::parts`, rustdoc links, and `docs/design/expression-formatting.md` linking directly to `src/expr/format.rs`. Geometry/Lie internal imports and export backend references occur throughout the relevant crates. `docs/design/local-structure.md` explicitly documents the old `ops` path; `docs/design/export.md`, examples and integration tests use export's feature-gated dependency re-exports. Repository searches also covered markdown, Cargo manifests and CI YAML; repeat searches during each batch because textual search does not prove all implicit Rust references or future changes have been found.

Before changing a module, compile representative external-consumer checks for the paths above and compare them after the move. Check alias type identity and method signatures; keep type-name/rustdoc text changes separate from actual path reachability. Run the feature-specific tests and documentation checks from the implementation plan. The full API inventory and prelude selection remain deferred.
