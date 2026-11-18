# Complexity and numerical stability of Lie algebra decomposition

Compared on 2026-11-18. The Julia reference is `LieAlgebra.jl` at `575029c` with
Julia 1.11.7. The Rust implementation is `syntaqs-lie` at `690b7b4`. Both split
ideals using the eigenspaces of a sampled element of the adjoint commutant.
The Julia reference samples on the whole algebra. Rust extracts the center,
samples on its HS-orthogonal complement, and verifies the resulting split.
[Design decision](../../decisions/lie-analysis.md#decisions),
[Julia decomposition](https://github.com/nattyo1226/LieAlgebra.jl/blob/575029c12fcad44b3b41206ee564437a7ec5fa45/src/decomposition.jl#L86).

The main complexity difference comes from the nullspace computation. The Julia
reference generates the full left singular-vector matrix as part of its SVD.
Rust computes only the right singular vectors, but runs additional SVDs to
verify simplicity and checks the final reconstructed expressions. On small,
well-conditioned orthonormal bases both split correctly. On the nonorthogonal,
abelian, and nonclosed cases below, their behavior differs before any issue of
eigensolver accuracy arises.

## Cost of forming the adjoints

Let `D` be the physical Hilbert-space dimension, `n` the number of basis
elements, and `d = n - z` the semisimple dimension, where `z` is the center
dimension. All complexity estimates below use conventional dense arithmetic.

- **Julia.** The input consists of dense `D × D` matrices. Computing `n²`
  brackets takes `O(n²D³)` arithmetic. Each bracket projection rebuilds the
  `n × n` Gram matrix with `O(n²D²)` contractions and solves a fresh system
  with a general dense upper bound of `O(n³)`. This gives an overall bound
  `O(n²D³ + n⁴D² + n⁵)`. The actual solve can be cheaper for a diagonal Gram
  matrix. The physical basis and retained adjoints occupy `O(nD² + n³)`
  entries. [Julia bracket projection](https://github.com/nattyo1226/LieAlgebra.jl/blob/575029c12fcad44b3b41206ee564437a7ec5fa45/src/brackets.jl#L11),
  [Gram solve](https://github.com/nattyo1226/LieAlgebra.jl/blob/575029c12fcad44b3b41206ee564437a7ec5fa45/src/basis.jl#L34).
- **Rust.** The Gram stage makes `n²` normalized Hilbert–Schmidt (HS)
  contractions. Closure verifies all ordered pairs and makes `n²(n+4)`
  contractions. One retained LU factorization gives an `O(n⁴)` total bound
  for the `n²` backsolves. Contraction cost depends on the expressions and
  tensor support. It includes collecting both operands and evaluating term
  products using local traces. The coordinate arrays include `O(n³)` retained
  structure/adjoint data. Physical tensor expressions bypass `D × D` storage.
  [Closure analysis](README.md#expression-evaluation-and-closure).

Julia can be cheaper at this stage for small `D` because its dense products
are small and its adjoint construction projects each bracket once. Rust
additionally tests the residual of every projected bracket against the
original operator expression. In the Pauli `n=15` case, Julia's adjoints took
1.79 ms. Rust's closure stage alone took 29.58 ms and included that residual
evaluation. [Rust public API timings](README.md#public-api-timings).

## Cost of the spectral split

For real endomorphisms `C` of an `n`-dimensional algebra, the equations
`C ad(B_i) - ad(B_i) C = 0` for all `i` give `m=n³` scalar rows and `p=n²`
unknowns. In Julia the adjoints and the constraint array use `ComplexF64`.
Its `nullspace` in Julia 1.11.7 calls `svd(A; full=true)`, which produces
an `m × m` left singular-vector matrix even though the returned basis comes
from `V`. For tall matrices the usual QR-based full-vector LAPACK path adds
`O(m²p)` work to the `O(mp²)` reduction and uses `O(m² + mp)` storage. Thus
the Julia reference has an `O(n⁸)` dense work bound for this step and an
`O(n⁶)` memory cost. Its pivoted QR for extracting independent real symmetric
matrices from the kernel is a separate step. [Julia adjoints](https://github.com/nattyo1226/LieAlgebra.jl/blob/575029c12fcad44b3b41206ee564437a7ec5fa45/src/adjoint.jl#L1),
[Julia commutant](https://github.com/nattyo1226/LieAlgebra.jl/blob/575029c12fcad44b3b41206ee564437a7ec5fa45/src/commutant.jl#L16),
[LAPACK full-vector driver](https://netlib.org/lapack/complex16/zgesdd.f),
[explicit Q generation](https://netlib.org/lapack/complex16/zungqr.f).

In Rust, `svd(false, true)` uses `O(mp+p²)` storage and the usual tall dense
`O(mp²)` work estimate. The center SVD has shape `n² × n`, giving `O(n⁴)`
work and `O(n³)` storage. The public full adjoint commutant has time `O(n⁷)`
and storage `O(n⁵)` under this model. For decomposition, a symmetric
commutant solve on a `d`-dimensional subalgebra has `d³` rows and
`d(d+1)/2` columns. It also builds the full `d³ × d²` array before projecting
into the symmetric coordinates. With candidate ideal dimensions `d_i`, an
accepted first attempt uses `O(n⁴ + d⁷ + Σᵢ d_i⁷)` work for the SVDs. Retries
that reach candidate validation can repeat the restricted solves. The
semisimple solve is shared across attempts. The largest pair of symmetric
solves occurs for an already simple algebra, with `d = d_1 = n`.
Purely abelian inputs take the zero-constraint path for the center and skip
the symmetric solves in decomposition.
[Rust SVD](../../../crates/syntaqs-lie/src/numerical/kernels.rs#L126),
[symmetric constraints](../../../crates/syntaqs-lie/src/invariant.rs#L364),
[candidate validation](../../../crates/syntaqs-lie/src/decomposition/spectral.rs#L122).

The final expression check has a separate cost. Reconstructing a component of
dimension `d_i` can mix all `n` source expressions into each new basis element.
Its orthonormality check contracts all `d_i²` pairs of those potentially dense
combinations. For `fermion-4` that check took 0.94 s, comparable to the 0.95 s
spent across all decomposition SVDs in the instrumented run. The values of
`D`, `n`, and `d_i` alone do not predict the contraction time.
[Rust decomposition profile](README.md#adjoint-constraints-and-decomposition).

### Same-size measured examples

The Julia timings are medians in milliseconds of five post-warmup runs with
one OpenBLAS thread on the same Apple M4 host as the Rust measurements.
The Julia split column includes sampling, diagonalization, grouping, and
reconstruction. The total comes from a separate call to `decompose`.
[Rust measurement setup](README.md#measurement-and-reproduction).

| Case | `D` | `n` | Adjoint | Commutant | Split | Julia total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `spin-half-1` | 2 | 3 | 0.0085 | 0.0347 | 0.0138 | 0.0558 |
| `spin-half-2` | 4 | 6 | 0.0663 | 0.676 | 0.0192 | 0.754 |
| `pauli-2` | 4 | 15 | 1.79 | 1077 | 0.0521 | 1085 |
| `fermion-3` | 8 | 15 | 5.64 | 1081 | 0.0808 | 1084 |

On the same `3375 × 225` Pauli constraint matrix, the isolated nullspace
computation took 1079 ms with the full SVD and 111 ms with the thin SVD.
Julia-reported allocation fell from 201 to 37.4 MiB. The two nullspaces had
matching projectors within `1e-12` in Frobenius norm, using the same absolute
rank cutoff. A tall thin SVD still returns all `p` right directions. When
`m<p`, a thin SVD omits `p-m` structural null directions. At `n=28`, the Julia
full left array alone would occupy about 7.18 GiB. The `n=28` Julia figure is
an allocation estimate from shape and element size.
[Julia measurement driver](compare_julia.jl).

## Numerical stability and acceptance criteria

### Basis geometry and closure

For an independent Hermitian basis with normalized HS Gram matrix `G`, the
adjoint matrices satisfy `AᵢᵀG + GAᵢ = 0` in exact arithmetic. Skew-symmetry in
ordinary Euclidean coordinates follows when `G=I`. In general the adjoint
operation on a coordinate endomorphism is `G⁻¹CᵀG`. That distinction matters
when extracting a symmetric commuting operator. Julia projects the complex
commutant using ordinary conjugate transpose and requires each projected
Hermitian part to be real. If the real kernel contains a nonsymmetric matrix,
its skew part divided by `i` can fail this reality check. The kernel contains
such matrices whenever the center has dimension at least two. Rust first
orthonormalizes in the HS metric and isolates the center, then solves directly
in a real symmetric parameter space. [Julia reality check](https://github.com/nattyo1226/LieAlgebra.jl/blob/575029c12fcad44b3b41206ee564437a7ec5fa45/src/commutant.jl#L25),
[Rust orthonormal coordinates](../../../crates/syntaqs-lie/src/invariant.rs#L219).

Both upstream projection paths solve a Gram system. For the operator-basis
column map `B`, `G=B*B` gives `κ₂(G)=κ₂(B)²`. A near-linear dependence can
therefore affect the adjoint coefficients before the commutant SVD starts.
Rust diagnoses independence from the Gram spectrum and rejects a basis
below its relative eigenvalue cutoff. For the independent family
`[X₁, X₁ + 10⁻⁷Y₁, Z₁]`, the smallest Gram eigenvalue was about `5×10⁻¹⁵`
against a cutoff of `2×10⁻¹²`, so Rust reported numerical rank two. HS residual
evaluation can also be sensitive to cancellation in nonorthogonal expressions,
as exercised by the tests with an explicit residual tolerance override.
[Gram factorization](../../../crates/syntaqs-lie/src/family/projection.rs#L275),
[nonorthogonal expression tests](../../../crates/syntaqs-lie/tests/decomposition.rs#L273).

### Rank and eigenspace separation

Julia passes its absolute `atol` (default `1e-10`) to the constraint
nullspace, real-matrix check, pivoted QR rank decision, and eigenvalue grouping.
The sampler checks symmetry with its own absolute `1e-10` default. Julia
compares a candidate eigenvalue to the first eigenvalue in its group. Rust
uses distinct rank, Gram, gap, and norm
tolerances. The constraint rank cutoff scales with `σ_max`. The gap cutoff
scales with the largest absolute eigenvalue, and grouping compares adjacent
sorted eigenvalues. Rust verifies each returned kernel direction against
`||L v||`, and validates the number, invariance, mutual commutation, and
simplicity of the candidate ideals. A failed spectral split can trigger a
seeded retry. [Julia grouping](https://github.com/nattyo1226/LieAlgebra.jl/blob/575029c12fcad44b3b41206ee564437a7ec5fa45/src/decomposition.jl#L50),
[Rust tolerances](../../../crates/syntaqs-lie/src/numerical/options.rs),
[constraint residuals](../../../crates/syntaqs-lie/src/invariant.rs#L136).

The eigenprojectors of either method depend on the gap between different
clusters, and the computed kernel depends on how well zero and nonzero
singular values separate. Checks establish acceptance at the selected
tolerances. For example, the Rust default relative rank cutoff for the
`21952 × 406` symmetric constraints at `d=28` is about `4.9e-12` in `f64` and
`2.6e-3` in `f32`. Small nonzero singular values near that decision can change
the inferred commutant dimension. A residual failure or exhausted sampling
budget can surface an ambiguous split. [LAPACK SVD error bounds](https://www.netlib.org/lapack/lug/node97.html),
[LAPACK symmetric eigenproblem error bounds](https://www.netlib.org/lapack/lug/node90.html).

### Observed behavior

The following Pauli probes use the same physical spaces and basis changes in
both implementations, with default numerical settings. The collision probe
forces a zero first sample in the Rust unit test and a zero sample in Julia.

| Input or perturbation | Julia reference | `syntaqs-lie` |
| --- | --- | --- |
| Two independent Pauli triples | Returns two 3-dimensional ideals | Returns two 3-dimensional simple ideals |
| Replace `X₂` by `X₂ + X₁` in the same closed basis | `ArgumentError` from the real-matrix check | Returns two 3-dimensional simple ideals |
| Commuting basis `[Z₁, Z₂]` | `ArgumentError` from the real-matrix check | Returns a 2-dimensional center and no simple ideals |
| Nonclosed span of `[X₁, Y₁, (Z₁ + X₂)/√2]` | Returns one 3-dimensional component of the projected bracket | `NonClosedPair`, residual about `1.414` |
| Forced zero sample for the two independent triples | Returns the full 6-dimensional algebra as one component | Rejects the merged group, retries, and reports exhaustion for persistent collisions |

The Rust unit tests exercise collision recovery and exhaustion at both
precisions. Compare component projectors or invariant spans across sampling
seeds and numerical backends. Eigenvectors within a degenerate cluster can
rotate even when the subspace is well determined.
[Rust collision tests](../../../crates/syntaqs-lie/src/decomposition/tests.rs#L29),
[Julia driver](compare_julia.jl), [Rust probe](stability_probe.rs).

## Implications for the current implementation

Removing the full left output from the Julia SVD preserves the measured tall
kernel and changes its leading storage bound from `O(n⁶)` to `O(n⁵)`. The
reality and closure behavior come from other stages. For Rust, constructing
the symmetric constraints directly would remove the full intermediate while
keeping the same right singular problem. At `d=28` the intermediate alone is
131.3 MiB in `f64`. With verified skew-symmetric adjoints, `CAᵢ-AᵢC` is
symmetric for symmetric `C`, so the rows could also use weighted independent
symmetric coordinates. That further reduction needs to preserve Frobenius
norms, account for the skew-symmetry defect, and explicitly address the
dimension-dependent rank cutoff because the constraint row count changes.

In exact arithmetic a direct sum of `k` nonzero ideals in a centerless compact
algebra splits into simple ideals if `k` already equals the global symmetric
commutant dimension. This explains the overlap between the global count and
the per-ideal simplicity checks. The current checks also supply independent
numerical evidence about the restricted problems. Eliminating them would need
an acceptance criterion that still resolves ambiguous ranks and detects
misclassified or non-invariant components. Forming `LᵀL` to avoid a rectangular
SVD would square the constraint singular values, making the numerical
nullspace harder to resolve. [Rust validation contract](../../design/lie.md#7-complete-center-and-simple-ideal-decomposition).

## Reproduction

The two probes passed, along with 35 tests in the Julia reference and 111
tests in `syntaqs-lie` including doctests. The existing Rust measurements
use the original benchmark driver and a separate profiling build.

Run the [Julia driver](compare_julia.jl) in the referenced checkout with
Julia 1.11.7, passing the absolute path of the driver.

```sh
julia --startup-file=no --project=. /path/to/compare_julia.jl
```

For Rust, use the [isolated benchmark checkout](README.md#measurement-and-reproduction),
copy `stability_probe.rs` to `examples/src/bin/lie_stability_probe.rs` in that
checkout, and run the example from the checkout root.

```sh
OPENBLAS_NUM_THREADS=1 cargo run --locked --release -p examples \
  --no-default-features --features lie --bin lie_stability_probe
```
