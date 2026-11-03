# Operator families and real HS projection

Updated: 2026-11-04

## 1. Scope and contexts

`syntaqs-lie` currently provides ordered operator families, explicit Hermiticity
and independence diagnostics, Gram factorizations, real projection coordinates,
reconstruction, residuals, and span membership. Brackets, closure validation,
adjoint representations, and decomposition remain in the
[active implementation plan](../plans/v4-lie-algebra/implementation-plan.md).

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
[numerical specification](../plans/v4-lie-algebra/spec.md#numerical-contract).

Before contraction, Lie operations explicitly collect equal natural primitives
using core's `collect_terms`/`collect_terms_in`. Local sums remain factored;
compatible tensor terms merge without Cartesian expansion. This reduces
cancellation before squaring without changing ordinary evaluation or imposing an approximate-zero
policy. Collection uses primitive equality, not a complete independent operator
basis; distinct natural primitives may still have linear relations. No general
accuracy guarantee beyond the recorded numerical checks is asserted.

## 4. Verification

[Public family tests](../../crates/syntaqs-lie/tests/operator_family.rs) compare
against independent small matrices at both precisions, covering nonorthonormal
projection, real basis changes, in/out-of-span targets, complex expression
coefficients, context errors, singularity, overrides, and tiny residuals.
An ordinary 32-factor product of local sums checks that contraction and projection
do not enumerate all primitive strings.
[Collection tests](../../tests/tests/core_collect_terms.rs) independently check
ordinary/graded signs and primitives without Clone or Dimension. General native
linkage and check commands are in [verification](verification.md).
