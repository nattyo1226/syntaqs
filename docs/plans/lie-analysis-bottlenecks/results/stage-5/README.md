# Stage 5 indexed commutators in closure

Indexed term selection cut `heisenberg-16` closure time from 45.08 to
4.88 ms, a 9.23× speedup. Collecting each bracket once with the existing
evaluator took 7.53 ms. The indexed path was 1.54× faster than that
control and finished ahead in all nine paired trials.
[Closure timings and peak RSS](timings.csv).

| Workload | Symbolic baseline (ms) | Eager collection (ms) | Indexed closure (ms) |
| --- | ---: | ---: | ---: |
| `heisenberg-2` | 1.55 | 0.59 | 0.73 |
| `heisenberg-8` | 12.28 | 2.69 | 2.49 |
| `heisenberg-12` | 25.89 | 4.81 | 3.72 |
| `heisenberg-16` | 45.08 | 7.53 | 4.88 |
| `heisenberg-2-in-64` | 1.59 | 0.60 | 0.76 |
| `pauli-2-mixed` | 42.70 | 16.76 | 36.00 |
| `fermion-4` | 107.26 | 44.30 | 106.79 |

The table reports medians for closure using a retained Gram
factorization. Eager collection has lower medians on shorter chains and
on the controls with fewer eligible brackets. Indexed closure was within
1.2% of the symbolic baseline on each measured fermion case. On
`spin-half-2` the median increased from 0.779 to 0.804 ms.
[Full timing samples](timings.csv).

For the 16-site Hamiltonian and total spin along one axis, each product
visited 90 pairs and built 90 terms, down from 720 of each. Across all
eligible products in that closure call, visits and terms built both
fell from 17,298 to 2,142. The median total index setup time was 0.019 ms.
The work measurement compares the same eligible products with a Cartesian
loop and with the index. Each row records one product direction before
coefficient aggregation. [Work counts](pairs.csv).

Core applies the index when both operands evaluate to overlapping single
sparse blocks. It retains the original product order for the selected
pairs and applies the existing local products and graded signs. Layouts
that do not qualify and coefficient bounds that may overflow use the
symbolic path. Lie selects this operation for closure targets. The
standalone bracket and bracket projection APIs still construct symbolic
targets. [Core selection](../../../../../crates/syntaqs-core/src/evaluate/operator/commutator.rs),
[closure contract](../../../../design/lie.md#4-brackets-closure-and-retained-representations).

The 42,145 structure constants and 2,689 ordered checks matched across
34 workloads, including residual norms, scales, and thresholds. All
351 targeted release tests and doctests passed across `syntaqs-core`,
`syntaqs-lie`, and `syntaqs-tests`. New tests cover ordinary and graded
mixed parity, two-site overlap, disjoint odd terms, overflow fallback,
and space validation in both precisions. Formatting and targeted Clippy
checks passed. [Diagnostic comparison](diagnostics.csv),
[regression tests](../../../../../tests/tests/lie_analysis.rs).

## Measurement and reproduction

The baseline is `833495d5f4df8384e2b6957ab798e9131eb3e33a`. The indexed
candidate uses the six crate source files listed below. The eager control
collects every placed closure bracket once with `collect_terms_in`.
Measurements used Apple M4, macOS 15.8.1 arm64, Rust and Cargo 1.99.1,
the pinned dependencies, and OpenBLAS 0.3.33 with one thread. The
`f64` driver ran nine fresh processes per case and variant, rotating
execution order. Work profiles ran three fresh processes per case and
variant. [Shared measurement setup](../../README.md#measurement-and-reproduction),
[probe](../../probe/commutator.py).

Run from the repository root with the stage 5 source and Cargo environment.

```bash
set -euo pipefail
export PYTHONDONTWRITEBYTECODE=1
probe="$PWD/docs/plans/lie-analysis-bottlenecks/probe"
work=$(mktemp -d)
mkdir "$work/bin"
expr=crates/syntaqs-lie/src/expression.rs
core=crates/syntaqs-core/src/evaluate/operator/commutator.rs
driver=examples/src/bin/lie_analysis_bench.rs

build() {
  cargo build --locked --release --manifest-path "$snapshot/Cargo.toml" \
    --target-dir "$snapshot/target" -p examples --no-default-features \
    --features lie --bin lie_analysis_bench
  cp "$snapshot/target/release/lie_analysis_bench" "$work/bin/$1"
}

for side in pre post; do
  snapshot="$work/$side"
  mkdir "$snapshot"
  git archive 833495d5f4df8384e2b6957ab798e9131eb3e33a | tar -x -C "$snapshot"
  cp "$probe/dependencies.lock" "$snapshot/Cargo.lock"
  if [[ $side == post ]]; then
    for file in "$expr" "$core" crates/syntaqs-core/src/evaluate/{operator,collect}.rs \
      crates/syntaqs-lie/src/{lib,representation}.rs; do
      cp "$file" "$snapshot/$file"
    done
  fi
  mkdir -p "$snapshot/examples/src/bin"
  cp "$probe/benchmark.rs" "$snapshot/$driver"
  build "$side"
  git -C "$snapshot" apply "$probe/closure-diagnostics.patch"
  build "$side-diagnostics"
  cp "$probe/benchmark.rs" "$snapshot/$driver"
done

for variant in eager indexed-profile all-pairs-profile; do
  python3 "$probe/commutator.py" prepare "$snapshot" "$variant"
  build "$variant"
  cp "$expr" "$snapshot/$expr"
  cp "$core" "$snapshot/$core"
done

python3 "$probe/commutator.py" measure "$work/bin" "$work/results"
python3 "$probe/measure.py" "$work/bin" "$work/comparison" \
  --revisions 833495d 833495d+indexed --suite diagnostics
```
