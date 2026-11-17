# Conventions corrections

The conventions phase is complete. C01–C06 and E01/E02/E03/E05 from the
[baseline audit](audit.md) have been addressed. Corrections were checked against
the installed documentation skills and local implementation at revision
`c1aeaf779c6894b2743ec2855a8fa4cc9240fdf9`, with documentation edits applied in
the working tree. Production Rust sources, manifests, and the lockfile are
unchanged.

## Resolutions

| Finding | Resolution |
| --- | --- |
| C01: Quick start paths | [Root README](../../../README.md#quick-start) imports primitives and the basis from their defining public modules. The complete sample compiled and ran against local crates. |
| C02: primitive comparison | [Local-expression design](../../design/primitive-expression.md#5-linear-dependence-and-basis-connections) distinguishes primitive/coefficient equality used by state collection from the absent public expression-tree comparison API, and links to the canonical capability table. |
| C03: prelude status | [API usability](../../future-work/api-usability.md#5-onboarding-and-custom-algebra-guidance) distinguishes existing per-crate preludes from the unadopted unified entry crate. All nine library roots were checked. |
| C04: export module layout | [Export design](../../design/export.md#8-types-and-module-structure) describes public matrix/vector metadata and arguments, private conversion implementations, and crate-root abstractions consistently with the declarations. |
| C05: verification pointer | The root README points minimum Rust version and no_std verification questions to the [verification guide](../../design/verification.md#3-ci-and-verification-limits); future work remains the owner of deferred investigations. |
| C06: v5 plan | The [v5 plan](../v5/README.md) now states objective, scope, stages, evidence, remaining integration/version work, and completion criteria. Recorded implementation checks are distinguished from pending candidate validation; merge/release readiness is not asserted. |
| E01: environment instructions | Permanent verification, Lie, and provider documents use Cargo and actual system OpenBLAS/LAPACK/pkg-config requirements. Local Nix/direnv instructions were removed; application-owned provider selection remains documented. |
| E02: plan trigger | The [documentation guide](../../README.md#1-document-responsibilities), [future-work overview](../../future-work.md#2-unimplemented-features-and-proposals), and API-usability proposal require plans for work benefiting from stages or investigation and permit straightforward changes without one. |
| E03: Lie ownership | [Lie design](../../design/lie.md) owns detailed APIs, numerical contracts, algorithms, and feature verification targets. [The crate README](../../../crates/syntaqs-lie/README.md) provides usage navigation and practical native linkage; [verification](../../design/verification.md) owns common commands, CI, and linkage checks. Missing kernel details were carried into design before removing duplicate descriptions. |
| E05: future-work overview | Detailed numerical/performance investigations, cache/sharing conditions, and component-pruning conditions were moved to [evaluation improvements](../../future-work/evaluation-improvements.md). The overview retains summaries and links; incoming residual-accuracy links were updated. |

## Verification

- The Rust block under the root README's Quick start was copied verbatim into a
  temporary standalone package. Only the dependency source was changed from Git
  to local `syntaqs-core` and `syntaqs-spin-half` paths; `num-complex` remained
  version 0.4.6. `cargo run --offline` with that manifest passed on Rust/Cargo
  1.99.1, checked the example's expectation assertion, and printed `1: 1+0i`.
- The local/composite evaluation bounds, spin-half public paths, all library
  preludes, export modules, native-provider manifest, and Lie numerical kernel
  declarations were inspected for the corrected claims.
- The three moved numerical/performance investigations were compared with their
  original text, ignoring relative-link changes. The pruning formula and cache/
  reuse conditions were also checked for preservation. All comparisons passed.
- Local links, heading anchors, permanent-to-plan boundaries, and fenced blocks
  passed the [current mechanical checks](conventions-checks.json). These cover
  the 42 repository documents including the new proposal, plus this task's
  Markdown records. Reproduce with:

  ```sh
  python3 docs/plans/documentation-guidelines/check_docs.py --include docs/future-work/evaluation-improvements.md
  ```

- `git diff --check` passed. The final diff was inspected for scope, document
  ownership, preserved contracts, and affected references. The original
  [audit check snapshot](checks.json) is retained separately.

This phase did not run the full workspace test suite, an additional build on the
minimum Rust version, native Lie calculations, or hosted CI. The example validates the
current local API and behavior; it does not test fetching the advertised Git
dependencies. External URLs were not checked. The mechanical checker retains
the parser limitations described in the baseline audit.

## Remaining work

The later writing-focused task retains E04 (composite explanation order), E06
(migration-relative wording), and E07 (repeated basis/identity explanations).
The baseline audit and coverage inventory remain historical findings rather
than claims that the corrected issues are still present. The v5 plan remains
active; this documentation task does not establish its release completion.
