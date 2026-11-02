# Documentation guide

Updated: 2026-11-02

## 1. Document responsibilities

| Document | Contents |
| --- | --- |
| [requirements](requirements.md) | What the project builds: its purpose, scope, and functional requirements. |
| [architecture](architecture.md) | An overview of what it provides: the overall structure and allocation of responsibilities. |
| [future-work](future-work.md) | Features to implement and candidates not yet adopted. Details belong in `future-work/`. |
| `design/` | The API, mathematical contracts, constraints, and implementation of each feature currently provided. |
| [plans/v3](plans/v3/README.md) | Plans and progress for the ongoing v3 release preparation. |

Keep detailed current specifications in the corresponding design documents and link to them from requirements and architecture. Separate proposals for unimplemented features from current specifications, and create a plan in `plans/` when implementation starts. Completed steps within an active plan remain part of its progress record; the resulting specifications belong in `design/`.

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
