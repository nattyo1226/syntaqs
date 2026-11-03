# syntaqs v4: Lie algebra implementation

Updated: 2026-11-03
Status: scope and main decisions agreed; implementation has not started.
Next action: [Step 1](implementation-plan.md#step-1-crate-and-numerical-foundation).

Implement the agreed LieAlgebra.jl functionality on syntaqs in the dedicated
`syntaqs-lie` crate. This directory is the temporary implementation workspace.

| Document | Responsibility |
| --- | --- |
| [Specification](spec.md) | Single source of truth for agreed behavior and numerical defaults. |
| [Implementation plan](implementation-plan.md) | Six ordered steps, deliverables, acceptance checks, and remaining implementation choices. |
| [References](references.md) | Existing extension points, pinned Julia sources, and intentional compatibility differences. |

Read the specification, then execute the plan in order. Record completed steps
and dated checks in the plan; do not duplicate specifications in progress logs.
Concrete Rust signatures and supporting type names can be settled within each
step under the agreed contracts. No further broad design review is a prerequisite.

## Verification record

| Date | Check | Result |
| --- | --- | --- |
| 2026-11-03, initial investigation | `direnv exec . cargo test -p syntaqs-core -p syntaqs-tests --all-features` | Passed, including integration tests and doctests. Existing manifest warnings about both license fields were emitted. |
| 2026-11-03, document consolidation | Local links/anchors, Markdown structure/whitespace, stale-reference search, and final scope review | Passed. Six implementation steps remain pending; changes are confined to this workspace. |

These results cover existing foundations, not the new crate. The full workspace
matrix and the reference's full Julia test suite have not been run for this
project. Version compatibility has been checked from metadata; LAPACK linking,
Rust 1.85 builds with the new dependency, and the proposed numerical defaults
still require verification in Step 1 and subsequent feature tests.

## Maintenance

Follow the [documentation guide](../../README.md). Once the implementation is
complete and merged, move lasting specifications to permanent documentation,
update incoming links, and remove this workspace. “v4” is the project label;
the current Cargo version is `0.3.0`. Release numbering is a release-time task.
