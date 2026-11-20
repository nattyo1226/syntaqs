# Stage 2 measurements

The initial Stage 2 checkpoint used the same ordered sequence for local and
composite operator products, including `*`, named products, and reconstructed
basis terms. Empty products become identity and single-operand products reuse
the operand. Each nested product retains its shared handle.
[Implementation plan](../../README.md#2-ordered-operator-products) · [Operand storage follow-up](#operand-storage-follow-up)

## Product construction and basis expansion

Timing includes destruction of the result. Each cell reports the medians for
thread CPU and elapsed time. Allocation counts are requests per call, and bytes
are cumulative requested bytes.

| Workload | Stage 1 CPU / elapsed | Stage 2 CPU / elapsed | Requests, Stage 1 → 2 | Requested bytes, Stage 1 → 2 |
| --- | ---: | ---: | ---: | ---: |
| 8-operand sequence (µs) | 0.208 / 0.208 | 0.112 / 0.112 | 10 → 3 | 608 → 304 |
| 64-operand sequence (µs) | 1.404 / 1.404 | 0.572 / 0.572 | 66 → 3 | 4,640 → 2,096 |
| 1,024-operand sequence (µs) | 24.409 / 24.409 | 8.221 / 8.241 | 1,026 → 3 | 73,760 → 32,816 |
| Binary `*`, prebuilt 8-operand products (ns) | 65.198 / 65.196 | 66.455 / 66.453 | 3 → 2 | 128 → 64 |
| Binary `*`, prebuilt 64-operand products (ns) | 64.802 / 64.800 | 69.074 / 69.071 | 3 → 2 | 128 → 64 |
| Binary `*`, prebuilt 1,024-operand products (ns) | 68.529 / 68.527 | 66.905 / 66.903 | 3 → 2 | 128 → 64 |
| Heisenberg 64, `H²` coordinates (ms) | 35.969 / 35.968 | 36.514 / 36.513 | 795,088 → 797,332 | 89,261,200 → 89,293,312 |
| Heisenberg 64, expand `H²` (ms) | 105.782 / 105.781 | 85.102 / 85.137 | 3,390,185 → 2,331,320 | 260,353,056 → 217,277,936 |
| Graded 128-mode far hopping, evaluate (µs) | 114.221 / 114.220 | 114.036 / 114.035 | 3,101 → 3,098 | 594,664 → 594,568 |
| Graded 128-mode far hopping, coordinates (µs) | 2.078 / 2.078 | 2.078 / 2.078 | 57 → 57 | 10,784 → 10,784 |
| Heisenberg 512, `build_model` (µs) | 353.293 / 353.289 | 390.305 / 390.301 | 13,934 → 13,922 | 815,488 → 758,128 |

The 64-site `H²` query returns 17,395 coordinates at both checkpoints. Each
coordinate had 63 binary product nodes in Stage 1. It now has one product node
with a single reserved operand buffer. This gives 1,095,885 → 17,395 product
nodes and 0 → 17,395 operand-buffer allocations by construction. The counts
derive from the measured coordinate count and the expression shape checked by
the expansion test. Expansion spent 20% less CPU time and issued 31% fewer
allocation requests. The coordinate-only control keeps the same 140,635 local
basis expansion calls. [Expansion construction](../../../../../crates/syntaqs-core/src/basis/expansion/composite.rs) · [Shape test](../../../../../crates/syntaqs-core/src/basis/expansion/operator/tests.rs)

The first sequence prototype measured 72.8–74.3 ns for binary `*` with three
allocation requests. The final shared representation reuses the owned operand
buffer in composite destruction and measured 66.5–69.1 ns with two requests.
The same destruction path can reuse the owned child buffer of a sum.
[Initial binary timings](before-drop-reuse-timings.csv) · [Initial binary allocations](before-drop-reuse-allocations.csv)

In a focused rerun of 512-site `build_model`, Stage 1 took 363.661 µs and
Stage 2 took 390.283 µs. This workload constructs and retains a short product
at each interaction term, along with the space and binding. The rerun confirms
the increase in the end-to-end time. The measurements give total allocation
traffic and leave the individual source of the time increase unverified.
[Stage 1 recheck](model-recheck-stage-1.csv) · [Stage 2 recheck](model-recheck-stage-2.csv)

## Reproduction and verification

The checkpoint uses production revision
`74344a44d63b7caee4aac94d277e67876d8d1a4b` plus the Stage 2 changes. It ran on
November 20, 2026 on the Stage 0 host with Rust 1.99.1, the release profile, and
the same `Complex<f64>` probe. The 53 timing and allocation cases match Stage
1 and use `operator_product_construction`, `heisenberg,64,64,2`,
`hop_far,128,128,1`, `heisenberg_construction`, and `single_x_on_neel`.
The diagnostic run covered all cases. [Run commands](../../README.md#reproduction)

All 246 diagnostic values and case IDs matched Stage 1. Targeted tests compare
three-operand local and composite products and their adjoints with independent
ordinary and graded matrices in both precisions. Updated tests cover product
arity, state action order, display, and deeply nested products. Other checks
cover fallible input precedence, nested sharing, and basis reconstruction.
Workspace tests and doctests with all features, all-feature Clippy with warnings
denied, the all-targets check without default features, and formatting passed.
[Stage 1](../stage-1/README.md) · [Timing](timings.csv) · [Allocations](allocations.csv) · [Diagnostics](diagnostics.csv)

## Operand storage follow-up

The follow-up uses `Operands` directly for both node types and shares
singleton handling, placed-operator binding checks, and immediate local-factor
classification. It used the same host, profile, and 53 timing and allocation
cases as the initial Stage 2 checkpoint. All 246 diagnostic values matched.

| Workload | Initial CPU / elapsed | Follow-up CPU / elapsed | Requests | Requested bytes |
| --- | ---: | ---: | ---: | ---: |
| Binary `*`, prebuilt 64-operand products (ns) | 69.074 / 69.071 | 55.610 / 55.608 | 2 | 64 |
| 1,024-operand sequence (µs) | 8.221 / 8.241 | 7.531 / 7.531 | 3 | 32,816 |
| Heisenberg 64, expand `H²` (ms) | 85.102 / 85.137 | 82.409 / 82.409 | 2,331,320 | 217,277,936 |
| Heisenberg 512, `build_model` (µs) | 390.305 / 390.301 | 377.348 / 377.344 | 13,922 | 758,128 |

The allocation counts and bytes in this table match the initial checkpoint.
Use the follow-up data as the final Stage 2 comparison point.
Workspace tests and doctests with all features, all-feature Clippy, the
all-targets check without default features, and formatting passed.
[Timing](operand-storage/timings.csv) · [Allocations](operand-storage/allocations.csv) · [Diagnostics](operand-storage/diagnostics.csv)
