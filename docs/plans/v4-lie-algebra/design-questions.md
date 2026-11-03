# Design questions and provisional work order

Status: adopted mathematical decisions and remaining design questions; public
APIs are not yet settled.

## Adopted decisions

### Mathematical domain

The target is the real Lie algebra spanned by Hermitian operators, with bracket
`-i(AB-BA)`. Real coefficients describe combinations of Lie algebra basis
elements; complex coefficients remain necessary inside operator expressions.
Using complex storage does not imply taking the complex span. General complex
Lie algebras are not an additional requirement of this implementation.

### Basis construction and validation

Keep construction lightweight and separate mathematical validation from it:

- Store a nonempty, ordered collection of existing operator expressions with a
  common ambient Hilbert space. Check space compatibility at construction.
  Keep this collection distinct from core's complete primitive `Basis` trait.
- Permit candidate collections before Hermiticity, independence, or closure is
  established, including redundant or nonclosed collections.
- Provide explicit checks for Hermiticity, real linear independence, and closure
  under the selected bracket, with caller-specified tolerances. Diagnostics
  identify offending elements or pairs and report relevant residuals.
- Operations such as structure-constant construction and decomposition verify
  their required mathematical preconditions before returning a result. Do not
  silently report projected brackets of a nonclosed collection as Lie algebra
  structure constants. The exact preconditions of each operation will be
  documented with its API.
- Return a validation report that records the tolerances used, rather than
  attaching a generic validated flag to the collection. Allow explicit reuse
  of computed data, such as Gram matrices, when applicable; this does not adopt
  automatic caching.
- Do not introduce a separate validated-basis type initially. Revisit it only
  when a concrete API need is established.

Public signatures, diagnostic layouts, numeric thresholds, and detailed error
precedence remain implementation design work. In particular, handling small
imaginary residuals in mathematically real coordinates needs an explicit
numerical contract.

### Inner product and singular bases

Fix the operator inner product to the dimension-normalized HS inner product
`<A,B>=Tr(A†B)/D` on the selected ambient Hilbert space. Use it for Gram
matrices, projection, orthonormalization, and residual norms. Custom inner
products are outside this iteration's API. This does not change the separately
adopted classical Killing form, which is a bilinear form rather than this
operator inner product.

Candidate collections may still be constructed before independence is
established. Independence checks report dependent or numerically singular
collections in their diagnostics. Operations requiring an independent basis
must report an error for such inputs, rather than silently choose coefficients
with a pseudoinverse or discard redundant elements. This includes coefficient
solves and the basis-dependent operations built on them.

Numerical rank/conditioning decisions follow the configured tolerance policy;
specific thresholds and diagnostic/error types remain to be designed. A
nonorthonormal but numerically independent basis is supported. Singular
constraint matrices used to compute a legitimate nullspace, center, or
commutant are not themselves invalid bases and are not errors under this rule.

### Coefficients, projection, and span membership

Adopt projection-coefficient semantics, as in the reference library, and keep
span membership as a separate explicit check. The operation names below
describe the selected semantics; concrete Rust signatures remain undecided.

- `coefficients(A)` returns the coefficients of the projection onto the supplied
  basis span, even when A lies outside that span.
- `project(A)` reconstructs the projected operator from these coefficients.
- `residual(A)` returns the norm of `A-project(A)`, using the normalized HS norm
  specified below.
- `in_span(A, tolerance)` compares that residual with the selected tolerance.

Support nonorthonormal, independent Hermitian bases. For normalized HS
projection, form `G_ij=<B_i,B_j>` and `b_i=<B_i,A>`, then solve `Gc=b`.
For an orthonormal basis this reduces to `c_i=<B_i,A>`. These are real
coordinates under the adopted mathematical domain, subject to the explicit
roundoff policy still to be defined.

Reconstruction equals A only when A lies in the span, within the selected
numerical tolerance. Operations requiring an exact representation in that span,
such as Lie algebra structure constants and decomposition, must verify the
necessary membership/closure conditions rather than silently use projection to
hide a missing component. This follows the adopted validation policy above.
Dependent or numerically singular bases produce errors for coefficient solves
under the adopted policy above; accepting a candidate collection does not
guarantee a coefficient solve.

### Residual norm

Measure projection residuals with the dimension-normalized HS norm. For a
projection P(A) on a selected ambient Hilbert space of dimension D, define
`R=A-P(A)` and

```math
\|R\|_{\mathrm{HS,norm}}
=\sqrt{\operatorname{Tr}(R^\dagger R)/D}.
```

Use the existing normalized HS contraction rather than constructing a dense
physical matrix or requiring an integer total dimension. This differs from the
reference library's unnormalized Frobenius residual. Numerical tolerances and
handling of roundoff in squared norms remain to be specified.

The adopted coefficient API returns projection coefficients and checks
membership separately, as specified above. This distinction is independent of
whether the basis is orthonormal.

### Killing form

Use the classical bilinear Killing form:

```math
K(X,Y)=\operatorname{Tr}(\operatorname{ad}_X\operatorname{ad}_Y).
```

The trace is taken on the Lie algebra, with no adjoint/conjugation and no
dimension normalization. The definition applies to the chosen Lie bracket and
does not require an orthonormal basis. Do not substitute the reference library's
normalized HS Gram matrix of adjoint representations. A separate adjoint-Gram
API is not required by this decision.

This definition was selected by the user during design discussion. It changes
the planned syntaqs behavior relative to LieAlgebra.jl; the reference checkout
has not been modified.

### Adjoint-representation commutant

For a Lie algebra basis B, `commutant` refers to endomorphisms of the Lie
algebra commuting with all adjoint-representation matrices:

```math
\operatorname{Comm}(\operatorname{ad}\mathfrak{g})
=\{C\in\operatorname{End}_{\mathbb R}(\mathfrak{g})
\mid C\operatorname{ad}_{B_i}=\operatorname{ad}_{B_i}C\ \text{for all }i\}.
```

Its matrices are expressed in the chosen Lie algebra basis. This is the
commutant used for decomposition and matches the intended role of
`commutant(MatrixBasis)` in the reference. It is not the centralizer of the
original operators in the physical Hilbert space; this decision does not add
a separate physical-centralizer API.

Concrete result types and nullspace routines remain open. The adopted
decomposition algorithm below uses the symmetric part in orthonormal
coordinates. Distinguish that restricted subspace from the full commutant
rather than treating them as the same mathematical object.

### Decomposition target

Decompose the adopted real Hermitian Lie algebra into its full center and
nonabelian simple ideals:

```math
\mathfrak{g}=\mathfrak{z}(\mathfrak{g})
\oplus\mathfrak{g}_1\oplus\cdots\oplus\mathfrak{g}_k.
```

- Return the center distinctly from the simple ideals. Keep a multidimensional
  center as one central subspace; choosing one-dimensional directions in it is
  not a simple-ideal decomposition requirement.
- Each noncentral component must be a simple ideal, not merely a closed
  subalgebra or an arbitrary invariant block.
- The components must span the input with no overlap; each must be an ideal of
  the input algebra, and different components must commute. Verify these
  properties within explicitly stated numerical tolerances under the adopted
  validation policy.
- A purely abelian input has the entire input as its center and no simple
  ideals. A centerless input has zero-dimensional center.
- Component bases and the order of the simple ideals need not be unique.
  Compare the represented subspaces and algebraic properties, rather than
  individual basis vectors or return order.

The earlier discussion of a remembered reference-library error is set aside
and is not an implementation requirement or an outstanding investigation for
this work.

### Decomposition algorithm

Adopt the reference library's commutant/random-symmetric-element/eigenspace
method, with the agreed center handling, coordinate preparation, and result
validation:

1. Validate the required Hermiticity, independence, and closure properties.
   Internally use normalized-HS orthonormal coordinates, retaining the
   transformation from the original basis for reconstruction.
2. Extract the full center from the common kernel of the adjoint action.
   Return it as a distinct subspace and apply the splitting algorithm to its
   orthogonal complement, the semisimple part in the adopted domain.
3. Compute the adjoint representation and its commutant on that semisimple
   part. Obtain the real symmetric commutant subspace in these orthonormal
   coordinates.
4. Form a random real linear combination of these symmetric matrices,
   diagonalize it, and group eigenvalues under the configured tolerances.
   Use the eigenspaces as candidate simple-ideal components.
5. Validate the result against the adopted decomposition target, including
   complete splitting into simple ideals. Retry random sampling when an
   accidental eigenvalue coincidence or an insufficient split prevents a
   complete validated result. A failed attempt must not be returned as a
   successful complete decomposition.
6. Reconstruct the center and ideal bases as linear combinations of the
   original syntaqs expressions, retaining the original ambient space.

Allow the caller to specify a random seed for reproducibility. Numerical
thresholds, the concrete RNG and sampling configuration, retry limits and
exhaustion diagnostics, and Rust result types remain implementation details
to specify. Reproducibility does not require bitwise agreement across different
LAPACK implementations or platforms.

### Numerical error policy (provisional)

Use precision-preserving, operation-specific tolerances with recommended
defaults and caller-supplied overrides. The user adopted the initial values
below for modest Lie algebra dimensions, subject to verification with the
selected backend and representative regression cases. These are planned
defaults, not measured accuracy guarantees or executed verification results.

| Test | f32 initial default | f64 initial default |
| --- | --- | --- |
| Residuals, Hermiticity, and small imaginary parts | `atol=rtol=2e-5` | `atol=rtol=1e-12` |
| Rank/nullspace relative cutoff | `max(2e-5,d*epsilon)` | `max(1e-12,d*epsilon)` |
| Eigenvalue-grouping relative cutoff | `max(2e-5,n*epsilon)` | `max(1e-12,n*epsilon)` |

Here epsilon is the machine epsilon of the actual real precision,
`d=max(rows,columns)` for the matrix under analysis, and n is the symmetric
eigenproblem size. Defaults are independent settings even where their initial
numeric values coincide.

For residual-based tests, use `residual <= atol + rtol*scale`, with a documented
scale appropriate to the operation. For span membership the scale is the
normalized HS norm of the target. Hermiticity and imaginary-part checks must
similarly specify the operator/coefficient norm being compared. Norm-squared
roundoff handling is a separate rule still to be specified.

For rank/nullspace tests on a matrix, classify singular values
`sigma <= relative_cutoff*sigma_max` as numerical zero. The dimension-times-
epsilon component follows the standard defaults documented by
[NumPy matrix_rank](https://numpy.org/doc/stable/reference/generated/numpy.linalg.matrix_rank.html)
and [SciPy null_space](https://docs.scipy.org/doc/scipy/reference/generated/scipy.linalg.null_space.html).
The `2e-5` / `1e-12` floors are the adopted syntaqs initial recommendation,
not defaults taken from those libraries. When checking independence through
Gram-matrix eigenvalues, document and report the eigenvalue cutoff separately:
the Gram matrix has squared singular-value scaling and its own rounding error.
Do not silently identify its eigenvalue threshold with a singular-value
threshold for the original operator family.

For eigenvalue grouping, use the relative cutoff multiplied by the largest
absolute eigenvalue of the sampled symmetric matrix as the gap threshold.
This avoids introducing a fixed absolute eigenvalue scale when the sample is
rescaled. Preserve the agreed validation/retry behavior for an insufficient
split.

- Preserve f32/f64 throughout the calculation; do not implicitly promote or
  narrow coefficient precision to implement tolerance checks.
- Choose defaults appropriate to precision, scale, and problem size. Consider
  absolute and relative tolerances rather than copying one fixed `1e-10`
  threshold to every operation.
- Distinguish tolerances for Hermiticity, span/closure residuals, rank and
  nullspace decisions, normalization, and eigenvalue grouping. Their tested
  quantities and scales differ; a norm and a squared norm must not silently use
  an interchangeable threshold.
- Allow explicit overrides through per-operation options or a shared
  configuration object; the concrete form remains open. Propagate the selected
  settings to internal checks rather than replacing overrides with hidden
  defaults.
- Validation reports record the effective tolerances and relevant diagnostics,
  including the normalization/scale used to interpret a residual.
- Define an explicit policy for small imaginary parts of mathematically real
  coordinates and for roundoff in squared norms. Thresholds must be visible and
  configurable; materially invalid values must not be silently corrected.
- Keep these approximate decisions separate from ordinary expression
  evaluation and coordinates, which retain their existing exact-zero rules.

Verify these initial values with the chosen numerical backend and
representative regression cases. Concrete Rust configuration signatures,
remaining operation-specific scale rules, and edge-case behavior still need
implementation design.

### Algebraic and numerical boundary

Use existing syntaqs expressions and algebraic operations wherever possible,
and dense ndarray representations for numerical coefficient calculations.
The initial implementation targets modest Lie algebra basis dimensions; dense
storage and direct factorizations are the intended starting point.

| Work | Responsibility |
| --- | --- |
| Operator expressions, placement, linear combinations, brackets, adjoints, traces, and normalized HS contractions | Existing syntaqs capabilities. |
| Gram matrices, coefficient vectors, structure constants, adjoint matrices, and basis transformations in coordinates | Dense ndarray arrays, populated using syntaqs operations. |
| Linear solves, rank/nullspace calculations, and eigenvalue decompositions | Use ndarray-linalg as needed for routines on these arrays. |
| Reconstruction of projected operators and decomposition component bases | Linear combinations of the original syntaqs expressions using computed coefficients. |

The dense dimensions refer to the chosen Lie algebra basis, not the physical
Hilbert-space action matrices. A list of generators can generate a larger
algebra; cost estimates use the dimension of the actual basis being analyzed.
This boundary does not require full physical action-matrix export or sparse
and iterative numerical backends for the initial implementation. It preserves
the existing core dependency direction and no_std contract.

The workspace already uses ndarray 0.17.2. Its
[documentation](https://docs.rs/ndarray/0.17.2/ndarray/#the-ndarray-ecosystem)
places eigenvalue and SVD routines in a companion library.
[ndarray-linalg](https://docs.rs/ndarray-linalg/0.18.1/ndarray_linalg/) provides
linear solves, QR/SVD, and eigendecomposition using LAPACK. The user approved
using it when these routines are needed. Its
[published dependency metadata](https://docs.rs/crate/ndarray-linalg/0.18.1)
specifies ndarray `^0.17.1`, which includes the workspace's ndarray 0.17.2.
Use a compatible version, initially targeting this pairing, while checking
the workspace's Rust 1.85 compatibility in actual builds. This planning update
does not yet change Cargo dependencies.

### User-selected LAPACK implementation

Let the final application choose the LAPACK implementation at build/link time.
The syntaqs-lie production dependency on ndarray-linalg must not select a
provider, and should use `default-features=false`. A planned dependency entry
is:

```toml
ndarray-linalg = { version = "0.18.1", default-features = false }
```

An application can enable the provider on its matching direct dependency,
for example:

```toml
ndarray-linalg = { version = "0.18.1", default-features = false, features = ["openblas-system"] }
```

The application's feature selection then applies to the shared ndarray-linalg
dependency through Cargo feature unification. This is the
[upstream recommendation for library developers](https://docs.rs/crate/ndarray-linalg/0.18.1#for-library-developer).
Other upstream provider features include `netlib-system` and `openblas-static`;
see the [versioned feature list](https://docs.rs/crate/ndarray-linalg/0.18.1/features).
Select a single compatible LAPACK provider for the final linked application.
This is a compile-time configuration, not runtime backend switching.

Keep provider choice for CI, development, tests, and examples separate from
the production library dependency. Exact CI/Nix configuration and whether to
offer any convenience forwarding features remain implementation details. If
forwarding features are offered, account for mutually incompatible providers
when adapting the existing workspace all-features checks.

### Crate placement

Place the Lie algebra functionality in a dedicated workspace crate,
`syntaqs-lie`, planned at `crates/syntaqs-lie`.

- Own the Lie algebra basis collection, validation reports, projection and
  residual operations, Lie-specific bracket helpers, structure constants,
  adjoint representations, Killing form, commutant, orthogonalization, and
  center-plus-simple-ideal decomposition in this crate.
- Build on public syntaqs-core expressions, spaces, and algebraic capabilities;
  reuse existing operations rather than introducing another expression tree or
  evaluator.
- Keep dense ndarray coefficient data and ndarray-linalg/LAPACK integration at
  this crate's numerical boundary. Do not add these dependencies to core.
- Keep production functionality independent of concrete spin/fermion crates;
  use those algebras in examples and tests as appropriate.

The core dependency direction and its existing no_std contract remain intact.
Internal module layout, feature flags, and concrete dependency configuration
will be designed during implementation. This planning update does not create
an empty crate or modify workspace manifests.

### Representation storage scope

Implement coefficient generation, storage, and interpretation entirely within
syntaqs-lie for this iteration. Use Lie-specific result types for structure
constants, adjoint matrices, Gram/Killing matrices, and basis transformations,
with the basis, space, ordering, and other metadata needed to interpret them.
Retain the adopted output-first structure-constant convention `f[k,i,j]` and
the corresponding adjoint-matrix column convention.

Generalization with the existing
[map-representation proposal](../../../docs/future-work/representation-matrix.md)
is deferred to future work. Do not make general map abstractions, shared
coefficient containers, or adapters for arbitrary map bases prerequisites of
this implementation. In particular, this iteration does not require a new
general map-representation API in core or export.

Continue to reuse existing public syntaqs capabilities. The concrete Lie-specific
types and ownership strategy remain implementation design work; future
commonality can be assessed after this functionality is established.

### Adopted type names

Use names reflecting the mathematical role and construction guarantees rather
than copying matrix-oriented names from Julia.

| Reference type | Adopted Rust name | Meaning |
| --- | --- | --- |
| `MatrixBasis` | `OperatorFamily` | A nonempty ordered collection of syntaqs operator expressions on a common space, which may still await independence/Hermiticity/closure validation. |
| `Decomposition` | `LieDecomposition` | The full center together with nonabelian simple ideals, represented through operator expressions. |

`OperatorFamily` does not imply that construction establishes a basis. It can
serve as a basis after the appropriate validation; no separate validated-basis
type is introduced.

For `LieDecomposition`, use distinct center and simple-ideal accessors rather
than one undifferentiated `ideals` list. Represent a zero-dimensional center
explicitly; `Option<OperatorFamily<...>>` remains a possible layout compatible
with the nonempty family constructor. Concrete generics, ownership, and result
layout remain to be designed.

## Decisions to resolve

| Question | Initial direction / evidence |
| --- | --- |
| What are the validation APIs and diagnostics? | Follow the adopted construction/validation policy above. Define operation-specific preconditions, tolerance/report structures, diagnostic details, and error precedence. |
| What are the concrete projection APIs and singular-basis diagnostics? | Follow the adopted projection-coefficient semantics and fixed normalized HS inner product. Define Rust signatures, numerical singularity thresholds, and error/diagnostic types; custom inner products are out of scope. |
| How is the adjoint commutant computed and represented? | Its mathematical target is fixed above. Choose the numerical algorithm, result representation, and explicit handling of any symmetric or metric-self-adjoint subspace used for decomposition. |
| What are the concrete decomposition options and result types? | Follow the adopted Julia-style algorithm with center extraction, internal orthonormalization, validation/retry, and caller-supplied seeds. Define result types, validation details, RNG configuration, retry limits, and exhaustion errors. |
| Which factorization routines and test/build configuration are used? | Use a compatible ndarray-linalg version, initially 0.18.1 for ndarray 0.17.2. LAPACK choice belongs to the final application. Select routines and CI/Nix/example configurations, and verify Rust 1.85 and actual linking. |
| What are the concrete Lie-specific representation types? | Keep coefficient data and interpretation within syntaqs-lie under the adopted storage scope. Define types and ownership retaining basis, space, ordering, bracket, metric, and precision as appropriate. General map commonality is deferred. |
| What are the concrete tolerance options and remaining scale rules? | The initial default values are adopted above, with explicit overrides. Define remaining per-operation scale rules, Gram-eigenvalue handling, effective settings/reporting, edge cases, and the configuration API; verify defaults on representative cases. |
| What is the compatibility target? | Preserve supported mathematical functionality while documenting intentional changes from Julia-specific storage, indexing, precision promotion, and restrictive algorithms. API naming and release numbering remain open. |

### Supporting type naming candidates

The following supporting names remain candidates; the main public names
`OperatorFamily` and `LieDecomposition` are adopted above.

| Reference type / role | Proposed Rust name | Meaning |
| --- | --- | --- |
| Validation result | `ValidationReport` | Outcomes, diagnostics, and effective tolerances; not a validated-basis marker. |
| Retained structure constants | `StructureConstants` | Lie-specific coefficient data associated with its basis and conventions. |
| Retained adjoint representation | `AdjointRepresentation` | Matrices of the Lie algebra adjoint action, distinguished from an operator's Hermitian adjoint. |

## Provisional implementation sequence

1. Translate the adopted domain and construction/validation policies into
   concrete APIs, tolerances, and acceptance cases. Resolve remaining
   semantic differences identified in
   [the reference inventory](reference-functionality.md).
2. Add the dedicated syntaqs-lie crate and configure compatible ndarray-linalg
   routines at its dense ndarray boundary, keeping production LAPACK selection
   with the final application and choosing an explicit test backend.
   Implement basis collections, Gram evaluation, reconstruction, coefficients,
   projection, residuals, and independence using existing expressions and
   numerical coefficient arrays.
3. Add the chosen Lie bracket, closure diagnostics, structure constants, and
   adjoint representations with explicit conventions and metadata.
4. Add the classical Killing form defined above and orthogonalization.
5. Add commutants and transformations, then implement the adopted Julia-style
   decomposition algorithm for the full center and nonabelian simple ideals.
6. Add public examples and lasting specifications, complete the repository
   verification commands relevant to implementation, and retire this workspace
   after merge.

This sequence is a planning aid; implementation has not started. Use of
ndarray-linalg when needed is authorized, while its concrete configuration
remains to be selected.

## Acceptance candidates

- Reproduce su(2) from syntaqs spin-half `[Sx,Sy,Sz]`: for the reference bracket,
  `-i[Sx,Sy]=2Sz`, with cyclic structure constants of 2.
- Reproduce su(2) plus su(2) using two identified factors: two three-dimensional
  commuting ideals, verified independently of basis order and signs.
- Verify center-plus-simple-ideal decomposition for `su(2) + u(1)`, purely
  abelian inputs, centerless inputs, and multiple simple factors. Include
  nonorthonormal input bases and real changes of basis; compare resulting
  subspaces and algebraic properties rather than basis vectors or component
  order. Check that the returned center is the full center.
- Compare Gram matrices, projection, and bracket coefficients with independent
  small dense matrices, including a nonorthonormal basis and a target outside
  the span. Test change-of-basis covariance where applicable.
- Compare residual norms with independent `sqrt(Tr(R†R)/D)` references. Cover
  targets inside and outside the span for orthonormal and nonorthonormal bases;
  do not substitute the unnormalized Frobenius norm.
- For the orthonormal Pauli subspace basis `[Sx,Sz]` and `A=Sx+Sy`, verify
  coefficients `[1,0]`, projection Sx, normalized residual 1, and failed span
  membership at a suitably small tolerance. Also verify successful membership
  and reconstruction for an in-span target. Repeat the projection/reconstruction
  checks after an invertible nonorthonormal change of the subspace basis.
- Verify antisymmetry, Jacobi identity, reconstruction residuals, ideal
  invariance, direct-sum span, and cross-ideal commutation.
- Verify the adopted decomposition algorithm on normalized and nonorthonormal
  input bases, purely abelian cases, and multiple simple factors. Check seed
  reproducibility within the same numerical environment, handling of
  insufficient eigenvalue separation, retry/exhaustion behavior, and that
  incomplete invariant blocks are not reported as a complete simple-ideal
  decomposition.
- Verify commutant results against all adjoint matrices in the selected Lie
  algebra basis. Distinguish these endomorphisms from physical-Hilbert-space
  operator centralizers and explicitly identified restricted subspaces.
- Verify the classical Killing form independently: the Pauli basis with
  `-i[Sx,Sy]=2Sz` gives `K=-8 I_3`; abelian Lie algebras give zero. For an
  invertible real basis change with columns U, verify `K_new=U^T K_old U`,
  including nonorthonormal bases. These checks distinguish it from the
  reference's positive, dimension-normalized adjoint Gram matrix.
- Cover dependent/nearly dependent lists, nonclosed spans, abelian/central
  components, invalid spaces, and the documented numerical/decomposition limits.
- Verify that dependent and numerically singular bases produce errors in
  operations requiring independence, without implicit pseudoinverse or
  redundant-element removal. Keep valid nonorthonormal bases supported, and
  distinguish expected nullspaces of constraints from invalid basis Gram
  matrices.
- Verify that construction accepts compatible candidate collections before
  mathematical validation, rejects empty collections and incompatible spaces,
  and preserves order. Check validation diagnostics and recorded tolerances.
  Ensure operations requiring Lie algebra properties reject inputs that fail
  their preconditions, including nonclosed inputs for structure constants.
- Preserve f32/f64 precision with declared tolerances. Keep tiny nonzero
  expression coefficients unless the caller explicitly requests approximation.
- Verify recommended defaults and explicit tolerance overrides at both
  precisions, including boundary cases whose classification changes with the
  selected tolerance. Check that internal validation respects overrides and
  reports the effective settings without changing ordinary exact-zero rules.
- For supported graded inputs, compare with ordinary action matrices and use
  the selected ordinary Lie bracket. Do not add a superbracket implicitly.
- Demonstrate that algebraic Gram and bracket operations avoid full physical
  matrices; numerical matrices on the chosen Lie algebra basis remain allowed.
- Verify the boundary end to end: populate dense ndarray coefficient data from
  syntaqs expressions, perform the selected numerical factorizations, and
  reconstruct output bases as syntaqs expressions with the original space.

These are candidates to refine when the mathematical contracts are chosen, not
newly executed tests or claims of current feature support.
