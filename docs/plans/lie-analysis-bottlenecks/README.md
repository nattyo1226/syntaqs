# Lie analysis bottlenecks

Measured on 2026-11-18 at revision `690b7b4ca6a59f88c48d0310d9faa80ab258aff1`.

## Findings

- **Closure follows expression cost.** The Heisenberg family has `n=5`
  throughout, and its closure time grows from 3.36 ms on two sites to 136 ms
  on eight. At fixed `D=4` and `n=15`, mixing pairs of Pauli strings raises
  closure time from 29.58 to 77.18 ms. Term collection and HS contraction
  account for most of the profiled closure time in both comparisons.
- **Dense adjoint constraints are the largest memory cost.** At `n=28`, the
  four-mode Majorana family takes 1.21 s for the full adjoint commutant and
  1.92 s for decomposition. Separate runs through these stages peak at 275
  and 280 MiB, respectively.
- **Decomposition can also be expression-bound.** For three Majorana modes
  (`n=15`), the output component's orthonormality check takes about 70 ms of
  a 100 ms instrumented split. At four modes, the check and the constraint
  SVDs take about 0.94 and 0.95 s, respectively.

## Objective and scope

The investigation measures existing `syntaqs-lie` APIs on known closed operator
families from the spin-half, spin-one, and fermion crates. It varies the physical
Hilbert-space dimension `D`, the real Lie algebra dimension `n`, and the operator
expressions used as a basis. Each workload supplies a complete basis for closure
verification. The timings start at Gram evaluation after input construction.

The investigation is complete. It inspected source paths, measured eight public
stages for 15 families, measured peak RSS for selected endpoints, and instrumented
the dominant stages in a temporary copy. The driver checks closure and the
expected center, full adjoint commutant, and simple-ideal dimensions on each full
run.

## Workloads

| Case | Construction | Lie algebra |
| --- | --- | --- |
| `spin-half-1`, `spin-half-2`, `spin-half-4` | `Sx`, `Sy`, `Sz` on each of 1, 2, or 4 sites | One copy of `su(2)` per site |
| `spin-half-2-in-8` | The `spin-half-2` operators on the first two sites with six spectator factors | `su(2) ⊕ su(2)` |
| `pauli-2`, `pauli-2-mixed` | All 15 nonidentity two-site Pauli strings. The mixed basis replaces adjacent pairs by their sum and difference, leaving the final string on its own | `su(4)` |
| `spin-one-dipole`, `spin-one-cartesian` | One spin-1 site. Three dipoles, or those dipoles together with five Cartesian quadrupoles | `su(2)`, `su(3)` |
| `heisenberg-2`, `heisenberg-4`, `heisenberg-8` | The isotropic, open-chain Heisenberg Hamiltonian at coupling `1`, the three total Pauli components, and identity | `su(2) ⊕ u(1) ⊕ u(1)` |
| `pauli-z-4` | All 15 nonidentity strings of `I` and `Sz` on four sites | `u(1)^15` |
| `fermion-2`, `fermion-3`, `fermion-4` | All Hermitian Majorana bilinears `-i γ_a γ_b` for `a < b`, on 2, 3, or 4 graded modes | `so(4)`, `so(6)`, `so(8)` |

The workload definitions and checks are in the [benchmark driver](benchmark.rs).

## Public API timings

Values are median wall times in milliseconds across five runs per case. `Factor`
includes Hermiticity and independence checks and real LU factorization on the
retained Gram matrix. `Closure` uses that factorization. `Adjoint` includes
converting the closure report to structure constants. Killing form, full adjoint
commutant, center split, and decomposition each start from the same retained
adjoint representation. Decomposition includes its own center split. Times are
rounded to 0.001 ms. [Timing samples](timings.csv).

| Case | `D` | `n` | Gram | Factor | Closure | Adjoint | Killing | Commutant | Center | Decompose |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `spin-half-1` | 2 | 3 | 0.056 | 0.078 | 0.277 | 0.001 | 0.000 | 0.056 | 0.061 | 0.165 |
| `spin-half-2` | 4 | 6 | 0.141 | 0.071 | 1.819 | 0.001 | 0.001 | 0.208 | 0.263 | 0.783 |
| `spin-half-4` | 16 | 12 | 0.761 | 0.100 | 16.210 | 0.003 | 0.015 | 7.306 | 1.579 | 7.704 |
| `spin-half-2-in-8` | 256 | 6 | 0.326 | 0.083 | 3.495 | 0.001 | 0.001 | 0.200 | 0.640 | 1.611 |
| `pauli-2` | 4 | 15 | 0.718 | 0.112 | 29.577 | 0.003 | 0.035 | 27.888 | 1.694 | 34.824 |
| `pauli-2-mixed` | 4 | 15 | 1.421 | 0.158 | 77.175 | 0.003 | 0.035 | 28.132 | 3.200 | 36.880 |
| `spin-one-dipole` | 3 | 3 | 0.034 | 0.055 | 0.142 | 0.001 | 0.000 | 0.053 | 0.033 | 0.114 |
| `spin-one-cartesian` | 3 | 8 | 0.071 | 0.060 | 1.359 | 0.002 | 0.003 | 0.872 | 0.150 | 2.017 |
| `heisenberg-2` | 4 | 5 | 0.226 | 0.076 | 3.364 | 0.001 | 0.001 | 0.102 | 0.464 | 0.727 |
| `heisenberg-4` | 16 | 5 | 0.950 | 0.116 | 17.763 | 0.001 | 0.001 | 0.102 | 1.922 | 2.907 |
| `heisenberg-8` | 256 | 5 | 4.920 | 0.245 | 136.003 | 0.001 | 0.001 | 0.108 | 10.082 | 14.072 |
| `pauli-z-4` | 16 | 15 | 1.117 | 0.126 | 31.980 | 0.003 | 0.035 | 9.804 | 2.507 | 4.086 |
| `fermion-2` | 4 | 6 | 0.243 | 0.086 | 4.051 | 0.001 | 0.001 | 0.255 | 0.475 | 1.790 |
| `fermion-3` | 8 | 15 | 1.661 | 0.144 | 53.030 | 0.003 | 0.035 | 27.600 | 3.583 | 99.137 |
| `fermion-4` | 16 | 28 | 7.026 | 0.274 | 335.538 | 0.022 | 0.421 | 1213.329 | 15.799 | 1922.868 |

## Expression evaluation and closure

Closure checks every ordered pair, including the diagonal. For each pair it
constructs the bracket, computes its norm and Hermiticity defect, contracts it
against each basis operator, solves for coefficients using the existing LU,
and evaluates the projection and residual norms. The result stores `n³` real
coefficients. The current path therefore makes `n²(n+4)` normalized
Hilbert–Schmidt (HS) contractions through the family adapter. Each adapter call
collects the natural terms of both operands before contraction. [Closure](../../../crates/syntaqs-lie/src/representation.rs#L240),
[projection](../../../crates/syntaqs-lie/src/family/projection.rs#L325),
[expression adapter](../../../crates/syntaqs-lie/src/expression.rs#L103).

The composite evaluator traverses tensor factors and term products. It groups
local expressions, supplies spectator identities during collection, and merges
compatible tensor terms. HS uses normalized local traces. The amount of work
depends on factor count, terms in sums and products, and cancellations. Dense
numeric arrays use family coordinates. [Term collection](../../../crates/syntaqs-core/src/evaluate/collect.rs#L142),
[HS contraction](../../../crates/syntaqs-core/src/evaluate/trace.rs#L193).

The spectator control increases `D` by 64× at fixed `n=6` and increases closure
time by 1.9×. In the Heisenberg series, each total spin axis grows from 2 to 8
terms and the Hamiltonian grows from 3 to 21 bond terms. Closure increases by
about 40× while the full commutant stays close to 0.1 ms. At fixed physical
dimension `D=3`, the spin-1 Cartesian basis increases `n` from 3 to 8 and
increases decomposition time from 0.11 to 2.02 ms.

Instrumented closure medians across three runs per case show the cost of
collecting both operands of every HS call. `HS calls` counts calls to the
family adapter. `LU solve` sums all retained-factor solves in the stage.
[Substage samples](profile.csv).

| Case | HS calls | Collection (ms) | HS contraction (ms) | LU solve (ms) | Closure (ms) |
| --- | ---: | ---: | ---: | ---: | ---: |
| `heisenberg-8` | 225 | 96.19 | 35.71 | 0.018 | 133.94 |
| `pauli-2` | 4275 | 22.46 | 6.58 | 0.118 | 31.92 |
| `pauli-2-mixed` | 4275 | 61.04 | 12.61 | 0.107 | 77.82 |
| `fermion-4` | 25088 | 260.82 | 78.70 | 0.867 | 358.56 |

## Adjoint constraints and decomposition

The full adjoint commutant has `n²` unknown matrix entries and `n³` scalar
constraints. The implementation builds a dense `n³ × n²` array and uses
`svd(false, true)` to obtain all right singular vectors. The installed
`ndarray-linalg` copies the constraint array for the destructive LAPACK SVD.
With `f64`, the input alone takes `8n⁵` bytes. The usual dense tall-SVD work
estimate is `O(n⁷)` from rows times columns squared. The center kernel uses
`n² × n` constraints. Its public method also evaluates the Gram matrix during
orthonormalization and again to form the coordinate inverse. The Killing form
uses `n⁴` scalar products. [Constraints and center](../../../crates/syntaqs-lie/src/invariant.rs#L189),
[SVD kernel](../../../crates/syntaqs-lie/src/numerical/kernels.rs#L126),
[Killing form](../../../crates/syntaqs-lie/src/representation.rs#L121).

For a semisimple complement or a candidate ideal of dimension `d`, the
symmetric commutant builds `d³ × d²` full constraints and projects them into
`d³ × d(d+1)/2` symmetric constraints. Both arrays coexist with the SVD input
copy during factorization. The decomposition solves the semisimple symmetric
commutant once, then solves each candidate ideal's symmetric commutant during
verification. A simple algebra of dimension `n` therefore incurs two of the
largest symmetric solves in an accepted first attempt. Validation can repeat
per candidate when sampling retries. After the split, the emitted operator
expressions are reconstructed, collected, and checked for HS orthonormality.
[Symmetric constraints](../../../crates/syntaqs-lie/src/invariant.rs#L364),
[spectral decomposition](../../../crates/syntaqs-lie/src/decomposition/spectral.rs).

At `n=15`, the Pauli and three-mode Majorana full commutants both take about
28 ms. At `n=28`, the full commutant SVD call, including its input copy, takes
1.16 s of the 1.21 s instrumented stage. The abelian `pauli-z-4` takes the
all-zero SVD shortcut. It still verifies the entire `n²`-dimensional nullspace
with dense constraint multiplication and residual checks, taking 9.1 ms of the
9.9 ms instrumented commutant. Its decomposition skips the symmetric solves
because the center spans the algebra. [Zero-constraint path](../../../crates/syntaqs-lie/src/numerical/kernels.rs#L134),
[residual checks](../../../crates/syntaqs-lie/src/invariant.rs#L136),
[substage samples](profile.csv).

The following decomposition times include the two smaller center kernel SVDs
in `Constraint SVDs`. The three- and four-mode Majorana outputs have one simple
ideal and all `n²` source-coordinate coefficients are nonzero in the measured
runs. Their HS rechecks therefore receive dense linear combinations of graded
bilinears. The two-site Pauli case has a dense reconstructed basis and its
recheck takes 7.63 ms. The comparison varies the physical representation and
grading as well as the resulting expressions. [Timing samples and coordinate
counts](timings.csv), [substage samples](profile.csv).

| Case | Constraint SVDs (ms) | Building symmetric constraints (ms) | Component orthonormality (ms) | Decomposition (ms) |
| --- | ---: | ---: | ---: | ---: |
| `pauli-2` | 20.76 | 1.95 | 7.63 | 35.53 |
| `fermion-3` | 21.26 | 1.90 | 69.93 | 99.81 |
| `fermion-4` | 948.80 | 51.93 | 936.09 | 2013.05 |

## Peak resident memory

Values are median process peak RSS in MiB across three fresh processes per
endpoint. All three modes compute Gram, factorization, and closure. Commutant
and decomposition each add the adjoint and their respective computation.
[RSS samples](memory.csv).

| Case | Through closure | Through commutant | Through decomposition |
| --- | ---: | ---: | ---: |
| `spin-half-2` | 4.36 | 4.86 | 4.92 |
| `pauli-2` | 4.42 | 17.97 | 17.38 |
| `pauli-z-4` | 4.42 | 17.41 | 4.70 |
| `heisenberg-8` | 4.67 | 5.03 | 5.09 |
| `fermion-4` | 4.80 | 275.00 | 279.50 |

At `n=28` the full commutant constraint is `21,952 × 784` (131.3 MiB). Its
symmetric counterpart is `21,952 × 406` (68.0 MiB). The full matrix, symmetric
matrix, and SVD input copy account for about 267.3 MiB of array payload
during a symmetric solve. Peak RSS includes the program, BLAS/LAPACK workspace,
and other allocations. By shape alone, the full traceless Pauli algebra on three
spins would have `D=8`, `n=63`, and a `250,047 × 3,969` full commutant constraint
of about 7.39 GiB per array. The three-spin full Pauli calculation is a size
estimate.

## Measurement and reproduction

- Apple M4, 24 GB, macOS 15.8.1 arm64, Rust 1.99.1, Cargo 1.99.1,
  `ndarray` 0.17.2, `ndarray-linalg` and `lax` 0.18.1, system OpenBLAS 0.3.33
  with `OPENBLAS_NUM_THREADS=1`. The binary linked to OpenBLAS as checked
  with `otool -L`.
- Cargo release profile, `f64` coefficients. The residual absolute and relative
  tolerances were both `1e-10`. Other numerical options used defaults. The
  decomposition seed was `816`, with up to 16 attempts. `pauli-z-4` took
  zero sampling attempts. Every other measured split took one.
- Each sample used a fresh process. Stage times use `Instant`. Input creation,
  correctness assertions, logging, and final result destruction are outside
  the stage intervals. Peak RSS is `wait4`'s `ru_maxrss` for the entire process.
  Substage timings come from a separate build with timers around selected calls.
  The timers add overhead. The component orthonormality timer includes its
  own HS work, so these measurements overlap. The `coordinate_tiny` column
  counts nonzero source-coordinate coefficients of magnitude at most `1e-12`.
- The timings characterize these inputs and this host. Thread count, BLAS
  backend, precision, basis structure, and sampling outcomes can affect the
  results. The study used supplied closed bases and one decomposition seed.

From the repository root, with Rust, Python 3, pkg-config, and OpenBLAS available,
build the driver against a temporary copy of the measured source. The commands
use a local `Cargo.lock` because the workspace ignores that file. The measured
lockfile SHA-256 was
`915aa770e0ec140f6766d8c429d184b0acc80eb6b27829c7c42ce3f86d0fd4cd`.
With a different lockfile, record its dependency versions with the measurements.

```sh
work=$(mktemp -d)
plan="$PWD/docs/plans/lie-analysis-bottlenecks"
git archive 690b7b4ca6a59f88c48d0310d9faa80ab258aff1 | tar -x -C "$work"
cp Cargo.lock "$work/"
cp "$plan/benchmark.rs" "$work/examples/src/bin/lie_analysis_bench.rs"
cargo build --locked --release --manifest-path "$work/Cargo.toml" \
  -p examples --no-default-features --features lie --bin lie_analysis_bench
bench="$work/target/release/lie_analysis_bench"
python3 "$plan/measure.py" "$bench" "$work/results"
```

To collect the substage timings, apply the [instrumentation patch](profile.patch)
to that temporary copy and rebuild.

```sh
git -C "$work" apply "$plan/profile.patch"
cargo build --locked --release --manifest-path "$work/Cargo.toml" \
  -p examples --no-default-features --features lie --bin lie_analysis_bench
python3 "$plan/measure.py" "$bench" "$work/results" --profile
```

## Verification

All 111 [Lie crate tests](../../../crates/syntaqs-lie/Cargo.toml), including
doctests, passed in release. The 14 shared `lie_analysis` and `lie_decomposition`
integration tests passed with the `lie` feature enabled and default features
disabled.
