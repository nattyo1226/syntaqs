# Follow-up reference measurements

The reference covers all 322 Stage 8 cases, 45 new space-construction cases,
and three products with an exact-zero result before an operand containing a
sum. The original 246 diagnostic values match Stage 8. Production sources
match Stage 8. The median of the per-case CPU and elapsed time ratios is 1.017
for the original cases. Six CPU medians increased by more than 10%, and
allocation totals differ in 54 original cases. Use this run as the matched
reference for the follow-up checkpoints. [Implementation plan](../../README.md#follow-up-implementation-sequence) · [Stage 8 reference](../stage-8/README.md)

## Space construction and retention

The new controls use `usize` factor IDs on ordinary spaces with 8, 32, 128,
512, and 1,024 factors. The workload name describes the prebuilt inputs.

| Workload | Prebuilt inputs |
| --- | --- |
| `space_flat_ordered` | Individual factors with ascending IDs |
| `space_flat_unsorted` | Individual factors with alternating highest and lowest remaining IDs |
| `space_nested_unsorted` | Four-factor blocks made from the same unsorted sequence |

`tensor_all` combines the input parts in one call. `tensor_successive` appends
one input part to the accumulated space at a time and returns the final
space. `tensor_successive_retained` additionally returns the intermediate
handles in a vector, excluding the prebuilt first part and the final space.
The diagnostic run checks factor order in the final and retained spaces and
counts ID comparisons.

| 1,024 factors | CPU / elapsed (µs) | Allocation requests | Requested bytes | Retained bytes |
| --- | ---: | ---: | ---: | ---: |
| Ordered factors, `tensor_all` | 47.495 / 47.494 | 1,206 | 61,160 | 16,432 |
| Ordered factors, successive | 12,641.500 / 12,641.333 | 99,314 | 21,457,496 | 81,840 |
| Ordered factors, successive with intermediate handles | 12,483.375 / 12,483.166 | 99,316 | 21,473,880 | 98,192 |
| Unsorted factors, `tensor_all` | 52.741 / 52.740 | 1,206 | 61,160 | 16,432 |
| Unsorted factors, successive | 13,107.125 / 13,107.000 | 99,991 | 21,565,440 | 81,840 |
| Unsorted blocks, `tensor_all` | 39.045 / 39.044 | 692 | 38,632 | 4,144 |
| Unsorted blocks, successive | 2,999.292 / 2,999.156 | 24,821 | 3,330,368 | 20,400 |

Timing and allocation traffic include destroying the returned objects.
Retained bytes are the net change in live allocations measured just before
dropping those objects. Prebuilt factors and blocks stay alive outside the
measurement. Retained bytes describe this change in requested heap storage.
The 16,352-byte difference between the two ordered successive runs comes
from the vector of 1,022 intermediate handles. The final space already keeps
the nested intermediate spaces reachable. All construction cases retain zero
factor-index entries in the current implementation.
[Retained bytes](before/retained.csv) · [Construction diagnostics](before/diagnostics.csv)

The 1,024-site Heisenberg `bind()` takes 139.098 µs and requests 114,608 bytes
in this reference. It builds one index on both a fresh and a reused root.
Explicit `act_in()` on an equivalent root and 64-site `H²` coordinates also
build one per call. The already bound `act()` and the single-reference `act()`
control build zero. The index entries are all released before these calls
return. [Work counters](before/work.csv)

## State action work

Each zero-before-sum control evaluates ((B C + I)|\mathrm{vac}\rangle), with a
two-term (B\) and annihilation (C\). The three expressions use a local
operator product, a local product placed on a graded space, and a product
with a sum spanning two graded factors. The diagnostics check that the sole
output coordinate is the original vacuum with coefficient 1.

| Evaluation workload | Product continuations with zero | Sum branches scheduled with zero | Tensor child tasks | Input / result arrays | Tensor key lookups |
| --- | ---: | ---: | ---: | ---: | ---: |
| Local zero before sum | 1 | 2 | 0 | 0 / 0 | 0 |
| Graded placement of a local zero-before-sum product | 1 | 2 | 4 | 2 / 2 | 2 |
| Composite zero before sum | 1 | 2 | 4 | 2 / 2 | 2 |
| Hubbard vacuum, 64 sites | 316 | 0 | 40,448 | 316 / 316 | 1 |
| Hubbard alternating occupations, 64 sites | 158 | 0 | 60,672 | 474 / 474 | 285 |
| Single `X`, 1,024 sites | 0 | 0 | 1,024 | 1 / 1 | 2 |
| Heisenberg 10, one action | 0 | 0 | 540 | 54 / 54 | 55 |
| Heisenberg 10, eight actions | 0 | 0 | 438,480 | 43,848 / 43,848 | 43,849 |

`zero_product_continuations` counts product continuations that receive an
exact-zero state. `zero_operator_sum_branches` counts child tasks dispatched
when an operator sum receives zero. The tensor child and array counters apply
to tensors reached while placing a local operator. The input array comes
from extracting the children and the result array comes from taking their
evaluated results off the value stack. `tensor_key_lookups` includes all
tensor keys probed in the shape table, including initial normalization, and
counts both hits and misses. These are logical call counts. Allocator traffic
is measured separately. [Work counters](before/work.csv)

## Reproduction and verification

The reference uses production revision
`5120b348e6af1f6358794d94e9452ce3f2469ccc` and the accompanying standalone
probe on November 21, 2026. It uses an Apple M4 MacBook Air with 24 GB RAM,
macOS 15.8.1 (24H32), Rust 1.99.1, the default release profile, and the same
`Complex<f64>` coefficients as Stage 8. The standard `time`, `allocations`,
and `diagnostics` commands run without a filter. Set `results` to
`"$plan/measurements/follow-up/before"` in the reproduction commands. Add
the following command for the standalone retention run.
[Probe commands](../../README.md#reproduction)

```sh
cargo run --release --locked --manifest-path "$probe" --target-dir target --features allocations -- retained > "$results/retained.csv"
```

For work counters, apply the patch only to a separate source copy. Use the
same probe sources as for the API measurements. The patch updates the
evaluation and factor lookup routines to count work and adds diagnostic
calls on the prebuilt inputs. It is dedicated to this reference revision.
[Reference instrumentation patch](before/work-counts.patch)

```sh
set -o pipefail
copy=$(mktemp -d)
git archive 5120b348e6af1f6358794d94e9452ce3f2469ccc | tar -x -C "$copy"
cp "$plan/probe/src/"*.rs "$copy/$plan/probe/src/"
git -C "$copy" apply "$PWD/$results/work-counts.patch"
cargo run --release --locked --manifest-path "$copy/$probe" --target-dir "$copy/target" -- diagnostics |
  awk -F, 'NR == 1 || $7 ~ /^(zero_product_continuations|zero_operator_sum_branches|tensor_child_tasks|tensor_input_arrays|tensor_result_arrays|tensor_key_lookups|factor_index_builds|retained_index_entries)$/' > "$results/work.csv"
```

The raw files contain 370 timing and allocation rows, 384 regular diagnostic
rows, 45 retained-byte rows, and 368 work-count rows. The timing files include
all seven samples and their medians for both clocks.
[Timings](before/timings.csv) · [Allocations](before/allocations.csv) · [Diagnostics](before/diagnostics.csv)

The stored instrumentation patch reproduced the work counts in a separate
source copy. Standalone probe formatting, Clippy with warnings denied, and
the allocation-counter test passed. The shared state-collection and tensor
product integration tests also passed.
