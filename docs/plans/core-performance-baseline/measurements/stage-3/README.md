# Stage 3 measurements

State evaluation uses factor IDs directly in the shape lookup, walks the
normalized input in tensor order for graded placements, and materializes the
reachable result syntax after evaluation. The three checkpoints add these
changes in that order. Composite state evaluation and the APIs that call it
require `I: Ord`. [Implementation plan](../../README.md#3-state-evaluation)

## Evaluation time

The table reports medians for thread CPU and elapsed time in microseconds per
call. Heisenberg runs on 10 sites and the hopping controls use 128 modes.

| Workload | Before CPU / elapsed | Factor keys CPU / elapsed | Ordered traversal CPU / elapsed | Deferred syntax CPU / elapsed |
| --- | ---: | ---: | ---: | ---: |
| Single `X`, 1,024 sites | 778.654 / 778.643 | 480.590 / 480.585 | 469.379 / 469.374 | 448.414 / 448.408 |
| Near hopping | 67.947 / 67.946 | 64.577 / 64.576 | 50.627 / 50.626 | 48.374 / 48.373 |
| Middle hopping | 93.691 / 93.690 | 88.792 / 88.791 | 63.848 / 63.867 | 59.757 / 59.756 |
| Far hopping | 117.061 / 117.060 | 104.978 / 104.977 | 76.729 / 76.729 | 70.696 / 70.695 |
| Heisenberg, one action | 47.322 / 47.322 | 49.658 / 49.657 | 46.008 / 46.008 | 36.025 / 36.024 |
| Heisenberg, eight actions | 41,902 / 41,902 | 42,839 / 42,839 | 39,092 / 39,092 | 28,920 / 28,920 |
| 1,024-site action-free input | 0.674 / 0.674 | 0.680 / 0.680 | 0.672 / 0.672 | 0.656 / 0.656 |

At 1,024 sites, the single-`X` evaluation changed from 526,848 equality
comparisons to 2,048 equality and 43,618 ordering comparisons when the
evaluator began using factor IDs as keys. That isolated change increased the
one-action Heisenberg time by about 5% at 10 sites. Both Heisenberg cases
improved by the final checkpoint.

At 128 modes, the ordered traversal reduced equality comparisons from
891 to 131 for near hopping, 12,798 to 194 for middle hopping, and 16,767 to
257 for far hopping. Ordering comparisons matched the preceding checkpoint
in all three cases. A placement visits factors in the order of its input and
shares the suffix after the target. Each branch of a sum starts at the same
position. The selected action space determines whether the placement is graded.

The 1,024-site single-`X` delayed-action construction control measured
0.688 µs with factor keys, 0.815 µs with ordered traversal, and 0.689 µs in the
final run. The production changes apply to evaluation, and the isolated timing
increase did not recur in the final run. Action-free evaluation made zero
allocation requests at every checkpoint.

## Allocation requests and requested bytes

Each table cell gives allocation requests and cumulative requested bytes.

| Workload | Before requests / bytes | Factor keys | Ordered traversal | Deferred syntax |
| --- | ---: | ---: | ---: | ---: |
| Single `X`, 1,024 sites | 9,843 / 1,788,352 | 9,827 / 1,735,384 | 9,826 / 1,734,824 | 8,799 / 1,729,736 |
| Far hopping, 128 modes | 3,098 / 594,568 | 3,092 / 588,408 | 2,584 / 326,280 | 2,313 / 313,904 |
| Heisenberg 10, one action | 1,267 / 170,464 | 1,264 / 169,640 | 1,264 / 169,640 | 979 / 148,408 |
| Heisenberg 10, eight actions | 979,799 / 118,114,008 | 979,796 / 118,095,960 | 979,794 / 118,034,520 | 670,248 / 90,597,952 |

The final eight-action query retained 222 coordinates, matching the independent
bit-state reference. All diagnostic values apart from index comparison counts
matched the previous implementation. The timings and allocation totals come
from the release probe with the production code.

## State-expression node counts

The counts report calls to `StateExpr::new` during `evaluate()`. The instrumented
runs used temporary source copies immediately before and after deferral. Each
copy incremented an atomic counter in `StateExpr::new`, reset it before applying
the `evaluate()` call to the prebuilt delayed state, and read it afterward.

| Heisenberg 10 | Before deferral | After deferral |
| --- | ---: | ---: |
| One action | 132 | 30 |
| Eight actions | 132,922 | 454 |

[Counts before deferral](state-nodes-before.csv) · [Counts after deferral](state-nodes-after.csv)

## Reproduction and verification

The initial run on November 20, 2026 used revision
`7a30e8f696cc768693d9e0553bc0706494ab5832`.
The later checkpoints used the same revision plus the named changes. The
host, Rust 1.99.1, default release profile, and `Complex<f64>` coefficient type
match Stage 0. In each of the four checkpoints, timing, allocations, and
diagnostics used `single_x_on_neel`, `hop_`, and `heisenberg_repeated_neel`, for
54 matched timing and allocation cases and 64 diagnostic rows. The raw CSVs
retain seven samples for each timed case. [Probe commands](../../README.md#reproduction)

- Stage entry. [Timings](before/timings.csv) · [Allocations](before/allocations.csv) · [Diagnostics](before/diagnostics.csv)
- Ordered factor keys. [Timings](factor-keys/timings.csv) · [Allocations](factor-keys/allocations.csv) · [Diagnostics](factor-keys/diagnostics.csv)
- Graded traversal. [Timings](graded-traversal/timings.csv) · [Allocations](graded-traversal/allocations.csv) · [Diagnostics](graded-traversal/diagnostics.csv)
- Deferred syntax. [Timings](deferred-syntax/timings.csv) · [Allocations](deferred-syntax/allocations.csv) · [Diagnostics](deferred-syntax/diagnostics.csv)

Tests exercise ordered keys with equal IDs on distinct handles and a graded
factored sum before and around the target, using an independent matrix in both
precisions. Existing state tests also check first-occurrence order, zeros,
nonreflexive values, shared-source normalization, and deep delayed actions.
Workspace tests and doctests with all features, all-feature Clippy with
warnings denied, the all-targets check without default features, and formatting
passed.
