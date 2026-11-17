# syntaqs-lie

Operator-family projection, Lie representations, adjoint commutants, centers,
and complete center/simple-ideal decomposition for [syntaqs](../../README.md)
operator expressions.

The analysis uses the real span of Hermitian operators and the normalized
Hilbert–Schmidt metric. It works directly on shared core expressions, without a
state basis or physical action matrices. The crate uses std and complex f32/f64
coefficients; the application supplies native BLAS/LAPACK linkage.

## Using the crate

Construct an `OperatorFamily` from local or placed operators, supplying an
ambient space with `new_in` for entirely unbound placed inputs. Validate
Hermiticity and independence before projection; a Gram factorization can be
reused for multiple targets. Lie representations and decomposition additionally
require closure. See the [crate-level example](src/lib.rs) and the runnable
[Lie analysis example](../../examples/src/bin/lie_algebra.rs) for the public
workflow.

Types live under `family`, `representation`, `invariant`, `decomposition`, and
`numerical`; the prelude provides common entry points. The design document owns
the detailed contracts:

| Topic | Specification |
| --- | --- |
| Family contexts, validation, and retained results | [Families and validation](../../docs/design/lie.md#1-scope-and-contexts) |
| Projection, numerical settings, and cancellation limits | [Projection and numerical scales](../../docs/design/lie.md#3-projection-and-numerical-scales) |
| Brackets, closure, adjoints, and Killing forms | [Lie representations](../../docs/design/lie.md#4-brackets-closure-and-retained-representations) |
| Basis changes and orthogonalization | [Family transformations](../../docs/design/lie.md#5-family-transformations-and-orthogonalization) |
| Full adjoint commutant and center | [Invariant subspaces](../../docs/design/lie.md#6-adjoint-commutant-and-center) |
| Sampling, validation, and complete simple-ideal splitting | [Decomposition](../../docs/design/lie.md#7-complete-center-and-simple-ideal-decomposition) |

Numerical validity is relative to explicit settings. Some nonorthonormal
expressions need caller-selected residual overrides because of cancellation.
Primitive `PartialEq` supports exact term collection; small nonzero coefficients
are retained. Sampling reproducibility is limited to the same numerical
environment. The linked contracts specify these conditions and diagnostics.

## Native linkage

This library enables ndarray BLAS and uses ndarray-linalg without provider
features. Its provider-neutral blas-src dependency retains native linkage.
The final application selects one source supplying both BLAS and LAPACK:

```toml
blas-src = { version = "0.14.0", features = ["openblas"] }
openblas-src = { version = "0.10", features = ["system"] }
```

Retain that provider in the application binary with:

```rust
extern crate blas_src as _;
```

The library does not expose provider-selection features. The linked source must
supply LAPACK as well as BLAS. For a system provider, pkg-config and linker search
paths must find the installed implementation.

Workspace tests and examples select system OpenBLAS through the internal
[blas-provider package](../../support/blas-provider/README.md); it is not a
production library dependency. Provide system OpenBLAS, including LAPACK, and
pkg-config when running them. The
[downstream fixture](../../tests/fixtures/lie-blas/Cargo.toml) demonstrates
application-owned provider selection. Workspace commands, CI coverage, and
verification limits belong to the [verification guide](../../docs/design/verification.md).
