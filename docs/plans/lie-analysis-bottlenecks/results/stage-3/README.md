# Stage 3 residual diagnostics for exact-zero constraints

For exact-zero constraints, rank analysis now supplies the proof used
to construct residual diagnostics directly. On the abelian `pauli-z-4`
case, all 225 nullspace residual checks matched, and the full adjoint
commutant took 9.85 → 0.845 ms, a median speedup of 11.7×.
[Public timings](timings.csv), [diagnostic comparisons](invariants.csv).

| Metric for `pauli-z-4` | Baseline | Exact-zero diagnostics |
| --- | ---: | ---: |
| Peak process RSS through full commutant | 17.48 MiB | 11.22 MiB |
| Dense residual products per full commutant | 1 | 0 |

The eliminated product took 6.63 ms in the instrumented baseline.
The nonzero `pauli-2` and `fermion-4` constraints each still ran the
product. [RSS samples](memory.csv), [internal profiles](hotspots.csv.gz).

The commutant, center, and decomposition diagnostics matched on all 34
distinct workloads, as did their retained center and component coordinates
and the closure diagnostics. All 345 targeted release tests and doctests
passed across `syntaqs-core`, `syntaqs-lie`, and `syntaqs-tests`. The
new tests cover exact-zero and empty constraints and rejection of a
nonzero constraint classified as rank zero, in both `f32` and `f64`.
[Closure comparison](diagnostics.csv),
[residual regression tests](../../../../../crates/syntaqs-lie/src/invariant/tests.rs).

## Measurement and reproduction

The baseline is `722793719f26a3f01e55d6c64d08ed5902d15cbf`. The candidate
includes this stage's edits to `invariant.rs` and `numerical/kernels.rs`
in `syntaqs-lie`. Both builds used the pinned dependencies and `f64`
driver settings on Apple M4, macOS 15.8.1 arm64, Rust and Cargo 1.99.1,
and OpenBLAS 0.3.33 with one thread. The public API medians use five
fresh processes per workload and revision. RSS endpoints and phase
profiles use three. Execution order alternates across cases and
trials. [Shared measurement setup](../../README.md#measurement-and-reproduction).

From a stage 3 checkout, adapt the [stage 1 build recipe](../stage-1/README.md#measurement-and-reproduction).
Archive the baseline above for both builds and use the following
`post` setup in the build loop.

```bash
if [[ $side == post ]]; then
  cp crates/syntaqs-lie/src/invariant.rs "$snapshot/crates/syntaqs-lie/src/"
  cp crates/syntaqs-lie/src/numerical/kernels.rs \
    "$snapshot/crates/syntaqs-lie/src/numerical/"
fi
```

For the diagnostics builds apply [center-diagnostics.patch](../../probe/center-diagnostics.patch)
and [commutant-diagnostics.patch](../../probe/commutant-diagnostics.patch)
after the closure diagnostics patch. For the profile builds, apply
[constraint-residual-profile.patch](../../probe/constraint-residual-profile.patch)
after the stage 1 phase timer setup.

```bash
python3 "$probe/measure.py" "$work/bin" "$work/results" \
  --revisions 7227937 7227937+zero-residual \
  --suite timings --suite memory --suite hotspots --suite diagnostics
```

The invariant comparison runs both diagnostics binaries in `all` mode
once per distinct workload in the primary and site suites. It compares
the lines beginning with `# invariant ` and records the number of
matched lines and their SHA-256 after joining them by newlines.
