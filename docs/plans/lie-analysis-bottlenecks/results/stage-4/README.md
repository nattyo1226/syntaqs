# Stage 4 direct symmetric adjoint constraints

Direct construction removes the full-commutant matrix from the
symmetric solve. For `fermion-4`, each symmetric solve now builds the
desired `21,952 × 406` matrix directly. The intermediate
`21,952 × 784` array held 131.3 MiB of `f64` data. Peak process RSS
through decomposition fell by 46.8%.
[RSS samples](memory.csv), [internal profiles](hotspots.csv.gz).

| Metric for `fermion-4` | Baseline | Direct construction |
| --- | ---: | ---: |
| Peak RSS through decomposition | 279.39 MiB | 148.67 MiB |
| Decomposition time in that endpoint | 1,013.72 ms | 987.21 ms |
| Symmetric constraint construction | 49.77 ms | 29.96 ms |
| Symmetric constraint SVD, including input copy | 883.65 ms | 907.66 ms |
| Full intermediate matrices in the two symmetric solves | 2 | 0 |

The first two rows are medians for fresh uninstrumented processes in
`decompose` mode. The phase timings sum the semisimple and ideal
symmetric solves in each instrumented process before taking medians.
The SVD uses the same input shape and rank settings. In these runs
the per-process SVD totals ranged from 880 to 951 ms before the change
and from 875 to 931 ms afterward.
[Public timings](timings.csv), [internal profiles](hotspots.csv.gz).

The coefficient regression test matches the previous full projection
bit for bit in `f32` and `f64`, including shared coefficient contributions.
All 34 workload comparisons matched for commutant, center, decomposition,
and closure diagnostics, including the retained center and component
coordinates. The 347 targeted release tests and doctests passed across
`syntaqs-core`, `syntaqs-lie`, and `syntaqs-tests`.
[Invariant comparisons](invariants.csv), [closure comparison](diagnostics.csv),
[coefficient regression test](../../../../../crates/syntaqs-lie/src/invariant/tests.rs).

## Measurement and reproduction

The baseline is `2fa250c5e0cfe51ddbcc0d241248ffdeb3666b99`. The candidate
uses this stage's `invariant.rs` from `syntaqs-lie`. Both builds used
the pinned dependencies, `f64` driver settings, and single-threaded
OpenBLAS 0.3.33 on Apple M4, macOS 15.8.1 arm64, and Rust and Cargo
1.99.1. There were five paired trials for the public timings and three
for the RSS endpoints and profiles.
[Shared measurement setup](../../README.md#measurement-and-reproduction).

Adapt the [stage 1 build recipe](../stage-1/README.md#measurement-and-reproduction)
from a stage 4 checkout. Archive the baseline above for both builds
and use the following `post` setup in the build loop.

```bash
if [[ $side == post ]]; then
  cp crates/syntaqs-lie/src/invariant.rs "$snapshot/crates/syntaqs-lie/src/"
fi
```

Use the [stage 3 diagnostic patches](../stage-3/README.md#measurement-and-reproduction)
for the diagnostics binaries. After applying the stage 1 phase timers
to each profile snapshot, add the symmetric constraint instrumentation.
[Profiling script](../../probe/profile-symmetric-constraints.py).

```bash
python3 "$probe/profile-symmetric-constraints.py" "$snapshot"
```

Run the paired suites with the stage labels. Compare the diagnostic
lines and record their hashes as in stage 3.

```bash
python3 "$probe/measure.py" "$work/bin" "$work/results" \
  --revisions 2fa250c 2fa250c+symmetric-direct \
  --suite timings --suite memory --suite hotspots --suite diagnostics
```
