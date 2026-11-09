# syntaqs v4: Lie algebra implementation

Updated: 2026-11-09
Status: Steps 1–6 complete; implementation and local full verification finished.
Next action: review and merge. Retain this workspace until merge.

The agreed LieAlgebra.jl functionality is implemented in `syntaqs-lie`. This
directory retains the agreed target and verification history pending merge.
Current contracts belong to [the permanent Lie design](../../design/lie.md).

| Document | Responsibility |
| --- | --- |
| [Specification](spec.md) | Agreed implementation target and numerical defaults; current contracts are in the permanent design. |
| [Implementation plan](implementation-plan.md) | Completed steps, deliverables, acceptance checks, and dated verification results. |
| [References](references.md) | Existing extension points, pinned Julia sources, and intentional compatibility differences. |

Use the permanent design and crate documentation for current public APIs.
This workspace preserves implementation decisions and actual check results;
no implementation steps remain pending.

## Verification record

| Date | Check | Result |
| --- | --- | --- |
| 2026-11-03, initial investigation | `direnv exec . cargo test -p syntaqs-core -p syntaqs-tests --all-features` | Passed, including integration tests and doctests. Existing manifest warnings about both license fields were emitted. |
| 2026-11-03, document consolidation | Local links/anchors, Markdown structure/whitespace, stale-reference search, and final scope review | Passed. Six implementation steps remain pending; changes are confined to this workspace. |

Numerical, family, representation, invariant, and decomposition tests, workspace checks, and
Rust 1.85 results are recorded in
[the implementation plan](implementation-plan.md#step-6-results-2026-11-09).
Earlier step results also record application linkage and provider isolation.
The public workflow and full local matrix passed; the reference's Julia suite
and GitHub-hosted Linux CI were not run in this session.

## Maintenance

Follow the [documentation guide](../../README.md). Once the implementation is
complete and merged, move lasting specifications to permanent documentation,
update incoming links, and remove this workspace. “v4” is the project label;
the current Cargo version is `0.3.0`. Release numbering is a release-time task.
