# Audit coverage

All 41 tracked Markdown documents were assessed against both documentation
conventions and technical writing guidelines at baseline revision
`3856a311230bc9dd29952c20eaa82d1ac91235c2`. This inventory records
document responsibility, current/proposed/historical status, relevant ownership
and writing concerns, and applicable findings from [the audit](audit.md).

“No specific issue” means no actionable conventions/writing discrepancy was
identified in that reading. It does not certify every API example, mathematical
claim, or linked external source. Line counts are recorded in [checks.json](checks.json).

## Entry points, requirements, and architecture

| Document | Assessment |
| --- | --- |
| [Root README](../../../README.md) | Useful user-facing purpose/install/usage entry point. Correct unavailable sample imports (C01) and verification ownership pointer (C05). Brief crate/convention summaries are appropriate here. |
| [Documentation guide](../../README.md) | Clear responsibility map, design navigation, language, and maintenance policy. Clarify whether every implementation requires a plan or only substantial tasks (E02). |
| [Requirements](../../requirements.md) | No specific issue. Goals, functional guarantees, exclusions, and acceptance scenarios have design links; mathematical conventions affecting physical results are legitimate requirements. |
| [Architecture](../../architecture.md) | No specific issue. Components, dependencies, data flow, and compact module responsibilities describe the system. Detailed contracts remain delegated to design. |

## Current feature designs

| Document | Assessment |
| --- | --- |
| [Local algebras](../../design/local-structure.md) | Appropriate mathematical owner for primitive meaning, normalization, grading, and local rules. Remove the undefined historical normalization reference (E06). Product/action tables are meaningful contracts. |
| [Numerics](../../design/numerics.md) | No specific issue. Defines coefficients, operation-dependent capabilities, division, exact zero, and test tolerances with relevant conditions and implementation references. |
| [Index](../../design/index.md) | Correct separation of identity, issuance, algebra, and display. Consolidate repeated registry explanations (E07). UUID uniqueness is explicitly practical rather than guaranteed. |
| [Hilbert space](../../design/hilbert-space.md) | No specific issue. Defines factors/grouping before compatibility/subsystems, illustrates unit and mixed-kind cases, and keeps basis/algebra separate. |
| [Expression construction](../../design/expression-construction.md) | No specific issue. Construction inputs, error precedence, binding behavior, structural grouping, and coefficient inference are explicit and linked to detailed owners. |
| [Local expressions](../../design/primitive-expression.md) | Correct responsibility and mathematical laws overall; broad comparison claim conflicts with state collection bounds (C02). |
| [Composite expressions](../../design/composite-expression.md) | Extensive current contracts and algorithm descriptions belong in design. Reorder public contracts and Act scope ahead of optimization internals (E04), and remove migration-relative wording (E06). |
| [Normalization](../../design/normalization.md) | No specific issue. API ownership, denominator, exact-zero behavior, square-root bounds, errors, and floating-point limitations are distinguished. |
| [Basis](../../design/basis.md) | Appropriate owner for basis capabilities, enumeration, coordinates, and generation. Consolidate repeated expansion/metadata explanations (E07), preserving intrinsic/action-matrix distinctions. |
| [Expression formatting](../../design/expression-formatting.md) | No specific issue. User syntax and examples precede implementation; bounds, truncation, errors, and Display/Debug resource differences are explicit. |
| [Geometry](../../design/geometry.md) | No specific issue. Separates geometry, relations, mapping, sites, and leaves; examples explain ordering and boundary cases. |
| [Models](../../design/models.md) | No specific issue. Physical Hamiltonians, coupling scales, constant shifts, placement, bounds, and boundary validation are explicit and linked to primitive conventions. |
| [Export](../../design/export.md) | Correct owner for result interpretation and conversion contracts. Update obsolete module visibility/layout (C04) and migration-relative wording (E06). Version constraints and backend requirements have local/primary-source references; external contents were not revalidated. |
| [Lie](../../design/lie.md) | Suitable canonical owner of family, numerical, representation, and decomposition contracts. Consolidate parallel detailed statements and test inventories with README/verification (E03). |
| [Verification](../../design/verification.md) | Appropriate cross-component concern under design. Distinguishes CI configuration from executions and performance claims from guarantees. Review environment-specific instructions (E01) and duplicate Lie inventories (E03). |

## Accepted decisions

| Document | Assessment |
| --- | --- |
| [Lie analysis](../../decisions/lie-analysis.md) | No specific issue. Context, choices, alternatives, and consequences are explicit; current contracts are delegated to design and native setup to the crate README. |
| [Core evaluation performance](../../decisions/core-evaluation-performance.md) | No specific issue. Accepted choices and important alternatives have rationale, workload conditions, regressions, and historical provenance. Archived Git retrieval does not link to a live plan. Measurements were not reproduced. |

## Future proposals

| Document | Assessment |
| --- | --- |
| [Future-work overview](../../future-work.md) | Proposals are visibly separate from implementation. Clarify plan initiation (E02) and consider extracting detailed numerical/performance investigations to retain an overview (E05). |
| [API usability](../../future-work/api-usability.md) | Candidates are explicitly unadopted and reference current APIs. Correct prelude status (C03) and clarify plan initiation (E02). |
| [External connections](../../future-work/external-connections.md) | No specific issue. Future AD/tensor-network responsibilities and parameter-dependency conditions are distinguished from existing export. |
| [Identity and storage](../../future-work/identity-and-storage.md) | No specific issue. Unimplemented restoration and independent copies have clearly different identity contracts; current index rules are linked. |
| [Partial trace](../../future-work/partial-trace.md) | No specific issue. Unimplemented status is explicit, and the graded counterexample explains why ordinary trace semantics require care. |
| [Representation matrices](../../future-work/representation-matrix.md) | No specific issue. Defines the proposed map domain, input conjugation, numbering, and retained coefficients before construction/verification; APIs remain undecided. |
| [State/operator review](../../future-work/state-and-operator.md) | No specific issue. Unstarted review questions are distinguished from current ket/operator facts and do not presume a unification outcome. |

## Active working material

| Document | Assessment |
| --- | --- |
| [Version 0.5 plan](../v5/README.md) | Correct temporary location and outward link to a permanent decision. Needs actionable remaining stages and completion criteria (C06); merge/release completion was not established. |

## Crate and supporting READMEs

| Document | Assessment |
| --- | --- |
| [Core](../../../crates/syntaqs-core/README.md) | No specific issue. Compact responsibilities/API overview with integration boundaries and links to canonical designs. |
| [Basic](../../../crates/syntaqs-basic/README.md) | No specific issue. User-relevant identity, issuance, names, and display summary; detailed rules remain in design. |
| [Geometry](../../../crates/syntaqs-geometry/README.md) | No specific issue. Explains relation ordering and caller ownership with useful design/example links. |
| [Normalize](../../../crates/syntaqs-normalize/README.md) | No specific issue. Concise extension trait and integration guidance, including denominator and numeric requirements. |
| [Export](../../../crates/syntaqs-export/README.md) | No specific issue. Compact feature/result table and metadata/context distinctions; detailed contracts are linked. |
| [Lie](../../../crates/syntaqs-lie/README.md) | Review parallel detailed contracts (E03) and Nix/direnv specificity (E01). Keep this document's practical native linkage instructions and provider ownership. |
| [Spin-half](../../../crates/syntaqs-spin-half/README.md) | No specific issue. Short primitive/model overview with physically necessary normalization and coupling conventions. |
| [Spin-one](../../../crates/syntaqs-spin-one/README.md) | No specific issue. Clearly distinguishes normalized primitives from physical spin and the normalized Zero state from a zero vector. |
| [Fermion](../../../crates/syntaqs-fermion/README.md) | No specific issue. Explains Majorana/parity conventions, graded composition, and Hubbard integration concisely. |
| [Examples](../../../examples/README.md) | Runnable commands and per-example feature information serve discovery. Remove migration-relative passages from current usage guidance (E06); link detailed construction contracts rather than expanding their restatement. |
| [Workspace tests](../../../tests/README.md) | No specific issue. Explains fixture ownership, package/feature selection, and test integration; does not present coverage descriptions as freshly executed results. |
| [BLAS provider](../../../support/blas-provider/README.md) | Correct internal-provider responsibility and production boundary. Review the local Nix-shell wording while retaining system-provider requirements (E01). |

The audit directory's own Markdown files were checked separately for links and
fences; they are deliverables, not part of the original 41-document assessment.
