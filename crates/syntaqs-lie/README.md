# syntaqs-lie

Operator-family projection, Lie representations, adjoint commutants, centers,
and complete center/simple-ideal decomposition for [syntaqs](../../README.md)
operator expressions.

The analysis uses the real span of Hermitian operators and the normalized
Hilbert–Schmidt metric. It works directly on shared core expressions, without a
state basis or physical action matrices. The crate uses std and complex f32/f64
coefficients; the application supplies native BLAS/LAPACK linkage.

For a release, follow the [version-tagged installation guide](../../docs/installation.md)
and use documentation at the chosen tag. This README describes current-checkout APIs.

## Using the crate

The usual workflow is:

1. Construct an `OperatorFamily` from local or placed operators. Use `new_in`
   with an ambient space when all placed inputs are unbound.
2. Validate Hermiticity and independence. Reuse a Gram factorization when
   projecting multiple targets onto the same family.
3. Check closure before obtaining Lie representations or a center/simple-ideal
   decomposition. Numerical settings and diagnostics remain attached to results.

The [crate-level example](src/lib.rs) and runnable
[Lie analysis example](../../examples/src/bin/lie_algebra.rs) show these steps together.

Types live under `family`, `representation`, `invariant`, `decomposition`, and
`numerical`; the prelude provides common entry points. Follow the relevant
contract for precise preconditions and result conventions:

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

Select one provider supplying both BLAS and LAPACK in the final application.
For an installed system OpenBLAS, add:

```toml
blas-src = { version = "0.14.0", features = ["openblas"] }
openblas-src = { version = "0.10", features = ["system"] }
```

Retain that provider in the application binary with:

```rust
extern crate blas_src as _;
```

The system implementation must be discoverable through pkg-config and the linker
search paths. syntaqs-lie enables ndarray BLAS and uses ndarray-linalg without
provider features; its provider-neutral blas-src dependency leaves selection to
the application. It exposes no provider-selection features of its own.

Workspace tests and examples select system OpenBLAS through the internal
[blas-provider package](../../support/blas-provider/README.md); it is not a
production library dependency. Provide system OpenBLAS, including LAPACK, and
pkg-config when running them. The
[downstream fixture](../../tests/fixtures/lie-blas/Cargo.toml) demonstrates
application-owned provider selection. Workspace commands, CI coverage, and
verification limits belong to the [verification guide](../../docs/design/verification.md).
