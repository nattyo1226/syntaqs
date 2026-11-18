# Internal bottlenecks in Lie analysis

Profiled on 2026-11-18 at `5be3033a1013b8d085cb85efca75b38ef658e041`. This
follow-up uses six cases from the original benchmark to attribute time inside
`GramFactorization::closure`, `AdjointRepresentation::commutant`, and
`AdjointRepresentation::decompose`. [Workloads](README.md#workloads).

- **Closure.** The largest phase in all six cases builds the right-hand
  side of the Gram solve. Each of its `n` contractions collects the same
  bracket again. The final projection residual also costs 43 ms on the
  eight-site Heisenberg case.
- **Full adjoint commutant.** The LAPACK SVD driver and its wrapper account
  for about 96% of the four-mode Majorana case. On the all-zero abelian
  constraint, dense residual multiplication is the largest cost.
- **Decomposition.** At four Majorana modes, the two symmetric commutant
  solves together take about 0.97 s, and the final orthonormality check takes
  0.93 s. In that check, enumerating homogeneous graded terms takes 478 ms,
  followed by 191 ms evaluating local products and traces. Reconstructing
  and collecting the output operator expressions takes about 1.5 ms.

## Closure and projection

For each of the `n²` ordered pairs, `bracket_projection` builds the bracket
expression and `project` checks its norm and Hermiticity. It obtains the
coordinates from `G c = b`, using `b_i = HS(B_i, bracket)`. The expression
adapter collects both operands of each `HS` call. This causes `n³` collections
of the right operand in the RHS phase, with `n` copies of the collection work
for each original bracket expression. The LU is retained across all bracket
pairs. `project` reconstructs the projected expression and evaluates both
its norm and the norm of the difference from the bracket. [Closure loop](../../../crates/syntaqs-lie/src/representation.rs#L244),
[projection phases](../../../crates/syntaqs-lie/src/family/projection.rs#L325),
[collection in the HS adapter](../../../crates/syntaqs-lie/src/expression.rs#L103).

The timings below are instrumented medians in milliseconds. Bracket norm,
Hermiticity, RHS, projection norm, and residual norm are separate phases.
Bracket construction, projection expression reconstruction, and bookkeeping
are included in the total.

| Case | RHS | Bracket norm | Hermiticity | Projection norm | Residual norm | LU solve | Closure |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `pauli-2` | 19.33 | 1.89 | 2.86 | 3.94 | 5.19 | 0.10 | 34.43 |
| `pauli-2-mixed` | 47.63 | 5.34 | 9.77 | 7.44 | 12.02 | 0.11 | 84.17 |
| `heisenberg-8` | 56.49 | 23.66 | 29.75 | 3.10 | 43.48 | 0.02 | 156.88 |
| `fermion-4` | 225.30 | 10.85 | 16.15 | 54.12 | 58.45 | 0.82 | 371.81 |

The Pauli basis change keeps the number of RHS contractions fixed at 3,375.
In these contractions, collecting the bracket grows from 8.92 to 29.01 ms.
The two operand collections together visit 10,125 intrinsic tensor terms for
`pauli-2` and 31,755 for `pauli-2-mixed`. At `fermion-4`, collecting the right
operand takes 91.90 ms across 21,952 RHS contractions. [Raw profiles](hotspots.csv).

The Heisenberg RHS spends 34.36 ms collecting the bracket and 18.91 ms
contracting the collected expressions. Within both operand collections,
the traversal, local evaluation, and merging loop takes 33.79 ms, of which
18.54 ms is inside `merge_tensor`. That function searches previously retained
terms and merges compatible factors. The reduced RHS call count on this
case, 125, still entails visiting 22,310 intrinsic tensor terms across both
operand collections. [Composite collection](../../../crates/syntaqs-core/src/evaluate/collect.rs#L142),
[merge implementation](../../../crates/syntaqs-core/src/evaluate/collect.rs#L76).

## Full adjoint commutant

The profile separates the input copy from `svd_into(false, true)`. This is
the same copy and consuming SVD used by `ndarray-linalg` 0.18.1 in its
borrowed-array `svd`. The driver timer includes layout handling, workspace
setup, the backend call, and wrapping the returned arrays. The constraint
residual checks include a Frobenius norm over the full matrix and the dense
product with the computed nullspace basis. [Constraint construction](../../../crates/syntaqs-lie/src/invariant.rs#L195),
[rank kernel](../../../crates/syntaqs-lie/src/numerical/kernels.rs#L126),
[residual checks](../../../crates/syntaqs-lie/src/invariant.rs#L136).

| Case | Constraint fill (ms) | SVD input copy (ms) | SVD driver (ms) | Residual checks (ms) | Full commutant (ms) |
| --- | ---: | ---: | ---: | ---: | ---: |
| `pauli-2` | 0.36 | 0.34 | 25.42 | 1.32 | 27.82 |
| `pauli-z-4` | 0.38 | — | — | 9.10 | 9.96 |
| `fermion-3` | 0.41 | 0.36 | 25.07 | 1.32 | 27.58 |
| `fermion-4` | 9.19 | 7.73 | 1164.24 | 28.84 | 1218.55 |

For `fermion-4`, the Frobenius norm alone accounts for 21.46 ms of the
remaining residual checks. For `pauli-z-4`, the all-zero constraint skips the
SVD and returns a basis of 225 null directions. Checking them multiplies the
`3375 × 225` constraint by a `225 × 225` basis. That dense product takes
6.64 ms of the 9.10 ms residual-check phase. [Zero-constraint shortcut](../../../crates/syntaqs-lie/src/numerical/kernels.rs#L134),
[raw profiles](hotspots.csv).

## Center and simple-ideal decomposition

The candidate-validation phase includes both the restricted center and the
symmetric commutant of each candidate ideal. The sampled eigensolve is
timed separately from those SVDs. After accepting a split, `component`
reconstructs and collects its expressions. The orthonormality check then
builds a Gram matrix for each emitted family and verifies the expressions.
[Candidate validation](../../../crates/syntaqs-lie/src/decomposition/spectral.rs#L122),
[reconstruction](../../../crates/syntaqs-lie/src/decomposition/spectral.rs#L332),
[orthonormality](../../../crates/syntaqs-lie/src/family/transform.rs#L117).

| Case | Center (ms) | Semisimple commutant (ms) | Sample and eigensolve (ms) | Candidate validation (ms) | Reconstruct and collect (ms) | Orthonormality check (ms) | Decomposition (ms) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `pauli-2` | 1.91 | 11.97 | 0.02 | 13.27 | 0.27 | 7.97 | 35.47 |
| `heisenberg-8` | 10.27 | 0.02 | <0.01 | 0.02 | 0.32 | 3.81 | 14.45 |
| `fermion-3` | 3.92 | 12.58 | 0.02 | 13.26 | 0.40 | 71.90 | 102.40 |
| `fermion-4` | 16.93 | 474.16 | 0.06 | 496.08 | 1.49 | 927.67 | 1918.07 |

For `fermion-4`, the candidate's symmetric commutant accounts for 494.26 ms of
the 496.08 ms candidate-validation phase. The LAPACK driver takes 422.98 ms in
the semisimple solve and 436.00 ms in the candidate solve. For `heisenberg-8`,
most decomposition time is in the center phase, which takes 5.23 ms to
orthonormalize the source family and 5.00 ms to recompute the Gram matrix for
the inverse coordinate transformation. [HS orthonormal coordinates](../../../crates/syntaqs-lie/src/invariant.rs#L219).

### Expression verification after the split

For `pauli-2` and `fermion-3`, the output checks make the same 240 HS calls.
The collected Pauli expressions produce 3,600 ordinary tensor terms during
those contractions. The graded Majorana expressions produce 18,225. On the
four-mode Majorana case, the walker enumerates 200,704 terms and requests
319,872 local traces. The term counter measures iterations of the ordinary
trace walker after the adapter has collected both input expressions.
[Ordinary term walker](../../../crates/syntaqs-core/src/expression/traversal.rs#L353),
[trace loop](../../../crates/syntaqs-core/src/evaluate/trace.rs#L111).

| Case | HS contraction (ms) | Term iterator (ms) | Graded enumeration (ms) | Parity lowering (ms) | Local trace reduction (ms) | Tensor terms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `pauli-2` | 5.22 | 0.33 | — | — | 3.98 | 3,600 |
| `fermion-3` | 60.32 | 38.08 | 32.90 | 3.97 | 14.77 | 18,225 |
| `fermion-4` | 839.68 | 549.29 | 478.42 | 59.05 | 190.93 | 200,704 |

Graded enumeration is `HomogeneousTerms::next`. It walks the sum/product
branches of the HS expression, evaluates local expressions into parity
parts, and builds homogeneous placement branches. Parity lowering then
converts each yielded term to ordinary factors. Even the 903,168 inserted
parity factors in the four-mode check take less elapsed time than enumerating
the homogeneous terms. The local reduction timer covers factor lookup, local
product evaluation, normalized trace, and exact-zero early exit. The iterator
timer includes both graded enumeration and lowering, and the contraction
timer includes the iterator and local reduction. [Homogeneous enumeration](../../../crates/syntaqs-core/src/expression/traversal.rs#L254),
[parity lowering](../../../crates/syntaqs-core/src/expression/traversal.rs#L377).

## Verification and reproduction

The baseline and instrumented release binaries used identical Rust sources
and benchmark inputs before applying the instrumentation patch. The runs
used the same Apple M4 environment, `f64` settings, and one OpenBLAS thread
as the original benchmark. Three paired trials ran each case once per build
in fresh processes. Tables show medians of the instrumented runs.
[Environment and numerical settings](README.md#measurement-and-reproduction),
[samples from both builds](hotspots.csv).

The instrumentation adds nested wall timers and integer counters. In paired
runs the instrumented closure medians were 7–17% higher. The four-mode full
commutant and decomposition medians differed by less than 3% between builds.
Nested times overlap. The SVD driver interval aggregates layout and workspace
handling with the backend work.

Build the uninstrumented release binary following the isolated baseline setup
and use the source revision at the top of this report. Save that binary, then
apply the instrumentation patch to the same source snapshot. [Baseline build](README.md#measurement-and-reproduction),
[hotspot patch](hotspots.patch).

```sh
cp "$bench" "$work/baseline"
git -C "$work" apply "$plan/hotspots.patch"
cargo build --locked --release --manifest-path "$work/Cargo.toml" \
  -p examples --no-default-features --features lie --bin lie_analysis_bench
python3 "$plan/measure.py" "$bench" "$work/results" --hotspots "$work/baseline"
```

The six cases pass the driver's closure, commutant, center, and simple-ideal
dimension assertions in both builds. The 111 Lie crate tests and the six core
contraction early-exit tests passed in release. `hotspots.csv` records build, public
stage, inner phase, operation, elapsed milliseconds, and the relevant counts.
`trace_parity_insertions` stores the number of inserted factors in the `calls`
column. The time for inserting them is included in `trace_graded_lower`.
