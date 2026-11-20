# Factor positions indexed at space construction

Space construction now checks for duplicate factor IDs while assembling a
sorted table of existing factor handles and their tensor positions. A
parent with one nonempty child reuses the child's table. Validation and
operator evaluation look up positions on the selected space. The stored
tree still supplies child and leaf order for enumeration and reconstruction.
[Implementation plan](../../../README.md#follow-up-implementation-sequence)

## Construction time and retained memory

Each case starts with prebuilt factors or prebuilt four-factor groups. CPU
time includes construction and result destruction. Retained bytes measure
additional heap storage while the result is live, excluding the prebuilt
inputs. The table shows the 1,024-factor cases with only the final root
retained.

| Input | API | CPU (µs), before → after | Retained bytes, before → after | Additional index entries |
| --- | --- | ---: | ---: | ---: |
| Ordered factors | `tensor_all` | 42.80 → 8.65 | 16,432 → 32,872 | 1,024 |
| Unsorted factors | `tensor_all` | 43.78 → 15.82 | 16,432 → 32,872 | 1,024 |
| Unsorted four-factor groups | `tensor_all` | 35.71 → 18.26 | 4,144 → 20,584 | 1,024 |
| Ordered factors | Successive `tensor` | 10,743.67 → 4,628.50 | 81,840 → 8,535,912 | 524,799 |
| Unsorted factors | Successive `tensor` | 12,408.88 → 3,379.29 | 81,840 → 8,535,912 | 524,799 |
| Unsorted four-factor groups | Successive `tensor` | 2,731.89 → 745.60 | 20,400 → 2,139,960 | 131,580 |

For 8, 32, 128, 512, and 1,024 factors, CPU time fell by 30–80% for
`tensor_all()` and by 16–74% for the successive paths. The 1,024-factor
ordered `tensor_all()` makes 13 allocation requests instead of 1,206 and
requests 49,224 bytes instead of 61,160. Successive `tensor()` on that input
requests 8,617,704 bytes over the call, down from 21,457,496.

Each branching node keeps its own table. The final root of a left-deep tree
holds all the child nodes, with table sizes from 2 to `n` for `n` factors.
The total is `n(n + 1) / 2 − 1` entries. Keeping explicit handles for
the intermediate roots adds handle storage and shares those same tables.
At 1,024 ordered or unsorted factors it adds 16,352 bytes to the
after result. This quadratic retention is the construction limitation for
the next checkpoints to measure. [Before retained bytes](before/retained.csv) · [After retained bytes](after/retained.csv) · [Before work counts](before/work.csv) · [After work counts](after/work.csv)

The factor comparison counters distinguish equality from ordering. For
1,024 ordered factors, `tensor_all()` goes from 22,782 ordering calls to
1,023 ordering and 1,023 equality calls. Successive `tensor()` goes from
10,609,898 ordering calls to 523,776 calls of each kind. The constructor
sorts singleton inputs in place and merges already sorted child tables for
nested inputs. [Before diagnostics](before/diagnostics.csv) · [After diagnostics](after/diagnostics.csv)

## Validation and evaluation

These cases use prebuilt spaces. A fresh call receives an independently
constructed, equivalent root outside the timing and allocation windows.
The single-placement action uses an unbound operator on the last factor.
The same-root action uses an operator already bound to the input space.

| 1,024-site input | CPU (µs), before → after | Allocation requests, before → after | Index builds, before → after |
| --- | ---: | ---: | ---: |
| Heisenberg `bind()`, reused root | 138.165 → 73.886 | 17 → 3 | 1 → 0 |
| Heisenberg `bind()`, fresh root | 139.118 → 73.426 | 17 → 3 | 1 → 0 |
| Heisenberg `act_in()`, reused equivalent root | 139.823 → 75.583 | 23 → 9 | 1 → 0 |
| Heisenberg `act_in()`, fresh equivalent root | 140.521 → 74.880 | 23 → 9 | 1 → 0 |
| Heisenberg `act()`, same bound root | 0.044 → 0.045 | 2 → 2 | 0 → 0 |
| Single spin action `act()`, last factor | 0.693 → 0.069 | 5 → 3 | 0 → 0 |

The single-placement check goes from 1,024 equality calls to 11 ordering
calls. Evaluating that delayed action still takes about 0.43 ms at this
size. The placement traversal during state evaluation is a separate
checkpoint. In the eleven Heisenberg `H` and `H²` Cartesian coordinate
controls, CPU changes range from a 3.1% decrease to a 1.0% increase. All
eleven reuse the space's index.

For the measured ordered Heisenberg path, the sum of the separate
`tensor_all()` and reused-root `bind()` CPU medians falls from 179.01 µs to
82.40 µs at 1,024 sites and from 3.08 µs to 1.49 µs at 32 sites. This sum
estimates one construction and one validation. Construction costs decreased
on their own throughout the sweep, so they require no repeated validations
to break even in CPU time. The extra retained bytes persist for the
lifetime of the root.
The end-to-end 1,024-site model builder falls from 774.44 µs to 681.62 µs.
[Before timings](before/timings.csv) · [After timings](after/timings.csv) · [Before allocations](before/allocations.csv) · [After allocations](after/allocations.csv)

## Reproduction and verification

The before library uses `6b216280e34d15e7630644b60b266aebcd005d95`. The
after library uses that revision with the immutable index change. Both use
the checkpoint probe, including `act_in_fresh`, in the default release profile
on the follow-up reference host with Rust 1.99.1 and `Complex<f64>`.
Build separate executables for each source and allocation mode. For
`time` and `allocations`, run the filters below in order, starting with the
before binary on odd-numbered filters and the after binary on even-numbered
filters.
[Probe commands](../../../README.md#reproduction) · [Host configuration](../README.md#reproduction-and-verification)

| Case filter | APIs saved |
| --- | --- |
| `space_` | All |
| `heisenberg_construction` | All |
| `single_x_on_neel` | `act` and `evaluate` |
| `,heisenberg,` | `operator_coordinates_cartesian` |

Run `retained` and `diagnostics` without a filter. The files contain 98
matched timing and allocation rows, 45 retained-heap rows, and 390
diagnostic rows on each side. The diagnostic rows agree except for factor
comparison counts. The probe checks construction order and compares
representative ordinary and graded numerical results with independent
references.

For the work counts, copy both sources. Apply the reference instrumentation
using the include patterns below. Include the reference `space.rs` hunk only
on the before copy. Apply the query cases patch to both copies and the
eager index counter patch to the after copy. Run `diagnostics` and retain
the header and the `factor_index_builds` and `retained_index_entries` metrics.
The output has 158 matched rows. Entry counts describe newly live table
entries at call return, so reused tables and prebuilt inputs contribute
zero. The production benchmarks do not include the instrumentation.
[Reference instrumentation](../before/work-counts.patch) · [Shared work cases](work-cases.patch) · [Eager index counters](after/work-counts.patch)

```sh
git -C "$copy" apply \
  --include=crates/syntaqs-core/src/lib.rs \
  --include=crates/syntaqs-core/src/probe_work.rs \
  --include=docs/plans/core-performance-baseline/probe/src/measure.rs \
  --include=docs/plans/core-performance-baseline/probe/src/space_construction.rs \
  "$reference_patch"
```

The index tests cover interleaved IDs, tensor-order offsets, borrowed
leaf sharing, singleton and unit composition, duplicate detection, and
mixed-kind error precedence. The test ID supports ordering without
`Clone` and the shared space satisfies `Send + Sync`.
[Space tests](../../../../../../crates/syntaqs-core/src/space.rs)

All-feature workspace tests and doctests, core tests without default
features, and the no-default all-targets workspace check passed. Workspace
and probe Clippy and formatting checks passed, as did the probe's
allocation-counter test.
