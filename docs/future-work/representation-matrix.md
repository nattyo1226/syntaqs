# Proposed map representation matrices and coefficients

Updated: 2026-11-02

## 1. Scope and responsibilities

Consider representation coefficients with explicit input/output bases, multiple inputs, conjugation per input, and retained/reusable coefficient data as an unimplemented feature. Timing, public APIs, and internal design are undecided.

Generation and interpretation would belong to the algebra core, with numerical storage in export. Combine basis enumeration, ordinary operations, expansion, and coordinates without requiring a dedicated trait per local algebra. Current state/operator coordinates and action-matrix export exist independently of this general mechanism.

Assume each system uses one pair of primitive types S/O, with one local basis per input or output applied to all its factors. Different inputs and outputs may choose different bases. Maps connecting heterogeneous local structures require separate consideration.

## 2. Representation matrices of linear maps

For a complex-linear map `T: V→W` between finite-dimensional state/operator spaces, let `{B_μ}` be the input basis and `{F_ρ}` the output basis. Its matrix M satisfies:

```math
T(B_\mu)=\sum_\rho M_{\rho\mu}F_\rho,
\qquad
X=\sum_\mu x_\mu B_\mu
\Longrightarrow
[T(X)]_F=Mx.
```

Column μ contains the output coordinates of `T(B_μ)`. Input and output bases may differ, but each must span its full target space. Composite bases are tensor products of local bases on basic factors.

For example, fixing an ordinary operator A gives the state-space map `T(ψ)=Aψ` and operator-space maps `T(X)=AX`, `XA`, and `AX−XA`. Fixed means A is not a variable input to this representation; it introduces no special operator kind. A fixed state's `T(A)=Aψ` uses the same framework.

## 3. Conjugate-linear and multiple-input maps

Treat maps linear or conjugate-linear in each input separately. Specify input order and conjugation explicitly as a mathematical contract of the supplied operation; do not infer them from its action on basis elements. General nonlinear maps are outside this framework.

For a k-input map T, define coefficients by:

```math
T(B^{(1)}_{\mu_1},\ldots,B^{(k)}_{\mu_k})
=\sum_\rho C_{\rho,\mu_1,\ldots,\mu_k}F_\rho.
```

For coefficient vector `x^(j)`, take `q^(j)=x^(j)` for a linear input and `q^(j)=overline(x^(j))` for a conjugate-linear input. Combining input indices into one column gives:

```math
M_{\rho,\operatorname{index}(\mu_1,\ldots,\mu_k)}
=C_{\rho,\mu_1,\ldots,\mu_k},
\qquad
z=M\left(q^{(1)}\otimes\cdots\otimes q^{(k)}\right).
```

Columns use mixed-radix order in the specified argument order, with the last input's basis index varying fastest. Each input retains its ordinary basis order. Tensor products here combine numerical coordinates; signs from graded physical operations are already part of evaluating the operation and must not be reapplied during coefficient contraction.

| Kind | Action on coefficients |
| --- | --- |
| One linear input | `z = Mx` |
| One conjugate-linear input | `z = K overline(x)` |
| Two bilinear inputs | `z = M(x ⊗ y)`; unflattened coefficients are `C_{ρμν}`. |
| First input conjugate-linear, second linear | `z = M(overline(x) ⊗ y)` |

Scalar output uses the one-dimensional space C with basis `{1}`. Bilinear scalar maps may be written `xᵀCy`, and sesquilinear ones `x†Gy`. Turning a column into a row does not conjugate it: distinguish transpose `xᵀ` from adjoint `x†`. Although `z = K overline(x)` can also be written `zᵀ = x†Kᵀ`, row/column orientation alone must not specify conjugation.

## 4. Construction from existing capabilities

Build the common operation as follows:

1. Enumerate input basis elements, or tuples of them, in the specified order.
2. Apply the ordinary operation to each input.
3. Expand the result in the output basis and obtain its coordinates, or use its scalar value.
4. Associate coefficients with output rows and input columns.

Custom local structures, primitives, and rules can use this mechanism through existing operation/basis contracts without another mathematical capability. Evaluate basis elements with coefficient 1 even for conjugated inputs; apply the specified conjugation to general input coefficients later.

Allow computation of only needed columns/components. Do not require complete large matrices, operator multiplication tables, or input tensor-product coordinates to be materialized together.

No dedicated trait is currently required. Consider replaceable direct, sparse, or factorized computation paths if needed. Reusing an explicitly computed matrix does not imply adding automatic evaluation-result caching in core.

## 5. Coordinate, space, and numerical contracts

- Reconstructed coefficient actions must mathematically match direct operations, without projection or component pruning.
- Even in non-orthonormal bases, matrix entries are output expansion coordinates, not raw inner products with basis elements.
- Inner-product coefficients are `G_{μν}=⟨B_μ,B_ν⟩`, generally not identity; the corresponding contraction is `x†Gy`.
- Retain enough information to interpret input/output spaces, bases/orders, argument order, conjugation, and precision. Concrete storage and transfer remain undecided.
- Preserve existing precision, special-value, approximate-comparison, and space-selection contracts. Bitwise floating-point equality is not required.

Ordinary expression trees do not retain basis metadata. Numerical coefficient vectors/matrices must nevertheless identify their basis. This does not require basis labels on expanded expression trees. Concrete dense/sparse storage belongs at the external numerical-library boundary.

## 6. Verification and open questions

Compare representation-based and direct state action, left/right operator multiplication, and commutators. Verify conjugate-linearity of adjoints, bilinearity of two variable operators, and sesquilinearity of inner products with complex input coefficients. Confirm that transpose alone cannot replace conjugation.

Cover non-orthonormal output coordinates and Gram matrices, different input/output bases, composite order, multi-input column numbering, graded signs, and ordinary expression reuse without basis metadata. State tolerances under existing numeric verification conventions.

Signatures, map/input/conjugation descriptors, component enumeration/export, and optimization paths remain undecided. Preserve construction from existing capabilities when making these decisions. Track the common linear, conjugate-linear, bilinear, and sesquilinear operations and their verification in [future work](../future-work.md#2-unimplemented-features-and-proposals).
