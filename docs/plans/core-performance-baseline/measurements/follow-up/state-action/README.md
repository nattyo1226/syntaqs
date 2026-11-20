# Selective tensor traversal during state action

Each local placement gets its target position from the selected Act space.
Normalized shapes carry the number of factors they cover. Ordinary and
graded even placements schedule the tensor child containing the target.
Graded odd placements also schedule the prefix that receives parity.
A sum uses the same starting position for each branch.

The evaluator reads unmodified children from the original tensor key and
takes the affected results from the value stack. A coefficient-only change
keeps the original shape ID. A changed shape triggers one tensor key
construction, including promotion of a tensor exposed when a sum collapses.
[State evaluation contract](../../../../../design/composite-expression.md#114-natural-state-collection-between-actions)

## Evaluation cost

Times are CPU medians of calls on prebuilt inputs, including destruction
of the output. Requested bytes count cumulative allocator traffic.

| Workload | CPU (µs), before → after | Allocation requests, before → after | Requested bytes, before → after |
| --- | ---: | ---: | ---: |
| `X` on the last of 1,024 sites | 433.203 → 427.750 | 8,784 → 8,754 | 1,723,928 → 1,642,832 |
| Heisenberg 10, one action | 36.780 → 23.391 | 979 → 690 | 148,408 → 78,104 |
| Heisenberg 10, eight actions | 29,747.542 → 18,823.375 | 670,247 → 436,391 | 90,597,496 → 41,052,760 |
| Near hopping, 128 modes | 45.942 → 43.023 | 1,178 → 1,161 | 232,200 → 209,248 |
| Far hopping, 128 modes | 68.991 → 66.120 | 2,313 → 2,296 | 313,904 → 290,952 |
| Hubbard vacuum, 64 sites | 3,104.802 → 2,966.458 | 148,939 → 146,406 | 15,028,016 → 10,730,288 |
| Hubbard alternating occupations, 64 sites | 5,034.145 → 4,810.542 | 225,253 → 221,420 | 23,300,680 → 16,915,752 |
| W-state norm, 64 sites | 3,677.177 → 3,519.365 | 126,531 → 126,536 | 5,973,248 → 5,980,064 |

CPU time for the ten-site eight-action case drops by 35.2% at the median
of four alternating-order pairs. The median differences for the 1,024-site local
action and 128-mode near hop are −1.1% and −1.3%. The far hop ranges from
−3.2% to +1.3% with a median difference near zero. Most of its prefix
still needs parity.

Action-free normalization still runs before the first placement on a
prebuilt action. For action-free W states at 8–32 sites, normalization
takes 1.2–1.7% more CPU time in the sweep. At eight sites it requests
480 extra bytes, or 0.8%. On the 64-site W state,
normalization time differs by under 0.1% in the sweep and the recheck
pairs span −3.2% to +2.4%. The already-normalized 16-site product state
has zero allocation requests in both revisions. Its recheck median is
1.5% slower, under one nanosecond per call.
[Before timings](before/timings.csv) · [After timings](after/timings.csv) · [Before allocations](before/allocations.csv) · [After allocations](after/allocations.csv) · [Alternating recheck](recheck-timings.csv)

## Placement work

Child tasks count scheduled children of a tensor during placement. The two
array counters track extraction of tensor children and collection of their
results. Both after counters are zero in all measured cases. Key lookups
include initial tensor normalization and cover both hits and misses in the
shape table.

| Workload | Tensor child tasks, before → after | Input and result arrays, each, before → after | Tensor key lookups, before → after |
| --- | ---: | ---: | ---: |
| `X` on the last of 1,024 sites | 1,024 → 1 | 1 → 0 | 2 → 2 |
| Heisenberg 10, one action | 540 → 54 | 54 → 0 | 55 → 37 |
| Heisenberg 10, eight actions | 438,480 → 43,848 | 43,848 → 0 | 43,849 → 29,233 |
| Near hopping, 128 modes | 256 → 3 | 2 → 0 | 3 → 3 |
| Far hopping, 128 modes | 256 → 129 | 2 → 0 | 3 → 3 |
| Hubbard vacuum, 64 sites | 40,448 → 16,318 | 316 → 0 | 1 → 1 |
| Hubbard alternating occupations, 64 sites | 60,672 → 24,477 | 474 → 0 | 285 → 253 |

The 1,024-site case still normalizes all factors once and changes one
shape. For the far hop, annihilation at the first mode uses one child
task. Creation at the last mode still uses all 128, including the parity
prefix. The repeated Heisenberg case saves tensor work on every placement.
[Before work counts](before/work.csv) · [After work counts](after/work.csv)

## Reproduction

The reference is `f01998d116828ac9488008cefbdada05cdd8a323`. The after
source adds selective tensor traversal. Both use the same probe, the
follow-up reference host, Rust 1.99.1, the default release profile, and
`Complex<f64>`. Build timing and allocation executables separately. Run
each case filter below and retain the named APIs.
[Probe commands](../../../README.md#reproduction) · [Host configuration](../README.md#reproduction-and-verification)

| Filter | APIs |
| --- | --- |
| `single_x_on_neel` | `evaluate`, `evaluate_input` |
| `heisenberg_repeated_neel`, `,hop_`, `hubbard_neel`, `hubbard_vacuum`, `zero_before_sum` | `evaluate` |
| `,w,` | `evaluate`, `norm_squared` |

For timing and allocations, alternate binary order across the filters
through `zero_before_sum`, starting with the before binary. Run the
W-state filter before, then after. The recheck alternates order between
passes for each of its seven cases. The raw files retain all seven time
samples per call in each pass.

The matched sweep has 48 timing and allocation rows per revision. Run
`diagnostics` without a filter to reproduce the 615 diagnostic rows.
Only 26 factor-comparison counts differ. Physical reference values agree,
including all 222 final ten-site Heisenberg coordinates and the graded
hopping signs.
[Before diagnostics](before/diagnostics.csv) · [After diagnostics](after/diagnostics.csv)

For work counts, apply instrumentation to separate source copies. Apply
the reference instrumentation to both copies with include patterns for
`crates/syntaqs-core/src/lib.rs`, `crates/syntaqs-core/src/probe_work.rs`,
and the probe's `src/measure.rs`. Include
`crates/syntaqs-core/src/evaluate/state_terms.rs` from the reference patch on
the before copy. Apply the exact-zero checkpoint patch on the before copy
with include patterns for `crates/syntaqs-core/src/evaluate/composite.rs` and
`crates/syntaqs-core/src/probe_work.rs`. Then use the new counters on the
after copy and the shared work cases on both. Run `diagnostics` and retain
the four `tensor_*` metrics. Each revision has 192 work rows.
[Reference instrumentation](../before/work-counts.patch) · [Exact-zero instrumentation](../zero-product/after/work-counts.patch) · [New counters](after/work-counts.patch) · [Shared work cases](work-cases.patch)

The regression tests check a coefficient-only result sharing the original
tensor node and an action on a tensor of two sum blocks. The second block
collapses to a tensor after one branch becomes zero. The ordinary and
graded cases verify the final shape and the sign from the parity prefix.
[Regression tests](../../../../../../crates/syntaqs-core/src/evaluate/tests/stopping.rs)

Workspace tests and doctests with all features, core tests without default
features, and the no-default all-targets workspace check passed. Workspace
and probe Clippy and formatting checks passed, along with the probe's
allocation-counter test.
