# Stage 1 measurements

Tensor construction uses an ordered set for duplicate checks. Delayed action
reuses a binding to the selected space's shared root. A query-local index
serves membership validation, graded ordering, and operator basis placement.
The index uses `core::cell::OnceCell`. The build uses the existing production
dependencies. [Implementation plan](../../README.md#1-space-construction-and-binding)

## Construction and binding

The table reports thread CPU and elapsed medians in microseconds per call.
Each intermediate checkpoint adds one change to the previous checkpoint.

| Heisenberg at 1,024 sites | Stage 0 CPU / elapsed | Ordered set CPU / elapsed | Binding reuse CPU / elapsed | Query-local index CPU / elapsed |
| --- | ---: | ---: | ---: | ---: |
| `tensor_all` | 163.424 / 163.422 | 41.761 / 41.761 | 41.570 / 41.569 | 44.839 / 44.839 |
| `build_model` | 3,109.469 / 3,109.427 | 3,016.365 / 3,016.323 | 2,961.688 / 2,961.656 | 768.979 / 769.083 |
| `bind_fresh` | 2,381.297 / 2,380.938 | 2,392.052 / 2,392.000 | 2,362.146 / 2,362.125 | 136.312 / 136.288 |
| `bind` with reused space | 2,372.016 / 2,371.995 | 2,400.938 / 2,400.917 | 2,365.052 / 2,365.026 | 135.856 / 135.855 |
| `act` with shared bound space | 2,374.750 / 2,374.729 | 2,393.219 / 2,393.198 | 0.042 / 0.042 | 0.044 / 0.044 |
| `act_in` with equivalent space | 2,384.432 / 2,384.406 | 2,410.526 / 2,410.505 | 2,371.682 / 2,371.651 | 136.592 / 136.591 |

| Change at 1,024 sites | Equality / ordering comparisons, before → after | Allocation requests, before → after | Requested bytes, before → after |
| --- | ---: | ---: | ---: |
| Ordered set in `tensor_all` | 523,776 / 0 → 0 / 22,782 | 1,045 → 1,207 | 57,360 → 61,192 |
| Binding reuse in `act` | 3,145,725 / 0 → 0 / 0 | 12,281 → 2 | 50,405,352 → 88 |
| Query-local index in `bind` | 3,145,725 / 0 → 1,024 / 68,530 | 12,279 → 17 | 50,405,264 → 114,608 |
| Query-local index in `act_in` | 3,146,749 / 0 → 2,048 / 68,530 | 12,285 → 23 | 50,421,752 → 131,096 |

The temporary ordered set saved 74% of `tensor_all` CPU time at 1,024 sites
in its isolated checkpoint. At 32 sites the final run took 1.037 µs, up from
0.984 µs, with one more allocation request. `act` measures construction and
destruction of delayed syntax. Its fast path checks state compatibility and
uses the validated binding. The explicit-space control still validates on
the separately constructed root, then compares ordered factors with the
state's space.

The 1,024-site product-state construction control rose from 78.334 µs to
83.673 µs and took 79.916 µs on a focused rerun. The 16-site action-free
evaluation control rose from 14.593 ns to 17.932 ns and took 18.119 ns on
rerun. Both controls retained their original allocation counts. The cause
of the small timing shift in action-free evaluation is unverified.
[Focused rerun](recheck-timings.csv)

For membership validation, a single local reference uses a scan. Building
the index on the first reference took 3.193 µs in a 1,024-site single-`X`
trial. With the scan, the final run took 0.712 µs versus 0.693 µs in Stage 0,
and both requested 8,296 bytes in five allocations. In the measured Heisenberg
`bind`, the second reference initializes one sorted position table. Fresh and
reused space handles each initialize a table per query. A unit test checks lazy
initialization, reuse within the query, lookup by equal IDs, and preservation
of tensor order for unsorted IDs in a nested space.

## Operator basis controls

| Workload | Stage 0 CPU / elapsed | Stage 1 CPU / elapsed | Requests, Stage 0 → 1 | Requested bytes, Stage 0 → 1 |
| --- | ---: | ---: | ---: | ---: |
| Heisenberg 64, `H²` coordinates (ms) | 38.031 / 38.031 | 35.969 / 35.968 | 798,113 → 795,088 | 90,051,008 → 89,261,200 |
| Heisenberg 64, `H²` expansion (ms) | 109.772 / 109.783 | 105.782 / 105.781 | 3,393,210 → 3,390,185 | 261,142,864 → 260,353,056 |
| Graded 128-mode hopping, far coordinate (µs) | 2.436 / 2.436 | 2.078 / 2.078 | 68 → 57 | 14,960 → 10,784 |

The `H²` query returned 17,395 coordinates and made 140,635 basis expansion
calls at both checkpoints. Far hopping returned one coordinate. These controls
exercise factor lookup and graded placement while keeping local basis work
fixed.

## Reproduction and verification

The checkpoints were measured on November 20, 2026 from
`49b7fbc86353c26cdc8d61d4af943a844b9ec386` plus the corresponding Stage 1
changes. They use the Stage 0 host, Rust 1.99.1, and release probe. The first
two checkpoints used the `heisenberg_construction` filter for all three modes.
The final checkpoint ran all diagnostics. Timing and allocation filters were
`heisenberg_construction`, `single_x_on_neel`, `heisenberg,64,64,2`,
`hop_far,128,128,1`, and `operator_product_construction`. The latter preserves
controls for Stage 2. [Stage 0 comparison point](../stage-0/README.md) · [Run commands](../../README.md#reproduction)

- Ordered set checkpoint. [Timing](duplicate-check/timings.csv) · [Allocations](duplicate-check/allocations.csv) · [Diagnostics](duplicate-check/diagnostics.csv)
- Binding reuse checkpoint. [Timing](binding-reuse/timings.csv) · [Allocations](binding-reuse/allocations.csv) · [Diagnostics](binding-reuse/diagnostics.csv)
- Query-local index checkpoint. [Timing](factor-positions/timings.csv) · [Allocations](factor-positions/allocations.csv) · [Diagnostics](factor-positions/diagnostics.csv)

All 246 diagnostic rows matched the Stage 0 case and metric keys. Coordinate,
basis, and term-count metrics matched their Stage 0 values. The affected
space and expression tests cover mixed-kind error precedence, compatible
distinct roots, explicit-space overrides, and missing factor IDs.
Workspace tests and doctests with all features passed, along with the
all-targets check without default features, all-feature Clippy with warnings
denied, and formatting.
