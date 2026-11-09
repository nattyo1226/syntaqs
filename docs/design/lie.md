# Operator families, Lie representations, and real HS projection

Updated: 2026-11-09

## 1. Scope and contexts

`syntaqs-lie` currently provides ordered operator families, explicit Hermiticity
and independence diagnostics, Gram factorizations, real projection coordinates,
reconstruction, residuals, span membership, ordinary Lie brackets, closure
diagnostics, retained Lie representations, real family transformations, the
full real adjoint commutant, and the full center with its HS-orthogonal
complement. Simple-ideal decomposition remains unimplemented.

`OperatorFamily<E>` uses the sealed `FamilyExpression` adapter for core's
`LocalOperator<O,Complex<R>>` and `Operator<I,O,Complex<R>>`, with R=f32/f64.
Primitives supply Product, Adjoint, Trace, Identity, and PartialEq; placed inputs
also use core's parity traits and I: Eq. No Clone on primitives or indices,
Dimension, state type, state basis, or physical matrix export is required.
The metric is fixed normalized HS, `Tr(A†B)/D`.

`new(inputs)` rejects empty input and preserves order and original expression
handles. Local context is `()`. For placed operators it selects the first
available binding, requires all existing bindings to agree, and validates every
referenced factor. Entirely unbound placed inputs require `new_in(&space, inputs)`.
The explicit family context also rejects conflicting old bindings. Rebind in
core before construction if replacement is intended. Zero and zero coefficients
do not bypass context checks. Targets obey the same checks.

## 2. Validation and retained results

Construction does not require mathematical validity. `gram(&options)` evaluates
the full complex `G[i,j]=<B_i,B_j>` and retains the family, options, and norms.
`GramMatrix::validate()` and `OperatorFamily::validate(&options)` return
`FamilyValidation`: ordered checks of `||B_i-B_i†||` at scale `||B_i||`, and an
`IndependenceReport`. No closure or Lie-algebra validity is claimed.

Real independence uses the eigenvalues of `Re(G)`, the real Hilbert-space Gram
matrix even for non-Hermitian candidates. The report records eigenvalues,
dimension, rank, relative cutoff, and absolute eigenvalue threshold. Dependent
and non-Hermitian candidates produce failed reports; invalid settings, nonfinite
data, or a materially indefinite Gram matrix produce errors.

`GramMatrix::factorize()` additionally requires Hermiticity, checks imaginary
Gram entries against norm-product scales, rejects singularity under the Gram
cutoff, and retains an LU factorization. It does not apply the separate SVD rank
cutoff to Gram eigenvalues. There is no pseudoinverse, input removal, custom
metric, validated-family flag, or automatic caching.

Gram matrices, factorizations, coordinates, projections, and validation reports
borrow the originating immutable family. Coordinates follow its input order;
their `reconstruct()` uses that exact family and context. Explicit reconstruction
from a finite real vector also works on dependent/non-Hermitian candidates.

## 3. Projection and numerical scales

The family and target must be Hermitian for projection. Coordinates solve
`G c=b`, `b_i=<B_i,A>`; allowed imaginary roundoff is checked at scale
`||B_i||*||A||`. An out-of-span target still returns projection coordinates.
`project` retains the target, coordinates, reconstructed projection, residual
expression `A-project(A)`, and the effective span check. `residual` returns its
normalized HS norm. `in_span` uses `residual <= atol + rtol*||A||`.

Hermiticity uses norm units; imaginary norm-squared checks and negative-norm
roundoff use squared units. Residual squared-norm roundoff uses
`(||A||+||project(A)||)^2`; Hermiticity squared-norm roundoff uses `(2||B||)^2`.
Self-contractions also use their scalar magnitude as a minimum squared scale.
Invalid or overflowing bounds error. Small positive values and reconstruction
coefficients are not pruned. See the
[numerical settings](../../crates/syntaqs-lie/README.md).

Before contraction, Lie operations explicitly collect equal natural primitives
using core's `collect_terms`/`collect_terms_in`. Local sums remain factored;
compatible tensor terms merge without Cartesian expansion. This reduces
cancellation before squaring without changing ordinary evaluation or imposing an approximate-zero
policy. Collection uses primitive equality, not a complete independent operator
basis; distinct natural primitives may still have linear relations. No general
accuracy guarantee beyond the recorded numerical checks is asserted.

## 4. Brackets, closure, and retained representations

The scalar field is real and the operators are Hermitian. `lie_bracket(A,B)`
constructs `-i(AB-BA)` without numerical validation. `family.bracket(i,j)` also
places the result in the family's common context. This is the ordinary Lie
bracket even on graded spaces; core evaluates physical grading signs.

`bracket_coefficients` returns real projection coordinates, including for
out-of-span brackets. `bracket_projection` additionally retains the residual
and span check. Both are available on families and reusable Gram factorizations.
`closure` requires Hermiticity and independence, reuses one factorization for
all ordered pairs including diagonals, and returns `ClosureReport`. Each
`BracketClosureCheck` records input indices and the residual check, using the
bracket's normalized HS norm as scale. Nonclosed candidates return failed checks;
dependent and non-Hermitian candidates must first be diagnosed with `validate`.
No inputs are removed and no pseudoinverse is used.

Only a passing closure report can become `StructureConstants`; otherwise
`NonClosedPair` records the first failed pair and its effective check. Values
obey `-i[B_i,B_j]=sum_k f[k,i,j] B_k`, with output index first. The constants
retain the full closure report. `adjoint()` returns `AdjointRepresentation`
with matrices `ad_i[k,j]=f[k,i,j]`, whose columns act on real family coordinates.
`killing_form()` on the adjoint representation returns `KillingForm`, with
`K[i,j]=Tr(ad_i ad_j)` and no conjugation, sign change, or normalization.
All retained results borrow the exact originating family and retain numerical
settings. Its context, element order, and real precision define the metadata.
All claims of closure are relative to the recorded numerical tolerances.

## 5. Family transformations and orthogonalization

`transform(U)` constructs candidates with `B'_j=sum_i B_i U[i,j]`. The finite
real matrix has one row per original input and at least one column; rectangular
and singular transforms are allowed. `select(indices)` constructs the equivalent
selection matrix, preserving requested order and allowing repeated indices.
Neither operation demands Hermiticity, independence, or closure.
`FamilyTransform` owns the new family, borrows its source, and retains `U`.
It preserves the common context and supports reconstruction through its columns.

`orthogonalize(options)` requires independent Hermitian inputs and uses Gram
eigenvectors as columns of `U`. `orthonormalize(options)` additionally divides
each column by the square root of its Gram eigenvalue. Both retain settings,
reject Gram singularity, and preserve the span without pruning. Eigenvector
signs and ordering within degenerate eigenspaces are not prescribed.
`orthonormality(options)` reports Hermiticity, independence, and the Frobenius
norm of complex `G-I`, checked with the residual tolerance at scale `sqrt(n)`.
Its `passed()` requires all these checks to pass. Closure is not required.

## 6. Adjoint commutant and center

`AdjointRepresentation::commutant()` solves the real homogeneous equations
`C ad_i-ad_i C=0` for every generator. Unknowns are row-major matrix entries;
constraint rows enumerate `(generator,output_row,output_column)`. Full SVD right
nullspaces yield `AdjointCommutant::matrices()`, a Frobenius-orthonormal basis of
the full real endomorphism space in original family coordinates. Under a real
invertible basis transformation `U`, its matrix subspace transforms by
`C_new=U^-1 C_old U`. Individual SVD basis matrices are not prescribed.
This is distinct from a physical-operator centralizer and from the smaller
symmetric commuting subspace. In particular an n-dimensional abelian algebra
has a full n²-dimensional real commutant.

Center extraction constructs internal normalized-HS orthonormal coordinates.
With Gram matrix `G`, the retained forward transform `U` obeys `U^T G U=I`,
and its inverse is `V=U^T G`. Generators and action coordinates both transform:
`ad'_j=V (sum_i U[i,j] ad_i) U`. The common-kernel SVD stacks these transformed
adjoint matrices. Its nullspace columns `Z` span the full center; the remaining
right singular vectors `S` span the Euclidean orthogonal complement. Both come
from the same rank classification, without applying another cutoff to obtain
the complement.

`AdjointRepresentation::center()` returns `CenterSplit` with original coordinate
columns `U Z` and `U S`, reconstructed as HS-orthonormal operator families in the
original common context. `center()` and `complement()` accessors return `Option`:
centerless input has no center family, and purely abelian input has no complement
family. Absent coordinate matrices retain n rows and zero columns. No empty
`OperatorFamily` is constructed. The split retains forward/inverse orthonormal
transforms and centrality checks in original coordinates. In the accepted
Hermitian compact domain, this complement is the semisimple part; simple-ideal
splitting is not performed here.

Family convenience methods `commutant(options)` and `center(options)` require
Hermiticity, independence, and closure before obtaining the adjoint representation.
Methods on a retained adjoint reuse its coefficients and settings. All results
borrow the originating family. A legitimate constraint nullspace, including the
whole ambient space for zero constraints, is not a singular operator basis.

`ConstraintDiagnostics` records constraint shape, descending singular values,
rank, effective relative cutoff, absolute threshold, and each nullspace vector's
residual check. Rank uses `options.rank_cutoff(rows,columns)`. For a real constraint
map `L` and candidate vector `v`, the check measures `||L v||_2` at scale
`||L||_F ||v||_2`, using the residual tolerance. Matrix unknowns use row-major
vectorization, so their vector norm is Frobenius. Candidates exceeding this bound
return `ConstraintResidual`, even if the rank cutoff classified them as null.
Center diagnostics refer to internal orthonormal coordinates; `centrality()`
additionally checks the mapped columns against the original stacked adjoints.
All dimensions and validity claims are relative to the recorded tolerances.

The internal `SymmetricCommutant` solves constraints restricted to real symmetric
matrices in HS-orthonormal coordinates, with Frobenius-orthonormal parameter basis
`E_aa` and `(E_ab+E_ba)/sqrt(2)`. It is a distinct internal object for subsequent
spectral ideal splitting and is not exposed as the public full commutant. Its
kernel uses the same rank/residual diagnostics. For a three-dimensional abelian
algebra its dimension is six, whereas the public full commutant has dimension nine.

## 7. Verification

[Public family tests](../../crates/syntaqs-lie/tests/operator_family.rs) compare
against independent small matrices at both precisions, covering nonorthonormal
projection, real basis changes, in/out-of-span targets, complex expression
coefficients, context errors, singularity, overrides, and tiny residuals.
An ordinary 32-factor product of local sums checks that contraction and projection
do not enumerate all primitive strings.
[Collection tests](../../tests/tests/core_collect_terms.rs) independently check
ordinary/graded signs and primitives without Clone or Dimension. General native
linkage and check commands are in [verification](verification.md).

[Representation tests](../../crates/syntaqs-lie/tests/representations.rs) check
Pauli su(2) constants and `K=-8I`, abelian zero Killing forms, antisymmetry,
Jacobi, direct and coefficient adjoint actions, nonorthonormal transforms,
`K_new=U^T K_old U`, closure failures, orthogonalization, and numerical/context
errors at both precisions. A two-mode odd-Majorana example compares the ordinary
graded bracket against independent Jordan–Wigner matrices. Capability coverage
also exercises representations and orthonormalization without primitive/index
Clone, Dimension, or state traits.

[Invariant tests](../../crates/syntaqs-lie/tests/invariants.rs) compare full
commutant spans and center projectors against independent su(2) block matrices,
including a single factor, two simple factors, one- and multidimensional centers,
purely abelian families, real nonorthonormal basis changes, and a graded example.
They check original-coordinate residuals, reconstruction, complete orthogonal
center/complement spans, absence cases, rank/residual overrides, and invalid
preconditions at f32/f64. Internal tests distinguish symmetric/full dimensions
and verify orthonormal generator/action transforms and symmetric constraints.
Capability tests also cover these APIs without primitive/index Clone, Dimension,
state, or state-basis traits.
