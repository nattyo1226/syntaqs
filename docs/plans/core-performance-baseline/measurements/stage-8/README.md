# Stage 8 end-to-end comparison

The final implementation lowers median thread CPU time in 296 of the 310
original cases. The median of the per-case ratios is 0.622, a 37.8% reduction.
Bound action construction, operator contractions, and the factored XX echo
show the largest gains.
Small operator-coordinate queries expose a remaining setup cost. The final
run covers all original cases and 12 controls added in Stage 0.
[Implementation plan](../../README.md#8-run-the-full-comparison)

## Results by workload

Ratios compare Stage 8 with Stage 0. Each case has equal weight. The median
describes the cases in this probe and does not weight them by query cost or
frequency.

| Original cases | Cases | Median CPU ratio | Median elapsed ratio | Median allocation request ratio | Median requested byte ratio |
| --- | ---: | ---: | ---: | ---: | ---: |
| Spin-1/2 | 191 | 0.641 | 0.641 | 0.682 | 0.768 |
| Spin-1 | 35 | 0.637 | 0.637 | 0.684 | 0.742 |
| Fermion | 84 | 0.592 | 0.592 | 0.608 | 0.653 |
| All three | 310 | 0.622 | 0.622 | 0.660 | 0.750 |

The allocation medians use cases with nonzero Stage 0 values. Allocation
requests fell in 289 original cases and rose in 11. Requested bytes fell
in 277 and rose in 25. All 12 Stage 0 controls improved in both time and
allocation counts. The per-case comparison contains absolute differences
and ratios, with blank allocation ratios for a zero baseline.
[All 322 paired cases](comparison.csv)

The tables report median CPU and elapsed time per call. Allocation requests
and requested bytes include allocation and reallocation traffic for one
measured call.

| Construction and placement | CPU, Stage 0 → 8 | Elapsed, Stage 0 → 8 | Requests, Stage 0 → 8 | Requested bytes, Stage 0 → 8 |
| --- | ---: | ---: | ---: | ---: |
| Heisenberg 1,024 `tensor_all` | 163.42 → 41.83 µs | 163.42 → 41.83 µs | 1,045 → 1,206 | 57,360 → 61,160 |
| Heisenberg 1,024 `build_model` | 3,109.47 → 765.57 µs | 3,109.43 → 765.57 µs | 39,949 → 27,836 | 51,919,616 → 1,518,088 |
| Heisenberg 1,024 `bind` | 2,372.02 → 129.91 µs | 2,371.99 → 129.91 µs | 12,279 → 17 | 50,405,264 → 114,608 |
| Heisenberg 1,024 `bind_fresh` | 2,381.30 → 130.02 µs | 2,380.94 → 130.02 µs | 12,279 → 17 | 50,405,264 → 114,608 |
| Heisenberg 1,024 bound `act` | 2,374.75 µs → 44.3 ns | 2,374.73 µs → 44.3 ns | 12,281 → 2 | 50,405,352 → 88 |
| Heisenberg 1,024 `act_in` on a distinct equivalent root | 2,384.43 → 131.15 µs | 2,384.41 → 131.14 µs | 12,285 → 23 | 50,421,752 → 131,096 |
| 1,024 placed operands, sequence product | 24.20 → 7.47 µs | 24.20 → 7.47 µs | 1,026 → 3 | 73,760 → 32,816 |
| 1,024 placed operands, binary `*` | 66.9 → 56.8 ns | 66.9 → 56.8 ns | 3 → 2 | 128 → 64 |

`act()` measures delayed syntax construction and destruction. At 1,024 sites
it makes zero factor-ID comparisons on the shared bound space, down from
3,145,725 equality comparisons. `bind()` on both a fresh and reused root
makes 1,024 equality comparisons and 68,530 ordered comparisons. Explicit
`act_in()` on the distinct equivalent root also validates placements. The
per-query index still accounts for work in repeated bindings.
[Stage 1 results](../stage-1/README.md)

| Evaluation and contractions | CPU (ms), Stage 0 → 8 | Elapsed (ms), Stage 0 → 8 | Requests, Stage 0 → 8 | Requested bytes, Stage 0 → 8 |
| --- | ---: | ---: | ---: | ---: |
| Last-site `X` on 1,024 factors, `evaluate` | 0.783 → 0.429 | 0.783 → 0.429 | 9,828 → 8,781 | 1,781,704 → 1,722,464 |
| Directed hop across 128 modes, `evaluate` | 0.114 → 0.069 | 0.114 → 0.069 | 3,101 → 2,310 | 594,664 → 312,536 |
| Heisenberg 10, eight actions, `evaluate` | 42.832 → 28.638 | 42.832 → 28.637 | 1,024,034 → 670,247 | 119,529,528 → 90,597,496 |
| Heisenberg 64 on Néel, `expectation` | 1.565 → 0.607 | 1.565 → 0.607 | 35,034 → 12,030 | 5,581,424 → 3,666,984 |
| BLBQ 64 on Néel, `evaluate` | 3.371 → 1.844 | 3.371 → 1.844 | 43,222 → 29,543 | 15,089,136 → 12,363,528 |
| BLBQ 64 on Néel, `expectation` | 6.211 → 2.468 | 6.211 → 2.468 | 144,631 → 51,700 | 21,101,976 → 13,628,856 |
| Hubbard 64 on alternating occupations, `evaluate` | 16.464 → 5.355 | 16.464 → 5.354 | 395,126 → 225,253 | 99,432,216 → 23,300,680 |
| Hubbard 64 on alternating occupations, `expectation` | 19.327 → 5.843 | 19.327 → 5.843 | 488,047 → 242,444 | 105,728,904 → 24,541,680 |
| W64, squared norm | 34.587 → 3.586 | 34.587 → 3.586 | 1,309,676 → 126,523 | 78,210,024 → 5,969,784 |
| Heisenberg 64, HS self product | 55.759 → 0.261 | 55.758 → 0.261 | 3,305,658 → 4,307 | 131,871,072 → 486,064 |
| Hubbard 64, trace | 5.753 → 0.255 | 5.753 → 0.255 | 219,555 → 7,157 | 13,269,256 → 817,384 |

On-demand alignment reduces per-pair work for the W64 norm. Conjugate
symmetry reduces the pair count from 4,096 to 2,080 including the diagonal.
Direct HS contraction visits 17,955 pairs for the 189-term Heisenberg sum. Local parity rejects
252 Hubbard hopping terms in `trace()` and eliminates explicit parity
strings for this query. State evaluation of a far hop still visits more
placements than a near hop, reflecting the longer parity prefix.
[State contractions](../stage-4/README.md) · [Operator contractions](../stage-6/README.md)

| Basis queries | CPU (ms), Stage 0 → 8 | Elapsed (ms), Stage 0 → 8 | Requests, Stage 0 → 8 | Requested bytes, Stage 0 → 8 |
| --- | ---: | ---: | ---: | ---: |
| 16-site XX echo, Cartesian coordinates | 178.998 → 0.017 | 179.378 → 0.017 | 6,983,451 → 710 | 582,376,640 → 71,648 |
| Heisenberg 64 `H²`, Cartesian coordinates | 38.031 → 11.886 | 38.031 → 11.886 | 798,113 → 167,422 | 90,051,008 → 25,769,152 |
| Heisenberg 64 `H²`, Cartesian expansion | 109.772 → 23.668 | 109.783 → 23.668 | 3,393,210 → 306,924 | 261,142,864 → 45,524,048 |
| BLBQ 64 on Néel, state coordinates | 8.684 → 3.810 | 8.684 → 3.810 | 165,689 → 92,213 | 52,614,448 → 24,200,936 |
| BLBQ 64, Cartesian expansion | 3.635 → 1.278 | 3.635 → 1.278 | 113,482 → 37,588 | 9,852,040 → 3,759,584 |
| Hubbard 64, Majorana expansion | 4.275 → 1.131 | 4.275 → 1.131 | 138,586 → 14,466 | 12,284,784 → 3,249,184 |
| Hubbard 64 on alternating occupations, state coordinates | 20.793 → 7.609 | 20.793 → 7.609 | 524,325 → 306,559 | 145,030,168 → 38,620,968 |
| Plus18, state coordinates | 154.725 → 59.078 | 158.345 → 59.078 | 1,160,147 → 306,203 | 271,960,064 → 58,322,512 |

In the XX echo, the operator evaluator combines overlapping gate and
adjoint blocks before basis conversion. Both revisions yield one identity
coordinate. The 65,536 syntactic products are still present in the probe's
input count. Stage 5 instrumentation counted 32 block-local products in
evaluation. The 64-site `H²` query retains all 17,395 coordinates and reduces
basis rule calls from 140,635 to 4. Plus18 still produces 262,144 entries.
[Operator evaluation](../stage-5/README.md) · [Basis generation](../stage-7/README.md)

## Rechecked regressions and allocation increases

The Cartesian coordinates of the first-order Heisenberg Hamiltonian take
longer at 4, 8, and 16 sites in both focused rechecks. Stage 5 introduced
the increase. Its block evaluator collects local and composite terms and
tracks factor support before converting to basis indices. The simple `H`
control already has one output coordinate per input bond component, so
it offers little opportunity to cancel terms before conversion. Stage 7
recovers part of the added cost. The 32-site query is close to its Stage 0
time and the 64-site query is faster.
[Block evaluator](../../../../../crates/syntaqs-core/src/evaluate/operator.rs) · [Stage 5 results](../stage-5/README.md)

| Cartesian coordinates | Stage 0 CPU (µs) | Stage 4 CPU (µs) | Stage 5 CPU (µs) | Stage 8 CPU (µs) | Stage 0 → 8 CPU, second recheck (µs) |
| --- | ---: | ---: | ---: | ---: | ---: |
| Heisenberg `H`, 4 sites | 6.793 | 6.196 | 12.045 | 7.520 | 6.276 → 7.495 |
| Heisenberg `H`, 8 sites | 14.804 | 11.569 | 21.322 | 16.435 | 15.169 → 16.694 |
| Heisenberg `H`, 16 sites | 31.812 | 25.105 | 45.071 | 35.254 | 32.081 → 34.828 |
| Heisenberg `H`, 32 sites | 79.371 | 57.076 | 103.967 | 79.231 | 79.316 → 77.841 |
| Heisenberg `H`, 64 sites | 216.391 | 143.403 | 239.096 | 174.753 | 215.561 → 172.741 |

Small Cartesian BLBQ queries take less time but request more bytes. At
eight sites the full run drops from 4,111 to 3,723 allocation requests and
from 107.54 to 95.61 µs, while cumulative requested bytes rise by 22%.
The growth first appears with the Stage 5 evaluator and persists in the
focused allocation runs. At 64 sites the final query requests 35% fewer
bytes than Stage 0.

| Cartesian coordinates | Stage 0 bytes | Stage 4 bytes | Stage 5 bytes | Stage 8 bytes |
| --- | ---: | ---: | ---: | ---: |
| Heisenberg `H`, 4 sites | 17,496 | 15,904 | 34,040 | 27,440 |
| BLBQ, 4 sites | 110,088 | 101,656 | 163,264 | 144,160 |
| BLBQ, 8 sites | 271,040 | 243,280 | 376,904 | 329,496 |
| BLBQ, 16 sites | 666,728 | 572,664 | 818,288 | 708,520 |
| BLBQ, 64 sites | 4,845,608 | 3,579,480 | 3,616,808 | 3,146,176 |

Separate allocation counters locate most of this traffic in block evaluation
after local grouping. The 4-site Heisenberg query requests 20,856 bytes in
that phase, or 76% of its final total. The 8-site BLBQ query requests
304,992 bytes, or about 93% of its final total. For each case these phase
counts are the same in Stages 5 and 8. The savings after Stage 5 are in
the surrounding query. [Stage 5 allocation phases](stage5-phase-allocations.csv) · [Stage 8 allocation phases](stage8-phase-allocations.csv)

Two smaller effects from Stage 1 also recur. `tensor_all()` at 32 sites
takes 1.028–1.061 µs in the final rechecks against 0.947–0.978 µs in the
reference rechecks. The ordered set raises requested bytes from 1,808 to
1,912, while the 1,024-site time improves by 3.9×. The allocation-free
16-site `evaluate_input` increases from 15.1 to 18.1–18.3 ns. Both
revisions inspect the direct tensor's primitive leaves. The source of the
nanosecond difference remains unverified. The 1,024-site `product_state`
increase in the full sweep did not recur in either focused recheck.
[Stage 1 controls](../stage-1/README.md#construction-and-binding)

The case-level CSV records the smaller changes as well. For example, the
two-site Hubbard trace requests 2,480 more bytes in the full sweep and
takes 4.55 µs versus 6.17 µs in Stage 0. Cumulative requested bytes measure
allocator traffic. Peak live memory was outside this measurement.
[All paired allocation and timing results](comparison.csv)

## Follow-up investigations

1. **Reduce setup cost for small operator-coordinate queries.** Use the
   4–16-site Heisenberg and Cartesian BLBQ cases to test lower-cost term
   and support storage. Retain the 64-site and XX echo controls to check
   that block-local cancellation remains effective. Measure time, requests,
   and bytes together.
2. **Reduce work in state action when many branches vanish or repeat.**
   Evaluation takes 3.18 ms and requests 15.03 MB on the 64-site Hubbard
   vacuum, whose result is zero. Eight Heisenberg actions at ten sites take
   28.64 ms and request 90.60 MB for a result with 222 coordinates. Measure
   how many branches vanish and how much work precedes each rejection, and
   compare state-shape reuse across successive applications. Use the near
   and far hop controls to preserve the parity-prefix behavior.
3. **Assess reuse of the factor-position table across calls.** Fresh and
   reused `bind()` on a 1,024-site root still build a table for every call,
   at about 130 µs and 114,608 requested bytes. A shared table needs a
   retained-memory budget and compatibility with `no_std`, thread sharing,
   and the existing dependency set. Keep the single-reference path as a
   control.
4. **Measure compact or streamed coordinates for dense support.** Plus18
   takes 59.08 ms and requests 58.32 MB to return 262,144 full-length keys.
   Evaluate storage changes separately from the number of output entries.
   Preserve factor order, block correlations, and sparse index overflow
   behavior. Use GHZ as a two-coordinate control.

Primitive ordering and orthogonality-based pruning remain separate
experiments. [Follow-up design topics](../../../../future-work/evaluation-improvements.md) · [Primitive ordering experiment](../../README.md#measure-the-effect-of-ordering-primitive-values)

## Reproduction

Stage 8 uses production and probe revision
`775baa825235f443c07b3f57cdc97ade85f89518`. Stage 0 uses production revision
`49e9fc51270ad51885c0c5b507fcb903eee305fc` with the updated probe committed in
`49b7fbc86353c26cdc8d61d4af943a844b9ec386`. Both runs use an Apple M4 MacBook
Air with 24 GB of memory, macOS 15.8.1 (24H32), Rust 1.99.1, the default
Cargo release profile, and `Complex<f64>` coefficients. Run the recorded
reproduction commands with
`results="$plan/measurements/stage-8"` and no mode filter. The Stage 8 run used
`--offline` with the locked dependency cache. Input construction stays outside
the measured calls, and result destruction is included.
[Probe commands](../../README.md#reproduction) · [Stage 0 reference](../stage-0/README.md)

All 322 timing and allocation case IDs and all 246 diagnostic keys match
Stage 0, including all 310 original cases. All result assertions passed.
The 126 `nnz` counts and 13 `syntactic_term_products` counts agree with
Stage 0. The diagnostic comparison observes changes in index equality and
ordering comparisons and in all 11 basis rule call counts.
[Timing samples](timings.csv) · [Allocations](allocations.csv) · [Diagnostics](diagnostics.csv)

The focused comparison uses Stage 4 revision
`bbd01f532cf52a7679cbc2ddce6dfe5a46376474` and Stage 5 revision
`538ae88f685756ae87365e363a48b355780a8562` as intermediate checkpoints.
It selects 28 cases from first-order Heisenberg operator basis queries
at 4, 8, 16, 32, and 64 sites, BLBQ operator basis queries at 4, 8, 16,
and 64 sites, `tensor_all` at 32 sites, `product_state` at 1,024 sites,
and `evaluate_input` at 16, 64, 256, and 1,024 sites. The first pass runs
Stage 0, 4, 5, and 8 in succession. The second timing pass runs Stage 8
before Stage 0 to check run-order effects.

- Stage 0. [Timings](recheck-stage0-timings.csv) · [Allocations](recheck-stage0-allocations.csv) · [Second timing pass](recheck-stage0-timings-2.csv)
- Stage 4. [Timings](recheck-stage4-timings.csv) · [Allocations](recheck-stage4-allocations.csv)
- Stage 5. [Timings](recheck-stage5-timings.csv) · [Allocations](recheck-stage5-allocations.csv)
- Stage 8. [Timings](recheck-stage8-timings.csv) · [Allocations](recheck-stage8-allocations.csv) · [Second timing pass](recheck-stage8-timings-2.csv)

For the phase allocation files, an instrumented copy of each revision counts
requests during `EvaluatedOperator::new()` separately. Calls inside
`group_local()` have their own columns. `block_requests` and `block_bytes`
count the rest of evaluation before the constructor returns. The difference
between the query totals and the two phase counts covers the rest of
`coordinates()`. These runs cover the nine Cartesian-coordinate cases in the
focused sweep and use the same warmup and one measured call per row.

## Verification

The all-feature workspace build and tests with default, all, and no default
features passed on Rust 1.99.1. The all-target check without default features,
Clippy with warnings denied, rustdoc with warnings denied, and formatting
checks passed. The standalone probe also passed Clippy and formatting.
Downstream integration tests passed for Vec, ndarray, and sprs, as did the
Lie analysis tests, both Lie example configurations, native BLAS linkage,
and dependency-provider isolation checks.
The local run uses `aarch64-apple-darwin`. The Rust 1.85.0 and Linux CI matrix
is outside the scope of this host run. [Repository checks](../../../../design/verification.md)
