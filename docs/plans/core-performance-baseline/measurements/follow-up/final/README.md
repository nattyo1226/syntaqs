# End-to-end follow-up comparison

The final implementation reduces CPU time for construction, binding, and
repeated state action. The alternating recheck gives a 47% reduction for
1,024-site Heisenberg binding and a 36% reduction for eight applications
at ten sites. The largest retained-memory cost comes from the index
tables of successive tensors. Releasing earlier roots after flattening
reduces that cost.
[Follow-up plan](../../../README.md#follow-up-implementation-sequence)

## Stage 8 sweep

The table uses the fresh full sweep on both revisions. Time includes
output destruction. Requested bytes are cumulative allocation traffic.

| Case | CPU time, Stage 8 → final | Requested bytes, Stage 8 → final |
| --- | ---: | ---: |
| Build the 1,024-site Heisenberg model | 778.33 → 684.04 µs | 1,518,088 → 1,465,208 |
| Rebind the 1,024-site Heisenberg operator | 138.33 → 73.15 µs | 114,608 → 73,664 |
| `act_in()` on a fresh equivalent 1,024-site root | 140.94 → 74.88 µs | 131,096 → 90,152 |
| Construct a single-reference `act()` at 1,024 sites | 0.694 → 0.068 µs | 8,296 → 96 |
| Evaluate that delayed action | 437.98 → 427.97 µs | 1,724,392 → 1,650,304 |
| Evaluate one Heisenberg application at ten sites | 43.05 → 23.26 µs | 148,408 → 78,104 |
| Evaluate eight Heisenberg applications at ten sites | 33.78 → 18.53 ms | 90,597,496 → 41,053,216 |
| Evaluate the 64-site Hubbard vacuum | 3.090 → 2.878 ms | 15,027,104 → 10,730,288 |
| Obtain 128-mode far-hop operator coordinates | 1.863 → 1.134 µs | 11,464 → 6,360 |

The repeated-action recheck gives medians of 28.89 ms and 18.50 ms, and all four
pairs give reductions between 35.7% and 36.2%. Fresh and reused Heisenberg
binding at 1,024 sites improve by 46.7–47.1% at the recheck medians.
Single-reference evaluation gains around 2% on the recheck because it
still normalizes its full input. The 128-mode near and far state hops
have paired median reductions of 4.2% and 2.0%, with a small increase in
one far-hop pair. The far hop still processes the long parity prefix.
[Matched comparisons](comparison.csv) · [Alternating pairs](recheck-timings.csv)

## Construction and validation

The probe prebuilds a Heisenberg operator on the same ordered factors.
It measures construction, `bind()` on a prebuilt root, and construction
followed by one or eight `bind()` calls on the newly built root. The
intermediate construction handles are released in these cases. The
separate retention sweep also measures keeping all intermediate roots.
Stage 8 uses successive `tensor()` as the reference for the two new
flattening paths.

| Ordered factors and construction path | Build CPU (µs) | Prebuilt `bind()` CPU (µs) | Build + one `bind()` CPU (µs) | Build + eight `bind()` CPU (µs) |
| --- | ---: | ---: | ---: | ---: |
| 1,024, `tensor_all()` | 43.83 → 8.44 | 139.40 → 74.28 | 182.40 → 84.31 | 1,158.07 → 610.46 |
| 1,024, successive `tensor()` | 12,225.33 → 4,307.44 | 144.75 → 73.88 | 11,930.88 → 4,262.45 | 12,881.00 → 4,850.01 |
| 1,024, successive direct flattening | 12,225.33 → 6,582.04 | 144.75 → 73.17 | 11,930.88 → 6,692.98 | 12,881.00 → 7,203.52 |
| 32, successive two-step flattening | 12.06 → 12.79 | 2.44 → 1.10 | 14.61 → 13.96 | 31.83 → 22.18 |
| 32, successive direct flattening | 12.06 → 11.35 | 2.44 → 1.11 | 14.61 → 12.51 | 31.83 → 20.60 |

Thirty-five of the 36 paths already gain CPU time at construction. The
exception is ordered 32-factor `tensor().flatten()`. A break-even estimate
divides added construction time by the saving per prebuilt binding and
rounds up to the next whole call. This estimates one call for the
exception. Its alternating recheck gives construction medians of
12.07 µs and 12.81 µs and combined medians of 14.43 µs and 14.02 µs after one
binding. Construction-only and combined timings have separate samples
and cache states.

Among the final successive APIs, prebuilt bindings at 1,024 ordered factors
all take about 74 µs. The small differences between those binding times
give no reliable crossover between the current APIs. With released
intermediates, flattening trades additional construction time for lower
retained memory.
[All construction and binding combinations](construction-validation.csv) · [Alternating pairs](recheck-timings.csv)

## Retained memory and allocation traffic

The table shows the 1,024-factor ordered inputs. Retained bytes are the
extra live heap storage while the construction result is held. The input
leaves already exist. Index entries count the entries in newly live shared
tables, including tables reached through child roots.

| Revision and construction path | Roots held | Cumulative requested bytes | Retained bytes | Retained index entries |
| --- | --- | ---: | ---: | ---: |
| Stage 8 `tensor_all()` | Final | 61,160 | 16,432 | 0 |
| Final `tensor_all()` | Final | 49,224 | 32,872 | 1,024 |
| Stage 8 successive `tensor()` | Final | 21,457,496 | 81,840 | 0 |
| Final successive `tensor()` | Final | 8,617,704 | 8,535,912 | 524,799 |
| Final successive `tensor_flatten()` | Final | 28,133,816 | 32,872 | 1,024 |
| Stage 8 successive `tensor()` | All | 21,473,880 | 98,192 | 0 |
| Final successive `tensor()` | All | 8,634,088 | 8,552,264 | 524,799 |
| Final successive `tensor_flatten()` | All | 28,150,168 | 16,916,312 | 524,799 |

The final nested root owns its earlier child roots, so successive
`tensor()` retains the tables with 2 through 1,024 entries. Successive
flattening leaves one table live when callers release intermediate roots.
Keeping all flattened roots also keeps their wider child lists. Direct
flattening and the two-step equivalent retain identical bytes and index
entries in all matched cases. Direct construction makes fewer allocation
requests, as measured in the preceding checkpoint.
[Stage 8 retention](stage-8/retained.csv) · [Final retention](final/retained.csv) · [Stage 8 index data](stage-8/index-work.csv) · [Final index data](final/index-work.csv) · [Direct flattening checkpoint](../tensor-flatten/README.md)

## Rechecked slowdowns and allocation increases

XX layer and echo operator coordinates took 6–12% more CPU time on the
initial sweep. On the four alternating pairs, seven XX cases have paired
medians 1.5–6.3% lower. The 16-site layer has individual differences from
−2.7% to +1.5% and a median of +0.6%, about 0.35 µs. The 4-site BLBQ
model-build change also reverses in the recheck. No repeatable slowdown
of the original sweep magnitude appears in these rechecks.

The largest increase in cumulative requested bytes among the original
cases is 19,408 bytes for the 64-site W state. It occurs in evaluation,
norm, and coordinate queries. The norm goes from 5,967,696 to 5,987,104
bytes, an increase of 0.33%. The norm's paired CPU median differs by
−0.08%. Across the W-state timing controls, paired median differences
range from −0.75% to +1.08%. Retained tensor indices account for the
material memory trade-off in the construction cases above.
[Full comparisons](comparison.csv) · [Alternating pairs](recheck-timings.csv)

## Reproduction

The run compares the Stage 8 library
`775baa825235f443c07b3f57cdc97ade85f89518` and the final library
`d1d33cc8a28faf6e0dc3a94020bd7c52ddc8ddd2` with this checkpoint's probe.
It uses the follow-up reference host, Rust 1.99.1, the default Cargo
release profile, and `Complex<f64>`. Copy the current probe source files
into an archive of the Stage 8 tree and apply the adapter to that copy.
The adapter omits the flattening cases that require the new APIs. Build
release binaries with separate target directories. Run unfiltered
`diagnostics`, `time`, `allocations`, and `retained`, Stage 8 before final.
`allocations` and `retained` use the executable with the probe's
`allocations` feature.
[Stage 8 adapter](stage-8-probe.patch) · [Probe commands](../../../README.md#reproduction) · [Host configuration](../README.md#reproduction-and-verification)

The 427 paired timing and allocation rows include all 322 original Stage 8
cases. The final sweep has 556 rows with the additional flattening cases.
The binding controls use 32, 128, and 1,024 ordered, interleaved, or grouped
factors. In the new rows, `space_bind` times binding on a prebuilt root and
`space_build_bind` counts bindings in the `order` column. All numerical
assertions passed. Of 390 common diagnostic values, 172 index comparison
counts differ and the other 218 values agree. The recheck repeats 26
common and eight final-only cases four times. It covers Stage 8 cases
with an initial increase above 4% and 80 ns and targeted controls for
W states, binding, action, and the 32-factor construction crossover.
[Stage 8 times](stage-8/timings.csv) · [Final times](final/timings.csv) · [Stage 8 requests](stage-8/allocations.csv) · [Final requests](final/allocations.csv) · [Stage 8 diagnostics](stage-8/diagnostics.csv) · [Final diagnostics](final/diagnostics.csv)

To reproduce the index counts, work on separate copies of the measured
sources. Apply the reference instrumentation to both with include
patterns for `crates/syntaqs-core/src/lib.rs`,
`crates/syntaqs-core/src/probe_work.rs`, and the probe's
`src/measure.rs`. Include the reference `src/space.rs` and
probe `src/space_construction.rs` hunks on the Stage 8 copy.
Apply the eager index counters and flatten cases patch on the final copy.
Run `diagnostics space_` and keep `factor_index_builds` and
`retained_index_entries`.
[Reference instrumentation](../before/work-counts.patch) · [Eager index counters](../factor-index/after/work-counts.patch) · [Flatten case hooks](../flatten/after/work-cases.patch)

## Verification

All 24 checks passed on `aarch64-apple-darwin` with Rust 1.99.1.
They cover the workspace build, tests and doctests with default, all,
and no default features, the no-default all-target check, formatting,
Clippy, and rustdoc with warnings denied. The standalone probe test and
quality checks passed. Downstream coverage includes Vec, ndarray, and
sprs export and integration tests, both Lie example configurations,
isolated Lie analysis, native f32/f64 BLAS and LAPACK linkage, and
provider selection. The verification scope is the local macOS toolchain.
[Check results](verification.tsv) · [Repository verification commands](../../../../../design/verification.md)
