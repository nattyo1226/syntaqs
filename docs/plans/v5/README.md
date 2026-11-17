# Version 0.5.0

## Objective and scope

Prepare version 0.5.0 for integration into `main`, covering the public module/API
reorganization, core evaluation performance work, and documentation alignment
below. Current specifications belong in [architecture](../../architecture.md)
and the feature designs; this plan records integration progress and remaining
work.

The workspace now declares version 0.5.0. The source and accepted performance
record establish implementation progress; release-preparation checks are
recorded below. Review and integration into `main` remain to be confirmed.
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

## Documentation guidelines alignment

Completed the conventions audit and writing revision across repository Markdown
documentation. Corrected API and capability descriptions, consolidated detailed
contracts in their owning designs, and clarified current specifications and
future proposals. The [documentation guide](../../README.md) records the ongoing
maintenance and writing rules.

The [root README](../../../README.md) now contains only the introduction, Quick
start, crates, documentation links, and license. Crate READMEs lead with purpose
and usage. The [installation guide](../../installation.md) recommends matching
Git version tags across crates and using release-matched documentation.

Eight standalone README examples passed against local crates. The final root
Quick start also ran against a local `v0.4.0` archive; both versions printed
`Expectation: -1`. Local links, heading anchors, plan boundaries, code fences,
and whitespace checks passed. Example builds used local path dependencies;
remote installation and external URL reachability were not tested.
Release-candidate validation is tracked separately below. The completed
documentation task's temporary audit files and helper were removed; production
code and dependencies were unchanged.

## Release preparation checks (2026-11-17)

The workspace version and release-installation examples now select 0.5.0.
Cargo metadata confirms that all 12 workspace packages inherit this version.
The root and downstream fixture lockfiles were regenerated locally; they remain
ignored under the existing repository policy.

Seventeen temporary standalone documentation programs passed against local
dependencies constrained to `=0.5.0`: eight complete README examples, eight
design snippets with caller-owned setup where needed, and the Lie README's
native provider configuration combined with the crate-level example. No example
source changes were needed. The root Quick start printed `Expectation: -1`.

All ten example binaries ran successfully in 38 feature configurations. The
nine non-Lie binaries ran with default features, no default features, export,
and export without default features. The Lie binary ran with Lie alone and
all features, recovering center dimension 1 and simple-ideal dimensions `[3,3]`.

On host stable Rust 1.99.1 and Rust 1.85.0, workspace default/all/no-default-feature
tests, the all-feature build, and the all-target no-default-feature check passed.
Stable also passed workspace and downstream formatting, all-target/all-feature
Clippy, and rustdoc with warnings denied. Separate Vec/ndarray/sprs tests for
export and shared integration, the Lie-only shared test, and downstream f32/f64
native linkage passed on stable.

Release tag consistency, local document link targets, code fences, and
whitespace were checked. Remote Git installation, external URL reachability,
GitHub-hosted CI status, review status, and integration into `main` were not
verified by these local checks.

## Stages and progress

| Stage | Status and evidence |
| --- | --- |
| Module/API reorganization | Implemented. Defining public module paths and crate preludes are present; the [architecture](../../architecture.md#5-module-structure) owns the current conventions. |
| Core evaluation performance | Implemented and validated as recorded in the [accepted decision](../../decisions/core-evaluation-performance.md). These are the recorded development checks, not a new release-candidate run. |
| Documentation guidelines alignment | Completed. Conventions corrections, writing revisions, README examples, and local document checks are summarized above; the temporary task directory has been removed. |
| Integration validation and review | Local [workspace validation](../../design/verification.md#2-common-checks) completed on stable and MSRV as recorded above. GitHub-hosted CI and review status remain unconfirmed. |
| Version preparation and integration | Version preparation completed. Workspace packages, local lockfiles, and release-installation examples agree on 0.5.0; documentation examples passed. Integration into `main` remains to be confirmed before closeout. |

## Completion criteria

- All scoped changes are reviewed and their current contracts are documented
  in permanent architecture, design, and decision records.
- Candidate validation is recorded for the configurations in the verification
  guide, with failures and unverified targets identified explicitly.
- Workspace version and lockfile agree on 0.5.0, and integration into `main` is
  confirmed.
- Any durable information remaining here is preserved in permanent documents
  before the completed plan is removed.
