# Version 0.5.0

## Objective and scope

Prepare version 0.5.0 for integration into `main`, covering the public module/API
reorganization and core evaluation performance work below. Current specifications
belong in [architecture](../../architecture.md) and the feature designs; this
plan records integration progress and remaining work.

The workspace currently declares version 0.4.0. The source and accepted
performance record establish implementation progress; integration validation,
version preparation, and merge confirmation remain separate stages below.
Publishing and further feature development are outside this plan's scope.

## Module structure and public API

The implemented API uses defining module paths, with a prelude for frequently
used items in each library. Compatibility paths and non-prelude re-exports were
removed, so this is a breaking change. Examples construct Neel states from
expression building blocks instead of the removed spin-half helper.

## Core evaluation performance

Implemented and validated local-operator grouping, state-term collection between
actions, exact-zero contraction stopping, and shared spectator identity expansion
for operator coordinates. Decisions, performance comparisons,
and remaining limits are recorded in the
[permanent task record](../../decisions/core-evaluation-performance.md).

## Stages and progress

| Stage | Status and evidence |
| --- | --- |
| Module/API reorganization | Implemented. Defining public module paths and crate preludes are present; the [architecture](../../architecture.md#5-module-structure) owns the current conventions. |
| Core evaluation performance | Implemented and validated as recorded in the [accepted decision](../../decisions/core-evaluation-performance.md). These are the recorded development checks, not a new release-candidate run. |
| Integration validation and review | Pending. Check the candidate with the [verification commands](../../design/verification.md#2-common-checks), record results and limitations here, and confirm review status. |
| Version preparation and integration | Pending. Update workspace version/lockfile consistently with 0.5.0 and confirm integration into the intended branch before closeout. |

## Completion criteria

- Both scoped changes are reviewed and their current contracts are documented
  in permanent architecture, design, and decision records.
- Candidate validation is recorded for the configurations in the verification
  guide, with failures and unverified targets identified explicitly.
- Workspace version and lockfile agree on 0.5.0, and integration into `main` is
  confirmed.
- Any durable information remaining here is preserved in permanent documents
  before the completed plan is removed.
