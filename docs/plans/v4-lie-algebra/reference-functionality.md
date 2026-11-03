# LieAlgebra.jl reference functionality

Inspected: 2026-11-03. Source revision:
[`575029c12fcad44b3b41206ee564437a7ec5fa45`](https://github.com/nattyo1226/LieAlgebra.jl/tree/575029c12fcad44b3b41206ee564437a7ec5fa45).
The local package declares `1.0.0-DEV`, Julia 1.11, LinearAlgebra, and Random.
The following records source behavior, not a guarantee that all mathematical
inputs are supported or that the tests have been run in this investigation.

## Feature inventory

| Area | Existing reference API and behavior | syntaqs starting point / missing capability |
| --- | --- | --- |
| Basis container | `MatrixBasis`, iteration/indexing, `elements`, `matrix_size`, and `basis_matrix` (columns are column-major vectorizations). Construction requires a nonempty list of equally sized square matrices, but does not establish independence or closure. | Need a separate basis of operator expressions with a selected common space. |
| Independence | `rank` via LinearAlgebra, `is_independent`, `assert_independent`. | No general numerical rank operation. |
| Inner products | `AbstractInnerProduct`, callable metric, `TraceInnerProduct(normalize=true/false)`, and `gram_matrix`. Default is `Tr(A†B)/d`. | Reuse normalized operator HS for the planned Gram API; custom inner products will not be exposed in this iteration. |
| Coefficients and projection | `coefficients` solves `Gc=b`, with `Gij=<Bi,Bj>` and `bi=<Bi,A>`; `reconstruct`, `project`, `residual`, `in_span`. | Can reconstruct expressions and compute Gram entries; need a solver and a separate subspace/projection contract. |
| Brackets and closure | `commutator=AB-BA`; `lie_bracket=-i(AB-BA)`; `bracket` indexes basis elements; `bracket_coefficients`; `is_closed`, `assert_closed`. | Existing commutator and scaling suffice for bracket expressions; closure needs projection and residuals. |
| Structure constants | `structure_constants` returns `f[k,i,j]`, the coefficients of `-i[Bi,Bj]` along `Bk`. | Need retained coefficients and their basis/order metadata. |
| Adjoint representation | `adjoint_matrix(B,i)` has column `j = f[:,i,j]`; `adjoint_matrices` returns one matrix per basis element. | Implement Lie-specific coefficient data in syntaqs-lie; general commutator-map commonality is deferred. |
| Function named Killing form | `killing_form` computes pairwise inner products of adjoint matrices using the supplied metric. | syntaqs will use the classical bilinear Killing form instead; see the adopted decision below. |
| Commutant | `commutant(matrices)` uses Kronecker constraints and a nullspace, then retains real Hermitian representatives and uses pivoted QR to select independent elements. `commutant(B)` operates on adjoint matrices. | Need numerical nullspace/rank operations and a clear real/complex result contract. |
| Basis transformations | `transform_basis(B,U)` uses columns of U as new basis coefficients; `component_basis` selects indexed elements. | Existing expression sums/scaling can implement reconstruction. |
| Decomposition | `generic_commutant_element` forms a random real symmetric combination; `decompose` diagonalizes it, groups eigenvalues within tolerance, transforms the basis, and returns `Decomposition.ideals`. Explicit-RNG overloads exist. | Need factorization, RNG policy, metric assumptions, and validation of resulting ideals. |
| Orthogonalization | `orthogonalize` applies sequential Gram-Schmidt and normalization to a basis or each decomposition component; `is_orthonormal` compares its Gram matrix with identity. Dependent input raises an error. | State normalization exists, but operator-subspace orthogonalization does not. |

The public exports are listed in
[`src/LieAlgebra.jl`](https://github.com/nattyo1226/LieAlgebra.jl/blob/575029c12fcad44b3b41206ee564437a7ec5fa45/src/LieAlgebra.jl).
`rank` is an extension of LinearAlgebra.rank. Supporting functions such as
`closure_residuals`, `normalize_matrix`, and the complex-nullspace helpers are
defined internally and are not exported.

## Semantics to resolve before translating algorithms

### Scalars, closure, and projection

The bracket convention preserves Hermiticity for Hermitian inputs. However,
`MatrixBasis` does not check Hermiticity, and the coefficient solve permits
complex coefficients. The adopted syntaqs target is nevertheless the intended
real Lie algebra of Hermitian operators, rather than an additional general
complex Lie algebra API. See [the mathematical domain](design-questions.md#mathematical-domain).

`coefficients` projects even when the target is outside the span. Likewise,
structure constants and adjoint matrices do not first enforce closure or
independence. A projected bracket must not silently become a claim of exact
closure. The reference checks closure of supplied lists; it does not generate
the Lie closure of arbitrary generators.

syntaqs adopts the reference's projection-coefficient semantics for
`coefficients`, with explicit `project`, scalar `residual`, and `in_span`
operations. Nonorthonormal bases use a Gram solve; operations requiring Lie
algebra properties still verify their mathematical preconditions. See
[the adopted projection policy](design-questions.md#coefficients-projection-and-span-membership).
The operator inner product is fixed to normalized HS, and operations requiring
independence report dependent or numerically singular bases as errors. See
[the adopted inner-product and singular-basis policy](design-questions.md#inner-product-and-singular-bases).

For syntaqs, construction remains separate from mathematical validation, but
operations verify their required preconditions; validation reports retain the
selected tolerances and diagnostics. See
[the adopted basis policy](design-questions.md#basis-construction-and-validation).

The metric used for projection is configurable, but `residual` uses Julia's
ordinary matrix norm of `A-project(A)`, rather than the chosen metric norm.
Thus the default projection uses normalized HS while the residual uses the
unnormalized Frobenius norm. Porting residual thresholds unchanged would produce
dimension-dependent differences if syntaqs uses its normalized HS norm.
The user selected the normalized HS residual norm for syntaqs; see
[the adopted residual definition](design-questions.md#residual-norm).

### Killing form

[`killing.jl`](https://github.com/nattyo1226/LieAlgebra.jl/blob/575029c12fcad44b3b41206ee564437a7ec5fa45/src/killing.jl)
computes `ip(ad_i,ad_j)`. With the default metric this is
`Tr(ad_i† ad_j)/n`, where n is the Lie algebra basis length. The classical
bilinear Killing form is `Tr(ad_i ad_j)` without conjugation or normalization.
These are different contracts. The source behavior above records the inspected
reference implementation. For syntaqs v4, the user selected the classical
bilinear definition `Tr(ad_i ad_j)`, with neither conjugation nor dimension
normalization. See [the adopted decision](design-questions.md#killing-form).
The reference checkout has not been changed.

### Commutant and decomposition

[`commutant.jl`](https://github.com/nattyo1226/LieAlgebra.jl/blob/575029c12fcad44b3b41206ee564437a7ec5fa45/src/commutant.jl)
starts with complex nullspaces but the exported result path requires the
Hermitian parts to be real within tolerance. It is not an unrestricted complex
commutant basis API. For a MatrixBasis, the commutant is in the adjoint
representation, not the centralizer of the original physical matrices.
syntaqs adopts that adjoint-representation meaning; see
[the commutant decision](design-questions.md#adjoint-representation-commutant).

[`decomposition.jl`](https://github.com/nattyo1226/LieAlgebra.jl/blob/575029c12fcad44b3b41206ee564437a7ec5fa45/src/decomposition.jl)
requires a symmetric commutant for its random sample. A nonorthonormal original
basis need not make the adjoint family compatible with the ordinary transpose
used here. Compactness, real structure, metric, input closure, center handling,
and repeated/randomly coincident eigenvalues therefore need explicit contracts
and tests before a general decomposition guarantee is made.

For syntaqs, the decomposition target is now fixed as the full center plus
nonabelian simple ideals. See
[the adopted target](design-questions.md#decomposition-target). The reference's
commutant/random-symmetric-element/eigenspace method is adopted with center
extraction, internal orthonormalization, result validation/retry, and seed
configuration; see [the algorithm](design-questions.md#decomposition-algorithm).
Concrete numerical options and result signatures remain to be designed.

### Precision and tolerance

Several paths force ComplexF64 or Float64, including adjoint matrices,
orthogonalization, commutants, and decomposition intermediates. Many comparisons
default to absolute `1e-10`; Gram-Schmidt compares a squared norm with that same
threshold. syntaqs must decide tolerances according to precision, scale, and
the quantity being measured while preserving the selected coefficient type.
The initial syntaqs policy is to provide precision-aware recommended defaults
and allow explicit tolerance overrides; see
[the provisional numerical error policy](design-questions.md#numerical-error-policy-provisional).

## Existing reference tests

The files under
[`test/`](https://github.com/nattyo1226/LieAlgebra.jl/tree/575029c12fcad44b3b41206ee564437a7ec5fa45/test)
cover the Pauli basis and two commuting Pauli triples on two factors:
independence, a bracket coefficient, cyclic structure constants of 2, a
one-dimensional adjoint commutant for su(2), decomposition into one or two
three-dimensional ideals, cross-component commutation, Hermiticity, and
orthonormalization. Decomposition uses the fixed RNG seed 816.

There are no dedicated Killing-form tests, and these tests do not establish
general behavior for singular/nonorthogonal inputs, custom metrics, centers,
arbitrary complex bases, or precision preservation. Reuse their examples as
acceptance seeds, then add independent checks for the contracts adopted in v4.
