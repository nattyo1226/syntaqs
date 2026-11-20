# Composite expressions, placement, binding, and evaluation

Last updated: 2026-11-17

## 1. Scope and responsibilities

Composite `State<I,S,O,C>` and `Operator<I,O,C>` expressions place local algebra on
identified factors and interpret it in an ordered Hilbert space. A state retains
its containing space; an operator may remain unbound until an ambient space is
supplied. The shared machinery extends [local expression rules](primitive-expression.md)
along the [recursive space structure](hilbert-space.md).

The current API supports placement, binding, action, inner products, and full
trace. [Partial trace](../future-work/partial-trace.md) remains a future proposal.

All elementary factors being composed use one local algebra represented by the same primitive types S and O. Placement, binding, and action remain within that local algebra. Different indices or nested groupings do not imply heterogeneous local structures.

| Subject | Responsibility |
| --- | --- |
| Local structure | Associates state and operator primitives with local mathematical capabilities and ensures consistent local contracts |
| Composite Hilbert space | Composes capabilities from child structures, elementary-factor indices and order, and ordinary / graded kind; ensures composite consistency |
| State and operator expressions | Are constructed and evaluated according to their spaces' rules |
| Shared composition, embedding, and evaluation machinery | Provides composition laws preserving local contracts and realizes composite Hilbert-space capabilities |

These responsibilities use the existing space and expression types, without a
separate composite-structure type or second space tree. Operations need not be
space methods. The [public API](#3-current-public-api) and
[shared evaluation contracts](#4-shared-ordinary-and-graded-evaluation) precede
the [implementation algorithms](#11-storage-and-evaluation-implementation).

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

An explicit space takes precedence over an existing binding. Applying an unbound
operator uses the state's space. Delayed action trusts a binding to the selected
space's shared root. Otherwise it checks each referenced factor before checking
compatibility with the state. Composite operator trace and basis operations
without either a binding or an explicit space return `UnboundOperator`.
Binding, delayed action, and composite operator queries that resolve factor
positions require `I: Ord`.

Operator sums and products compare spaces only when every operand is bound. Any unbound operand makes the result unbound, and old bindings are not retained in child nodes. Conditions known at construction are checked immediately; conditions depending on the full space are checked during binding or evaluation. Zero, identity, and zero coefficients do not bypass invalid-reference checks.

State exposes domain and codomain as `ℂ→H`; a bound Operator exposes `H→H`. Zero states, zero operators, and scalar identities retain a determined space. Empty operator sums are zero; empty products are identity; scalar c means `cI` on the selected space.

## 3. Current public API

| Subject | Main APIs and contracts |
| --- | --- |
| Local-expression placement | `LocalState::at(&factor)` and `LocalOperator::at(&factor)` validate placement and share local expressions. States have a containing space; operators are unbound |
| `State<I, S, O, C>` construction and spaces | `place`, `zero(&space)`, `unit()`, `sum`, `try_sum`, `tensor_in(&space, states)`, `try_tensor_in(&space, states)`, `scaled`, and `space`, `domain`, `codomain` |
| `State<I, S, O, C>` operations and evaluation | `act`, `act_in`, `evaluate`, `inner_product`, `norm_squared`, `expectation`, and `expectation_in`; action construction is delayed |
| `Operator<I, O, C>` construction and spaces | `place`, `zero()`, `identity()`, `scalar`, `sum`, `try_sum`, `product`, `try_product`, `scaled`, `adjoint`, `bind`, and `space`, `domain`, `codomain`; space queries return `None` when unbound |
| `Operator<I, O, C>` operations and evaluation | `group_local`, `commutator`, `anticommutator`, `trace`, `trace_in`, `hs_inner_product`, and `hs_inner_product_in`; grouping is structural and commutator construction is delayed |
| Errors | `SpaceError` for duplicate factors, `ExpressionError` for composite connection and evaluation, and `DimensionOverflow` for dimension queries |

Named sum and product constructors are sequence-based `sum` / `product`; binary operations use standard `+` / `*`. Index matching and numeric capabilities are required only by operations that need them. Validating composite operations return `Result`; state-action evaluation requires no operator-primitive `Product`. Basis expansion and coordinates follow [basis design](basis.md); norms, normalization, and normalized expectations follow [normalization API contracts](normalization.md#1-norm-and-normalization-apis-and-features). Hilbert-space APIs are in [space design](hilbert-space.md#9-public-apis).

## 4. Shared ordinary and graded evaluation

Ordinary and graded inputs use the same `HilbertSpace<I>`, `State<I,S,O,C>`, and
`Operator<I,O,C>` types. Space `TensorKind` and effective kind follow the
[space contract](hilbert-space.md#9-public-apis);
core supplies signs from [local parity capabilities](local-structure.md#7-local-grading-capabilities).
Here `p(A)` is the parity of a homogeneous component (even 0 or odd 1), and P is
the local parity operator. Geometry and factor identifiers are independent of
this sign processing.

### 4.1. Construction entry points

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

### 4.2. Shared APIs and minimal capabilities

`evaluate`, `trace`, `expand`, `coordinates`, and their explicit-space variants are shared ordinary / graded entry points, without kind-specific evaluation APIs or adapters. Rust bounds cannot switch on runtime kind, so even ordinary spaces require the grading capabilities used by an operation. Actual signs depend on the selected space's `effective_kind()`; ordinary spaces insert neither P nor crossing signs.

| Operation | Required grading capabilities in shared evaluation |
| --- | --- |
| Composite-state evaluate / inner product / norm squared / expectation / expansion / coordinates, including explicit-space variants | O: `HomogeneousParity + ParityOperator` |
| Composite-operator trace / Hilbert–Schmidt inner product, including explicit-space variants | O: `HomogeneousParity + ParityOperator` |
| Composite-operator expansion / coordinates, including explicit-space variants | O: `HomogeneousParity`; C: `Neg<Output=C>`. Intrinsic ordering needs no P |
| Export / norm / normalization / normalized expectation | Inherits capabilities from the invoked core evaluation path |

Construction, placement, binding, sums, products, delayed actions, and adjoints do not uniformly require evaluation capabilities; purely local API bounds are preserved. State action requires no Product on O. P acts through local Act and existing Scale nodes, so crossing signs alone add no One / Neg bounds on C and need no direct parity query on S.

State evaluation additionally requires the following local algebra and
coefficient capabilities:

| State evaluation | Required capabilities |
| --- | --- |
| Local | `S: PartialEq + Act<C, Operator=O, Term=S>`; `O: Adjoint<C, Term=O>`; `C: Clone + Zero + One + Conjugate<Output=C>` |
| Composite | Local bounds plus `I: Eq`, `C: PartialEq`, and `O: HomogeneousParity + ParityOperator` |

`Zero` and `One` supply addition and multiplication. No primitive/index Clone,
primitive Eq/Ord/Hash, coefficient ordering, or primitive operator Product is
required. Construction has its own operation-specific bounds. State inner products, norms,
expectations, basis queries, normalization, vector export and action-matrix
export inherit the corresponding evaluation bounds. Operator-only trace/HS and
operator-basis queries retain their existing bounds.

### 4.3. Scope of delayed Act

Composite Act retains the HilbertSpace selected during construction. Operator syntax selects the full space in order of explicit space, existing binding, then target state. Explicit space overrides an old binding. Composing bound operators also treats different effective kinds as incompatible spaces.

For odd B and odd state `s_a`, distinguish:

```text
tensor_in(&H_ab, [s_a, s_b.act(B_b)])      // B action was constructed within H_b
tensor_in(&H_ab, [s_a, s_b]).act(B_b)      // B acts on H_a ⊗g H_b
```

Only the latter has the negative sign from `s_a`'s parity. Evaluation tasks switch to each Act's scope. Inserting an action previously constructed on a single factor into an outer tensor does not enlarge its scope. Evaluating before tensor composition and composing while delayed agree; expansion and reconstruction preserve meaning.

Unevaluated operator syntax is interpreted in the selected full space. Rebinding an evaluated or basis-expanded result need not reproduce the original unevaluated expression's action in another space. There is no automatic re-evaluation or source retention.

## 5. Mathematical capabilities and derived operations

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
complete operator basis. This collection operation is explicit and separate from
ordinary construction and evaluation.

## 6. Relations preserved by placement and embedding

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

## 7. Ordinary and graded composition laws

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

## 8. Consistency for nested, empty, and single-factor spaces

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

## 9. Full trace and embedding

Full trace is ordinary unnormalized trace in both ordinary and graded spaces. For any complete orthonormal basis `{e_n}` of the full ambient space `H`, it is defined below and is independent of the basis:

```math
\operatorname{Tr}_H(X)=\sum_n\langle e_n,Xe_n\rangle.
```

It satisfies `Tr(XY) = Tr(YX)` and `Tr(I_H) = dim H`. Cyclicity gains no parity sign; supertrace is not used.

In the ordinary tensor representation, `Tr(A⊗B)=Tr(A)Tr(B)` contracts local products and traces without enumerating the full basis or generating dense or sparse matrices. Graded placements first lower to the ordinary representation, including parity strings in local actions. A syntactically unreferenced factor cannot immediately be treated as an identity spectator.

## 10. Consistency guarantees and verification targets

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

## 11. Storage and evaluation implementation

The [evaluation performance decision](../decisions/core-evaluation-performance.md)
records the rationale, alternatives, and dated workload comparisons for these
processing boundaries. The contracts below describe the current implementation.

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

### 11.1. Intrinsic terms and ordinary representations

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

State inner products add no signs. Full trace contracts local products and Trace in the ordinary representation. A factor that may receive P cannot be identified as an identity spectator before lowering. Tests also cover the trace of `c_1†c_1 = I_0 ⊗ n_1` for two-mode `c_1 = P_0 ⊗ a_1`. Ordinary trace cyclicity gains no parity sign and requires neither S, a state basis, nor a usize-representable total dimension. Hilbert–Schmidt inner products use the same lowering and local-trace traversal, dividing each local trace by `Tr(I)` before contraction. Only HS inner products add Divide; total dimension D is not converted to an integer. Supertrace and partial trace are outside the current API.

Space validation precedes evaluation even for zero, zero coefficients, and identity, so invalid indices and spaces are not missed. See [tensor_products.rs](../../examples/src/bin/tensor_products.rs) for usage. Concrete implementations using this shared machinery are documented under [Fermion](local-structure.md#8-fermion-local-algebra) and [Spin-one](local-structure.md#9-spin-one-local-algebra). Extensions outside the current scope appear in [future work](../future-work.md).

### 11.2. Ordinary representation of fermions

With occupied state `|1>`, empty state `|0>`, and local annihilation operator `a=|0><1|`, parity is `P=(-1)^n=diag(1,-1)`. An annihilation operator placed in factor order has ordinary representation:

```math
c_j=P_0\otimes\cdots\otimes P_{j-1}
\otimes a_j\otimes I_{j+1}\otimes\cdots.
```

For i<j, `c_i c_j` has I at k<i, `a_iP_i` at k=i, P at i<k<j, a at k=j, and I at j<k. The local convention gives `aP=-a`. Intermediate parity remains as `P_(i,j)=(-1)^(Σ_(i<k<j) n_k)`.

Occupation-basis states allow signs to be computed from occupation numbers; general superpositions require actual parity-operator action. A state can be replaced with a constant sign only if it is an eigenstate of the corresponding parity. In the spinful Site representation, `c_(i,σ)=(∏_(j<i) P_j) f_(i,σ)` distinguishes internal CAR from preceding Sites' total parity, avoiding duplicate signs.

### 11.3. Structural local-operator grouping

After checking bindings, operator construction groups immediate local sum
operands on one factor. It also groups the leading operands of a product
when they are local to the same factor. Composite products store the remaining
operands in the same ordered sequence type as sums and retain shared handles
to nested products.
The rightmost operand acts first, and adjoints reverse the order. Construction
needs factor index equality and performs no primitive or coefficient arithmetic.
Whole-expression grouping handles nested composite scales and adjoints.

`Operator::group_local(&self) -> Self` requires `I: Eq`, `O: Identity`, and
`C: Clone`. It groups any subtree whose references all belong to one factor,
translating scales, ordered sums/products, and adjoints into local syntax.
It also groups the leading single-factor prefix of an ordered product before
the first distinct factor, using the product's left-associative grouping.
Scalar children use local identity/zero syntax in that factor's context.
Entirely scalar roots remain scalar. It preserves binding, supports unbound
operators, leaves the input unchanged, and is structurally stable on reapplication.
It does not evaluate primitive products/adjoints, collect equal terms, simplify
zeros, choose a basis, or exchange operators on different factors. In particular,
zero products keep references needed for later ambient validation.

Grouping uses a bounded candidate scan to avoid a full memo for small trees
that cannot need a rewrite, then iterative postorder traversal with a query-local
node memo. The scan falls back to memoized traversal after its fixed work budget,
so shared DAGs cannot make it expand indefinitely. Leaf classifications are
read directly; the memo retains replacements only for changed containers.
Scalar conversion to local syntax is lazy and shared across uses. Already-local
roots are reused directly; unchanged multi-factor paths share their original
nodes. No persistent grouping flag or public support cache is stored.

Operator expand/coordinates, `collect_terms_in`, full trace, and HS queries
validate the original expression against the selected space before applying
this same preparation. Graded traversal still decomposes homogeneous components
and owns all crossing signs; ordinary trace lowering still inserts parity strings.
Grouping never gathers separated same-factor occurrences across another factor.
State action is basis-free and requires no operator-primitive Product.

Structural grouping differs from exact primitive collection in
[local expressions](primitive-expression.md): it changes expression placement,
not primitive algebra. Display reflects the retained structure, so constructing
a same-factor sum may display `(A + B)[i]` rather than `A[i] + B[i]`.

### 11.4. Natural-state collection between actions

`State::evaluate()` evaluates an operator product right to left and collects
matching state terms before the next action. Operator sums contribute from the
same normalized input to one incremental accumulator. The same boundaries apply
inside local actions. Action-free inputs are normalized as well.

Each normalized contribution has an outer coefficient and an action-free shape:
unit, placed local polynomial, ordered tensor, or retained sum. Local polynomials
flatten additive nesting and scales, combine `PartialEq`-matching primitives in
first-occurrence order, and remove exact-zero coefficients. A singleton's
coefficient moves to the enclosing contribution; a scale of a multi-term local
polynomial is absorbed into its coefficients. Tensor normalization flattens
parentheses, removes units, and pulls child outer coefficients to the tensor's
outer coefficient. A zero factor makes the tensor zero while retaining the
state's selected space. Factor order and delayed Act scope never change.

Composite matching uses the resulting structure, primitive equality, and
coefficient equality inside retained polynomials/sums. Ordered retained sums
are not permuted for matching. Products of sums stay factored: no distributive
search or common-factor extraction equates factored and distributed states.
Distinct but linearly dependent primitives remain distinct until basis expansion.
Only exact `Zero::is_zero()` removes numerical terms; tiny nonzero values remain.

Evaluation uses iterative tasks, a per-call source-node memo, and internal
normalized-result wrappers. Equality-only interning assigns scratch primitive,
coefficient, and structural IDs for a query-local index, with output order stored
separately. These IDs are not basis coordinates and add no user `Ord`/`Hash`
requirement. Pointer memoization reuses normalization work, but never certifies
term equality: nonreflexive primitives or retained coefficients cannot be merged
merely because a source handle or ID is shared. No persistent collected marker
is stored; action-presence flags do not certify normalization. Re-evaluation is
structurally stable, with no O(1) guarantee for action-free inputs.

The full state-evaluation bounds are listed with the
[public evaluation contracts](#42-shared-apis-and-minimal-capabilities).

State coordinates follow `evaluate -> basis expansion -> basis-index collection`.
Validation of spaces/references precedes removal of zero states. Basis expansion
checks primitives retained by evaluation; removed terms no longer run basis rules,
including a tensor annihilated by a zero state factor. This boundary differs from
operator coordinates, which still validate expansions of reached intrinsic terms.
See [basis errors](basis.md#74-errors-and-examples).

### 11.5. Incremental scalar contractions

State inner products validate compatible spaces, evaluate/collect both operands,
and align independent tensor groups. Contraction distributes sums only where
needed inside those groups. A product continuation retains its accumulated value
and remaining groups, schedules one group, and proceeds only while the value is
nonzero. An exact-zero local contraction or underflowed running product discards
the remaining contractions and pending coefficient scaling for that product.
A zero summand or zero partial sum never stops the enclosing sum.

Local state contractions skip coefficient multiplication and left conjugation
after a zero primitive overlap; explicit zero term coefficients can skip the
corresponding primitive call. Nonzero local contributions still accumulate in
order. Primitive equality is not used to infer orthogonality: each visited
primitive overlap comes from its InnerProduct rule.

Full trace and normalized HS use the existing ordinary lowering, including
parity strings. They contract local factors in selected ambient order and stop
only the current tensor term on an exact-zero local value or running product.
A zero primitive trace skips its HS normalization division and coefficient work;
nonzero primitive traces are still divided by `Tr(I)` before coefficient
multiplication and summation. The identity trace establishing the local dimension
remains separate from the per-factor work. No total dimension, basis, state type,
orthogonality metadata, or result cache is introduced for operator HS.

Space/reference validation precedes these shortcuts. Operand evaluation, graded
lowering, and structural group discovery retain their existing responsibilities;
a shortcut does not imply that all preparation of a later factor was avoided.
The test for zero is exactly `Zero::is_zero()`, with no threshold or finite-value
rejection. Nonzero tiny values continue; NaN alone does not stop a product.
Avoiding arithmetic after a proven zero follows mathematical zero propagation,
without guaranteeing a particular IEEE sequence's signed-zero or NaN/inf propagation.
Inner-product helpers gain no coefficient equality bound; public state paths
inherit only the [evaluation bounds](#42-shared-apis-and-minimal-capabilities).

### 11.6. Sparse operator basis generation

Operator expand/coordinates consume signed intrinsic factors and use
[direct coordinate generation](basis.md#operator-coordinate-generation):
query-local spectator identity choices, sparse specified-factor data, a working
full-length key, and one final accumulator. The single-component identity path
uses its actual basis digit and coefficient; general identities share their
local choices across spectator positions. Expression expansion additionally
shares local basis payloads. Operator queries use the public bounds and
coordinate format defined in basis design. State basis expansion and trace/HS
contraction use their own paths.

## 12. Verification targets and tests

[core_tensor_products.rs](../../tests/tests/core_tensor_products.rs) compares space kinds, identifications, mixed-kind rejection, construction entry points, fallible aggregation, signs, CAR, action, adjoints, inner products, trace, and HS inner products against independent ordinary matrices. It includes Act scope, embedding homomorphisms, separated factors, odd-component exchange, and mixed parity.

[core_basis_expansion.rs](../../tests/tests/core_basis_expansion.rs) covers bases and re-expansion; [export_tensor_products.rs](../../tests/tests/export_tensor_products.rs) covers dense and sparse output and metadata; [normalize_tensor_products.rs](../../tests/tests/normalize_tensor_products.rs) covers normalization. Verification also covers minimal capabilities, borrows, sharing, deep trees, and contractions requiring no dense storage. See [verification](verification.md) for commands and limits.
