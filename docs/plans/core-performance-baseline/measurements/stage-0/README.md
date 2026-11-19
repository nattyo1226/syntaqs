# Stage 0 reference measurements

The reference run contains 322 timing and allocation cases and 246 diagnostic
rows. The original 310 cases match by case ID, and all 186 original diagnostic
values match. Use this run as the comparison point for cumulative changes in
Stages 1–8. [Implementation plan](../../README.md#implementation-plan)

## Comparison with the historical run

Across the 310 matched cases, the median of the per-case timing ratios was
0.877 for thread CPU time and 0.876 for elapsed time. Both runs used identical
production sources. The table shows selected workloads and the differences
between runs under that condition. Allocation deltas are per call.

| Workload | CPU time in ms, historical → Stage 0 | CPU change | Elapsed time in ms, historical → Stage 0 | Stage 0 requests (change) | Stage 0 requested bytes (change) |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1,024 sites, `tensor_all` | 0.190 → 0.163 | −14.1% | 0.196 → 0.163 | 1,045 (0) | 57,360 (0) |
| Heisenberg 1,024, `build_model` | 3.608 → 3.109 | −13.8% | 3.724 → 3.109 | 39,949 (0) | 51,919,616 (0) |
| Heisenberg 1,024, `bind` | 2.749 → 2.372 | −13.7% | 2.841 → 2.372 | 12,279 (0) | 50,405,264 (0) |
| Heisenberg 1,024, `act` | 2.871 → 2.375 | −17.3% | 2.883 → 2.375 | 12,281 (0) | 50,405,352 (0) |
| Heisenberg 10, evaluate eight actions | 47.322 → 42.832 | −9.5% | 47.319 → 42.832 | 1,024,034 (0) | 119,529,528 (0) |
| W64 squared norm | 38.837 → 34.587 | −10.9% | 38.837 → 34.587 | 1,309,676 (+9) | 78,210,024 (+4,552) |
| Heisenberg 64, expand `H²` | 118.509 → 109.772 | −7.4% | 118.508 → 109.783 | 3,393,210 (+6) | 261,142,864 (+3,888) |
| XX echo 16, operator coordinates | 198.890 → 178.998 | −10.0% | 199.386 → 179.378 | 6,983,451 (0) | 582,376,640 (0) |

The first run showed CPU increases of 31% for the 128-site Néel expectation
and 37% for its state coordinates. Focused reruns with the old and updated
probes measured 5.629 and 5.669 ms for expectation, and 7.203 and 7.184 ms for
coordinates. The increases did not recur. [Old probe timing rerun](recheck-time-old.csv) · [Updated probe timing rerun](recheck-time-new.csv)

Allocation counts or bytes varied in 53 of 310 cases compared with the
historical run. The largest changes were 0.15% in requests and 0.48% in bytes.
Repeating the same updated probe changed 38 of 322 rows. Replaying the old
probe changed 51 of 310 rows relative to the historical data. These runs
establish the scale of allocation variation observed with the same production
code. [Old probe allocation rerun](recheck-allocations-old.csv) · [Updated probe allocation rerun](recheck-allocations-new.csv)

## Binding and product construction controls

`bind_fresh` prepares a distinct equivalent tensor root for each call before
starting either measurement. `bind` reuses an equivalent root and measures
after a warmup bind. `act` uses the operator's bound root. `act_in_equivalent`
overrides that binding with the separately constructed root used by `bind`.
The measured calls include destruction of their results.

| Heisenberg sites | Fresh `bind` CPU / elapsed (µs) | Reused `bind` CPU / elapsed (µs) | Explicit `act_in` CPU / elapsed (µs) | Fresh and reused `bind` equality comparisons | Explicit `act_in` equality comparisons |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 128 | 74.98 / 74.97 | 75.14 / 75.14 | 74.13 / 74.13 | 49,149 | 49,277 |
| 512 | 672.03 / 671.84 | 668.37 / 668.36 | 668.47 / 668.46 | 786,429 | 786,941 |
| 1,024 | 2,381.30 / 2,380.94 | 2,372.02 / 2,371.99 | 2,384.43 / 2,384.41 | 3,145,725 | 3,146,749 |

Fresh and reused `bind` had identical allocation totals at every measured
size. At 1,024 sites both made 12,279 requests for 50,405,264 bytes. Explicit
`act_in` added 1,024 equality comparisons to the `act` count at that size and
made 12,285 requests for 50,421,752 bytes. `index_comparisons` records equality
comparisons and `index_order_comparisons` records calls to `Ord::cmp`. The
ordering counter was zero in every diagnostic case that recorded it.

`product_sequence` calls `Operator::product` with preplaced local Pauli `X`
operators, one per site. `binary_mul` uses `*` to square the prebuilt and bound
product. Each operation is timed independently of its input construction.
The product yields one all-`X` coordinate with coefficient 1. Its square
yields one identity coordinate with coefficient 1.

| Sites | Sequence CPU / elapsed (µs) | Sequence requests / bytes | Binary `*` CPU / elapsed (ns) | Binary `*` requests / bytes |
| ---: | ---: | ---: | ---: | ---: |
| 8 | 0.208 / 0.208 | 10 / 608 | 64.1 / 64.1 | 3 / 128 |
| 64 | 1.403 / 1.403 | 66 / 4,640 | 65.0 / 65.0 | 3 / 128 |
| 1,024 | 24.196 / 24.196 | 1,026 / 73,760 | 66.9 / 66.9 | 3 / 128 |

## Reproduction and verification

Production revision `49e9fc51270ad51885c0c5b507fcb903eee305fc` has the same
production sources and workspace manifests as historical revision
`690b7b4ca6a59f88c48d0310d9faa80ab258aff1`. The run used the accompanying
updated probe on November 20, 2026, on an Apple M4 MacBook Air with 24 GB RAM,
macOS 15.8.1 (24H32), and Rust and Cargo 1.99.1 on `aarch64-apple-darwin`
(LLVM 23.1.1). The default Cargo release profile and original `Complex<f64>`
workloads were used. [Probe source](../../probe/src/main.rs) · [Run commands](../../README.md#reproduction)

The timing CSV records per-call medians and all seven batch means. The
first-use control caps batches at 256 prebuilt roots. Allocation counts use
one measured call per row in the instrumented binary. The timing rerun used
the filter `heisenberg_neel,128,128,1`, and the allocation rerun kept all
available cases. The old probe was built from the recorded production revision.
[Timing samples](timings.csv) · [Allocations](allocations.csv) · [Diagnostics](diagnostics.csv)

The full diagnostics run and all new result assertions passed. The release
tests and doctests passed for `syntaqs-core`, `syntaqs-spin-half`,
`syntaqs-spin-one`, `syntaqs-fermion`, and `syntaqs-tests` with default
features. The probe passed formatting and Clippy with all targets and features
and warnings denied.
