# Documentation adjustments before the v3 release

Created: 2026-11-02
Last updated: 2026-11-02

This temporary document records requested adjustments and agreed policies before the v3 release. Document responsibilities, English migration, and crate ordering in README and Cargo.toml have been completed.

## 1. Document responsibilities

Responsibilities across `docs/` were insufficiently separated. Each document's scope must be clear, with content placed according to that responsibility.

### Agreed responsibilities

| Document | Responsibility |
| --- | --- |
| `requirements.md` | Describe what the project is building: purpose, responsibility boundaries, functional requirements, and related requirements. |
| `architecture.md` | Describe broadly what the project provides, including its overall structure. |
| `future-work.md` | Describe features to implement in the future. |
| `design/xxx` | Describe in detail what each individual feature provides. |
| `plans/xxx` | Record plans and progress for work currently being implemented. |

Existing documents follow this division. `architecture.md` and `design/` provide an overview and details of current offerings. Future features and ongoing plans and progress belong in `future-work.md` and `plans/`, respectively.

### Completed reorganization

A documentation guide was added at [docs/README](../../README.md). Requirements now cover purpose, scope, functional and nonfunctional requirements, and acceptance criteria; architecture covers the overall structure; design covers current features. Unimplemented features and API candidates without active implementation belong in `future-work/`. Verification history relevant to release preparation belongs in the [v3 history](verification.md).

English migration and README / Cargo.toml ordering are tracked separately below.

## 2. English documentation

Documents under `docs/`, previously written in Japanese, have been migrated to English.

### Agreed migration policy

Use English throughout `docs/`, including `requirements.md`, `architecture.md`, `future-work.md`, `design/`, and `plans/`. Organize and translate documents according to the responsibilities in item 1.

All 27 documents have been translated, including headings, tables, code-example comments, and plan history. Mathematical conventions and API names are preserved. Repository links to translated headings and the root README's language description have been updated.

Verification on 2026-11-02 covered 38 Markdown files and 340 local links, including heading anchors, with no broken links or unclosed fences. All 45 math blocks were unchanged; Rust code in seven fenced examples was unchanged except for comments. External reference URLs were preserved. `git diff --check` and `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --quiet` passed. Implementation code and dependencies were unchanged; workspace tests were not rerun for this translation.

## 3. Crate ordering in the root README and Cargo.toml dependency lists

The crate list in the [root README](../../../README.md#crates) previously did not follow a semantic order based on responsibilities. It now follows each crate's role and relationships.

### Presentation policy

Start with the shared foundation, `syntaqs-core`, followed by identifiers, geometry, and shared capabilities. Group concrete local-algebra crates such as `syntaqs-spin-half` later.

The agreed order is:

| Order | Crate | Responsibility |
| --- | --- | --- |
| 1 | `syntaqs-core` | Shared algebraic contracts, expressions, spaces, evaluation, and bases. |
| 2 | `syntaqs-basic` | UUID and named indices identifying factors. |
| 3 | `syntaqs-geometry` | Geometry, relations between Sites, and external-data integration. |
| 4 | `syntaqs-normalize` | Shared norms, normalization, and normalized expectations. |
| 5 | `syntaqs-export` | Shared export to numerical containers. |
| 6 | `syntaqs-spin-half` | Spin-1/2 local algebra, bases, and models. |
| 7 | `syntaqs-spin-one` | Spin-1 local algebra, bases, and models. |
| 8 | `syntaqs-fermion` | Fermion local algebra, bases, and models. |

### Application to Cargo.toml

**Scope and policy are agreed.** Order Cargo.toml dependency lists semantically by each dependency's responsibility and role.

- Include external dependencies as well as `syntaqs-*`. Keep project crates in the relative order above; order external dependencies according to their responsibilities and roles.
- Apply the same policy to the root `[workspace.dependencies]` and each crate's Cargo.toml, including `examples` and `tests`.
- Follow this policy within each dependency section, including `[dependencies]`, `[dev-dependencies]`, and `[build-dependencies]`.

External dependencies follow this order: identifiers (`uuid`), error handling (`thiserror`), numeric traits (`num-traits`), complex coefficients (`num-complex`), dense containers (`ndarray`), and sparse containers (`sprs`). Each dependency section lists project crates first in the agreed relative order, followed by external dependencies in this role order. Blank lines separate responsibility groups; the workspace dependency table also labels the external groups.

On 2026-11-02, the root README crate table and installation snippet, workspace members and dependencies, and package dependency lists were aligned with this policy. All 11 manifests and 19 dependency sections were checked, including those already in the agreed order. Settings within each original table were preserved, including versions, features, optional flags, and paths. `cargo metadata --locked --offline --no-deps --format-version 1` and `git diff --check` passed. Source code and Cargo.lock were unchanged; builds and tests were not rerun for this ordering-only change.
