# syntaqs-lie

Operator families, numerical foundations, and Lie representations using syntaqs
expressions, including adjoint commutants, centers, and complete decomposition
into the full center and nonabelian simple ideals.

`OperatorFamily<E>` retains a nonempty ordered family of `LocalOperator` or
placed `Operator` expressions with complex f32/f64 coefficients. `new` infers a
local context or a bound input's ambient space; `new_in` supplies the space for
unbound inputs. Conflicting bindings and missing indices fail, including zero
inputs. Construction does not evaluate or certify the candidates.

`gram(&options)` returns the complex normalized-HS Gram matrix. Its `validate()`
reports per-element Hermiticity and real-linear independence, including failed
diagnostics for invalid candidates. `factorize()` rejects non-Hermitian or
singular families and retains an LU decomposition for repeated targets.
`coefficients`, `project`, `residual`, and `in_span` also work directly on a
family, using explicit numerical options. Projection solves `G c = b` for real
coordinates in independent, potentially nonorthonormal families. Out-of-span
Hermitian targets still have coefficients; membership compares the residual's
normalized HS norm against the target's normalized HS norm scale.

Coordinates, Gram matrices, and projections borrow the exact originating family.
Expressions remain shared core handles; no physical action matrix, state basis,
or custom inner product is used. Primitives need `PartialEq` for exact collection
of equal terms before HS contraction. This reduces cancellation in residuals
without tolerance-based pruning. See the [current API contract](../../docs/design/lie.md)
and the crate-level Rust example.

`NumericalOptions<f32/f64>` exposes independent residual, Hermiticity, imaginary,
negative-squared-norm roundoff, rank, Gram-eigenvalue, and eigenvalue-gap settings.
Residual defaults are `atol=rtol=2e-5` for f32 and `1e-12` for f64. Spectral
defaults are `max(default_floor, dimension*epsilon)`; `Some(relative)` overrides
this formula exactly. `Tolerance::check` returns the measurement, scale, and
effective bound. Numerical settings/data must be finite; invalid scales, bounds,
and materially nonreal/negative results return NumericalError.

Internal dense kernels provide real solves, symmetric eigensystems, full SVD
right nullspaces (including wide/empty matrices), and independent Gram checks.
Norms use scaled accumulation; small positive values are not pruned. Gram
eigenvalue cutoffs apply to Gram eigenvalues themselves, not to their roots.

## Lie brackets and representations

`lie_bracket(A,B)` constructs `-i(AB-BA)`, including on graded spaces. On a
family, `bracket(i,j)` uses its common context. `bracket_coefficients` and
`bracket_projection` obtain projected coordinates and diagnostics; `closure`
checks every ordered pair using a reusable Gram factorization. These numerical
operations require independent Hermitian family elements.

`structure_constants` requires closure and retains `f[k,i,j]`, output index
first, with the source family and full pairwise diagnostics. Its `adjoint()`
returns matrices `ad_i[k,j]=f[k,i,j]`; their `killing_form()` computes classical
`K[i,j]=Tr(ad_i ad_j)`. The result arrays refer to the selected operator family,
not physical action matrices. Nonclosed families return diagnostics or a
`NonClosedPair` error rather than silently promoting projections to constants.
All validity checks use the recorded numerical settings.

`transform(U)` retains a new family and its real column coordinates in the
original family. `select(indices)` retains a selection matrix. `orthogonalize`
and `orthonormalize` use the Gram eigensystem and reject dependent or
non-Hermitian inputs. `orthonormality` reports the Gram defect from identity,
Hermiticity, and independence. See the [API contract](../../docs/design/lie.md).

## Adjoint commutant and center

`commutant` on an `AdjointRepresentation` returns `AdjointCommutant`, a basis of
all real matrices commuting with every adjoint matrix. The matrices act on the
original family coordinates and are Frobenius-orthonormal there. The full real
space includes nonsymmetric matrices. An n-dimensional abelian algebra returns
dimension n². This operation does not compute a physical-operator centralizer.

`center` returns `CenterSplit`, containing the full center and its HS-orthogonal
complement as optional HS-orthonormal operator families. An absent component is
`None`, with an `(n,0)` coordinate matrix. Coordinate columns reconstruct each
component in the original context. The result also retains the internal
orthonormal coordinate transformation and its inverse. Family convenience
methods take explicit numerical options and first validate the Lie preconditions;
retained adjoints reuse their existing coefficients and settings.

Both results borrow their source family and retain settings and SVD rank
diagnostics. Constraint candidates must pass the residual tolerance as well as
the singular-value cutoff; failures return `ConstraintResidual`. Center residuals
are also checked in original coordinates. The symmetric commuting subspace used
for later decomposition is a separate internal object in orthonormal coordinates.
See the [API contract](../../docs/design/lie.md#6-adjoint-commutant-and-center).

## Center and simple-ideal decomposition

`decompose` on an `AdjointRepresentation` uses its retained numerical settings;
on a family, `decompose(&numerical, &sampling)` first validates the Lie
preconditions. `DecompositionOptions` supplies `seed` (default zero) and
`max_attempts` (default 32, required positive). Sampling uses a deterministic
internal stream without another production dependency. Reproducibility is
within the same numerical environment, not across LAPACK implementations.

`LieDecomposition::center()` returns the full center as one optional
`LieComponent`; `simple_ideals()` returns the nonabelian simple components.
Each component exposes its reconstructed `family()` and column `coordinates()`
in the original source family. Output is HS-orthonormal; signs, basis vectors,
and ideal order are not canonical. `orthogonalize` and `orthonormalize` on a
component compose its coordinates back to the original source.

The algorithm separates the center first, samples the symmetric commutant on
its complement, and groups a sampled eigensystem. `eigenvalue_relative` controls
the gap threshold. Every proposed split must pass full-span/orthonormality,
centrality, ideal invariance, cross-ideal commutation, nonabelianness, centerless
block, and restricted symmetric-commutant checks. An unsplit direct sum never
becomes a simple ideal. Failed candidates retry; `DecompositionExhausted` retains
all failed attempts when the budget is used up. Purely abelian input returns
only the center and needs no sampled attempts. Diagnostics retain effective
gaps, constraint cutoffs, retry history, and final invariant checks.

Reconstruction collects exact terms without coefficient pruning, and returned
families are checked again. Family decomposition still uses the existing
projection/closure residual evaluator: some nonorthonormal expressions can
suffer cancellation under strict tolerances. Any residual override is explicit;
decomposition never loosens a caller's settings. See the
[decomposition contract](../../docs/design/lie.md#7-complete-center-and-simple-ideal-decomposition).

## Native linkage

This library enables ndarray BLAS and uses ndarray-linalg without provider
features. Its provider-neutral blas-src dependency retains native linkage.
The final application selects one source supplying both BLAS and LAPACK:

```toml
blas-src = { version = "0.14.0", features = ["openblas"] }
openblas-src = { version = "0.10", features = ["system"] }
```

For Nix development, include `pkgs.openblas` and `pkgs.pkg-config` in the
devShell with the Rust toolchain. CI installs their system equivalents. Workspace
tests and examples select that provider through the internal
[blas-provider package](../../support/blas-provider/README.md); it is not a
production library dependency. See the downstream fixture under
`tests/fixtures/lie-blas` for a standalone application using the same provider
without ndarray-linalg provider features.

```sh
direnv exec . cargo test -p syntaqs-lie
direnv exec . cargo run --manifest-path tests/fixtures/lie-blas/Cargo.toml
```

The crate uses std and preserves the existing core's no_std and exact-zero
contracts. See the [verification guide](../../docs/design/verification.md) for
checks and verification limits.
