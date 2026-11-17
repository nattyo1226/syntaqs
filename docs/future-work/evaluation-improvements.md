# Numerical accuracy and expression evaluation improvements

Updated: 2026-11-17

These are deferred investigations and unadopted proposals. Public APIs,
algorithms, schedules, and priorities are undecided. Current evaluation contracts
belong in [composite expressions](../design/composite-expression.md), numerical
rules in [numerics](../design/numerics.md), and verification coverage and limits
in [verification](../design/verification.md).

## 1. Lie residual accuracy

Investigate reducing cancellation in projection, closure, and Hermiticity
residuals for nonorthonormal operator expressions. The current implementation
uses explicit exact term collection; strict tolerances can still reject some
mathematically valid families. Any improvement must preserve small positive
residuals, avoid tolerance-based coefficient pruning, and retain factored local
sums without global Cartesian expansion. No additional accuracy guarantee or
specific replacement algorithm is adopted. See the
[current numerical contract](../design/lie.md#3-projection-and-numerical-scales)
and [verification limits](../design/verification.md#3-ci-and-verification-limits).

## 2. Inner-product acceleration

Further state and operator Hilbert–Schmidt inner-product acceleration is deferred
beyond incremental local contraction and exact-zero short-circuiting. Candidate
approaches include sparse-coordinate contraction with suitable basis information,
prefiltering term pairs using guaranteed orthogonality or conserved sectors, and
reuse of partial contractions. Public APIs and algorithms are undecided.

Incremental contraction still prepares structural group metadata eagerly and
considers every pair of summands. Investigate generating later tensor groups
only when a nonzero prefix needs them. Separate this preparation cost from
operand collection and primitive contraction counts; preserve independent tensor
boundaries, original validation, and continuation of other sum branches.

Distinct primitives are not necessarily orthogonal, and an arbitrary basis need
not be orthonormal. Any coordinate-based method must account for the relevant
Gram matrix unless orthonormality is guaranteed. Structural pruning must preserve
ordinary/graded semantics and must not imply approximate component truncation.
Any caches require explicit ownership, scope, reuse criteria, and resource limits.

## 3. Natural-state collection overhead

Per-query normalization and equality-based interning can dominate a single local
action or a small factored norm even when they prevent explosive growth during
repeated actions. Investigate reducing context initialization and allocation,
and avoiding repeated preparation within one query. Preserve action-free
collection, first-occurrence order, nonreflexive PartialEq behavior, minimal
public bounds, and retained products of sums. A persistent normalized marker or
cross-query cache is not adopted; either would require an explicit validity and
ownership design. Distinct terms and intrinsically large coordinate output remain
separate costs.

## 4. Expression management and reuse

Automatic sharing of independently constructed equal-shaped expressions, result
caches, parallel evaluation, and cross-thread sharing require decisions on
whether to add them. Caches need scope, reuse criteria, ownership, invalidation,
and resource management; parallelism needs stored-type requirements and an
execution model.

Distributing sums inside operator products can increase term counts even when
the result is sparse. Further investigations beyond same-factor local grouping
include comparing general result caching with recomputation and retaining
interval parity as unevaluated structure. Query-local reuse of an identity's
basis expansion is separate from general result caching.

## 5. Explicit component pruning

Consider approximate component pruning as explicit postprocessing with a
caller-selected basis, tolerance, and application point. In a
dimension-normalized HS orthonormal basis, discarded `δA=Σμ δaμ Bμ` has squared
error `||δA||²=Σμ |δaμ|²`. This formula does not apply unchanged to
non-orthonormal bases. Current evaluation, coordinates, and export must not
silently prune components.
