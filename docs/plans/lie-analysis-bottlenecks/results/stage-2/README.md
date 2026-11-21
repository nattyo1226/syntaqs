# Stage 2 Gram reuse in center extraction

Center extraction now builds the source Gram matrix once and uses it
for both coordinate transforms. In all 27 profiled cases, Gram
evaluations per center extraction fell from two to one, both for
`center()` and for the center phase of `decompose()`.
[Phase profiles and work counts](hotspots.csv.gz).

| Case | Center before → after (ms) | Decompose before → after (ms) |
| --- | ---: | ---: |
| `heisenberg-16` | 1.09 → 0.61 | 2.07 → 1.59 |
| `pauli-z-4` | 0.81 → 0.48 | 1.32 → 1.00 |

The table uses uninstrumented medians. The instrumented Gram calls
inside `center()` took 0.94 → 0.48 ms for `heisenberg-16` and
0.59 → 0.30 ms for `pauli-z-4`.
[Public timings](timings.csv), [site scaling samples](site-scaling.csv).

On all 34 distinct workloads, the center/complement coordinates,
centrality and nullspace diagnostics, and the decomposition coordinates
and diagnostics matched between builds. Closure structure constants and
diagnostics also matched. All 341 targeted release tests and doctests
passed across `syntaqs-core`, `syntaqs-lie`, and `syntaqs-tests`, including
the existing `f32` and `f64` center and ideal subspace tests.
[Invariant comparisons](invariants.csv), [closure diagnostics](diagnostics.csv).

## Measurement and reproduction

The baseline is `6bff96703916845deda0964338dbc17516ae2dbb`.
The candidate includes this stage's edits to `family/projection.rs`,
`family/transform.rs`, and `invariant.rs` in `syntaqs-lie`.
Both use the pinned dependencies, `f64` driver settings, and
single-threaded OpenBLAS 0.3.33 on Apple M4 with macOS 15.8.1 arm64 and
Rust and Cargo 1.99.1. The primary cases use five paired runs per case.
The site sweep and profiles use three. Execution order alternates across
cases and trials.
[Shared measurement setup](../../README.md#measurement-and-reproduction).

From a stage 2 checkout, adapt the [stage 1 build recipe](../stage-1/README.md#measurement-and-reproduction).
Archive the baseline above for both builds. Use this `post` setup in
the build loop.

```bash
if [[ $side == post ]]; then
  cp crates/syntaqs-lie/src/family/{projection,transform}.rs \
    "$snapshot/crates/syntaqs-lie/src/family/"
  cp crates/syntaqs-lie/src/invariant.rs "$snapshot/crates/syntaqs-lie/src/"
fi
```

Apply [center-diagnostics.patch](../../probe/center-diagnostics.patch)
alongside the closure diagnostics patch for the diagnostics builds.
For the profile builds, use the same phase timers as stage 1 and apply
[center-gram-profile.patch](../../probe/center-gram-profile.patch) to
count completed `gram()` calls. Use the paired runner with the new labels.

```bash
python3 "$probe/measure.py" "$work/bin" "$work/results" \
  --revisions 6bff967 6bff967+center-gram \
  --suite timings --suite site-scaling --suite hotspots --suite diagnostics
```

The invariant comparison runs both diagnostics binaries in `all` mode
once per distinct workload in the primary and site suites. It compares
the lines beginning with `# invariant ` and records the number of matched
lines and the SHA-256 of those lines joined by newline characters.
