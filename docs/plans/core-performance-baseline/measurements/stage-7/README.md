# Stage 7 measurements

Basis queries aggregate indexed coefficients before rebuilding expressions.
Composite queries reuse primitive conversions, operator coordinates derive
identity spectators from gaps, and state tensors enumerate complete digit
tuples. Expanded states and operators share local expression nodes for equal
factor positions and basis indices. [Implementation plan](../../README.md#7-basis-coordinates-and-expanded-syntax)

## Time per query

Thread CPU medians are in milliseconds per call on prebuilt expressions.
The measurements include dropping the result.

| Workload and API | Before | Indexed conversion | Cached conversion | Operator gaps | State tensors | Placed-node sharing |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Heisenberg `H`, Cartesian coordinates | 0.235 | 0.233 | 0.194 | 0.174 | 0.171 | 0.175 |
| Heisenberg `H`, Cartesian expansion | 0.611 | 0.610 | 0.565 | 0.567 | 0.550 | 0.302 |
| Heisenberg `H²`, Cartesian coordinates | 21.798 | 20.689 | 14.474 | 12.110 | 12.066 | 12.192 |
| Heisenberg `H²`, Cartesian expansion | 65.657 | 57.956 | 63.294 | 52.754 | 50.285 | 23.457 |
| BLBQ Néel, state `Sz` coordinates | 7.146 | 7.193 | 7.326 | 7.294 | 3.840 | 3.959 |
| BLBQ, Cartesian coordinates | 1.123 | 1.106 | 1.002 | 0.936 | 0.939 | 0.968 |
| BLBQ, ladder coordinates | 1.420 | 1.390 | 1.239 | 1.171 | 1.138 | 1.181 |
| BLBQ, Cartesian expansion | 2.145 | 2.102 | 2.051 | 2.050 | 1.961 | 1.306 |
| Hubbard, ladder coordinates | 0.494 | 0.490 | 0.432 | 0.330 | 0.370 | 0.351 |
| Hubbard, Majorana coordinates | 0.898 | 0.898 | 0.814 | 0.614 | 0.666 | 0.661 |
| Hubbard, Majorana expansion | 2.741 | 2.665 | 2.549 | 2.496 | 2.576 | 1.170 |
| GHZ14, `Sz` coordinates | 0.0153 | 0.0157 | 0.0155 | 0.0154 | 0.0132 | 0.0137 |
| GHZ14, `Sz` expansion | 0.0187 | 0.0172 | 0.0169 | 0.0168 | 0.0146 | 0.0162 |
| Plus14, `Sz` coordinates | 5.576 | 5.269 | 5.268 | 5.300 | 2.606 | 2.710 |
| Plus14, `Sz` expansion | 15.717 | 16.357 | 16.297 | 16.189 | 14.288 | 7.396 |
| Plus18, `Sz` coordinates | 149.558 | 139.755 | 139.886 | 139.615 | 61.015 | 60.463 |

Heisenberg and BLBQ use 64 sites. Hubbard uses 64 sites with 128 graded modes.
Direct indexed conversion removes reverse lookups during coordinate collection.
It reduces `H²` coordinate time by 5% while keeping the same number of basis
rule calls. Reusing conversions reduces those calls from 68,827 to 4 and
coordinate time from 20.69 to 14.47 ms. The corresponding `expand()` sample
increases from 57.96 to 63.29 ms. Allocation requests fall from 1,712,570 to
1,419,885 in the same run. The seven cached samples range from 54.49 to
69.96 ms. The cause of the time increase remains unverified.

Using spectator gaps reduces `H²` coordinate time from 14.47 to 12.11 ms with
the same term collector, basis rule calls, and allocation count. Direct tensor
enumeration reduces Plus18 coordinate time from 139.62 to 61.02 ms by removing
proper-prefix maps. Its 262,144 final coordinates still occupy full-length keys.

Placed-node sharing reduces `H²` expansion time from 50.29 to 23.46 ms and
Plus14 expansion time from 14.29 to 7.40 ms. GHZ14 has two disjoint
coordinates and provides no repeated placed nodes. For GHZ14 the caches add
15 allocation requests and about 1.6 µs to the preceding checkpoint. The final
GHZ14 expansion is still about 14% faster than at stage entry. As a cumulative
reference, the Stage 2 run after common operand storage measured 82.41 ms for `H²`
expansion. The Stage 7 checkpoints isolate placement sharing with the same
ordered product representation. [Stage 2 timing data](../stage-2/operand-storage/timings.csv)

## Work and allocations

| Workload and counter | Before | Placed-node sharing |
| --- | ---: | ---: |
| Heisenberg `H` coordinates, basis rule calls | 379 | 4 |
| Heisenberg `H²` coordinates, basis rule calls | 68,827 | 4 |
| Heisenberg `H` coordinates, factor validation visits | 12,096 | 378 |
| Heisenberg `H²` coordinates, factor validation visits | 1,113,280 | 68,826 |
| GHZ14 coordinates, proper-prefix map entries | 26 | 0 |
| Plus14 coordinates, proper-prefix map entries | 16,382 | 0 |
| Plus18 coordinates, proper-prefix map entries | 262,142 | 0 |
| Heisenberg `H` expansion, placed nodes created | 12,096 | 256 |
| Heisenberg `H²` expansion, placed nodes created | 1,113,280 | 256 |
| GHZ14 expansion, placed nodes created | 28 | 28 |
| Plus14 expansion, placed nodes created | 229,376 | 28 |

| Workload and API | Requests before | Requests after | Requested bytes before | Requested bytes after |
| --- | ---: | ---: | ---: | ---: |
| Heisenberg `H²`, Cartesian coordinates | 461,693 | 167,424 | 55,975,448 | 25,770,512 |
| Heisenberg `H²`, Cartesian expansion | 1,850,205 | 306,926 | 159,438,216 | 45,525,408 |
| Hubbard, Majorana expansion | 79,221 | 14,467 | 7,508,304 | 3,249,816 |
| GHZ14, `Sz` expansion | 635 | 500 | 114,720 | 76,912 |
| Plus14, `Sz` expansion | 382,644 | 117,788 | 39,438,768 | 13,909,736 |
| Plus18, `Sz` coordinates | 1,160,142 | 306,203 | 271,960,160 | 58,322,512 |

## Reproduction and verification

The reference source matches the Stage 6 commit
`279f88473ed4934e73940ff7c5d1a6c08ed135f7` for the probe and its production
dependencies. All checkpoints use the Stage 0 host, Apple M4 and macOS 15.8.1,
Rust 1.99.1, the Cargo release profile, and `Complex<f64>` coefficients. Run
the probe in `time`, `allocations`, and `diagnostics` modes using the filters
`,heisenberg,64,64,`, `,blbq_neel,64,64,1,`, `,hubbard,64,128,1,`,
`,ghz,14,14,`, `,plus,14,14,`, and `,plus,18,18,`. Retain the state and
operator coordinate and expansion rows. Each checkpoint records 16 timing
and allocation cases and 13 diagnostic rows. The timing files contain all
seven samples and the thread CPU and elapsed medians. Coordinate counts
agree at all checkpoints. [Probe commands](../../README.md#reproduction)

Separately instrumented copies count validation visits, proper-prefix entries,
and newly constructed placed nodes. The factor validation counter increments
on every ambient position in the earlier scan and on each specified factor
in the gap traversal. It omits the gap boundaries and the subsequent choice
enumeration. The prefix counter counts entries in the intermediate tensor maps
after adding each child except the last. The placement counter counts new
`StateNode::Local` and `OpNode::Local` nodes during reconstruction.

- Stage entry. [Timings](before/timings.csv) · [Allocations](before/allocations.csv) · [Diagnostics](before/diagnostics.csv) · [Work counts](before/work.csv)
- Direct indexed conversion. [Timings](indexed/timings.csv) · [Allocations](indexed/allocations.csv) · [Diagnostics](indexed/diagnostics.csv)
- Query-local reuse. [Timings](cached/timings.csv) · [Allocations](cached/allocations.csv) · [Diagnostics](cached/diagnostics.csv)
- Operator gaps. [Timings](operator-gaps/timings.csv) · [Allocations](operator-gaps/allocations.csv) · [Diagnostics](operator-gaps/diagnostics.csv)
- State tensor blocks. [Timings](state-tensors/timings.csv) · [Allocations](state-tensors/allocations.csv) · [Diagnostics](state-tensors/diagnostics.csv)
- Placed-node sharing. [Timings](placed-sharing/timings.csv) · [Allocations](placed-sharing/allocations.csv) · [Diagnostics](placed-sharing/diagnostics.csv) · [Work counts](placed-sharing/work.csv)

Custom bases now implement `expand_indices()` and `element()` alongside
`index()`. The default primitive `Basis::expand()` returns `Result` and
resolves the indices into elements. Expression coordinates validate the
returned indices against the local dimension. Expression expansion checks
the elements for the indices that survive aggregation. Tests cover these
errors, correlated tensor blocks, identity with a unit or nonunit coefficient
or several components, shared placed nodes, and coordinate round trips.
Workspace tests and doctests passed with default, all, and no default
features. All-feature Clippy with warnings denied, the all-targets check
without default features, probe Clippy, and formatting checks also passed.
