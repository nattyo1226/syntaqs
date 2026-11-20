# One-level Hilbert space flattening

`HilbertSpace::flatten()` promotes the children of immediate tensor nodes
and shares the existing factor-position index. Factor order and effective
kind are preserved. When promotion makes the outer kind effective, the new
root uses the original effective kind. A factor or already-flat tensor
shares its source node.
[Hilbert space contract](../../../../../design/hilbert-space.md#4-stored-trees-and-compatibility)

## Flattening a prebuilt space

The space is built outside the probe window. CPU time includes destruction
of the returned handle. Retained bytes are additional heap storage while
that handle is live and the source space is still held.

| Input at 1,024 factors | CPU (µs) | Allocation requests | Requested bytes | Retained bytes |
| --- | ---: | ---: | ---: | ---: |
| Already-flat ordered tensor | 0.389 | 0 | 0 | 0 |
| Unsorted tensor with four-factor groups | 4.903 | 11 | 32,800 | 16,448 |

The standalone flatten cases perform zero index comparisons and build zero
index tables. The nested case keeps a new root and child list and shares
the factor nodes and index.

## Successive construction

All rows compare paths in the checkpoint executable at 1,024 factors.
The input factors or four-factor groups already exist.

| Input | Roots retained | CPU (ms), `tensor` → `tensor().flatten()` | Retained bytes, `tensor` → `tensor().flatten()` |
| --- | --- | ---: | ---: |
| Ordered factors | Final only | 4.534 → 6.625 | 8,535,912 → 32,872 |
| Unsorted factors | Final only | 3.430 → 5.359 | 8,535,912 → 32,872 |
| Unsorted groups | Final only | 0.762 → 1.364 | 2,139,960 → 32,872 |
| Ordered factors | All | 4.534 → 7.667 | 8,552,264 → 16,916,312 |
| Unsorted factors | All | 3.480 → 6.560 | 8,552,264 → 16,916,312 |
| Unsorted groups | All | 0.748 → 1.392 | 2,144,024 → 4,241,144 |

For ordered factors, both paths build 1,023 indices. With only the final
root retained, the new path releases earlier roots and keeps 1,024 index
entries, down from 524,799. When the caller keeps every intermediate
root, both paths retain 524,799 entries. The flattened intermediates
each have a wider immediate child list, which accounts for the increase
in retained bytes. With the four-factor groups, the corresponding
final-only counts are 131,580 and 1,024 entries.

The new path increases CPU time by 46–105% across the final-only sweep.
It creates a tensor and then copies the promoted child handles into a
second root at each step. For 1,024 ordered factors, cumulative requests
rise from 8,617,704 to 28,231,928 bytes despite the lower final retention.
Direct construction of a flattened tensor will measure the cost of
avoiding that intermediate root in the next checkpoint.
[Implementation plan](../../../README.md#follow-up-implementation-sequence)

## Reproduction

The before source is `c69cd12d47bc379d9a581032f819e761199dc336`.
The after source includes `flatten()` and the additional construction
cases. Both use the default release profile on the follow-up reference host
with Rust 1.99.1. Build the binaries separately and run `time space_`,
`allocations space_`, and `retained space_`, with the `allocations`
feature enabled for the last two commands. Run `diagnostics` unfiltered.
[Probe commands](../../../README.md#reproduction) · [Host configuration](../README.md#reproduction-and-verification)

There are 45 matched construction rows and 45 after-only rows in each of
the timing, allocation, and retention files. The original cases agree
on allocation counts, requested and retained bytes, and all 390 diagnostic
values. The after run adds 135 diagnostic rows.
[Before timings](before/timings.csv) · [After timings](after/timings.csv) · [Before allocations](before/allocations.csv) · [After allocations](after/allocations.csv) · [Before retention](before/retained.csv) · [After retention](after/retained.csv) · [Before diagnostics](before/diagnostics.csv) · [After diagnostics](after/diagnostics.csv)

To reproduce work counts, apply the reference instrumentation in separate
source copies. Include only `crates/syntaqs-core/src/lib.rs`,
`crates/syntaqs-core/src/probe_work.rs`, and the probe's `src/measure.rs`.
Include the reference `src/space_construction.rs` hunk on the before
copy and apply the flatten work cases patch on the after copy. Apply the
eager index counter patch on both. Run `diagnostics space_` and keep the
`factor_index_builds` and `retained_index_entries` metrics. The entry count
is the net number of new live table entries while the result is held,
excluding prebuilt data.
[Reference instrumentation](../before/work-counts.patch) · [Eager index counters](../factor-index/after/work-counts.patch) · [Flatten work cases](after/work-cases.patch) · [Before work counts](before/work.csv) · [After work counts](after/work.csv)

Workspace tests and doctests with all features, core tests without default
features, and the no-default all-targets workspace check passed. Workspace
and probe Clippy and formatting checks passed, along with the probe's
allocation-counter test.
