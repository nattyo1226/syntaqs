# Numerical stability of Lie algebra decomposition

The Pauli cases below exercise closure, basis geometry, and spectral
ambiguity. The Rust stability probe passed at the v6 merge `c26ea49`.
The Julia observations are from `LieAlgebra.jl` revision `575029c` with
Julia 1.11.7 on 2026-11-18. [Rust probe](probe/stability_probe.rs),
[Julia probe](probe/compare_julia.jl),
[Julia decomposition](https://github.com/nattyo1226/LieAlgebra.jl/blob/575029c12fcad44b3b41206ee564437a7ec5fa45/src/decomposition.jl#L86).

## Observed behavior

The two probes use the same physical spaces and basis changes with their
respective default numerical settings. The Rust unit tests also force
spectral collisions at both precisions. [Collision tests](../../../crates/syntaqs-lie/src/decomposition/tests.rs).

| Input or perturbation | Julia reference | `syntaqs-lie` |
| --- | --- | --- |
| Two independent Pauli triples | Returns two 3-dimensional ideals | Returns two 3-dimensional simple ideals |
| Replace `X₂` by `X₂ + X₁` in the same closed basis | `ArgumentError` from the real-matrix check | Returns two 3-dimensional simple ideals |
| Commuting basis `[Z₁, Z₂]` | `ArgumentError` from the real-matrix check | Returns a 2-dimensional center |
| Nonclosed span of `[X₁, Y₁, (Z₁ + X₂)/√2]` | Returns one 3-dimensional component of the projected bracket | `NonClosedPair`, residual about 1.414 |
| Forced zero sample for the two independent triples | Returns the full 6-dimensional algebra as one component | Rejects the merged group and retries, reporting exhaustion for persistent collisions |

The Rust probe also classifies `[X₁, X₁ + 10⁻⁷Y₁, Z₁]` as rank two in
`f64`. Its smallest Gram eigenvalue is about `5×10⁻¹⁵` against a cutoff of
`2×10⁻¹²`. For an operator-basis column map `B`, the Gram matrix
`G = B*B` satisfies `κ₂(G) = κ₂(B)²`. Near-linear dependence therefore
affects adjoint coefficients before the constraint SVD. Nonorthogonal
projections can also be sensitive to cancellation in the Hilbert–Schmidt
(HS) residual.
[Gram factorization](../../../crates/syntaqs-lie/src/family/projection.rs#L275),
[numerical contract](../../design/lie.md#3-projection-and-numerical-scales).

## Basis geometry and spectral tolerances

For a Hermitian basis with normalized HS Gram matrix `G`, the adjoints
satisfy `AᵢᵀG + GAᵢ = 0` in exact arithmetic. The adjoint of a coordinate
endomorphism `C` in this metric is `G⁻¹CᵀG`. The Julia reference projects
the complex commutant by ordinary conjugate transpose and requires the
Hermitian projections to be real. The kernel can contain real nonsymmetric
maps, including maps within a center of dimension two or greater. Rust
extracts the center, orthonormalizes in the HS metric, and solves for real
symmetric endomorphisms of the semisimple complement.
[Julia reality check](https://github.com/nattyo1226/LieAlgebra.jl/blob/575029c12fcad44b3b41206ee564437a7ec5fa45/src/commutant.jl#L25),
[Rust decomposition contract](../../design/lie.md#7-complete-center-and-simple-ideal-decomposition).

The Julia path uses an absolute tolerance for the nullspace, reality check,
pivoted QR rank, and eigenvalue grouping. Rust uses separate tolerances for
rank, Gram scale, eigenvalue gaps, and residuals. For a 28-dimensional
component, the Rust relative rank cutoff for the `21,952 × 406` symmetric
system is approximately `4.9×10⁻¹²` in `f64` and `2.6×10⁻³` in `f32`.
Small nonzero singular values near the cutoff can change the inferred
commutant dimension.
Compare component projectors or invariant spans across seeds and backends
because bases within degenerate eigenspaces can rotate.
[Julia grouping](https://github.com/nattyo1226/LieAlgebra.jl/blob/575029c12fcad44b3b41206ee564437a7ec5fa45/src/decomposition.jl#L50),
[Rust tolerance definitions](../../../crates/syntaqs-lie/src/numerical/options.rs).

A tall thin SVD produced the same nullspace projector within `1e-12` in the
original Julia probe on the `3,375 × 225` Pauli system. A thin SVD on a wide
matrix can omit the extra structural nullspace directions. Solver changes
must preserve all right null directions and their residual checks.
[Rust nullspace kernel](../../../crates/syntaqs-lie/src/numerical/kernels.rs#L126).

## Reproduction

Build and run the Rust probe using the [merge benchmark setup](README.md#measurement-and-reproduction).
Run the Julia probe from a checkout of the pinned Julia reference with its
project dependencies available, passing the absolute path to
[compare_julia.jl](probe/compare_julia.jl).

```sh
julia --startup-file=no --project=. /path/to/compare_julia.jl
```
