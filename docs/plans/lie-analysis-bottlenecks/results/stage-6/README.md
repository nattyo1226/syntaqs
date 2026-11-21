# Stage 6 prepared closure right-hand sides

Closure reuses the evaluated basis across ordered brackets and prepares each
target once for the right-hand side (RHS) of its Gram solve. The change made
`fermion-4` closure 3.51× faster than stage 5. In the stage 5 profile,
preparing both operands accounted for roughly 89% of the RHS time on
`fermion-4` and 92% on `pauli-2-mixed`.
[RHS profiles](hotspots.csv).

| Workload | Closure before → after (ms) | RHS before → after (ms) | One-time basis preparation (ms) | Peak RSS before → after (MiB) |
| --- | ---: | ---: | ---: | ---: |
| `fermion-4` | 107.24 → 30.54 | 83.01 → 5.22 | 0.018 | 4.84 → 4.88 |
| `pauli-2-mixed` | 36.28 → 12.43 | 26.00 → 2.11 | 0.025 | 4.48 → 4.53 |
| `heisenberg-16` | 4.88 → 2.94 | 2.31 → 0.29 | 0.049 | 4.55 → 4.58 |

The public closure time and peak RSS are medians for seven uninstrumented
processes per workload and build. RHS and one-time basis preparation
are measured separately in five instrumented processes. The RHS phase
includes target preparation, contractions, and conversion to real entries.
[Public timings and RSS](timings.csv), [operand preparation counts](hotspots.csv).

| Workload | Basis preparations before → after | Target preparations before → after | RHS contractions |
| --- | ---: | ---: | ---: |
| `fermion-4` | 21,952 → 28 | 21,952 → 784 | 21,952 |
| `pauli-2-mixed` | 3,375 → 15 | 3,375 → 225 | 3,375 |
| `heisenberg-16` | 125 → 5 | 125 → 25 | 125 |

These preparation counts cover only the RHS. Closure computes the target
norm and checks Hermiticity before forming the RHS. The span check
evaluates the full difference between the bracket and its reconstructed
projection. For placed operators, one closure call holds the prepared
basis blocks. Core releases the target
blocks and their additional primitive handles after each RHS. Local
operators reuse operands collected by the existing local evaluator.
[Closure contract](../../../../design/lie.md#4-brackets-closure-and-retained-representations),
[prepared HS implementation](../../../../../crates/syntaqs-core/src/evaluate/operator/contraction.rs).

All 42,145 structure constants and 2,689 ordered closure checks matched
stage 5 across 34 workloads. The invariant checks and retained center and
component coordinates matched on the same workloads. All 353 targeted
release tests and doctests passed. The added tests check successive
targets with different graded primitives, complex overlaps, and recovery
after invalid-space input in `f32` and `f64`. Targeted Clippy, formatting,
and workspace checks without default features passed.
[Closure comparisons](diagnostics.csv), [invariant comparisons](invariants.csv),
[regression tests](../../../../../tests/tests/lie_analysis.rs).

## Combined results

Against the v6 merge, closure became faster on all 15 primary workloads.
The median of the per-workload speedups was 4.42×. The largest active
Heisenberg chain finished closure in 2.94 ms, 21.74× faster. The abelian
`pauli-z-4` commutant gained from exact-zero residual diagnostics.
Direct symmetric constraints lowered `fermion-4` decomposition peak
RSS by 47.1%. The `fermion-4` full commutant still took about 1.21 s.
[Primary timings](combined/timings.csv), [site scaling](combined/site-scaling.csv),
[endpoint timings and RSS](combined/memory.csv).

| Workload and endpoint | v6 merge | Completed refactoring |
| --- | ---: | ---: |
| `fermion-4` closure | 169.90 ms | 30.81 ms |
| `pauli-2-mixed` closure | 65.16 ms | 12.40 ms |
| `heisenberg-16` closure | 64.00 ms | 2.94 ms |
| `pauli-z-4` commutant | 9.84 ms | 0.804 ms |
| `pauli-z-4` commutant peak RSS | 17.52 MiB | 11.27 MiB |
| `fermion-4` decomposition | 1,063.94 ms | 976.34 ms |
| `fermion-4` decomposition peak RSS | 279.77 MiB | 147.89 MiB |

Closure diagnostics, structure constants, invariant diagnostics, and the
retained center and component coordinates matched the v6 merge across all
34 workloads. [Combined closure comparisons](combined/diagnostics.csv),
[combined invariant comparisons](combined/invariants.csv).

## Measurement and reproduction

The stage 6 baseline is `d55bcc528ed0434671e28351a27d4af2e793d21a`.
The combined baseline is the v6 merge `c26ea49`. The candidate uses
the stage 6 crate sources listed below. The builds used Apple M4,
macOS 15.8.1 arm64, Rust and Cargo 1.99.1, the pinned dependencies,
the shared `f64` driver, and OpenBLAS 0.3.33 with one thread.
The primary combined comparison used five runs per workload and build.
Site scaling and endpoint RSS each used three. Paired suites alternate
the order of the two builds across workloads and trials.
[Shared setup](../../README.md#measurement-and-reproduction),
[RHS probe](../../probe/closure-rhs.py).

Run from the repository root with the stage 6 source and Cargo environment.

```bash
set -euo pipefail
export PYTHONDONTWRITEBYTECODE=1
probe="$PWD/docs/plans/lie-analysis-bottlenecks/probe"
work=$(mktemp -d)
mkdir "$work/bin" "$work/combined-bin"
driver=examples/src/bin/lie_analysis_bench.rs

build() {
  cargo build --locked --release --manifest-path "$snapshot/Cargo.toml" \
    --target-dir "$snapshot/target" -p examples --no-default-features \
    --features lie --bin lie_analysis_bench
  cp "$snapshot/target/release/lie_analysis_bench" "$work/bin/$1"
}

for side in pre post merged; do
  revision=d55bcc528ed0434671e28351a27d4af2e793d21a
  if [[ $side == merged ]]; then revision=c26ea49; fi
  snapshot="$work/$side"
  mkdir "$snapshot"
  git archive "$revision" | tar -x -C "$snapshot"
  cp "$probe/dependencies.lock" "$snapshot/Cargo.lock"
  if [[ $side == post ]]; then
    for file in crates/syntaqs-core/src/evaluate/operator/contraction.rs \
      crates/syntaqs-lie/src/{lib,expression,representation,family/projection}.rs; do
      cp "$file" "$snapshot/$file"
    done
  fi
  mkdir -p "$snapshot/examples/src/bin"
  cp "$probe/benchmark.rs" "$snapshot/$driver"
  build "$side"
  git -C "$snapshot" apply "$probe/closure-diagnostics.patch" \
    "$probe/center-diagnostics.patch" "$probe/commutant-diagnostics.patch"
  build "$side-diagnostics"
  if [[ $side != merged ]]; then
    cp "$probe/benchmark.rs" "$snapshot/$driver"
    git -C "$snapshot" apply \
      --include=crates/syntaqs-core/src/lib.rs \
      --include=crates/syntaqs-core/src/hotspot.rs \
      --include="$driver" "$probe/hotspots-post.patch"
    python3 "$probe/closure-rhs.py" prepare "$snapshot"
    build "$side-profile"
  fi
done

for name in pre pre-diagnostics post post-diagnostics; do
  source="$name"
  case "$name" in pre*) source="merged${name#pre}" ;; esac
  ln -s "$work/bin/$source" "$work/combined-bin/$name"
done

python3 "$probe/measure.py" "$work/bin" "$work/stage" \
  --revisions d55bcc5 d55bcc5+rhs-reuse --suite diagnostics
python3 "$probe/closure-rhs.py" measure "$work/bin" "$work/stage" \
  --revisions d55bcc5 d55bcc5+rhs-reuse
python3 "$probe/closure-rhs.py" invariants "$work/bin" "$work/stage/invariants.csv" \
  --revisions d55bcc5 d55bcc5+rhs-reuse

python3 "$probe/measure.py" "$work/combined-bin" "$work/combined" \
  --revisions c26ea49 d55bcc5+rhs-reuse --suite timings --suite site-scaling \
  --suite memory --suite diagnostics
python3 "$probe/closure-rhs.py" invariants "$work/combined-bin" \
  "$work/combined/invariants.csv" --revisions c26ea49 d55bcc5+rhs-reuse
```
