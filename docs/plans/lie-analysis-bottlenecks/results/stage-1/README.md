# Stage 1 direct composite HS results

Direct core HS shortened closure by a median factor of 1.59 across the
15 primary cases. The Pauli component orthonormality checks took about
half as long. Closure diagnostics and structure constants matched
numerically on all 34 distinct workloads.
[Public timings](timings.csv), [component profiles](hotspots.csv.gz),
[numerical comparison](diagnostics.csv).

| Case | Closure before → after (ms) | Decompose before → after (ms) |
| --- | ---: | ---: |
| `pauli-2` | 27.87 → 17.50 | 37.48 → 31.27 |
| `pauli-2-mixed` | 65.04 → 42.46 | 39.67 → 32.94 |
| `heisenberg-16` | 63.64 → 45.01 | 4.58 → 2.04 |
| `fermion-4` | 169.56 → 107.32 | 1,067.19 → 1,033.32 |

In the instrumented builds, the `pauli-2` component check went from
10.37 to 5.02 ms. The `pauli-2-mixed` check went from 10.28 to 5.01 ms.
The recorded 42,145 structure constants and 2,689 ordered bracket
diagnostics matched. All 341 targeted release tests and doctests passed
across `syntaqs-core`, `syntaqs-lie`, and `syntaqs-tests`, including
`f32` and `f64` tests for tiny placed projection residuals.
[Site scaling samples](site-scaling.csv), [process peak RSS](memory.csv).

## Measurement and reproduction

The baseline is `4194d4200d2dae9a159d73ca9273472a61071a85`.
The candidate uses the composite adapter change in
[direct-hs.patch](../../probe/direct-hs.patch). Measurements used
Apple M4, macOS 15.8.1 arm64, Rust and Cargo 1.99.1, and OpenBLAS 0.3.33
with one thread. Both builds used the same release driver, pinned
dependencies, and `f64` numerical settings as the merge comparison.
The primary cases use five runs per revision. The RSS endpoints, site
sweep, and phase profiles use three. Each trial alternates build order
across cases. Stage 1 phase builds use matching timers around decomposition
and public stages, with the patch limited to the paths in the recipe below.
[Shared measurement setup](../../README.md#measurement-and-reproduction).

From the repository root, use the pinned source revision and the
existing benchmark driver to build the paired executables.

```bash
set -euo pipefail
probe="$PWD/docs/plans/lie-analysis-bottlenecks/probe"
work=$(mktemp -d)
mkdir "$work/bin"
build() {
  cargo build --locked --release --manifest-path "$snapshot/Cargo.toml" \
    --target-dir "$snapshot/target" -p examples --no-default-features \
    --features lie --bin lie_analysis_bench
  cp "$snapshot/target/release/lie_analysis_bench" "$work/bin/$1"
}

for side in pre post; do
  snapshot="$work/$side"
  mkdir "$snapshot"
  git archive 4194d4200d2dae9a159d73ca9273472a61071a85 | tar -x -C "$snapshot"
  cp "$probe/dependencies.lock" "$snapshot/Cargo.lock"
  if [[ $side == post ]]; then
    git -C "$snapshot" apply "$probe/direct-hs.patch"
  fi
  driver="$snapshot/examples/src/bin/lie_analysis_bench.rs"
  cp "$probe/benchmark.rs" "$driver"
  build "$side"
  git -C "$snapshot" apply "$probe/closure-diagnostics.patch"
  build "$side-diagnostics"
  cp "$probe/benchmark.rs" "$driver"
  git -C "$snapshot" apply \
    --include=crates/syntaqs-core/src/lib.rs \
    --include=crates/syntaqs-core/src/hotspot.rs \
    --include=crates/syntaqs-lie/src/decomposition/spectral.rs \
    --include=examples/src/bin/lie_analysis_bench.rs \
    "$probe/hotspots-post.patch"
  build "$side-profile"
done
python3 "$probe/measure.py" "$work/bin" "$work/results" \
  --revisions 4194d42 4194d42+direct-hs
```
