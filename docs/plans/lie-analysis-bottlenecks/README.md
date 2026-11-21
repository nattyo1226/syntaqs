# Lie analysis bottlenecks after the v6 merge

Closure became faster in all 15 primary workloads after the v6 merge.
The median of the per-case speedups was 1.28×. At fixed Lie dimension,
closure on the 16-site Heisenberg chain fell from 1.82 s to 63.9 ms. The
remaining closure cost is concentrated in repeated expression evaluation
for the Gram solve. Dense adjoint constraint SVDs still dominate runtime
and memory at larger Lie dimension.

## Public API measurements

The comparison uses the merge's first parent `3cadca3` and the merge
`c26ea49`, which incorporates `v6` at `89b4835`. Each input is a complete
Hermitian basis of a known Lie algebra. Let `D` be the physical Hilbert-space
dimension and `n` the real Lie dimension. The driver builds ordinary spin
and graded Majorana families, including mixed Pauli bases, abelian Pauli
strings, and collective spin operators with a Heisenberg Hamiltonian.
[Workload definitions](probe/benchmark.rs).

Times are medians in milliseconds across five fresh processes per revision
and case. Closure starts from a retained Gram factorization. The full
adjoint commutant and decomposition each start from the retained adjoint
representation. Decomposition includes its own center split. The CSV also
records Gram, factorization, adjoint, Killing form, and center timings.
[Public API samples](results/timings.csv).

| Case | D | n | Closure before → after | Full commutant before → after | Decompose before → after |
| --- | ---: | ---: | ---: | ---: | ---: |
| `spin-half-1` | 2 | 3 | 0.260 → 0.248 | 0.052 → 0.053 | 0.164 → 0.144 |
| `spin-half-2` | 4 | 6 | 1.791 → 1.463 | 0.205 → 0.201 | 0.813 → 0.736 |
| `spin-half-4` | 16 | 12 | 15.211 → 8.809 | 7.365 → 7.245 | 7.454 → 5.579 |
| `spin-half-2-in-8` | 256 | 6 | 3.483 → 1.466 | 0.198 → 0.199 | 1.597 → 0.732 |
| `pauli-2` | 4 | 15 | 29.580 → 28.295 | 27.709 → 27.798 | 34.722 → 37.617 |
| `pauli-2-mixed` | 4 | 15 | 76.353 → 64.871 | 28.067 → 28.116 | 36.918 → 39.420 |
| `spin-one-dipole` | 3 | 3 | 0.141 → 0.137 | 0.048 → 0.051 | 0.117 → 0.108 |
| `spin-one-cartesian` | 3 | 8 | 1.403 → 1.330 | 0.868 → 0.863 | 2.027 → 1.954 |
| `heisenberg-2` | 4 | 5 | 3.330 → 2.593 | 0.100 → 0.101 | 0.722 → 0.615 |
| `heisenberg-4` | 16 | 5 | 17.339 → 6.378 | 0.102 → 0.102 | 2.856 → 1.154 |
| `heisenberg-8` | 256 | 5 | 134.734 → 17.724 | 0.104 → 0.098 | 14.208 → 2.239 |
| `pauli-z-4` | 16 | 15 | 32.176 → 26.285 | 9.893 → 9.927 | 4.106 → 2.968 |
| `fermion-2` | 4 | 6 | 4.101 → 2.691 | 0.257 → 0.248 | 1.825 → 0.950 |
| `fermion-3` | 8 | 15 | 52.929 → 30.209 | 27.433 → 27.529 | 99.225 → 39.288 |
| `fermion-4` | 16 | 28 | 335.508 → 168.748 | 1,223.768 → 1,229.332 | 1,918.677 → 1,070.923 |

### Site scaling at fixed Lie dimension

The active Heisenberg chain adds interaction terms as the site count grows.
The control holds the active chain at two sites and adds spectators. The
graded control places two adjacent active modes at the front or tail of the
ordered space. The table shows selected medians in milliseconds from the
22-case sweep, with three fresh processes per revision and case.
[Site scaling samples](results/site-scaling.csv).

| Case | n | Closure before → after | Decompose before → after |
| --- | ---: | ---: | ---: |
| `heisenberg-2` | 5 | 3.264 → 2.578 | 0.711 → 0.616 |
| `heisenberg-16` | 5 | 1,823.796 → 63.855 | 87.879 → 4.634 |
| `heisenberg-2-in-64` | 5 | 44.946 → 2.693 | 12.774 → 0.653 |
| `fermion-front-2` | 6 | 4.075 → 2.693 | 1.801 → 0.952 |
| `fermion-front-64` | 6 | 54.797 → 2.984 | 59.132 → 1.056 |
| `fermion-tail-64` | 6 | 64.244 → 2.920 | 71.577 → 1.004 |

The core evaluator preserves disjoint support blocks, leaves spectator
identities implicit, and contracts graded operators using local parity
selection without materializing parity prefixes. This accounts for the
reduced spectator and placement costs. Growth with active support persists.
At `n=5`, post-merge closure takes about 64 ms with 16 active sites and
2.7 ms with two active sites embedded in 64. The full commutant remains near
0.1 ms in both because the adjoint constraint shape is the same.
[Evaluation model](../../design/composite-expression.md#11-storage-and-evaluation-implementation).

## Closure and Hilbert–Schmidt evaluation

The closure path projects every ordered bracket. For each bracket it checks
the norm and Hermiticity, computes the `n` right-hand-side entries of the
Gram system, solves using the retained LU, and checks the projection and
residual norms. The `n³` right-hand-side Hilbert–Schmidt (HS) calls still
collect both operands through the Lie adapter. The core HS contraction
then evaluates the rebuilt expressions before contracting the aligned blocks.
[Closure](../../../crates/syntaqs-lie/src/representation.rs#L244),
[Gram solve](../../../crates/syntaqs-lie/src/family/projection.rs#L325),
[Lie HS adapter](../../../crates/syntaqs-lie/src/expression.rs#L103),
[core HS contraction](../../../crates/syntaqs-core/src/evaluate/operator/contraction.rs#L232).

In the instrumented `fermion-4` run, right-hand-side construction takes
143.0 ms of a 196.2 ms closure. Across 21,952 HS calls in that phase, collecting
the bracket takes 75.7 ms, collecting the basis operand takes 23.8 ms, and
the core HS call takes 35.1 ms. LU solves take about 0.8 ms across closure.
On `heisenberg-16`, the right-hand side takes 25.5 ms of a 64.4 ms closure.
The two bracket norm and Hermiticity phases take another 28.9 ms together.
The old tensor-term merge scan is absent from the new evaluator.
[Internal profiles](results/hotspots.csv.gz).

### Direct core HS probe

The experimental patch passes composite expressions straight to core HS.
It keeps the Lie adapter's local-operator path and the decomposition
reconstruction path. Across ten selected workloads, this makes closure
1.43–1.81× faster. The test uses the decomposition endpoint, with times
measured per stage. Values below are medians in milliseconds over five
paired runs. [Experimental patch](probe/direct-hs.patch),
[direct HS samples](results/direct-hs.csv).

| Case | Closure current → direct HS | Decompose current → direct HS |
| --- | ---: | ---: |
| `heisenberg-16` | 64.39 → 44.89 | 4.66 → 2.12 |
| `pauli-2` | 28.16 → 17.39 | 38.24 → 31.58 |
| `pauli-2-mixed` | 64.80 → 42.74 | 40.37 → 33.23 |
| `fermion-4` | 169.12 → 107.18 | 1,048.16 → 1,021.50 |

The patch establishes a measured optimization candidate for these inputs.
Preserve the numerical behavior of general nonorthogonal projections,
cancellation-sensitive residuals, and both supported precisions when
evaluating it for adoption. Consider reuse of evaluated basis operands and
one evaluation per ordered bracket after measuring the remaining RHS cost.
Preserve collection across the entire residual expression so the
reconstructed projection can cancel against the target.
[Residual accuracy considerations](../../future-work/evaluation-improvements.md#1-lie-residual-accuracy).

## Dense adjoint constraints

For Lie dimension `n`, the full adjoint commutant builds a dense
`n³ × n²` constraint matrix and uses SVD with all right singular vectors.
With `f64`, the constraint input alone occupies `8n⁵` bytes. The usual
tall dense SVD work estimate is `O(n⁷)`. For a semisimple complement or
candidate ideal of dimension `d`, the symmetric solve projects a full
`d³ × d²` intermediate into `d³ × d(d+1)/2` constraints. The solver also
makes a working copy. An accepted split of a simple algebra solves the
largest symmetric problem once for the whole semisimple part and again
for the candidate ideal's verification.
[Full constraints](../../../crates/syntaqs-lie/src/invariant.rs#L195),
[symmetric constraints](../../../crates/syntaqs-lie/src/invariant.rs#L364),
[ideal validation](../../../crates/syntaqs-lie/src/decomposition/spectral.rs#L122),
[SVD kernel](../../../crates/syntaqs-lie/src/numerical/kernels.rs#L126).

- At `n=28`, `fermion-4` takes 1.23 s for the full commutant. Its SVD driver
  takes 1.15 s of the 1.21 s instrumented stage. The full constraint matrix
  is `21,952 × 784` (131.3 MiB), and the symmetric matrix is
  `21,952 × 406` (68.0 MiB). Peak process RSS after the merge is
  274.7 MiB through the full commutant and 279.0 MiB through decomposition,
  within 0.3 MiB of the corresponding pre-merge medians.
- In `fermion-4` decomposition, the component orthonormality check falls
  from 941.1 ms to 66.7 ms, while the combined SVD driver time is 860.8 ms
  after the merge. In `pauli-2` and `pauli-2-mixed`, the check grows from
  about 7.9 ms to 10.5–10.6 ms because the two operand collections increase
  from about 2.4 ms to 5.1 ms. This accounts for the measured 7–8% increase
  in those public decomposition timings.
- The abelian `pauli-z-4` uses the all-zero SVD shortcut. Dense residual
  validation still takes 9.1 ms of its 9.9 ms instrumented full commutant,
  with 6.7 ms in the constraint-by-nullspace multiplication.
  [Residual validation](../../../crates/syntaqs-lie/src/invariant.rs#L136).

Constructing the symmetric constraints directly is a memory optimization
candidate. At `d=28` it would remove the 131.3 MiB full intermediate while
retaining the same right singular problem. Changing the constraint rows or
eliminating per-ideal checks would require verifying rank decisions and
residual acceptance. A normal-equation solve would square the constraint
singular values and can impair numerical nullspace separation.
[Decomposition acceptance criteria](../../design/lie.md#7-complete-center-and-simple-ideal-decomposition).

## Measurement and reproduction

The reference runs took place on 2026-11-21 on Apple M4 with 24 GB of memory,
macOS 15.8.1 arm64, Rust and Cargo 1.99.1, and OpenBLAS 0.3.33. Both
revisions use the [same lockfile](probe/dependencies.lock) with
`ndarray` 0.17.2 and `ndarray-linalg` and `lax` 0.18.1. The release driver
uses `f64`, OpenBLAS with one thread, residual absolute and relative
tolerances of `1e-10`, and decomposition seed `816` with up to 16
attempts. The runner rotates revision order across cases and trials.
The timings characterize this host, backend, seed, and input families.

Public stages use wall time from `Instant`. Input construction, logging,
and stage-result assertions run outside their timers. The endpoint RSS
samples use three fresh processes per case, endpoint, and revision.
Each endpoint includes Gram, factorization, and closure, followed by the
requested commutant or decomposition where applicable. RSS covers the
entire process and comes from `wait4`. The internal profiles cover 27
cases with three runs per revision. Nested profile timers overlap and
add overhead. The pre- and post-merge patches follow their respective
evaluation paths and record common phase boundaries.
[Peak RSS samples](results/memory.csv), [pre-merge patch](probe/hotspots-pre.patch),
[post-merge patch](probe/hotspots-post.patch).

From the repository root, use Bash, Cargo, pkg-config, OpenBLAS, and
Python 3.9 or later on macOS or Linux. Keep the measured revisions
available in Git history. Build in a fresh temporary directory and write
a separate set of measurements there.

```sh
probe="$PWD/docs/plans/lie-analysis-bottlenecks/probe"
work=$(mktemp -d)
bash "$probe/build.sh" "$work"
python3 "$probe/measure.py" "$work/bin" "$work/results"
OPENBLAS_NUM_THREADS=1 "$work/bin/lie-stability"
```

Use `--suite timings`, `--suite memory`, `--suite site-scaling`,
`--suite hotspots`, `--suite direct-hs`, or `--suite diagnostics` to select
individual runs. The archived internal profile is stored as
`results/hotspots.csv.gz` and can be read with `gzip -dc`.

## Verification and completion criteria

All workload assertions for closure, center, full commutant, and ideal
dimensions passed on both merge revisions. Across the 34 distinct workloads,
all 42,145 structure constants and 2,689 ordered bracket diagnostics
matched numerically between revisions and between the merge and direct-HS
builds. The merge passed 341 release tests and doctests across
`syntaqs-core`, `syntaqs-lie`, and `syntaqs-tests` with the integration
crate's `lie` feature enabled. The [numerical stability probe](probe/stability_probe.rs)
also passed. [Diagnostic comparison](results/diagnostics.csv),
[stability probe output](results/stability.txt),
[numerical stability cases](comparison.md).

For a subsequent optimization, repeat the paired timings, RSS endpoints,
and numerical comparisons. Include ordinary, mixed-basis, graded, and
nonclosed inputs. Record the constraint dimensions and effective
tolerances for changes to the dense solver path.
