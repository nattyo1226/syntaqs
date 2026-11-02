# v3 release preparation: plans and progress

Last updated: 2026-11-02

## 1. Current work and remaining items

This document records plans and progress for ongoing v3 release preparation. Implemented specifications belong in `design/`; additional features without a scheduled start belong in [future work](../../future-work.md).

| Item | Status and next steps |
| --- | --- |
| Document responsibilities | Reorganized requirements, overview, feature specifications, future features, and ongoing plans. |
| English documentation in `docs/` | Translated all documents, including `plans/`, and updated links to translated headings. |
| Crate order in README and Cargo.toml | Completed responsibility-based ordering, including external dependencies, each package's dependency lists, and workspace members. |

[Documentation final adjustments](documentation.md) records agreed policies and completion of all three documentation adjustments above.

## 2. Release preparation progress

### 2.1. Primitive naming and normalization

Implementation and verification completed on 2026-11-01. Current primitives and normalization are specified in [local algebras](../../design/local-structure.md), dimension-normalized HS inner products in [local expressions](../../design/primitive-expression.md), and physical coupling corrections in [model construction](../../design/models.md). Executed checks are recorded in the [verification history](verification.md).

### 2.2. Expression-tree formatting

Implementation and verification completed on 2026-11-01. Features include ordinary and multiline Display, Debug, the eight-character Index limit, and Tag and builtin primitive labels. See [expression formatting](../../design/expression-formatting.md) for current contracts and [expression_formatting](../../../examples/src/bin/expression_formatting.rs) for usage.

### 2.3. README and license

On 2026-11-02, installation instructions, crate READMEs, and the MIT license were prepared. Checks covered local links, examples, and consistency between license and Cargo metadata. Current instructions are in the [root README](../../../README.md) and license text in [LICENSE](../../../LICENSE). Copyright is `Copyright (c) 2026 Haruki SAITO`, covering the entire repository.

Example Git dependencies were verified by mapping them to the local repository. Installation after the renamed code reaches GitHub has not been checked. README crate ordering was subsequently completed as recorded in section 2.7.

### 2.4. GitHub Actions

On 2026-11-02, workflows were prepared. The user temporarily added `feat/workflow` to push triggers and confirmed successful execution on GitHub Actions. Push triggers were then restricted to `main`.

[Verification](../../design/verification.md) defines current workflow structure, commands, and verification limits. Dedicated no_std targets are not included in workflows.

### 2.5. Many-body relation enumeration

Implementation and acceptance checks completed on 2026-10-31. See [geometry](../../design/geometry.md) for current Relation, Group, SiteMapper, and many-body leaf selection; [examples](../../../examples/README.md) for usage; and [verification history](verification.md) for checks performed at the time.

### 2.6. Document responsibilities

Implementation details and migration history were separated from requirements; architecture became an overall structural overview. Design documents now describe current features, with separate construction, normalization, numeric-convention, model-construction, and verification documents. Unimplemented feature details and API candidates moved to `future-work/`; earlier verification records moved into this plan's history.

Repository Markdown links were updated for moved documents and headings. During reorganization on 2026-11-02, 38 files, 340 local links, and code fences were checked, with no broken links or unclosed fences. `git diff --check` and `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` passed. Implementation code and dependency settings were unchanged; workspace tests were not rerun for that reorganization.

### 2.7. Presentation order

On 2026-11-02, the root README crate list, workspace members, and dependency sections were aligned with the agreed core-to-concrete-algebra order. External dependencies use the role order defined in the [documentation adjustments](documentation.md#application-to-cargotoml). All 11 manifests and 19 dependency sections were checked for ordering and preserved settings. Locked offline Cargo metadata and `git diff --check` passed. Source code, dependency settings, and Cargo.lock were unchanged; builds and tests were not rerun for this ordering-only change.
