# Documentation guide

Updated: 2026-11-02

## 1. Document responsibilities

| Document | Contents |
| --- | --- |
| [requirements](requirements.md) | What the project builds: its purpose, scope, and functional requirements. |
| [architecture](architecture.md) | An overview of what it provides: the overall structure and allocation of responsibilities. |
| [future-work](future-work.md) | Features to implement and candidates not yet adopted. Details belong in `future-work/`. |
| `design/` | The API, mathematical contracts, constraints, and implementation of each feature currently provided. |
| `plans/` | Plans and progress for work currently being implemented. |

Keep detailed current specifications in the corresponding design documents and link to them from requirements and architecture. Separate proposals for unimplemented features from current specifications, and create a plan in `plans/` when implementation starts. Completed steps within an active plan remain part of its progress record. Once the work is complete and merged, move any lasting specifications and conventions into the appropriate permanent documents, update incoming references, and remove the completed plan. Create `plans/` as needed; it need not exist when no plans are active.

## 2. Feature details

| Feature | Document |
| --- | --- |
| Local algebras, primitives, and grading capabilities | [local-structure](design/local-structure.md) |
| Coefficient types, numeric capabilities, zero, and approximate comparison | [numerics](design/numerics.md) |
| Factor identity and standard indices | [index](design/index.md) |
| Spaces, factor order, grouping, and compatibility | [hilbert-space](design/hilbert-space.md) |
| Type selection, expression construction, and fallible collection | [expression-construction](design/expression-construction.md) |
| Local expressions and evaluation | [primitive-expression](design/primitive-expression.md) |
| Composite expressions, placement, binding, and ordinary/graded evaluation | [composite-expression](design/composite-expression.md) |
| Norms, normalization, and normalized expectations | [normalization](design/normalization.md) |
| Basis expansion, coordinates, and enumeration | [basis](design/basis.md) |
| Expression formatting | [expression-formatting](design/expression-formatting.md) |
| Geometry, relations, and site connections | [geometry](design/geometry.md) |
| Spin and Hubbard model construction | [models](design/models.md) |
| Export to numerical containers | [export](design/export.md) |
| Verification structure and commands | [verification](design/verification.md) |

## 3. Language and maintenance

Write all documents under `docs/` in English, including requirements, architecture, future work, designs, and temporary plans. This applies to headings, tables, and explanatory comments in code examples as well as prose. Keep mathematical notation and API names consistent with the implementation.

When moving documents or changing headings, update incoming repository links, including heading anchors. Check local links and code fences, and keep instructions and examples consistent with the current public APIs. Put current verification commands and coverage in [verification](design/verification.md); dated execution logs and implementation progress belong only in active plans.

## 4. Presentation order

Present project crates from shared foundations to concrete local algebras, following the [architecture crate table](architecture.md#2-crates-and-responsibilities): `syntaqs-core`, `syntaqs-basic`, `syntaqs-geometry`, `syntaqs-normalize`, `syntaqs-export`, `syntaqs-spin-half`, `syntaqs-spin-one`, and `syntaqs-fermion`.

Use this relative order in the root README crate list and dependency examples, workspace members, and every Cargo.toml dependency section. Workspace members list `examples` and `tests` after the library crates. The dependency convention applies to `[workspace.dependencies]`, `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`, and target-specific dependency sections in all packages, including examples and tests.

Within each dependency section, list project crates first, then external dependencies by role:

| Role | Current dependency |
| --- | --- |
| Identifiers | `uuid` |
| Error handling | `thiserror` |
| Numeric traits | `num-traits` |
| Complex coefficients | `num-complex` |
| Dense numerical containers | `ndarray` |
| Sparse numerical containers | `sprs` |

Place additional dependencies according to their responsibilities rather than alphabetically. Separate responsibility groups with blank lines and label the external groups in the workspace dependency table. Ordering changes preserve dependency sections, versions, features, optional flags, and paths.
