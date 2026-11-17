# Documentation conventions and writing audit findings

This is the baseline audit. Subsequent conventions corrections and their
verification are recorded in [conventions results](conventions-results.md).
The later [writing phase](writing-results.md) records the completed full-document
revision and remaining audit-item resolutions.

The documentation broadly follows the required directory layout, but it is not
fully consistent with the current implementation or writing policy. This audit
identifies **six confirmed issues** and **seven editorial or policy-alignment
candidates**. Fix the public API example and contradictory capability statements
first; broader consolidation can follow without changing mathematical contracts.

## Scope and assessment basis

Audit date: 2026-11-17. Baseline Git revision:
`3856a311230bc9dd29952c20eaa82d1ac91235c2`.

All 41 tracked Markdown documents, totaling 4,400 lines, were read. The
[coverage inventory](coverage.md) records each document's assessment. Existing
documents and implementation files were not modified.

The applicable references were the installed `doc-maintain` and `doc-write`
skills and their `references/documentation/conventions.md` and `writing.md`.
The two skills contained identical copies of each reference. Their SHA-256
digests at the time of reading were:

| Reference | SHA-256 |
| --- | --- |
| `conventions.md` | `180c6087f8bb826c20d91aa6ec43bff8b030d9087034cd295935c9dc0e0c8bfc` |
| `writing.md` | `6f0dab5678534a18e856ef11ff76bc8e8df32431c731adcd0ffae6a1d413f2e7` |

References below use `C2` for conventions section 2 (document responsibilities),
`C3` for current state/history/proposals, `C4` for ownership/links/environment,
and `C7` for consistency review. `W1` through `W6` refer to the writing
guidelines' reader orientation, information flow, clarity, precision,
presentation, and references respectively.

The repository's [documentation guide](../../README.md) requires English under
`docs/`; the audit artifacts follow that established language. Priority expresses
editing order, not a claim about implementation defects. A confirmed issue has
direct documentary or local-source evidence. An editorial candidate includes a
judgment about placement, emphasis, or policy and does not by itself establish
a technical error.

## Confirmed issues

### C01 — Quick start imports unavailable public paths

Priority: high. Rules: C3, C7, W4.

[Root README](../../../README.md), line 55, imports `SpinHalfOperator`,
`SpinHalfState`, and `SzBasis` directly from `syntaqs_spin_half`. The first two
root aliases are `pub(crate)` in [lib.rs](../../../crates/syntaqs-spin-half/src/lib.rs),
lines 103–104; `SzBasis` is defined in
[basis.rs](../../../crates/syntaqs-spin-half/src/basis.rs), line 18, without a
public root re-export. The crate even contains `compile_fail` examples for all
three root imports at the start of `lib.rs`.

A reader following the advertised standalone example cannot resolve those
imports. Update the example to use `operator::SpinHalfOperator`,
`state::SpinHalfState`, and `basis::SzBasis`, retaining the root-defined
`SpinHalf` alias. Recheck the complete sample when editing; this audit verified
the mismatch statically and did not compile the README example.

### C02 — Primitive-comparison statement contradicts state evaluation

Priority: high. Rules: C3, C7, W4.

[Local-expression design](../../design/primitive-expression.md), line 148,
says evaluation needs no primitive/tree structural comparison. Its own
lines 74–79 describe automatic primitive matching requiring `PartialEq`.
The actual public `evaluate` methods require `S: PartialEq` in
[local evaluation](../../../crates/syntaqs-core/src/evaluate/local.rs),
lines 230–235, and [composite evaluation](../../../crates/syntaqs-core/src/evaluate/composite.rs),
lines 55–59. Composite evaluation additionally requires coefficient `PartialEq`.

Custom-algebra authors receive conflicting guidance about the required traits.
Distinguish the absent public whole-tree comparison API from primitive equality
used internally by state collection. Link to the
[evaluation capability table](../../design/composite-expression.md#42-shared-apis-and-minimal-capabilities)
and avoid broad claims covering both state evaluation and operator-only queries.

### C03 — Future-work guidance says no prelude is adopted

Priority: medium. Rules: C3, C7, W4.

[API usability candidates](../../future-work/api-usability.md), line 69,
says “no prelude or unified entry crate is adopted.” The
[architecture](../../architecture.md#5-module-structure) describes a prelude
in every library crate. All nine library `src/lib.rs` files contain
`pub mod prelude`; [core](../../../crates/syntaqs-core/src/lib.rs), line 133,
and [spin-half](../../../crates/syntaqs-spin-half/src/lib.rs), line 132,
provide concrete examples.

Separate the already provided per-crate preludes from the still unadopted
unified entry crate. Preserve any still-relevant onboarding proposal without
presenting an implemented facility as absent.

### C04 — Export module overview describes obsolete private modules

Priority: medium. Rules: C2, C3, C7, W4.

[Export design](../../design/export.md), line 194, describes private `output`,
`metadata`, `args`, `coordinates`, `vector`, `matrix`, `error`, and backend
modules. [The crate root](../../../crates/syntaqs-export/src/lib.rs),
lines 59–62, actually declares private `backend`/`coordinates` and public
`matrix`/`vector`. [matrix.rs](../../../crates/syntaqs-export/src/matrix.rs)
exposes `args` and `metadata`; [vector.rs](../../../crates/syntaqs-export/src/vector.rs)
exposes `metadata`. `ExportError` is defined at the crate root, not in a
separate `error` module.

The paragraph conflicts with the document's own current API paths and can
mislead readers about accessibility. Replace it with the current semantic
responsibilities and the few public paths readers need, without transcribing
the full source tree.

### C05 — Root verification pointer does not contain its advertised MSRV limits

Priority: low. Rules: C4, C7, W6.

[Root README](../../../README.md), line 94, points to future work for
verification limits “including the declared minimum Rust version.” The linked
[future-work section](../../future-work.md#3-outstanding-verification-and-performance)
covers dedicated no_std targets and performance, but contains no MSRV
verification discussion. The authoritative
[verification guide](../../design/verification.md#3-ci-and-verification-limits)
does describe Rust 1.85.0/stable configurations and the distinction between
workflow configuration and executed checks. The
[CI source](../../../.github/workflows/ci.yml) confirms the configured matrix.

This is a valid file/anchor with a misleading description, not a broken link.
Point current verification coverage to its owner and keep future work for
outstanding investigations. Do not infer a passing hosted run from CI YAML.

### C06 — Version 0.5 plan lacks an actionable completion definition

Priority: medium. Rule: C2, plans.

The entire [v5 plan](../v5/README.md) consists of a merge/version intention and
two summaries of implemented work. It records progress and topics, but provides
no staged remaining work or explicit completion criteria for validation, version
preparation, and closeout. A later contributor cannot determine what remains
between “Ready for merge” and completion.

Add a concise objective/scope, stages with status, and completion criteria, or
close the plan only after its actual completion is established. This audit did
not verify merge/release status and does not conclude that the plan is abandoned
or ready for deletion. The workspace still declares version 0.4.0 in
[Cargo.toml](../../../Cargo.toml).

## Editorial and policy-alignment candidates

### E01 — Separate native requirements from local environment conveniences

Priority: medium. Rules: C4, W1, W4.

[Verification](../../design/verification.md), line 15, introduces a `direnv exec .`
prefix. [Lie's README](../../../crates/syntaqs-lie/README.md), lines 148–160,
prescribes Nix devShell packages and prefixes all four verification commands
with direnv. [blas-provider's README](../../../support/blas-provider/README.md),
lines 7–8, also names Nix shells. `git ls-files` returned no tracked `.envrc`,
`.sideflake`, `flake.nix`, or `flake.lock` configuration.

The real requirements are BLAS/LAPACK and provider discovery: the
[provider manifest](../../../support/blas-provider/Cargo.toml) selects system
OpenBLAS, and CI installs OpenBLAS/pkg-config. Retain those requirements and
the application-owned provider instructions. Prefer direct Cargo commands;
retain Nix/direnv recipes only if their supported, reproducible role is deliberate
and explained. The conventions allow supported environment procedures, so the
mere occurrence of Nix, Linux, or a host target is not a violation. Whether the
current Nix recipe is an intentional supported workflow remains unverified.

### E02 — Clarify when implementation work needs a plan

Priority: medium. Rules: C1, C2.

[Documentation guide](../../README.md), line 16,
[future-work overview](../../future-work.md), line 13, and
[API usability candidates](../../future-work/api-usability.md), line 5,
say to create a plan when implementation starts. The shared conventions limit
that expectation to work benefiting from multiple stages and permit skipping
plans for straightforward changes.

Qualify the trigger with task size/investigation needs. If the repository
deliberately requires a plan for every implementation, identify that as a
repository-specific override. Existing local conventions can be more specific;
this is a policy-alignment decision, not proof that every existing plan is wrong.

### E03 — Give detailed Lie contracts and verification coverage one owner

Priority: medium. Rules: C4, W3, W6.

[Lie's README](../../../crates/syntaqs-lie/README.md), lines 12–126, repeats
the family lifecycle, tolerance defaults, coordinate conventions, center
representation, retry algorithm, and decomposition diagnostics also specified
in [Lie design](../../design/lie.md), sections 1–7. Its 165-line length is not
itself a violation; the issue is maintaining parallel detailed contracts.
Likewise, Lie test inventories appear both in design section 8 and
[verification](../../design/verification.md), lines 130–151.

Keep the README's purpose, minimal usage path, integration limits, and native
linkage. Let the design own mathematical/API contracts and link directly to
relevant sections. Let verification own commands and environment/CI coverage;
keep feature-specific test rationale close to its design with concise links
from the central inventory. Preserve useful local summaries rather than
replacing every explanation with a link.

### E04 — Put public composite-expression contracts before optimization detail

Priority: medium. Rules: W1, W2, W5.

[Composite-expression design](../../design/composite-expression.md) places
implementation sections 9.1–9.4 at lines 233–371 before the public API table at
line 373 and shared construction/capability/Act-scope contracts at lines
390–432. A reader looking for available operations and required capabilities
must pass detailed candidate scans, interning, and accumulator behavior first.

Bring the public API, capability requirements, and Act-scope semantics next to
placement/binding. Follow with composition laws and ordinary/graded lowering,
then implementation algorithms and verification. Consolidate repeated
construction/binding summaries while retaining distinctions between structural
grouping, primitive collection, intrinsic coordinates, and ordinary lowering.
This is a reader-oriented ordering recommendation, not a mandatory template.

### E05 — Keep the future-work entry point an overview

Priority: low. Rule: C2, future work.

[Future work](../../future-work.md), sections 3.1–3.3, contains roughly forty
lines of residual-accuracy, contraction-preparation, and state-collection
investigation conditions. Together with the expression-management discussion
in section 2.2, these require more detail than the top-level roadmap's scanning
purpose needs.

Consider a focused performance/numerical-investigation proposal under
`future-work/`, leaving concise topics and links in the overview. Keep these
ideas explicitly deferred; do not transfer them to current design contracts
or create implementation plans before work is accepted.

### E06 — Replace migration-relative wording in current explanations

Priority: low. Rules: C3, W1, W3, W4.

[Local-algebra design](../../design/local-structure.md), line 68, refers to
“the previous normalization” without identifying a current concept.
[Composite design](../../design/composite-expression.md), line 388, says type
parameters “remain unchanged”; [export design](../../design/export.md), line
148, similarly says contracts “remain unchanged.” These describe change
history rather than directly stating the present contract.

[Examples README](../../../examples/README.md), lines 70 and 74, mixes removed
`Chain<I>`/`ToSpace` APIs and the previous `scaled` inference behavior into
example discovery and usage guidance. State current construction and inference
rules directly and link to the construction design. Preserve consequential
migration rationale in a decision record only when it merits historical
retention; do not create records for every removed phrase.

### E07 — Consolidate repeated basis and identity explanations

Priority: low. Rules: C4, W3, W6.

[Basis design](../../design/basis.md), lines 49, 83–95, and 114–120, repeats
that expansion produces ordinary trees without retained basis metadata and
that evaluation and expansion are separate. The same key contract is summarized
again in local/composite expression documents.
[Index design](../../design/index.md), lines 13, 30, and 73–77, repeatedly
explains the absence of an index-to-algebra registry.

Make each full explanation primary once, then retain only the summary needed
at each operation boundary. Describe factor identity and primitive-type
responsibilities directly, and link back when that distinction matters.
Do not remove meaningful exclusions, grading conditions, or capability bounds
merely because they use negative wording.

## Conforming aspects to preserve

- Requirements describe goals, mathematical capabilities, and constraints;
  architecture describes component responsibility and dependency direction.
  Its compact module overview is appropriate system-level context.
- `design/` owns APIs, mathematical definitions, conditions, and relevant
  algorithms. Detailed algebra product tables specify semantics and are not
  automatically mechanical code duplication.
- Decision records identify accepted choices, alternatives, rationale, and
  consequences. The performance record distinguishes dated measurements from
  guarantees and records reproducibility conditions.
- Future proposals explicitly distinguish unimplemented behavior from current
  APIs. Mathematical conditions in a proposal do not make it an adopted feature.
- Existing documents are consistently English. API names and established
  mathematical terminology do not constitute unnecessary language mixing.
- No live Markdown link from outside `plans/` into `plans/` was detected.
  The performance decision's `git show <revision>:docs/plans/...` command is
  historical evidence retrieval, not a dependency on a live plan document.

## Verification and limitations

[check_docs.py](check_docs.py) checks the working-tree content of Markdown files
listed in `HEAD`, plus this task's Markdown artifacts, using Python's standard
library. The [recorded output](checks.json) contains the baseline revision, per-document line
counts, link counts, and detected issues. Reproduce against the current tree:

```sh
python3 docs/plans/documentation-guidelines/check_docs.py
```

For the original 41 documents, the check found **386 local link occurrences**,
including **67 heading-fragment links**, and skipped **19 external URL
occurrences**. It found no missing local targets, missing checked heading
anchors, prohibited permanent-to-plan links, or unclosed fenced code blocks.
The task artifacts are counted separately so report links do not inflate the
baseline figures.

The checker handles the repository's inline Markdown links, ATX headings,
duplicate heading slugs, and backtick/tilde fences. It approximates GitHub
heading slugs; it is not a complete CommonMark parser or a renderer-based
validation. External URL reachability and external source contents were not
checked. Code spans and historical Git commands are not treated as Markdown
links. The misleading but syntactically valid pointer in C05 requires semantic
review and is not detectable by the link check.

Technical discrepancies were checked by reading the relevant public declarations,
module files, manifests, CI configuration, and existing compile-fail examples.
The final scope check used `git diff --exit-code HEAD --` to confirm that tracked
files were unchanged, and whitespace checks covered every new task artifact.
The audit did not run Cargo builds/tests, compile every Markdown example,
reproduce archived performance measurements, inspect all rustdoc comments, or
verify hosted CI/merge/release status. No new correctness or performance
guarantees are inferred from existing test descriptions.

## Proposed editing sequence

1. Correct C01–C04, then resolve the verification pointer C05 and plan criteria
   C06. Compile the corrected standalone example and recheck public paths/bounds.
2. Resolve environment and plan-policy wording (E01–E02), keeping real native
   dependencies and any deliberate repository-specific overrides explicit.
3. Consolidate Lie ownership and composite information flow (E03–E04), then
   address the roadmap and wording improvements (E05–E07) in bounded edits.
4. After each move or heading change, rerun local link checks and inspect incoming
   references. Keep permanent documents independent of this temporary directory.

These are proposed follow-up edits. This audit authorizes no implementation,
permanent-document rewrite, plan deletion, commit, or publication.
