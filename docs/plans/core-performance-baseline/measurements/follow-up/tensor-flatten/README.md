# Direct construction of flattened Hilbert spaces

`tensor_flatten()` validates the original operands and constructs the
promoted parent directly. It shares the validation and index merge with
`tensor()` and the one-level promotion with `flatten()`. The graded and
explicit-kind forms follow the same rules. The result preserves the child
handles, factor order, and kinds of the equivalent two-step composition.
[Hilbert space contract](../../../../../design/hilbert-space.md#4-stored-trees-and-compatibility)

## CPU time

The comparison uses successive construction in the same release binary.
CPU time includes destruction of the returned handles. The table shows
ordered factors.

| Factors | Roots retained | `tensor().flatten()` | `tensor_flatten()` |
| ---: | --- | ---: | ---: |
| 8 | Final only | 1.658 µs | 1.149 µs |
| 32 | Final only | 12.806 µs | 11.765 µs |
| 1,024 | Final only | 6.456 ms | 6.426 ms |
| 1,024 | All | 7.568 ms | 7.568 ms |

At 8 and 32 factors, CPU time decreases by 8–35% across the paired
cases. At 1,024 factors, the final-only cases improve by 0.5–1.2%.
Keeping all roots at that size gives changes within 1.3% in the sweep.
Index merges and wide child lists remain in both paths.

Four recheck passes alternate run order for the 128-factor ordered
`tensor_all()` and the 1,024-factor ordered flattening cases.
`tensor_all()` takes 1.238 µs before and 1.222 µs after. For the existing
two-step path, the median across the four passes changes from 6.564 ms
before to 6.548 ms after with final-only retention. The
all-roots case changes from 7.671 ms to 7.740 ms. In the after binary,
direct construction takes 6.506 ms and 7.643 ms on these cases.
[Recheck samples](recheck-timings.csv)

## Retention and construction work

At 1,024 ordered factors, keeping only the final root retains 32,872 bytes
in both paths. Keeping every intermediate root retains 16,916,312 bytes in
both paths. The input factors are already built. Both paths build 1,023
indices and retain 1,024 index entries with only the final root, or
524,799 entries with all roots.

With final-only retention, direct construction reduces allocation requests
from 15,353 to 13,309 and cumulative requested bytes from 28,231,928 to
28,133,816. Each step that promotes a tensor operand saves the grouped
parent allocation and its two-child list. Both paths still copy the
promoted child handles into the final list and merge the index once.
Across all 30 paired cases, retained bytes, index builds, and factor
comparison counts match.
[After allocations](after/allocations.csv) · [After retention](after/retained.csv) · [After work counts](after/work.csv)

## Reproduction

The before source is `748f29ccf0937bfd09ff062b525dc0204881200b`.
The after source includes the direct construction API and its probe cases.
Both use the default release profile and Rust 1.99.1 on the follow-up
reference host. Build separate binaries and run `time space_`,
`allocations space_`, and `retained space_`, with the `allocations`
feature enabled for the last two commands. Run `diagnostics` unfiltered.
The two-step control assigns the grouped result before calling
`flatten()`, keeping the previous checkpoint's drop order.
[Probe commands](../../../README.md#reproduction) · [Host configuration](../README.md#reproduction-and-verification)

There are 90 matched construction rows and 30 additional direct-construction
rows in each of the timing, allocation, and retention files. Inputs contain
8, 32, 128, 512, or 1,024 factors in ordered, interleaved, or interleaved
four-factor-group form. The matched rows agree on allocation requests,
requested and retained bytes, and all 525 original diagnostic values.
[Before timings](before/timings.csv) · [After timings](after/timings.csv) · [Before allocations](before/allocations.csv) · [Before retention](before/retained.csv) · [Before diagnostics](before/diagnostics.csv) · [After diagnostics](after/diagnostics.csv)

To reproduce work counts, apply the three instrumentation patches from
the flatten checkpoint to separate copies of both sources. Include only
`crates/syntaqs-core/src/lib.rs`, `crates/syntaqs-core/src/probe_work.rs`,
and the probe's `src/measure.rs` from the reference instrumentation.
Apply the eager index counters and the flatten work cases to both copies.
Run `diagnostics space_` and retain the `factor_index_builds` and
`retained_index_entries` metrics. The count covers net new table entries
while the result is live and excludes prebuilt inputs.
[Reference instrumentation](../before/work-counts.patch) · [Eager index counters](../factor-index/after/work-counts.patch) · [Flatten work cases](../flatten/after/work-cases.patch) · [Before work counts](before/work.csv)

Workspace tests and doctests with all features, core tests without default
features, and the no-default all-targets workspace check passed. Workspace
and probe Clippy and formatting checks passed, along with the probe's
allocation-counter test.
