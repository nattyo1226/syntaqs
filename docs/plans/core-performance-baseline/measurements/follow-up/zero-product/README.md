# Exact-zero stopping in product actions

Product actions use one continuation that holds the product and advances in
application order. An exact-zero result ends that continuation before it
dispatches the next operand. Local, composite, and graded placed products
share operand selection. Graded placement passes the remaining local operand
directly to placement. Other branches of an enclosing sum continue to run.
[Implementation plan](../../../README.md#follow-up-implementation-sequence)

## Evaluation cost

These are matched release runs on prebuilt actions. Time is per call in
microseconds, including destruction of the result.

| Workload | CPU before → after (µs) | Elapsed before → after (µs) | Requests before → after | Requested bytes before → after |
| --- | ---: | ---: | ---: | ---: |
| Local zero before sum | 0.496 → 0.343 | 0.496 → 0.343 | 20 → 16 | 1,992 → 1,488 |
| Graded placement of a local zero-before-sum product | 1.810 → 1.694 | 1.810 → 1.694 | 67 → 64 | 8,504 → 8,392 |
| Composite zero before sum | 1.801 → 1.701 | 1.801 → 1.701 | 66 → 64 | 8,456 → 8,392 |
| Hubbard vacuum, 64 sites | 3,127.958 → 3,118.677 | 3,127.927 → 3,118.635 | 148,939 → 148,939 | 15,028,016 → 15,028,016 |
| Hubbard alternating occupations, 64 sites | 5,302.854 → 5,105.583 | 5,302.792 → 5,105.520 | 225,253 → 225,253 | 23,300,680 → 23,300,680 |
| Heisenberg 10, eight actions | 29,272.625 → 29,596.875 | 29,272.458 → 29,596.750 | 670,248 → 670,248 | 90,597,952 → 90,597,952 |

The three zero-before-sum controls take less time in both passes. The
64-site vacuum's time drops by 0.3% in the sweep and 0.6% in the recheck.
Its 40,448 tensor child tasks and allocation totals stay the same. Those
tasks precede product termination. The alternating-occupation and repeated
Heisenberg cases change direction between passes.

The 128-mode hopping controls showed CPU increases of 13–16% in the sweep.
In the reverse-order recheck, the near and far hops took 4.8% and 2.1% less
time. The middle hop took 4.1% more time, 58.63 µs before and 61.05 µs after.
Its work and allocation counts match. Four further middle-hop pairs,
alternating binary order, give changes from −3.2% to +1.2% with a median of
+0.1%. [Recheck before](before/recheck-timings.csv) · [Recheck after](after/recheck-timings.csv) · [Alternating middle-hop pairs](hop-mid-recheck.csv)

The first implementation put the direction and offset in separate fields.
On this host the local task grew from 24 to 32 bytes, and the 64-site vacuum
requested 131,568 extra bytes. A signed offset now carries the direction and
position in one field. The local task is again 24 bytes and the composite
task is 80 bytes. In the final sweep, 36 of 40 allocation rows match the
reference. The three zero-before-sum rows decrease. The 1,024-site local
action differs by 12 requests and 4,880 bytes. Its reference request count
also varied by 12 between runs.

## Work removed

`zero_product_continuations` counts continuations that dispatch another
operand on an exact-zero state. `zero_products_stopped` counts the new
continuations that end a product when they receive zero.

| Workload | Continuations dispatching with zero, before → after | Sum branches dispatched with zero, before → after | Products stopped after change |
| --- | ---: | ---: | ---: |
| Local zero before sum | 1 → 0 | 2 → 0 | 1 |
| Graded placement of a local zero-before-sum product | 1 → 0 | 2 → 0 | 1 |
| Composite zero before sum | 1 → 0 | 2 → 0 | 1 |
| Hubbard vacuum, 64 sites | 316 → 0 | 0 → 0 | 316 |
| Hubbard alternating occupations, 64 sites | 158 → 0 | 0 → 0 | 158 |

Across the state-action controls the new counters report zero operands and
sum branches dispatched with zero. The bit-state and graded diagnostics
retain all 384 reference values. [Reference work counts](../before/work.csv) · [After work counts](after/work.csv) · [Diagnostics](after/diagnostics.csv)

## Reproduction and verification

The reference binary is revision `35b6d659bfc6cbd512e67eeb5dd24513e813ecd6`.
The after binary uses that revision plus the product continuation change.
Both use the follow-up reference host, Rust 1.99.1, the release profile, and
`Complex<f64>`. Build before and after release binaries outside the timing
run, with `allocations` enabled for the allocation executables. Run each of
`hubbard_vacuum`, `hubbard_neel`, `zero_before_sum`,
`heisenberg_repeated_neel`, `single_x_on_neel`, and `,hop_` as a filter.
Retain `evaluate` and `evaluate_input` rows. This yields 40 matched cases.
The second pass reverses the binary order for ten evaluation cases.
[Probe commands](../../../README.md#reproduction) · [Before timings](before/timings.csv) · [After timings](after/timings.csv) · [Before allocations](before/allocations.csv) · [After allocations](after/allocations.csv)

To reproduce the work counts, copy the after source. Apply the reference
instrumentation patch with `--exclude=crates/syntaqs-core/src/evaluate/composite.rs`
and `--exclude=crates/syntaqs-core/src/evaluate/local.rs`, then apply the
checkpoint patch. Run the diagnostics with the reference work-counter
filter extended to include `zero_products_stopped`. [Reference instrumentation](../before/work-counts.patch) · [Checkpoint instrumentation](after/work-counts.patch)

The regression tests exercise exact zero before a sum in local, ordinary,
and graded action. They check that later primitive adjoints are skipped,
that the nonzero outer branch is retained, and that invalid placements are
still rejected. A nonzero three-operand product checks ordinary and adjoint
action order and the graded sign. [Regression tests](../../../../../../crates/syntaqs-core/src/evaluate/tests/stopping.rs)

The zero-before-sum test reports three adjoint calls on the reference
implementation and zero after the change. All-feature workspace tests and
doctests, core tests without default features, the no-default all-targets
check, and workspace and probe Clippy and formatting checks passed. The
work-count CSV was generated by applying the stored patches to a separate
source copy.
