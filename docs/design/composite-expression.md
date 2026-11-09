# Composite expressions, placement, binding, and evaluation

Last updated: 2026-11-04

## 1. Scope and responsibilities

Based on the [requirements](../requirements.md) and [architecture overview](../architecture.md), this document defines local-expression placement, binding to a full Hilbert space, and mathematical capabilities and consistency of composite state and operator expressions. It extends the [mathematical capabilities of local primitives and expression trees](primitive-expression.md) along the [recursive Hilbert-space structure](hilbert-space.md).

Unimplemented partial trace is covered in [future work](../future-work/partial-trace.md). This document covers current placement, binding, action, inner products, and full trace.

All elementary factors being composed use one local algebra represented by the same primitive types S and O. Placement, binding, and action remain within that local algebra. Different indices or nested groupings do not imply heterogeneous local structures.

| Subject | Responsibility |
| --- | --- |
| Local structure | Associates state and operator primitives with local mathematical capabilities and ensures consistent local contracts |
| Composite Hilbert space | Composes capabilities from child structures, elementary-factor indices and order, and ordinary / graded kind; ensures composite consistency |
| State and operator expressions | Are constructed and evaluated according to their spaces' rules |
| Shared composition, embedding, and evaluation machinery | Provides composition laws preserving local contracts and realizes composite Hilbert-space capabilities |

A composite Hilbert space carries a consistency responsibility analogous to local structure. This conceptual division does not require a separate composite-structure type, another space tree, or individual operations as space methods. Section 9 describes implementation; section 10 lists current public APIs.

Assuming local providers satisfy their contracts, shared machinery supplies composition laws that preserve them. Elementary and composite children recursively follow the same mathematical contracts. Products, actions, and adjoints need not be independently redefined for each composite system.

## 2. Placement, binding, and expression construction

A local operator expression paired with an Index forms a composite-operator leaf. Likewise, a local state expression paired with an Index forms a state leaf; tensor products and finite linear combinations form composite states. Placed leaves retain their original local structure. The local expression in a leaf need not be a single primitive.

Assigning an Index is called placement; determining the full Hilbert space, including spectators, is binding. Placement requires no full ambient space and issues no new Index. The same Index applies to the entire local expression.

| Subject | Construction or operation | Result and contract |
| --- | --- | --- |
| State and operator expressions | Assign an Index to a local expression | A leaf belonging to or acting on the specified factor |
| State and operator expressions | Addition, scalar multiplication, zero | Finite linear combinations; state addition requires compatible spaces |
| State expressions | Tensor product | Composes states on independent factors in the specified order; rejects duplicate elementary factors and incompatible composition conditions |
| Operator expressions | Product | Composes operators on the same or different factors, retaining input order |
| Operator expressions | Bind to a full Hilbert space | Validates referenced factors, local structure, and composition conditions, and determines the full ambient space |
| State expressions | Determine and inspect the containing space | Derives the space from factor states and tensor structure |
| State and operator expressions | Retain space information | Once determined, containing spaces, domains, and codomains survive evaluation or basis expansion to zero or identity |

There is no separate operator tensor-product operation or node. Ordinary multiplication also composes operators on different factors. Operator tensor notation here describes mathematical composition laws. Constructing a product does not automatically construct its full ambient space.

Placing a state creates a state on the factor with that Index. Adding a new factor requires a state for that factor or an explicit embedding map; identity spectators do not fill missing state factors.

An explicit space takes precedence over an existing binding. Applying an unbound operator uses the state's space. Composite-operator trace and basis operations without either a binding or an explicit space return `UnboundOperator`. The selected space validates factor existence and compatibility with the state space, without modifying the original expression.

Operator sums and products compare spaces only when every operand is bound. Any unbound operand makes the result unbound, and old bindings are not retained in child nodes. Conditions known at construction are checked immediately; conditions depending on the full space are checked during binding or evaluation. Zero, identity, and zero coefficients do not bypass invalid-reference checks.

State exposes domain and codomain as `ℂ→H`; a bound Operator exposes `H→H`. Zero states, zero operators, and scalar identities retain a determined space. Empty operator sums are zero; empty products are identity; scalar c means `cI` on the selected space.

## 3. Mathematical capabilities and derived operations

| Subject | Capability | Input | Result and evaluation responsibility |
| --- | --- | --- | --- |
| State expressions | Inner product | Two states on compatible spaces | Complex scalar; uses sesquilinearity and tensor-product inner-product laws, delegating to local inner products |
| Operator expressions | Product | Two operator expressions | Operator expression; uses local products on the same factor and space composition laws across factors |
| State expressions | Act | State and operator expressions | State expression; follows linearity, factor order, and grading, delegating local action to Act on state primitives |
| Operator expressions | Adjoint | Operator expression | Operator expression; conjugates coefficients, reverses product order, and applies local adjoints |
| Operator expressions | Full trace | Operator with a determined full ambient space | Complex scalar; ordinary unnormalized trace |
| State and operator expressions | Grading in graded spaces | Homogeneous components | Handles signs from component parities without assigning a single parity to mixed expressions |

Products and actions are linear in each argument; inner products are conjugate-linear in the first and linear in the second; adjoints are conjugate-linear; full trace is linear. `AB` acts on a state starting with the rightmost `B`. Linear relations among local primitives are preserved; unique coefficient sequences or expression structures are not required.

| Subject | Derived operation | Underlying capabilities |
| --- | --- | --- |
| State expressions | Norm and normalization | `sqrt(⟨ψ,ψ⟩)` from the inner product and scalar multiplication; normalization rejects zero-norm states |
| Operator expressions | Commutator / anticommutator | Products and addition: `AB−BA` / `AB+BA` |
| Operator expressions | Hilbert–Schmidt inner product | `Tr(A†B)/D`, where D is the full ambient-space dimension |
| State and operator expressions | Expectation and normalized expectation | Action and inner product: `⟨ψ,Aψ⟩`, optionally divided by `⟨ψ,ψ⟩`; normalized expectation rejects zero-norm states |

`syntaqs-core` provides shared composite expressions, placement, binding, and evaluation by composing concrete algebras' local rules. Automatic source retention and structural expression comparison are not provided. Display and Debug follow [expression formatting](expression-formatting.md). Local expressions, indices, local structures, and ambient spaces remain subject to consistency checks; mathematical equality does not require identical expression structure.

Core provides expansion and coordinates in a selected basis separately from evaluation by local rules. It uses [local basis capabilities](basis.md) and composite-basis ordering, collecting coefficients of equal basis elements. Ordinary evaluation of products, actions, inner products, and trace requires no basis expansion. Composite-operator basis expansion also requires a determined full ambient space.

Composite bases are tensor products of local bases for elementary factors, following space order and composition laws. Grouping does not change the basis. Expanded results are ordinary expression trees without basis identifiers or expanded-state metadata, retaining indices, local structure, and determined spaces. Changing basis uses ordinary re-expansion; export contracts retain basis information needed to interpret numerical coefficients.

Construction does not expand or evaluate an entire expression. Requested results evaluate only necessary parts, preserve inputs, and share immutable substructures. Action, inner products, and trace require no complete numerical matrix, common fully expanded form, or cached evaluation result.

Distribution and local multiplication are internal evaluation or expansion steps; there is no independent operation for selecting individual simplification rules.

`Operator::collect_terms_in(&ambient)` collects equal local primitives and merges
tensor terms differing in at most one factor after resolving products/adjoints.
Local sums remain factored, avoiding Cartesian expansion. It uses the existing
ordinary/graded traversal, including Koszul signs, and retains the selected
space even for zero. Primitive/coefficient PartialEq is required; primitive
Clone, Dimension, and a state basis are not required. It removes exact zero
coefficients, does not select a
complete operator basis, and leaves ordinary construction/evaluation unchanged.

## 4. Relations preserved by placement and embedding

Write `E_i^H(A)` for embedding local operator expression `A` at Index `i` in the full space `H`. This requires an existing Index, matching local structure, and compatible composition conditions. For expressions `A,B` of the same local structure and complex scalars `a,b`:

```math
\begin{aligned}
E_i^H(aA+bB)&=aE_i^H(A)+bE_i^H(B),\\
E_i^H(AB)&=E_i^H(A)E_i^H(B),\\
E_i^H(A^\dagger)&=E_i^H(A)^\dagger,\\
E_i^H(I_i)&=I_H.
\end{aligned}
```

Computing locally before embedding and computing after embedding represent the same operator. Products on one Index use the local algebra. Operators on distinct indices `i,j` commute in ordinary spaces; in graded spaces homogeneous `A,B` satisfy:

```math
E_i^H(A)E_j^H(B)
=(-1)^{p(A)p(B)}E_j^H(B)E_i^H(A),\qquad i\ne j.
```

Ordinary embedding uses identity on other factors. Graded embedding includes order-dependent parity according to the action law below. A reference Index alone does not imply identity on other factors in the ordinary tensor representation.

## 5. Ordinary and graded composition laws

The following two-factor rules extend to finite linear combinations and multiple factors. `A,C` act on the first factor and `B,D` on the second; `ψ,ψ'` are first-factor states and `φ,φ'` are second-factor states.

Ordinary composition follows:

```math
\begin{aligned}
(A\otimes B)(C\otimes D)&=AC\otimes BD,\\
(A\otimes B)(\psi\otimes\phi)&=A\psi\otimes B\phi,\\
(A\otimes B)^\dagger&=A^\dagger\otimes B^\dagger,\\
I_{H\otimes K}&=I_H\otimes I_K.
\end{aligned}
```

For graded composition, components used in signs are homogeneous, with `p` equal to even `0` or odd `1`:

```math
\begin{aligned}
(A\otimes_g B)(C\otimes_g D)
&=(-1)^{p(B)p(C)}AC\otimes_g BD,\\
(A\otimes_g B)(\psi\otimes_g\phi)
&=(-1)^{p(B)p(\psi)}A\psi\otimes_g B\phi,\\
(A\otimes_g B)^\dagger
&=(-1)^{p(A)p(B)}A^\dagger\otimes_g B^\dagger,\\
I_{H\otimes_g K}&=I_H\otimes_g I_K.
\end{aligned}
```

Even in graded spaces, the state inner product is the usual positive-definite Hilbert-space inner product. Here `⊗_*` denotes either ordinary or graded composition, with the same rule for both:

```math
\langle\psi\otimes_*\phi,\psi'\otimes_*\phi'\rangle
=\langle\psi,\psi'\rangle\langle\phi,\phi'\rangle.
```

The inner product is conjugate-linear in the first argument; grading alone adds no sign. Even and odd subspaces are orthogonal. A homogeneous state or operator tensor has parity equal to the sum of component parities modulo 2. Identity is even and adjoints preserve parity. Mixed-parity expressions are handled component by component. Choosing a parity for zero must not affect results.

Ordinary operator multiplication obeys `(XY)† = Y†X†` even in graded spaces. The graded tensor-adjoint sign above agrees with the exchange law used to restore factor order after reversing a product.

These graded products, actions, and adjoints follow [Crismale, Rossi, and Zurlo, On C*-norms on Z₂-graded tensor products, sections 2–3](https://link.springer.com/article/10.1007/s43037-021-00168-0). syntaqs adopts these composition laws consistently with action and adjoints on ordinary Hilbert spaces.

Conversion to an ordinary tensor representation inserts parity operators using the traversal described below. Intrinsic graded rules and parity strings represent the same action; converted signs are not applied again.

For multiple homogeneous factors, the same rules become:

```math
\begin{aligned}
(\mathop{\otimes_g}_i A_i)(\mathop{\otimes_g}_i B_i)
&=(-1)^{\sum_{i>j}p(A_i)p(B_j)}\mathop{\otimes_g}_i(A_iB_i),\\
(\mathop{\otimes_g}_i A_i)(\mathop{\otimes_g}_i\psi_i)
&=(-1)^{\sum_{i>j}p(A_i)p(\psi_j)}\mathop{\otimes_g}_i(A_i\psi_i),\\
(\mathop{\otimes_g}_i A_i)^\dagger
&=(-1)^{\sum_{i<j}p(A_i)p(A_j)}\mathop{\otimes_g}_i A_i^\dagger.
\end{aligned}
```

## 6. Consistency for nested, empty, and single-factor spaces

Composite children recursively use the same composition laws through their own mathematical capabilities and parity. For the same factor order and tensor kind, reassociation does not change results:

```math
(\psi_1\otimes_*\psi_2)\otimes_*\psi_3
\longleftrightarrow
\psi_1\otimes_*(\psi_2\otimes_*\psi_3).
```

This correspondence exchanges no factors and adds no graded sign. Products, actions, inner products, adjoints, and trace agree under it. This is a mathematical consistency requirement; it adds no user-facing regrouping, permutation, or construction-time flattening operation.

- An empty Tensor is a one-dimensional unit space. Its state `1` and identity operator are even; identity has trace `1`.
- A single-factor Tensor has the capabilities of that factor.
- A one-dimensional Factor with an Index remains distinct from an empty Tensor and retains its Index.
- Structures with ordinary / graded mixing remaining after empty and single-factor identifications are rejected under the current restrictions.

## 7. Full trace and embedding

Full trace is ordinary unnormalized trace in both ordinary and graded spaces. For any complete orthonormal basis `{e_n}` of the full ambient space `H`, it is defined below and is independent of the basis:

```math
\operatorname{Tr}_H(X)=\sum_n\langle e_n,Xe_n\rangle.
```

It satisfies `Tr(XY) = Tr(YX)` and `Tr(I_H) = dim H`. Cyclicity gains no parity sign; supertrace is not used.

In the ordinary tensor representation, `Tr(A⊗B)=Tr(A)Tr(B)` contracts local products and traces without enumerating the full basis or generating dense or sparse matrices. Graded placements first lower to the ordinary representation, including parity strings in local actions. A syntactically unreferenced factor cannot immediately be treated as an identity spectator.

## 8. Consistency guarantees and verification targets

For children satisfying local contracts, composite Hilbert-space composition laws preserve:

- Associativity of Product, `(XY)ψ = X(Yψ)`, and identity laws.
- Conjugate symmetry and positive definiteness of inner products, and `⟨φ,Xψ⟩ = ⟨X†φ,ψ⟩`.
- `(XY)† = Y†X†` and `(X†)† = X`.
- Linearity, sesquilinearity, conjugate linearity, and linear relations among primitives.
- Ordinary trace and consistency with basis-coordinate calculations.
- Mathematical results under reassociation of the same tensor kind, and agreement of actions between graded and ordinary tensor representations.

No runtime proof of associativity or other laws is required for arbitrary user-defined rules. Operations validate Index existence, local-structure matching, compatible state and ambient spaces, duplicate factors, scalar precision, grading, and tensor kind. Information available at construction is checked early; space-dependent conditions are checked during binding or evaluation. There is no global registry associating indices with local structures.

Verification compares multiplication before action with sequential action, hierarchical composition with elementary-factor calculations, and algebraic calculations with independently constructed small matrices. Adjoints, full trace, and basis coordinates are also checked. Graded cases include odd, even, and mixed-parity sums, separated factors, grouping, and parity strings.

Equalities concern mathematical meaning rather than bitwise floating-point equality. Precision, special values, and tolerances follow [numeric conventions](numerics.md).

## 9. Storage and evaluation implementation

| Subject | Implemented policy |
| --- | --- |
| Composite `State` | Always has a determined containing space. Zero and empty sums require a space; an empty tensor product is state `1` on a zero-factor space |
| Composite `Operator` | One type with an expression body and optional binding, kept on the outer handle. Composition compares spaces only if all operands are bound; child nodes retain no old bindings |
| Space selection | Explicit space first, then existing binding. Unbound action uses the state space; unbound trace without a specified space is an error |
| Sharing | Immutable `Arc` handles share expression bodies and spaces; node enums are private. States and operators expose their containing spaces or determined domains and codomains |
| Errors | Named methods are primary. Construction, connection, and evaluation requiring validation return `Result`. Validating operator syntax also uses `Output = Result<Expression, Error>`; incompatible spaces do not panic |
| Delayed evaluation | Validates spaces once information is available, delaying local rules. Zero, identity, and zero coefficients do not bypass invalid-reference checks |
| Evaluation paths | Separate action, inner-product, and full-trace paths impose no common fully expanded form. Action needs no primitive Product; full trace delegates required local products and traces |
| Deep expressions | Evaluation, traversal, and destruction of nodes held by their last owner use explicit work stacks |

Evaluation by local rules and expansion in a selected basis are separate operations; there is no shared fully expanded representation across all paths. Public types are `State<I,S,O,C>` and `Operator<I,O,C>`, with dedicated state and operator nodes. Neither a separate composite-structure type nor composite-specific local primitives are required.

## 10. Current public API

| Subject | Main APIs and contracts |
| --- | --- |
| Local-expression placement | `LocalState::at(&factor)` and `LocalOperator::at(&factor)` validate placement and share local expressions. States have a containing space; operators are unbound |
| `State<I, S, O, C>` construction and spaces | `place`, `zero(&space)`, `unit()`, `sum`, `try_sum`, `tensor_in(&space, states)`, `try_tensor_in(&space, states)`, `scaled`, and `space`, `domain`, `codomain` |
| `State<I, S, O, C>` operations and evaluation | `act`, `act_in`, `evaluate`, `inner_product`, `norm_squared`, `expectation`, and `expectation_in`; action construction is delayed |
| `Operator<I, O, C>` construction and spaces | `place`, `zero()`, `identity()`, `scalar`, `sum`, `try_sum`, `product`, `try_product`, `scaled`, `adjoint`, `bind`, and `space`, `domain`, `codomain`; space queries return `None` when unbound |
| `Operator<I, O, C>` operations and evaluation | `commutator`, `anticommutator`, `trace`, `trace_in`, `hs_inner_product`, and `hs_inner_product_in`; commutator construction is delayed |
| Errors | `SpaceError` for duplicate factors, `ExpressionError` for composite connection and evaluation, and `DimensionOverflow` for dimension queries |

Named sum and product constructors are sequence-based `sum` / `product`; binary operations use standard `+` / `*`. Index matching and numeric capabilities are required only by operations that need them. Validating composite operations return `Result`; state-action evaluation requires no operator-primitive `Product`. Basis expansion and coordinates follow [basis design](basis.md); norms, normalization, and normalized expectations follow [normalization API contracts](normalization.md#1-norm-and-normalization-apis-and-features). Hilbert-space APIs are in [space design](hilbert-space.md#9-public-apis).

## 11. Shared ordinary and graded evaluation

Space `TensorKind` and effective kind follow the [space API](hilbert-space.md#9-public-apis); primitive parity and P follow [local capabilities](local-structure.md#7-local-grading-capabilities). Type parameters of `HilbertSpace<I>`, `State<I,S,O,C>`, and `Operator<I,O,C>` remain unchanged. Geometry and Index gain no grading ownership or sign processing.

### 11.1. Construction entry points

| Subject | Shared ordinary / graded entry point |
| --- | --- |
| State | `tensor_in(&space, states)` |
| Fallible State | `try_tensor_in(&space, states)` |
| ExpressionFactory | `tensor_in(&space, states)` |
| Fallible ExpressionFactory | `try_tensor_in(&space, states)` |
| Primitive placement and composition | `product_state(&space, primitives)` |

Construct the containing space first and share it when constructing a state. State Tensor nodes retain only child sequences, without duplicating kind information. Kind is specified at space construction; both kinds share state-construction entry points.

Concatenated input-state leaves must exactly match the specified space's leaves and order. Effective kinds of multifactor subsystems must also be compatible; grouping may differ. Missing, extra, duplicate, reordered, or kind-incompatible factors and mismatched primitive counts return `ExpressionError::IncompatibleSpaces`. try_ entry points preserve [construction](expression-construction.md) rules for input-error precedence and stopping consumption.

Empty input is accepted only for a zero-factor unit space, retaining the specified space handle and grouping. Single input also retains the specified space after validation. `State::unit()` remains a convenience for the canonical unit space; `State::zero(&space)` creates zero on the specified space. Construction adds no Clone bounds on indices or primitives, or coefficient arithmetic bounds.

### 11.2. Shared APIs and minimal capabilities

`evaluate`, `trace`, `expand`, `coordinates`, and their explicit-space variants are shared ordinary / graded entry points, without kind-specific evaluation APIs or adapters. Rust bounds cannot switch on runtime kind, so even ordinary spaces require the grading capabilities used by an operation. Actual signs depend on the selected space's `effective_kind()`; ordinary spaces insert neither P nor crossing signs.

| Operation | Required grading capabilities in shared evaluation |
| --- | --- |
| Composite-state evaluate / inner product / norm squared / expectation / expansion / coordinates, including explicit-space variants | O: `HomogeneousParity + ParityOperator` |
| Composite-operator trace / Hilbert–Schmidt inner product, including explicit-space variants | O: `HomogeneousParity + ParityOperator` |
| Composite-operator expansion / coordinates, including explicit-space variants | O: `HomogeneousParity`; C: `Neg<Output=C>`. Intrinsic ordering needs no P |
| Export / norm / normalization / normalized expectation | Inherits capabilities from the invoked core evaluation path |

Construction, placement, binding, sums, products, delayed actions, and adjoints do not uniformly require evaluation capabilities; purely local API bounds are preserved. State action requires no Product on O. P acts through local Act and existing Scale nodes, so crossing signs alone add no One / Neg bounds on C and need no direct parity query on S.

### 11.3. Scope of delayed Act

Composite Act retains the HilbertSpace selected during construction. Operator syntax selects the full space in order of explicit space, existing binding, then target state. Explicit space overrides an old binding. Composing bound operators also treats different effective kinds as incompatible spaces.

For odd B and odd state `s_a`, distinguish:

```text
tensor_in(&H_ab, [s_a, s_b.act(B_b)])      // B action was constructed within H_b
tensor_in(&H_ab, [s_a, s_b]).act(B_b)      // B acts on H_a ⊗g H_b
```

Only the latter has the negative sign from `s_a`'s parity. Evaluation tasks switch to each Act's scope. Inserting an action previously constructed on a single factor into an outer tensor does not enlarge its scope. Evaluating before tensor composition and composing while delayed agree; expansion and reconstruction preserve meaning.

Unevaluated operator syntax is interpreted in the selected full space. Rebinding an evaluated or basis-expanded result need not reproduce the original unevaluated expression's action in another space. There is no automatic re-evaluation or source retention.

### 11.4. Intrinsic terms and ordinary representations

| Internal representation | Content and use |
| --- | --- |
| `IntrinsicOpTerms` | Homogeneous operator terms ordered by the selected space's leaves; used for graded operator-basis expansion and coordinates |
| `OrdinaryOpTerms` | Local-factor sequences in the ordinary representation, inserting a parity string for each placement; used for full trace |

Intrinsic ordering adds one negative sign whenever odd components on distinct factors cross, preserving the original local-product order on the same factor. It does not sort by Index values. The same ordering follows product-adjoint reversal; tensor-adjoint signs are not added separately.

The ordinary representation of a homogeneous intrinsic term is:

```math
\rho(A_0\otimes_g\cdots\otimes_g A_{n-1})
=\bigotimes_{i=0}^{n-1}\left(A_i P_i^{q_i}\right),
\qquad q_i=\sum_{j>i}p(A_j)\pmod 2.
```

P is placed to the right of A: for example, `A ⊗g B` becomes `A P^p(B) ⊗ B`. Ordinary-factor sequences are not reinterpreted as intrinsic graded terms; P and signs are not applied twice. These are temporary private representations, with no public conversion API, common full expansion, or persistent cache.

State action inserts P on factors preceding an odd primitive within the selected Act scope. Mixed-parity and entangled states use P's actual local action rather than a constant sign. Local sums, products, and adjoints are handled primitive by primitive. A full state sum-of-products expansion, complete basis, or dense matrix is not shared preprocessing; unaffected branches are shared.

State inner products add no signs. Full trace contracts local products and Trace in the ordinary representation. A factor that may receive P cannot be identified as an identity spectator before lowering. Tests also cover the trace of `c_1†c_1 = I_0 ⊗ n_1` for two-mode `c_1 = P_0 ⊗ a_1`. Ordinary trace cyclicity gains no parity sign and requires neither S, a state basis, nor a usize-representable total dimension. Hilbert–Schmidt inner products use the same lowering and local-trace traversal, dividing each local trace by `Tr(I)` before contraction. Only HS inner products add Divide; total dimension D is not converted to an integer. Supertrace and partial trace are not added.

Space validation precedes evaluation even for zero, zero coefficients, and identity, so invalid indices and spaces are not missed. See [tensor_products.rs](../../examples/src/bin/tensor_products.rs) for usage. Concrete implementations using this shared machinery are documented under [Fermion](local-structure.md#8-fermion-local-algebra) and [Spin-one](local-structure.md#9-spin-one-local-algebra). Extensions outside the current scope appear in [future work](../future-work.md).

### 11.5. Ordinary representation of fermions

With occupied state `|1>`, empty state `|0>`, and local annihilation operator `a=|0><1|`, parity is `P=(-1)^n=diag(1,-1)`. An annihilation operator placed in factor order has ordinary representation:

```math
c_j=P_0\otimes\cdots\otimes P_{j-1}
\otimes a_j\otimes I_{j+1}\otimes\cdots.
```

For i<j, `c_i c_j` has I at k<i, `a_iP_i` at k=i, P at i<k<j, a at k=j, and I at j<k. The local convention gives `aP=-a`. Intermediate parity remains as `P_(i,j)=(-1)^(Σ_(i<k<j) n_k)`.

Occupation-basis states allow signs to be computed from occupation numbers; general superpositions require actual parity-operator action. A state can be replaced with a constant sign only if it is an eigenstate of the corresponding parity. In the spinful Site representation, `c_(i,σ)=(∏_(j<i) P_j) f_(i,σ)` distinguishes internal CAR from preceding Sites' total parity, avoiding duplicate signs.

## 12. Verification targets and tests

[core_tensor_products.rs](../../tests/tests/core_tensor_products.rs) compares space kinds, identifications, mixed-kind rejection, construction entry points, fallible aggregation, signs, CAR, action, adjoints, inner products, trace, and HS inner products against independent ordinary matrices. It includes Act scope, embedding homomorphisms, separated factors, odd-component exchange, and mixed parity.

[core_basis_expansion.rs](../../tests/tests/core_basis_expansion.rs) covers bases and re-expansion; [export_tensor_products.rs](../../tests/tests/export_tensor_products.rs) covers dense and sparse output and metadata; [normalize_tensor_products.rs](../../tests/tests/normalize_tensor_products.rs) covers normalization. Verification also covers minimal capabilities, borrows, sharing, deep trees, and contractions requiring no dense storage. See [verification](verification.md) for commands and limits.
