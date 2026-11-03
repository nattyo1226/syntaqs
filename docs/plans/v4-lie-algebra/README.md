# syntaqs v4: Lie algebra implementation workspace

Created: 2026-11-03
Status: design decisions in progress; implementation has not started.

This directory holds temporary investigation notes, design decisions, and
implementation progress for bringing the functionality of LieAlgebra.jl onto
syntaqs. The present task establishes the workspace and records the existing
implementations; it does not settle public APIs or add production code.

## Documents

| Document | Purpose |
| --- | --- |
| [Current implementation](current-implementation.md) | Existing syntaqs foundations, extension points, and constraints. |
| [Reference functionality](reference-functionality.md) | Source-based inventory of LieAlgebra.jl functionality and compatibility questions. |
| [Design questions](design-questions.md) | Adopted decisions, remaining questions, and a provisional order of work. |

## Investigation baseline

| Repository | Inspected revision | Working tree at inspection |
| --- | --- | --- |
| syntaqs | `de686db19630a7a2fadcaab660fd12ea4fb5dcdc` | Clean before these documents were added. |
| LieAlgebra.jl | `575029c12fcad44b3b41206ee564437a7ec5fa45` | Clean. |

The reference was read from the existing local clone at
`~/repos/github.com/nattyo1226/LieAlgebra`. Its source and tests, rather than its
minimal README, are the basis of the inventory. The upstream repository is
[LieAlgebra.jl](https://github.com/nattyo1226/LieAlgebra.jl); source links in these
notes are pinned to the inspected revision. No fetch or reference-repository
modification was performed. Local checkout freshness against upstream was not
verified.

The user called this effort “v4”; the current Cargo workspace version is
`0.3.0`. A release version or version bump is not decided here.

## Location and lifecycle

This workspace is under `docs/plans/v4-lie-algebra/`, alongside the existing
permanent documentation under [`docs/`](../../../docs/README.md). These notes
use English consistently with the documentation convention.

Keep proposals distinct from verified existing behavior. Record adopted
decisions and dated verification here during implementation. When the work is
complete and merged, transfer lasting specifications into the appropriate
permanent documents, update references, and remove this temporary workspace in
accordance with the documentation guide.

## Progress

- [x] Read the current architecture, requirements, relevant designs, and source.
- [x] Inspect the local LieAlgebra.jl implementation and its tests.
- [x] Record feature coverage, reuse opportunities, and unresolved contracts.
- [x] Create this temporary workspace.
- [x] Confirm the real Hermitian Lie algebra domain and the bracket `-i(AB-BA)`.
- [x] Adopt lightweight basis construction, explicit validation, and required
  precondition checks for mathematical operations, with tolerance-aware reports.
- [x] Adopt the classical Killing form `K(X,Y)=Tr(ad_X ad_Y)`, without
  conjugation or dimension normalization.
- [x] Adopt the normalized HS residual norm `sqrt(Tr(R†R)/D)`.
- [x] Adopt projection-coefficient retrieval with separate reconstruction,
  residual-norm, and span-membership operations, including nonorthonormal bases.
- [x] Adopt decomposition into the full center and nonabelian simple ideals.
- [x] Adopt the Julia-style commutant/eigenspace algorithm with center
  extraction, internal orthonormalization, result validation/retry, and
  caller-supplied random seeds.
- [x] Fix the Lie algebra commutant as the commutant of its adjoint
  representation, expressed in the selected Lie algebra basis.
- [x] Record the provisional numerical error policy: recommended
  precision-aware defaults with caller-supplied tolerance overrides. Initial
  residual tolerances are `atol=rtol=2e-5` for f32 and `1e-12` for f64, with
  dimension-aware spectral cutoffs; verification and configuration signatures
  remain implementation work.
- [x] Adopt syntaqs for algebraic expression operations and dense ndarray arrays
  for numerical coefficients, using ndarray-linalg for factorizations as
  needed. Initially target ndarray-linalg 0.18.1 with ndarray 0.17.2, subject
  to build verification.
- [x] Leave LAPACK provider selection to the final application; do not select
  a provider in syntaqs-lie's production dependencies. Test/CI configuration
  remains implementation work.
- [x] Place Lie algebra functionality and its numerical integration in the
  dedicated syntaqs-lie crate, using the existing public core capabilities.
- [x] Keep representation-coefficient generation and storage within syntaqs-lie
  for this iteration; defer commonality with general map representations.
- [x] Fix the operator inner product to normalized HS, without custom inner
  products, and report dependent/numerically singular bases as errors in
  operations requiring independence.
- [x] Adopt OperatorFamily for candidate operator collections and
  LieDecomposition for the center-plus-simple-ideal result.
- [ ] Resolve the remaining design questions and agree on concrete acceptance criteria.
- [ ] Implement and verify the selected functionality.

## Verification on 2026-11-03

| Check | Result |
| --- | --- |
| `direnv exec . cargo test -p syntaqs-core -p syntaqs-tests --all-features` | Passed, including unit/integration tests, doctests, and compile-fail doctests. Covers the existing foundations, not the proposed Lie algebra feature. |
| Local Markdown links and code-fence pairing in these four files | Passed using a local Python check. External link availability was not checked by that script. |
| Whitespace and final change-scope review | Passed; additions are limited to this directory. |

Cargo emitted existing manifest warnings about specifying both `license` and
`license-file`; no manifest was changed. The full workspace CI matrix was not
run for this documentation-only task. The reference's full Julia test suite was
not executed. Exploratory checks do not establish general decomposition support.
No runtime compatibility between the two libraries has yet been established.
