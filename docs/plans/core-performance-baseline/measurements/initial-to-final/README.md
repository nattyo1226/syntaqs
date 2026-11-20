# Initial-to-final performance comparison

The saved Stage 0 and final runs share 322 timing and allocation case
IDs. In these runs, CPU time decreases in 309 cases and increases in
13. The median per-case speedup is **2.04×**, calculated as Stage 0
CPU time divided by final CPU time. Cumulative requested bytes decrease
in 300 cases, match in four, and increase in 18.
[Case-by-case comparison](comparison.csv)

## Representative gains

The rows use thread CPU medians and cumulative requested bytes per call.
CPU timings include destruction of the result.

| Operation | Stage 0 → final CPU time | Speedup | Stage 0 → final requested bytes |
| --- | ---: | ---: | ---: |
| `tensor_all()`, 1,024 factors | 163.42 → 8.49 µs | 19.2× | 57,360 → 49,224 |
| Build the 1,024-site Heisenberg model | 3.109 → 0.684 ms | 4.5× | 51,919,616 → 1,465,208 |
| Heisenberg `bind()`, 1,024 sites | 2.372 ms → 73.15 µs | 32.4× | 50,405,264 → 73,664 |
| Evaluate eight Heisenberg actions at ten sites | 42.832 → 18.528 ms | 2.3× | 119,529,528 → 41,053,216 |
| Evaluate the 64-site Hubbard vacuum | 10.641 → 2.878 ms | 3.7× | 65,347,648 → 10,730,288 |
| Squared norm of a 64-site W state | 34.587 → 3.570 ms | 9.7× | 78,210,024 → 5,987,104 |
| Expand Heisenberg `H²` at 64 sites | 109.772 → 23.561 ms | 4.7× | 261,142,864 → 45,523,016 |
| Cartesian operator coordinates of a 16-site XX echo | 178.998 ms → 19.73 µs | about 9,100× | 582,376,640 → 71,024 |

The XX echo has one identity coordinate in both runs. The final operator
evaluator combines overlapping blocks before converting them to the
basis. The bound Heisenberg `act()` at 1,024 sites goes from 2.375 ms to
43.6 ns per call. This case measures construction of the delayed action
whose binding can be reused. The repeated-action row measures evaluation.
[Operator evaluation results](../stage-8/README.md#results-by-workload) · [Case-by-case comparison](comparison.csv)

## Measured increases

Four cases increase by more than 3% and 0.1 µs in the saved sweeps.

| Operation | Stage 0 → final CPU time | Increase |
| --- | ---: | ---: |
| Cartesian coordinates of Heisenberg `H`, 4 sites | 6.23 → 7.31 µs | 17.4% |
| Cartesian expansion of Heisenberg `H`, 4 sites | 8.54 → 9.29 µs | 8.7% |
| Cartesian coordinates of Heisenberg `H`, 8 sites | 14.74 → 16.28 µs | 10.5% |
| Cartesian operator coordinates of an XX layer, 4 sites | 2.53 → 2.75 µs | 8.5% |

The earlier Stage 0–Stage 8 rechecks also found increased coordinate
time for small Heisenberg `H`. The 16-site `evaluate_input` result rises
from 14.59 to 16.86 ns in the saved sweeps. Eight-site BLBQ Cartesian
coordinates take 107.54 → 94.83 µs and request 271,040 → 329,192 bytes
across the call, an increase of 21.5% in allocator traffic.
Retained space indices have their own measurement in the final
follow-up report.
[Earlier small-system rechecks](../stage-8/README.md#rechecked-regressions-and-allocation-increases) · [Retained memory](../follow-up/final/README.md#retained-memory-and-allocation-traffic)

## Data and scope

This comparison joins the stored Stage 0 production revision
`49e9fc51270ad51885c0c5b507fcb903eee305fc` and final production revision
`d1d33cc8a28faf6e0dc3a94020bd7c52ddc8ddd2` on system, workload, sites,
factors, order, and API. Both used the same macOS host, Rust 1.99.1,
and the release profile on separate runs. Small timing differences
are individual sweep observations. The CSV ratios use final divided
by Stage 0 and omit the ratio when Stage 0 is zero.

All 246 Stage 0 diagnostic keys appear in the final data. The changed
values are 81 index-comparison counts and 11 basis-expansion call counts.
The recorded output counts match. The comparison uses the existing
saved sweeps and their recorded numerical checks.
[Stage 0 source data](../stage-0/README.md) · [Final source data](../follow-up/final/README.md#reproduction)
