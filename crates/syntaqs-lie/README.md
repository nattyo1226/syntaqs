# syntaqs-lie

Operator families and numerical foundations for Lie algebra analysis using
syntaqs expressions. Lie brackets and decomposition remain subsequent steps.

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
contracts. The [implementation workspace](../../docs/plans/v4-lie-algebra/README.md)
records scope, numerical rules, progress, and verification limits.
