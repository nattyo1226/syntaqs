# Writing revision results

All 46 pre-existing Markdown documents were reviewed. The 42 project documents
listed below were revised, the active documentation plan was updated, and the
three historical audit/phase records retain their dated evidence with current
follow-up links. This report records the writing phase and links to the
permanent specifications.

## Main changes

- Workspace and crate entry points now lead from purpose to usage, prerequisites,
  and detailed contracts. Eight complete README examples explain observable results.
- Release installation recommends one version tag across all syntaqs dependencies.
  The example tag is `v0.4.0`, resolving locally to commit
  `8b18d8b65a8baffbf15d8cea674893bc7fe1e864`. Tagged documentation is distinguished
  from current-checkout APIs. Cargo syntax was checked against the
  [official Git dependency reference](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#choice-of-commit).
- E04 is resolved by placing composite public APIs, full capability requirements,
  and action scope before internal representations and algorithms. E06 is resolved
  by replacing migration-relative wording in current usage/contracts. E07 is
  resolved by consolidating basis-result and identity explanations.
- Definitions, conditions, and examples remain close to the claims they qualify.
  Diagrams, tables, and procedures are used where they clarify actual relationships.

## Document coverage

| Document | Revision |
| --- | --- |
| [README.md](../../../README.md) | Tag-based installation, release-matched documentation, explained Quick start results, and task-oriented navigation. |
| [crates/syntaqs-core/README.md](../../../crates/syntaqs-core/README.md) | Construction workflow before the API inventory, with clear extension boundaries and a linked complete example. |
| [crates/syntaqs-basic/README.md](../../../crates/syntaqs-basic/README.md) | UUID/name choice table and a complete example showing identity-preserving copies. |
| [crates/syntaqs-geometry/README.md](../../../crates/syntaqs-geometry/README.md) | Geometry/relation/mapping roles followed by a runnable edge-enumeration example. |
| [crates/syntaqs-normalize/README.md](../../../crates/syntaqs-normalize/README.md) | Runnable normalization example, operation/import table, and precise zero/error behavior. |
| [crates/syntaqs-export/README.md](../../../crates/syntaqs-export/README.md) | Runnable vector export, container selection, metadata purpose, and feature choices. |
| [crates/syntaqs-lie/README.md](../../../crates/syntaqs-lie/README.md) | Ordered public workflow and application provider setup, with release-installation navigation. |
| [crates/syntaqs-spin-half/README.md](../../../crates/syntaqs-spin-half/README.md) | Local action example before primitive/model details, with physical normalization explained. |
| [crates/syntaqs-spin-one/README.md](../../../crates/syntaqs-spin-one/README.md) | Concrete Zero-state/raising example and a clear distinction between physical and normalized spin. |
| [crates/syntaqs-fermion/README.md](../../../crates/syntaqs-fermion/README.md) | Local creation example before graded many-mode and Hubbard conventions. |
| [examples/README.md](../../../examples/README.md) | Task/binary and feature tables, prerequisites before Lie commands, and current usage instead of migration notes. |
| [tests/README.md](../../../tests/README.md) | Commands grouped by workspace/backend/Lie intent; fixture ownership and native prerequisites clarified. |
| [support/blas-provider/README.md](../../../support/blas-provider/README.md) | Direct explanation of internal linkage, requirements, and the downstream application boundary. |
| [docs/README.md](../../README.md) | Reader routes through requirements/architecture/design, shorter policy paragraphs, and writing principles. |
| [docs/requirements.md](../../requirements.md) | Accepted scope separated from proposals; ordered end-to-end acceptance steps and explicit conditions. |
| [docs/architecture.md](../../architecture.md) | Computation-flow diagram, progressive explanation, and module responsibility table. |
| [docs/design/local-structure.md](../../design/local-structure.md) | Primitive/coefficient definitions before use, direct normalization language, and a fermion capability table. |
| [docs/design/numerics.md](../../design/numerics.md) | Exact-zero versus tolerance policy first; complex-division algorithm expressed as conditional steps. |
| [docs/design/index.md](../../design/index.md) | Identity separated from site/basis indices; redundant registry explanations consolidated. |
| [docs/design/hilbert-space.md](../../design/hilbert-space.md) | Space responsibility before private structure; dimension and type correspondence split into coherent paragraphs. |
| [docs/design/expression-construction.md](../../design/expression-construction.md) | Construction purpose first; fallible input precedence as a procedure and inference conditions nearby. |
| [docs/design/primitive-expression.md](../../design/primitive-expression.md) | Types and local/composite boundary defined directly, with links to construction and numeric owners. |
| [docs/design/composite-expression.md](../../design/composite-expression.md) | Public APIs, full capabilities, and Act scope before laws and internal representations/algorithms; anchors updated. |
| [docs/design/normalization.md](../../design/normalization.md) | Normalization and normalized expectation distinguished first; numeric bounds, evaluation, and errors separated. |
| [docs/design/basis.md](../../design/basis.md) | Result-choice table and one detailed expansion/metadata explanation, retaining enumeration and error contracts. |
| [docs/design/expression-formatting.md](../../design/expression-formatting.md) | Display/Debug purpose before bounds; multiline alignment rules split by topic with examples retained. |
| [docs/design/geometry.md](../../design/geometry.md) | Geometry-to-relation-to-data flow and explicit context for the schematic code example. |
| [docs/design/models.md](../../design/models.md) | Model construction described through shared expressions, with direct public construction wording. |
| [docs/design/export.md](../../design/export.md) | Interpretation and action-matrix distinction first; ownership paragraphs and present-tense feature/size contracts. |
| [docs/design/lie.md](../../design/lie.md) | Candidate/validation distinction first and concise progression into projection and retained results. |
| [docs/design/verification.md](../../design/verification.md) | Reader task and coverage meaning first; toolchain/check table replaces mixed command-selection prose. |
| [docs/decisions/lie-analysis.md](../../decisions/lie-analysis.md) | Accepted decision summarized before context, while preserving alternatives and source provenance. |
| [docs/decisions/core-evaluation-performance.md](../../decisions/core-evaluation-performance.md) | Decision and trade-offs summarized first; historical measurements, tables, methods, and revisions retained. |
| [docs/future-work.md](../../future-work.md) | Proposal status and navigation first; local-system extensions grouped by purpose. |
| [docs/future-work/api-usability.md](../../future-work/api-usability.md) | User problems before candidates; current onboarding examples distinguished from remaining guidance. |
| [docs/future-work/evaluation-improvements.md](../../future-work/evaluation-improvements.md) | Investigation goals first and term normalization distinguished from physical state normalization. |
| [docs/future-work/external-connections.md](../../future-work/external-connections.md) | One continuous short explanation of responsibility, invariants, and differentiable parameter retention. |
| [docs/future-work/identity-and-storage.md](../../future-work/identity-and-storage.md) | Restoration versus independent copying explained before their detailed identity rules. |
| [docs/future-work/partial-trace.md](../../future-work/partial-trace.md) | Proposed operation/result first; ambient resolution and subsystem validation separated. |
| [docs/future-work/representation-matrix.md](../../future-work/representation-matrix.md) | Coefficient reuse as motivation, with scalar field, coefficient matrices, transpose, and adjoint distinguished. |
| [docs/future-work/state-and-operator.md](../../future-work/state-and-operator.md) | Review objective and current objects before questions; undecided outcomes remain explicit. |
| [docs/plans/v5/README.md](../v5/README.md) | Implemented facts, remaining integration stages, and breaking changes expressed directly. |

## Working records

| Record | Treatment |
| --- | --- |
| [Task README](README.md) | Updated scope, stages, completion criteria, and links for this writing phase. |
| [Baseline audit](audit.md) | Preserved original findings and line references; updated moved targets and linked the writing outcome. |
| [Baseline coverage](coverage.md) | Preserved the original 41-document inventory; linked the full current revision record here. |
| [Conventions results](conventions-results.md) | Preserved executed checks and resolutions; linked completion of the later writing work. |

## Verification

- Compiled and ran the eight executable README examples against current local
  crates: root Quick start, basic, geometry, normalize, export, spin-half, spin-one,
  and fermion. Their assertions passed. The snippets were formatted with rustfmt.
- Archived the local `v0.4.0` tag and ran the exact root Quick start against its
  core/spin-half crates. Both current and tagged runs printed `1: 1+0i` and passed
  the expectation assertion. Builds used temporary standalone packages and local
  path dependencies, not a GitHub fetch.
- Preserved all 45 existing mathematical fenced blocks across design, proposal,
  and decision documents. Existing Rust blocks in those documents and all three
  performance-measurement tables were compared with the starting tree and retained.
- Local links, heading anchors, permanent-to-plan boundaries, and code fences are
  checked by [writing-checks.json](writing-checks.json). Reproduce with:

  ```sh
  python3 docs/plans/documentation-guidelines/check_docs.py
  ```

- Reviewed the final diff and ran whitespace/scope checks. Production Rust code,
  manifests, lockfile, and the earlier audit/check snapshots remain unchanged.

The builds used Rust/Cargo 1.99.1. This writing phase did not run the full
workspace suite, native Lie calculations, dedicated no_std targets, or an
additional build on the minimum Rust version. Rustdoc comments were outside the
systematic Markdown review. External-link reachability was not checked; Cargo
dependency syntax was verified against the primary reference above. The local
Markdown checker approximates GitHub heading slugs rather than rendering them.

The retained v5 plan still tracks integration/version work.

## Root README follow-up

The root README now contains only the introduction, Quick start, crate table,
documentation links, and license, following the subsequent request for a minimal
entry point. The Quick start recommends matching version tags and demonstrates
one two-spin expectation calculation.

Detailed installation guidance moved to [installation](../../installation.md).
Updated all 12 incoming installation links and added the guide to the
documentation index. Development commands remain in their existing
[verification document](../../design/verification.md).

The revised example ran successfully against both current local crates and the
local `v0.4.0` archive, printing `Expectation: -1` in each case. These checks used
temporary standalone packages with local path dependencies, not a remote Git
installation. The example prints the real component to avoid differences in
complex signed-zero formatting between the two versions.

The latest local link, heading, boundary, and fence results are recorded in
[root-readme-checks.json](root-readme-checks.json). Reproduce them with:

```sh
python3 docs/plans/documentation-guidelines/check_docs.py --include docs/installation.md --write root-readme-checks.json
```

Whitespace checks passed. This follow-up did not change production code or
dependencies, or rerun the full workspace suite.
