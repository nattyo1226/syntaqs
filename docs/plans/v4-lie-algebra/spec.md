# Implementation specification

Status: numerical foundations, operator-family/projection APIs, brackets,
representations, family transformations, the full adjoint commutant, center
extraction, and complete simple-ideal decomposition are implemented. Final public
integration and full verification remain Step 6.
Implementation order and checks are in [the plan](implementation-plan.md).

## Scope and boundaries

- Add `crates/syntaqs-lie`. Keep all Lie-specific APIs, coefficient generation,
  metadata, validation, and numerical integration in this crate.
- Reuse public core expressions, spaces, products, adjoints, traces, and HS
  contractions. Do not introduce another expression tree/evaluator or a
  production dependency on concrete spin/fermion crates.
- Use dense ndarray coefficient arrays and ndarray-linalg as needed. Matrix
  dimensions concern the selected Lie algebra basis, not the full physical
  Hilbert space. Do not require physical action-matrix export.
- Preserve core's dependency direction, no_std contract, coefficient precision,
  and exact-zero behavior. Numerical paths support f32 and f64.
- Defer general map-representation commonality. Custom inner products, general
  complex Lie algebras, superbrackets, and automatic Lie closure generation
  from arbitrary generators are outside this iteration.

## Objects and validation

| Object | Contract |
| --- | --- |
| `OperatorFamily` | Nonempty ordered collection of existing operator expressions with a common local/ambient context. Preserve order; construction checks context compatibility, not mathematical validity. |
| Validation report | Records Hermiticity, real linear independence, closure, relevant element/pair diagnostics, residuals, and effective tolerances. No generic validated flag or separate validated-basis type. |
| Retained coefficient results | Associate data with the originating family, space/context, precision, and index conventions. Keep the association valid for the result's lifetime. |
| `LieDecomposition` | Full center plus nonabelian simple ideals, reconstructed as operator expressions in the original context. Distinguish center and simple-ideal accessors. Represent an absent center explicitly. |

Candidate families may contain dependent, non-Hermitian, or nonclosed inputs.
Explicit checks report validity; operations verify the preconditions they need.
Dependent/numerically singular bases cause errors in operations requiring
independence. Do not silently remove elements or use a pseudoinverse. A valid
constraint nullspace for a center/commutant is not an invalid basis.

Reuse explicitly computed Gram/factorization data where appropriate, without
adopting automatic result caching. Supporting names such as `ValidationReport`,
`StructureConstants`, and `AdjointRepresentation`, concrete generics, and
ownership layout remain implementation choices. A zero center may use
`Option<OperatorFamily<...>>` to preserve nonempty construction.

## Algebra and coordinates

The mathematical domain is the real span of Hermitian operators. Complex
coefficients inside operator expressions do not change that scalar field.
Use the ordinary Lie bracket, including on supported graded spaces; core
already resolves physical grading signs.

| Operation | Definition / behavior |
| --- | --- |
| Operator inner product | `<A,B> = Tr(A†B)/D`, fixed normalized HS on the selected Hilbert space. No custom metric API. |
| Gram matrix | `G_ij = <B_i,B_j>`. |
| `coefficients(A)` | Solve `Gc=b`, `b_i=<B_i,A>`, returning real projection coefficients even for an out-of-span Hermitian target. Support independent nonorthonormal bases. |
| Reconstruction / `project(A)` | Form `sum_i c_i B_i`; only equals A for an in-span target, within tolerance. |
| `residual(A)` | Scalar normalized HS norm of `R=A-project(A)`: `sqrt(Tr(R†R)/D)`. |
| `in_span(A, tolerance)` | Compare the residual with the configured bound. |
| Lie bracket | `-i(AB-BA)`; core's existing `commutator` remains `AB-BA`. |
| Structure constants | `-i[B_i,B_j] = sum_k f[k,i,j] B_k`, output index first. Verify closure before reporting Lie structure constants. |
| Adjoint representation | `ad_i[k,j]=f[k,i,j]`, acting on the Lie algebra coordinate vector. |
| Killing form | `K_ij=Tr(ad_i ad_j)`, without conjugation, sign reversal, or dimension normalization. |
| Basis transformation | Columns of U define the new operators: `B'_j=sum_i B_i U[i,j]`. Retain transforms for reconstruction and coordinate interpretation. |

Provide family access/enumeration, rank/independence checks, Gram evaluation,
projection/reconstruction, span/closure diagnostics, bracket coefficients,
structure constants, adjoint matrices, Killing form, basis transformation and
component selection, orthogonalization, and orthonormality checks. Orthogonalize
both individual families and decomposition components; dependent inputs error.
Method names/signatures may follow Rust conventions while preserving semantics.

## Commutant and decomposition

The public Lie commutant is the full real endomorphism space
`{C | C ad_i = ad_i C for every i}`, in the selected Lie algebra basis. It is
distinct from physical-operator centralizers and from its symmetric subspace.

Target `g = z(g) ⊕ g_1 ⊕ ... ⊕ g_k`, with the full center returned as one
subspace and each g_i a nonabelian simple ideal. Purely abelian input returns
only the center; centerless input has no central component. Bases and component
order need not be unique. Use the agreed Julia-style algorithm:

1. Validate Hermiticity, independence, and closure. Move internally to
   normalized-HS orthonormal coordinates, retaining the original-basis transform.
2. Extract the full center from the common kernel of the adjoint action.
   Split its orthogonal complement, the semisimple part in this domain.
3. Compute that part's adjoint commutant and its real symmetric subspace.
4. Sample a real linear combination of symmetric commutant matrices,
   diagonalize it, and group eigenvalues using configured tolerances.
5. Validate complete simple-ideal splitting, full span with no overlap, ideal
   invariance, and inter-component commutation. Retry insufficient splits or
   accidental eigenvalue collisions; never return them as complete success.
6. Reconstruct center/ideal bases from the original syntaqs expressions.

Accept a caller-supplied random seed. Define finite retry limits and an error
on exhaustion during implementation. Reproducibility is within the same
numerical environment, not bitwise across LAPACK implementations/platforms.

## Numerical contract

All settings can be independently overridden. These agreed initial defaults
require representative tests; they are not measured accuracy guarantees.

| Test | f32 | f64 |
| --- | --- | --- |
| Residuals, Hermiticity, small imaginary parts | `atol=rtol=2e-5` | `atol=rtol=1e-12` |
| Rank/nullspace relative cutoff | `max(2e-5,d*epsilon)` | `max(1e-12,d*epsilon)` |
| Eigenvalue-grouping relative cutoff | `max(2e-5,n*epsilon)` | `max(1e-12,n*epsilon)` |

Use the actual real precision's machine epsilon, `d=max(rows,columns)`, and
eigenproblem size n. Residual tests use `residual <= atol + rtol*scale`; span
membership uses the target's normalized HS norm as scale. Rank/nullspace tests
classify `sigma <= cutoff*sigma_max` as zero. Eigenvalue grouping uses
`cutoff*max(abs(eigenvalues))` as its gap threshold.

Step 1 exposes independent settings through NumericalOptions. Spectral
`None` selects the dimension-aware default; `Some(r)` overrides it exactly,
including zero. Gram checks apply `gram_cutoff*lambda_max` directly to Gram
eigenvalues, report that separate threshold, and reject numerical singularity
or materially negative spectra; this is not a cutoff on the original family's
singular values.

Dense symmetry checks measure Frobenius defects against the input Frobenius
scale before averaging triangles. Real-coordinate conversion checks the
imaginary part against its separately configured bound and caller-supplied
scale. Negative squared norms are clamped only within a squared-unit bound,
initially `atol=0, rtol=64*epsilon`; larger negatives error, and positive tiny
values remain. Invalid settings, nonfinite analysis data, and overflowing
bounds error without changing ordinary expression evaluation. Higher-level
operator-scale selection follows the relevant operation as it is implemented.
Propagate overrides and retain effective diagnostics; do not prune expression
coefficients.

## Dependencies and BLAS linkage

Initially pair ndarray 0.17.2 with ndarray-linalg 0.18.1, whose declared ndarray
dependency is `^0.17.1`. Verify actual builds and Rust 1.85 compatibility.
Enable BLAS use in syntaqs-lie's ndarray dependencies and separate provider
selection through blas-src. Planned production dependencies are:

```toml
ndarray = { workspace = true, features = ["blas"] }
ndarray-linalg = { version = "0.18.1", default-features = false, features = ["blas"] }
blas-src = { version = "0.14.0", default-features = false }
```

Keep provider features off the production blas-src dependency and include
`extern crate blas_src as _;` in the library to retain native linkage. The
application chooses the implementation through blas-src, not through
ndarray-linalg provider features. For example, system OpenBLAS uses:

```toml
blas-src = { version = "0.14.0", features = ["openblas"] }
openblas-src = { version = "0.10", features = ["system"] }
```

The chosen implementation must supply both BLAS and the compatible LAPACK
routines required by ndarray-linalg. The `blas` feature alone does not supply
those libraries. This is build/link-time selection; syntaqs-lie does not expose
provider-selection features. Centralize the development/test/example provider
configuration in the unpublished internal `blas-provider` package under
`support/blas-provider`, with system OpenBLAS supplied by Nix/CI initially.
Verify that its selection and native linkage do not leak into
downstream production dependency settings.
