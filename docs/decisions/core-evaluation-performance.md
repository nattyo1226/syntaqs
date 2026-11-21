# Core expression evaluation: collection boundaries and sparse coordinates

Status: Accepted  
Reference measurements: 2026-11-16

Preserve local expression structure, collect matching state terms between
actions, stop exact-zero contractions, and share spectator identity expansion
within a coordinate query. These choices reduce avoidable work while retaining
factored outputs; query preparation can still cost more on small workloads.
The measurements below record both improvements and regressions under explicit
conditions, without establishing general performance guarantees.

## Context

Realistic local gate layers, repeated spin rotations, Hamiltonian actions,
W-state norms, and sparse Hamiltonian coordinates exposed different costs:
composite distribution before local cancellation, duplicate intermediate states,
unnecessary contractions after zero, and repeated spectator identity expansion.
A small physical result does not imply a small syntactic evaluation path.
Conversely, a factored state can have an intrinsically large coordinate output.

## Decisions and alternatives

Use cheap direct same-factor sum/product grouping during construction, iterative
single-factor subtree grouping before operator queries, and an explicit
`Operator::group_local()` API with the same rules. This balances construction
cost with recovery of locality hidden by scales/adjoints. Eager whole-tree
normalization at every construction step would repeatedly traverse shared syntax.
Global distribution or gathering same-factor operands across other factors would
add work and complicate ordered products and graded crossing signs. Grouping is
structural; existing traversal remains responsible for signs. See the current
[grouping contract](../design/composite-expression.md#113-structural-local-operator-grouping).

Collect matching natural-state contributions inside `evaluate()` after each
action and on action-free inputs. A product applies its right operand, collects,
and then applies its left operand. Delaying collection until basis expansion
retains exponentially many action paths to a small state. A separate evaluator
that preserves those duplicates would not fulfill this evaluation contract.
Accept primitive PartialEq and the coefficient capabilities required by
collection, propagating them to consumers. Do not introduce operator Product as
an action requirement. See [capabilities and matching rules](../design/composite-expression.md#114-natural-state-collection-between-actions).

Match normalized ordered structure, including tensor parentheses and coefficient
placement, while preserving factored products of sums and first-occurrence order.
General distributive equivalence would destroy the locality the optimization
needs. Use query-local interning/memoization and normalized task results rather
than persistent collection flags or general caches. Node sharing can reuse work
but cannot override nonreflexive PartialEq. Primitive equality does not establish
linear dependence or orthogonality; basis expansion remains a separate operation.

Contract independent local groups incrementally and stop only the current
product when an exact zero is established. Keep all other sum branches. This
requires no orthogonality metadata or basis selection and also applies to
ordinary trace and normalized operator HS contractions. Graded HS retains
ordinary parity-string lowering; intrinsic operator coordinates remain distinct.
Approximate pruning and a basis-aware inner-product API were not adopted. See
[contraction semantics](../design/composite-expression.md#115-incremental-scalar-contractions)
and [numeric conventions](../design/numerics.md).

Keep sparse specified factors internally and expand spectator identity lazily
once per operator-basis query. For `I = gamma B_q`, use q as the working key's
default digit and multiply gamma at each missing factor in ambient order.
Identity is not universally digit zero with coefficient one. Division or a
precomputed gamma power could introduce additional requirements or numerical
failure. Multi-component identities share local choices, then enumerate genuine
output combinations. Emit complete keys directly, and create expanded syntax
only when a key first enters the accumulator. Preserve public full-length keys
and validation of every reached local expansion before numeric pruning. This
removes per-term spectator expansion and prefix maps without a public format
change or a primitive-result cache. See [operator coordinate generation](../design/basis.md#operator-coordinate-generation).

## Reference workloads and method

Measurements used aarch64-apple-darwin, Rust 1.99.1 / LLVM 23.1.1, default Cargo
release optimization, Complex<f64>, num-complex 0.4.6, and num-traits 0.2.19.
Each case warmed up once, calibrated a power-of-two batch toward 10 ms with a
65,536-iteration cap, and reported the median of seven per-call batch averages.
Output destruction was included. Tests/builds and measurements ran separately;
this does not assert isolation from all host activity. Nanosecond-scale controls
that hit the cap are rough measurements. Allocation requests and cumulative
requested bytes came from a separate single operation, including reallocations
and destruction-time allocations; these are not peak resident memory.

The reference H is the open-chain spin-half Heisenberg model at coupling 1
provided by the model API; the input is its alternating Neel product state.
Operator coordinates use CartesianBasis and state coordinates use SzBasis.
The placed Hadamard layer is a product of separately placed `(X_i + Z_i)` sums;
the measured query expands `2^-n L†L`, whose only coordinate is identity.
Rotations apply `R = cos(0.1) I - i sin(0.1) Y` repeatedly to Up. W is the normalized
sum of n product states with one Down among Up states. Plus18 is the tensor
product of 18 normalized `(Up + Down)` local sums. Input construction is excluded
except in the explicit construction and per-step controls.

The columns below identify successive algorithm combinations, followed by an
identical-source final rerun. They are dated reference observations, not universal
thresholds, confidence intervals, or a promise of improvement for all expressions.

| Case (median ms) | Baseline | Grouping | + Collection | + Stopping | + Coordinates | Final rerun | Overall speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Placed Hadamard n=8 | 251.076000 | 0.011048 | 0.010972 | 0.010947 | 0.009565 | 0.010840 | 23,162.5x |
| Rotation, 16 actions | 96.364458 | 107.029583 | 0.005378 | 0.005290 | 0.005474 | 0.005496 | 17,532.5x |
| Six-site H^4 on Neel | 55.237084 | 56.680000 | 0.895786 | 0.705893 | 0.695060 | 0.706271 | 78.2x |
| W64 norm squared | 99.744125 | 98.647625 | 60.936875 | 33.499375 | 33.966250 | 32.996125 | 3.0x |
| H32 HS self | 18.386458 | 18.418167 | 18.761333 | 7.150146 | 6.944312 | 7.282000 | 2.5x |
| H64 coordinates | 3.930281 | 3.979948 | 4.291927 | 3.957479 | 0.210430 | 0.208357 | 18.9x |
| H^2 64 coordinates | 752.744500 | 757.960917 | 774.564959 | 754.000542 | 36.955084 | 37.448167 | 20.1x |

| Allocation control | Baseline requests / bytes | Final requests / bytes |
| --- | ---: | ---: |
| Placed Hadamard n=8 | 10,555,044 / 1,094,632,312 | 444 / 24,920 |
| Rotation, 16 actions | 3,866,499 / 199,228,480 | 240 / 20,984 |
| Six-site H^4 on Neel | 1,788,119 / 108,566,400 | 23,697 / 2,167,128 |
| W64 norm squared | 2,438,984 / 133,681,016 | 1,309,691 / 78,216,064 |
| H32 HS self | 1,178,622 / 43,147,640 | 434,691 / 17,365,424 |
| H64 coordinates | 134,643 / 26,369,504 | 4,119 / 815,800 |
| H^2 64 coordinates | 25,135,284 / 4,896,265,824 | 798,111 / 90,049,744 |

| End-to-end regression control (median ms) | Baseline | Final | Interpretation |
| --- | ---: | ---: | --- |
| One-site action on Neel64 | 0.002105 | 0.023469 | Collection setup dominates a single action |
| Factored plus18 norm | 0.006088 | 0.019744 | Remains factored; preparation costs more |
| Plus18 coordinates | 152.033958 | 154.680459 | 262,144 genuine output entries |
| Construction, 128 placed sums | 0.045053 | 0.037246 | Cheap direct-child grouping |

The construction control builds and binds 128 placed sums and destroys the
result. The single-action control acts at one site of a 64-factor Neel state;
there is no contribution explosion to amortize collection setup. Its roughly
11x regression and the roughly 3.2x factored-norm regression are retained costs
of per-query collection. Action-free duplicate-sum evaluation now collects
terms, whereas the former evaluator returned the input unchanged; their timings
measure different responsibilities. Plus18 coordinates have 262,144 genuine
entries and remain output-bound.

### Work avoided and remaining costs

For W64, primitive inner-product calls fell from 262,144 to 91,456. For the
measured W representation and order, the final count is
`(n^3 + 3n^2 - n)/3`: all n² pairs are considered, diagonal pairs visit n factors,
and off-diagonal pairs visit min(i,j)+1. This remains cubic. End-to-end timing
also includes operand collection and eagerly prepared tensor-group metadata.

H64 coordinates reduced local basis calls from 12,096 to 379, including spectator
identity calls from 11,718 to one. H-squared64 reduced total basis calls from
2,286,144 to 140,635 and identity calls from 2,146,260 to 751. Of the latter, 750
come from explicit local products and one from spectators; all 17,395 output
keys remain. Sharing spectators does not recognize or cache every explicit
identity. Multi-component identity expansion and intrinsic term products can
still generate genuinely large outputs.

In 32-factor product controls, zero at positions 0/16/31 reduced local state
contractions to 1/17/32; corresponding HS contractions used 2/18/33 primitive
traces, including the dimension trace. No-zero controls still visited all factors.
The final 102-case rerun reproduced all work-count/output-size diagnostics of
the coordinate implementation. Small allocation variations occurred in 22 cases,
at most 20 requests and 9,488 cumulative bytes per query. Address-keyed memo
BTreeMap layout is a plausible, unisolated explanation; no source optimization
is attributed to the rerun. Further acceleration is tracked in
[future work](../future-work.md#3-outstanding-verification-and-performance).

## Evidence and provenance

The accepted implementation was checked locally with Rust 1.85.0 and stable
1.99.1 using default/all/no-default workspace tests, all-feature builds,
all-target no-default checks, Lie workflows, and downstream system-OpenBLAS
linkage/provider isolation. Stable additionally covered separate Vec/ndarray/sprs
backend tests, formatting, Clippy, and rustdoc with warnings denied. These are
local host results, not GitHub-hosted Linux or dedicated no_std target results.
Current commands and semantic/call-count regression ownership remain in
[verification](../design/verification.md). Baseline probe fixtures are covered
by retained state-collection, operator-expansion, and combined pipeline tests;
the temporary probe is not a maintained benchmark package.

The pre-optimization production revision was
`a0700711b4320f60e745c0a3dc96a00e286252f8`. The final measured implementation and
all six raw records, source/probe hashes, seven-sample measurements, dependency
snapshot, and temporary harness are preserved in Git revision
`aab754d8617246b0f7d5867aa87468f80d4cf821`. The initial five-single-call investigation
used a different method and is not the baseline in these tables.

For detailed auditing, read the archived file directly from that revision:

```sh
git show aab754d8617246b0f7d5867aa87468f80d4cf821:docs/plans/core-evaluation-performance/stage-5-results.txt
```

Sibling records are `stage-0-baseline.txt`, `stage-1-results.txt` through
`stage-4-results.txt`, and `stage-0-probe.lock`; the archived `probe/` contains
workload definitions and reproduction instructions are in `stage-0-baseline.md`.
The final rerun uses the same production/probe sources as the coordinate result.
The completed plan, raw working logs, and generated probe artifacts are removed
from the current tree rather than maintained as duplicate specifications.

## Follow-up performance baseline

The follow-up keeps reusable factor positions on each immutable space and
reduces intermediate work in state action, scalar contractions, and basis
conversion. Operator evaluation uses factor overlap and graded exchange signs
to decide which blocks to combine. The current behavior is described in the
[Hilbert space](../design/hilbert-space.md), [composite expression](../design/composite-expression.md),
and [basis](../design/basis.md) designs.

The Stage 0 and final release sweeps on macOS with Rust 1.99.1 share 322 timing
and allocation cases. Median thread CPU time per case decreased in 309 cases
and increased in 13, with a median per-case speedup of 2.04×. The sweeps
include result destruction and use `Complex<f64>`.

| Workload | Stage 0 CPU time | Final CPU time |
| --- | ---: | ---: |
| `tensor_all()` on 1,024 factors | 163.42 µs | 8.49 µs |
| Bind the 1,024-site Heisenberg operator | 2.372 ms | 73.15 µs |
| Evaluate eight Heisenberg actions at ten sites | 42.832 ms | 18.528 ms |
| Evaluate the 64-site Hubbard vacuum | 10.641 ms | 2.878 ms |
| Squared norm of the 64-site W state | 34.587 ms | 3.570 ms |
| Cartesian operator coordinates of a 16-site XX echo | 178.998 ms | 19.73 µs |

Small Heisenberg queries still pay preparation costs. The saved sweep for
four-site Cartesian coordinates increased from 6.23 to 7.31 µs. Retaining only
the final root of successive `tensor()` construction on 1,024 factors held
8.54 MB of additional heap data. Successive flattening paths held 32.9 KB
when intermediate roots were released. Retaining earlier roots also retains
their position tables.

The full sweeps compare production revisions `49e9fc5` and `d1d33cc`.
A separate review check compared `20ea94f` and the common-block extraction
change in `d18d199`. For a sum with 17 shared two-term factors, the five-run
release medians decreased from 242 ms to 42.7 µs for `collect_terms_in()`
and from 190 ms to 19.2 µs for `trace_in()`. Extracting those factors avoids
enumerating both addends' Cartesian products before collection or trace.

The completed plan, probe, and recorded sweeps are preserved in Git revision
`63330701b472e5aff54236c0ec323506fcea7418`. The initial-to-final comparison
contains the matched cases and the final follow-up report covers retained
memory, rechecks, and local verification.

```sh
git show 63330701b472e5aff54236c0ec323506fcea7418:docs/plans/core-performance-baseline/measurements/initial-to-final/README.md
git show 63330701b472e5aff54236c0ec323506fcea7418:docs/plans/core-performance-baseline/measurements/follow-up/final/README.md
```
