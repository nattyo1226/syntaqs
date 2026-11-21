# Numerical accuracy and expression evaluation improvements

Updated: 2026-11-21

The aim is to reduce expression-contraction cancellation and query overhead
while preserving exact-zero rules, small nonzero values, and factored structure.
These are deferred investigations and unadopted proposals; public APIs,
algorithms, schedules, and priorities are undecided. Current evaluation contracts
belong in [composite expressions](../design/composite-expression.md), numerical
rules in [numerics](../design/numerics.md), and verification coverage and limits
in [verification](../design/verification.md).

## 1. Lie residual accuracy

Investigate reducing cancellation in projection, closure, and Hermiticity
residuals for nonorthonormal operator expressions. HS evaluation collects
matching natural terms exactly. Strict tolerances can still reject some
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

State contraction prepares factor counts, aligns tensor blocks on demand, and
uses conjugate symmetry for matching sums. Many of the remaining pairs in a
W-state norm vanish at a local overlap. Measure pair rejection separately from
factor coverage preparation and primitive contraction. Any additional pruning
must preserve independent tensor boundaries and original validation.

Distinct primitives are not necessarily orthogonal, and an arbitrary basis need
not be orthonormal. Any coordinate-based method must account for the relevant
Gram matrix unless orthonormality is guaranteed. Structural pruning must preserve
ordinary/graded semantics and must not imply approximate component truncation.
Any caches require explicit ownership, scope, reuse criteria, and resource limits.

## 3. Natural-state collection overhead

Preparing normalized state terms and equality-based interning for each query
can dominate a single local action or a small factored norm, even when term
collection prevents explosive growth during repeated actions. Investigate
reducing context initialization and allocation,
and avoiding repeated preparation within one query. Preserve action-free
collection, first-occurrence order, nonreflexive PartialEq behavior, minimal
public bounds, and retained products of sums. A persistent normalized marker or
cross-query cache is not adopted; either would require an explicit validity and
ownership design. Distinct terms and intrinsically large coordinate output remain
separate costs.

Product actions stop on exact zero, and local actions visit only affected tensor
children. For graded Hubbard action on the vacuum, measure the work in each
branch up to the point where it vanishes. For repeated Hamiltonian action on
nonzero states, measure intermediate allocations and test whether adjacent
applications can reuse state shapes. Preserve exact-zero semantics and the order
of graded actions.
[State evaluation](../../crates/syntaqs-core/src/evaluate/composite.rs)

## 4. Expression management and reuse

Automatic sharing of independently constructed equal-shaped expressions, result
caches, parallel evaluation, and cross-thread sharing require decisions on
whether to add them. Caches need scope, reuse criteria, ownership, invalidation,
and resource management; parallelism needs stored-type requirements and an
execution model.

Overlapping operator blocks can generate many intermediate terms even when
the result is sparse. Compare caching evaluated blocks with recomputation
and investigate retaining interval parity as unevaluated structure.

Benchmark primitive interning with `PartialEq` scans, `Ord`, and `PartialOrd`.
Include spin and fermion workloads and cases with many distinct primitives.
Measure comparison counts and query time, and account for incomparable values
before considering changes to primitive bounds.

## 5. Explicit component pruning

Consider approximate component pruning as explicit postprocessing with a
caller-selected basis, tolerance, and application point. In a
dimension-normalized HS orthonormal basis, discarded `δA=Σμ δaμ Bμ` has squared
error `||δA||²=Σμ |δaμ|²`. This formula does not apply unchanged to
non-orthonormal bases. Current evaluation, coordinates, and export must not
silently prune components.

## 6. Ambient-space setup in operator evaluation

Operator blocks keep spectator identities implicit through collection,
reconstruction, and normalized HS contraction. Collection still gathers
the ambient leaves to reconstruct explicit placements, and HS allocates
its component table using the full ambient factor count.
[Block evaluation](../design/composite-expression.md#117-operator-block-evaluation),
[collection](../../crates/syntaqs-core/src/evaluate/collect.rs#L73),
[HS alignment](../../crates/syntaqs-core/src/evaluate/operator/contraction.rs#L98).

Measure the leaf gathering and factor alignment separately on a fixed
two-site operator with a growing ambient space. If setup becomes material,
investigate reconstructing placements by position and allocating alignment
metadata for the reached support. Preserve validation of the original
references, ambient order for graded signs, the full trace dimension
factor, and the identity contribution to normalized HS.

## 7. Setup cost in small operator basis queries

The operator evaluator builds primitive tables, term maps, and support vectors
before basis conversion. Coordinate queries for first-order Heisenberg sums
at 4–16 sites became slower when block evaluation was introduced. Small
Cartesian BLBQ queries now request more bytes, most of them during block
evaluation. Investigate lighter storage and fewer temporary collections when
blocks contain few terms. Use the small sums, a large lattice, and a factored
gate followed by its inverse as complementary controls. Measure requested
bytes as well as time. Preserve block-local cancellation and graded factor
order.
[Operator evaluator](../../crates/syntaqs-core/src/evaluate/operator.rs)

## 8. Storage for coordinates with dense support

Independent tensor choices can produce exponentially many nonzero coordinates.
An 18-site product of `|+⟩` factors produces 262,144 entries with 18 digits
per key. Compare compact key storage or streaming with the sparse map, using
a sparse GHZ input as a control. Measure both per-entry storage and the cost
of aggregating overlapping contributions. Preserve factor order, block
correlations, and access to sparse coordinates when the full dimension
exceeds `usize`.
[Coordinate contract](../design/basis.md#72-coordinate-interpretation-and-indices)

## 9. Extreme coefficients in composite Hilbert–Schmidt products

Let `Sp` be the spin-1/2 raising operator. With `Complex<f64>`, place
`1e308·Sp + 1e308·Sp − 1e308·Sp` and `Sp` on the same factor of a
two-factor ordinary space.
The normalized inner product is `5e307 + 0i`. In the performance review,
Stage 0 (`49e9fc5`) and the local `hs_inner_product()` returned that value.
The composite `hs_inner_product_in()` at `20ea94f` returned `NaN + NaN i`.
The shared evaluator overflowed while adding the first two coefficients
before contraction applied the normalized overlap of `0.5`.
[Coefficient collection](../../crates/syntaqs-core/src/evaluate/operator.rs)

This regression has low priority because the reproduced case combines
coefficients near the `f64` limit with cancellation. Changing when the
shared evaluator combines coefficients could affect the performance of typical
Hilbert–Schmidt queries. Revisit the fix if the failure occurs in a real
workload.
