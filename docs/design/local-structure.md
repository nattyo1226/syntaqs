# Local algebras and primitives

Last updated: 2026-11-02

## 1. Scope and type responsibilities

This document defines the meaning, dimensions, local rules, normalization, bases, and grading of state and operator primitives. S, O, and their individual capability implementations define the local algebra; there is no additional LocalStructure type, definition label, or definition ID.

Local expressions are `LocalState<S,O,C>` and `LocalOperator<O,C>`; composite expressions are `State<I,S,O,C>` and `Operator<I,O,C>`. State expressions also specify O to retain delayed actions. Operator expressions do not require a state type. If the same O acts on several S types, each S provides the corresponding Act implementation.

Storage, sharing, and delayed construction require neither evaluation capabilities nor Clone on primitives. Primitives may contain borrows; no uniform Send, Sync, or 'static constraint applies. Sharing and evaluation follow [local expressions](primitive-expression.md) and [composite expressions](composite-expression.md).

## 2. Associating space information with capabilities

| Responsibility | Implementation |
| --- | --- |
| Primitive meanings and local rules | S, O, and individual capability implementations; no additional definition ID or label |
| Local state-space dimension | Dimension on S: the dimension d of the span of all primitive values |
| Local operator-space dimension | Dimension on O: d² for the full operator space, distinct from the variant count |
| Identity operator | O implements `Identity::identity() -> Self`, independently of the coefficient type |
| Action type correspondence | Evaluation bounds such as `S: Act<C, Operator = O, Term = S>` and `O: Adjoint<C, Term = O>` |
| Scalar consistency | C and the bounds of each operation; different precisions are not mixed implicitly |
| Bases | `Basis<S, C>` and `Basis<O, C>`; enumeration size equals the dimension of the primitive span |
| Factor structure and geometry | `HilbertSpace<I>` retains factor order and grouping; Chain retains only site count and boundary conditions. An external sequence of local spaces specifies placement |

Dimension provides `fn dim() -> Result<NonZeroUsize, DimensionOverflow>`. If the dimension cannot be represented as an integer, querying it returns an error. `dim_state<S>()` and `dim_op<O>()` return the respective dimensions directly; the operator dimension d² is not squared again. Only operations that use dimensions require this capability.

The identity must be representable as a primitive value, but need not belong to the selected basis. Basis expands it when necessary. Linearly dependent primitives are allowed, so adding an identity value does not increase the basis dimension. Local expressions retain identity as `Primitive(O::identity())`, using the ordinary primitive paths for action, products, adjoints, trace, and basis expansion. `LocalOperator::identity()` and an empty `product` require `O: Identity` but no numeric coefficient capabilities. The symbolic `OpNode::Identity` remains available for an unbound empty product or the identity on the full ambient space.

The trace of local identity and spectator factors comes from O's Identity and Trace. It is not replaced with conversion of an integer dimension into a coefficient; local trace gains no additional One or integer-conversion bound. If Identity cannot be implemented for a foreign O, use an upstream implementation or a user-owned wrapper. `Identity` and `Dimension` are exported from `syntaqs_core::ops` and the crate root. `Identity` is also implemented for local expressions (`O: Identity`) and composite expressions (no additional capability bounds).

### 2.1. Provided primitives and normalization

| Algebra | State primitives | Operator primitives |
| --- | --- | --- |
| spin-half | `Up, Down` | `I, Sx, Sy, Sz, Sp, Sm` |
| fermion | `Empty, Occupied` | `I, Annihilate, Create, Number, Mx, My, Mz` |
| spin-one | `Plus, Zero, Minus` | `I, Sx, Sy, Sz, Sp, Sm, Dxy, Q0, Qxy, Qyz, Qzx, Q1p, Q1m, Q2p, Q2m` |

The state-space dimensions are respectively 2, 2, and 3; operator-space dimensions are 4, 4, and 9. These are distinct from variant counts and selected-basis sizes. Zero is represented by an empty linear combination or expression.

The spin-half CartesianBasis, fermion MajoranaBasis, and spin-one CartesianBasis are orthonormal under the [HS inner product](primitive-expression.md#41-dimension-normalized-hilbertschmidt-inner-products) divided by the state-space dimension d. This condition does not apply to every primitive or arbitrary custom Basis. Ladder operators are defined through their connection to Cartesian components.

Spin-half follows the [Pauli-matrix convention](#6-spin-12-conventions), with `Sp=(Sx+iSy)/2` and `Sm=(Sx−iSy)/2`. Physical spin J is `J=S/2` for spin-half and `J=√(2/3) S` for spin-one, expressed with existing scalar multiplication. There is no helper API for constructing physical spin. Fermion and spin-one details appear below; model coupling constants follow [model construction](models.md).

## 3. Consistency of mathematical capabilities

Here `A,B,C` are operators of the corresponding primitive type O, `ψ,φ` are states, and `I` is identity. The following contracts also apply to linear combinations of primitives.

- Closure and membership: Product, Act, and adjoints return values within the corresponding S and O types. Concrete representations and capability implementations must agree.
- Linearity: products and actions are linear in each argument; inner products are conjugate-linear in the first and linear in the second; adjoints are conjugate-linear; trace is linear.
- Associativity and compatible action: `(AB)C = A(BC)` and `(AB)ψ = A(Bψ)`.
- Identity: `IA = AI = A` and `Iψ = ψ`.
- Inner product: conjugate symmetry and positive definiteness on the state space. With dependent primitives, even a nonzero coefficient sequence may represent the zero state.
- Adjoint: the relation to inner product and action below, together with `(A†)† = A` and `(AB)† = B†A†`.
- Trace: ordinary unnormalized trace on the local Hilbert space, satisfying `Tr(I) = d`.
- Preservation of linear relations: operations on different linear combinations representing the same state or operator yield mathematically equal results. Unique coefficient sequences or expression trees are not required.
- Grading, where supported: homogeneous primitive parities agree with local rules. The shared composite-algebra machinery handles signs between factors.

```math
\langle\phi,A\psi\rangle=\langle A^\dagger\phi,\psi\rangle.
```

These are mathematical contracts and verification targets for local implementations. There is no runtime mechanism that automatically proves associativity or other laws for arbitrary implementations. S, O, and their capabilities determine meanings and rules without additional definition labels. Structures with runtime parameters are not supported.

Equalities concern mathematical meaning, separately from bitwise equality in floating-point computation. Precision, special values, and approximate comparisons follow [numeric conventions](numerics.md).

Local Product and Act return mathematically simplified final coefficients directly; analytically zero results are empty linear combinations. Unit coefficients use One, and private helpers consistently generate constants. Runtime rules do not repeatedly convert to and from the previous normalization.

## 4. Conditions for matching local structures

S, O, and their capability implementations determine primitive meanings and mathematical conventions. The same S and O are not distinguished by another L or definition ID. Different rules or representations must be reflected in primitive types and capabilities. A separate definition label cannot isolate the same primitive type.

State tensors, sums, and inner products take the same S, O, and C; operator sums and products take the same O and C. Action combines a state expression and an operator expression with corresponding O, requiring the Act relationship at evaluation. Matching types do not prove mathematical laws; concrete algebra implementations and verification carry that responsibility.

Index, Site, display names, selected bases and enumeration order, object identity in memory, and sharing strategies do not create new definition labels. There is no mathematical-isomorphism detection, implicit conversion across types, or automatic primitive correspondence.

## 5. Connection to the Index contract

[Index](index.md) and local-algebra semantics have separate responsibilities. Spaces retain factor indices, order, and grouping; expressions retain primitive types. Space structure validates duplicate factors and placement or binding targets. S, O, and C bounds reject inappropriate composition of heterogeneous expressions. There is no global registry of indices and local definitions, or uniform TypeId or 'static bound.

Space dimensions apply the span of the primitive specified in `space.dim::<P>()` to each factor, or use an explicit local dimension in `space.dim_with(dim_local)`. S queries state-space dimension; O queries operator-space dimension. This does not register a primitive definition in the space. An empty tensor has dimension 1; ordinary action, inner products, and trace do not require an integer total dimension.

## 6. Spin-1/2 conventions

With `hbar = 1`, primitives are Pauli matrices and physical angular momentum is `J = S/2`. Their relation to Pauli matrices is

```math
S^\alpha = \sigma^\alpha
```

The ladder operators are

```math
S^+ = (S^x + iS^y)/2, \qquad
S^- = (S^x - iS^y)/2
```

The default local basis order is

```math
(|\uparrow\rangle, |\downarrow\rangle)
```

These correspond to `Sz` eigenvalues `+1` and `-1`, and physical `Jz` magnetic quantum numbers `+1/2` and `-1/2`. Local matrix representations in this basis are:

```math
S^x =
\begin{pmatrix}0&1\\1&0\end{pmatrix}, \qquad
S^y =
\begin{pmatrix}0&-i\\i&0\end{pmatrix}, \qquad
S^z =
\begin{pmatrix}1&0\\0&-1\end{pmatrix}
```

```math
S^+ =
\begin{pmatrix}0&1\\0&0\end{pmatrix}, \qquad
S^- =
\begin{pmatrix}0&0\\1&0\end{pmatrix}
```

The state basis `SzBasis` is ordered `[Up,Down]`; operator bases `CartesianBasis` and `LadderBasis` are `[I,Sx,Sy,Sz]` and `[I,Sp,Sm,Sz]`. State-space dimension is 2 and operator-space dimension is 4. CartesianBasis is orthonormal under `Tr(A†B)/2`; ladder elements need not have the same unit norm.

All primitives are even and the parity operator is I. See the [spin-half source](../../crates/syntaqs-spin-half/src/lib.rs) for local rules and bases, and [contracts.rs](../../crates/syntaqs-spin-half/tests/contracts.rs) for independent matrix references covering all local rules, algebraic laws, and bases. Rules return mathematically simplified coefficients directly; expression construction and storage require no evaluation capabilities.

## 7. Local grading capabilities

`syntaqs_core::ops` and the crate root export these coefficient-independent capabilities:

```rust,ignore
enum Parity { Even, Odd }
trait HomogeneousParity {
    fn parity(&self) -> Parity;
}
trait ParityOperator: Sized {
    fn parity_operator() -> Self;
}
```

S and O in a graded local algebra are homogeneous primitives, both providing `HomogeneousParity`. Mixed parity is expressed through sums in `LocalState` / `LocalOperator`; a mixed expression has no single parity. No blanket implementation implicitly treats unsupported types as even.

O returns the local parity operator P as a primitive value through `ParityOperator`. Like identity, it need not be a selected-basis element; a basis may expand it as `P = I - 2n`. Adding a linearly dependent P value does not change the span dimension.

The contracts are `P† = P`, `P² = I`, `Pψ = (-1)^p(ψ) ψ`, and `POP = (-1)^p(O) O`. Identity and P are even; adjoints preserve parity. Nonzero output terms of local Product / Act have the sum of input parities modulo 2. Even and odd state subspaces are orthogonal. Basis primitives obey the same contracts. Concrete algebra implementations and tests establish these laws; core does not automatically prove them at runtime.

Custom ordinary-only algebras also explicitly implement the necessary grading traits when using the shared composite-evaluation API. Trivially graded algebras make every state and operator even and set P to I.

```rust,ignore
impl HomogeneousParity for MyState {
    fn parity(&self) -> Parity { Parity::Even }
}
impl HomogeneousParity for MyOperator {
    fn parity(&self) -> Parity { Parity::Even }
}
impl ParityOperator for MyOperator {
    fn parity_operator() -> Self { Self::identity() }
}
```

Spin-half explicitly implements this trivial grading. Existing model spaces remain ordinary and give identical numerical results in graded spaces. Purely local evaluation, construction, placement, and binding do not uniformly require grading capabilities. Operation-specific bounds are in [composite expressions](composite-expression.md#11-shared-ordinary-and-graded-evaluation).

## 8. Fermion local algebra

`syntaqs-fermion` uses `no_std + alloc`, forbids unsafe, and provides coefficient-independent `FermionState` and `FermionOperator`. States are `Empty/Occupied`; the seven operator variants are `I/Annihilate/Create/Number/Mx/My/Mz`. State-space dimension is 2 and operator-space dimension is 4.

`Fermion<C>` aliases `ExpressionFactory<FermionState,FermionOperator,C>`. Public entry points are in [lib.rs](../../crates/syntaqs-fermion/src/lib.rs), local rules in [state.rs](../../crates/syntaqs-fermion/src/state.rs) and [operator.rs](../../crates/syntaqs-fermion/src/operator.rs), and bases in [basis.rs](../../crates/syntaqs-fermion/src/basis.rs). Models follow the [Hubbard model design](models.md#3-fermionic-hubbard-models).

### 8.1. Conventions, dimensions, and parity

State order is `(|0>,|1>)=(Empty,Occupied)`, with Kronecker-delta inner products. Write `a=Annihilate`, `a†=Create`, and `n=Number`.

```math
a=\begin{pmatrix}0&1\\0&0\end{pmatrix},\quad
a^\dagger=\begin{pmatrix}0&0\\1&0\end{pmatrix},\quad
n=\begin{pmatrix}0&0\\0&1\end{pmatrix},
```

```math
M_x=a+a^\dagger=\begin{pmatrix}0&1\\1&0\end{pmatrix},\quad
M_y=\frac{a-a^\dagger}{i}=\begin{pmatrix}0&-i\\i&0\end{pmatrix},\quad
M_z=-iM_xM_y=I-2n=\begin{pmatrix}1&0\\0&-1\end{pmatrix}.
```

Mx and My are Majoranas with the standard normalization of [Kitaev, equations (1)–(2)](https://arxiv.org/pdf/cond-mat/0010440). Mz is an even bilinear operator and the parity operator P itself, rather than a third odd Majorana.

```math
[M_x,M_y]=2iM_z,\quad [M_y,M_z]=2iM_x,\quad [M_z,M_x]=2iM_y,\quad
M_\alpha^2=I,\quad \sum_\alpha M_\alpha^2=3I.
```

Empty is even and Occupied is odd. a, a†, Mx, and My are odd; I, n, and Mz are even. `ParityOperator::parity_operator()` returns Mz, with P satisfying section 7. Ordinary local multiplication within one mode is distinct from graded composition between modes.

### 8.2. Local capabilities

#### 8.2.1. Complete Product rules

Rows are left operands and columns are right operands; the rightmost operator acts first.

| Left × right | I | a | a† | n | Mx | My | Mz |
| --- | --- | --- | --- | --- | --- | --- | --- |
| I | I | a | a† | n | Mx | My | Mz |
| a | a | 0 | I−n | a | I−n | i(I−n) | −a |
| a† | a† | n | 0 | 0 | n | −in | a† |
| n | n | 0 | a† | n | a† | ia† | −n |
| Mx | Mx | n | I−n | a | I | iMz | −iMy |
| My | My | in | −i(I−n) | −ia | −iMz | I | iMx |
| Mz | Mz | a | −a† | −n | iMy | −iMx | I |

All 49 pairs use direct matches. Mathematically zero products return empty linear combinations; products with identity return the other operand as one term. Local rules satisfy `a²=(a†)²=0`, `{a,a†}=I`, `n=a†a`, and parity laws. Runtime local products use neither matrices nor basis conversions.

#### 8.2.2. Act and inner products

| Operator | Action on Empty | Action on Occupied |
| --- | --- | --- |
| I | Empty | Occupied |
| a | 0 | Empty |
| a† | Occupied | 0 |
| n | 0 | Occupied |
| Mx | Occupied | Empty |
| My | i Occupied | −i Empty |
| Mz | Empty | −Occupied |

Core provides linearity and sesquilinearity for state expressions.

#### 8.2.3. Adjoint and Trace

Adjoint swaps a and a†, preserving the other five elements. Ordinary unnormalized trace is 2 for I, 1 for n, and 0 otherwise. Only the HS inner product applies dimension normalization as `Tr(A†B)/2`.

### 8.3. Bases and expansion

| Basis | Enumeration order |
| --- | --- |
| `OccupationBasis` | `[Empty,Occupied]` |
| `LadderBasis` | `[I,Annihilate,Create,Number]` |
| `MajoranaBasis` | `[I,Mx,My,Mz]` |

MajoranaBasis is orthonormal under `Tr(A†B)/2`. LadderBasis is nonorthogonal because I and n have a nonzero inner product; normalization is not applied uniformly.

```math
a=(M_x+iM_y)/2,\qquad a^\dagger=(M_x-iM_y)/2,\qquad n=(I-M_z)/2.
```

Inverse transformations use the definitions in section 8.1. index returns enumeration indices only for members of the selected basis: Mz has index 3 in MajoranaBasis and None in LadderBasis. Re-expansion, coordinates, and action matrices are checked across both bases.

### 8.4. Coefficient types and capabilities

Identity, Dimension, parity, and expression construction require no numeric bounds. InnerProduct and Trace require Zero + One; adjoint and OccupationBasis require One. Product requires One + Add + Neg + ImaginaryUnit; Act requires One + Neg + ImaginaryUnit; LadderBasis requires One + Add + Neg + ImaginaryUnit. MajoranaBasis additionally uses Div. Arithmetic Output is C.

Operations using fractions require a representation of the mathematical value 1/2. Local rules require neither Float nor Clone / Copy on C. Shared expression evaluation, HS inner products, and export impose their own operation-specific bounds. HS inner products use the existing Divide capability to divide by `Tr(I)`.

### 8.5. Graded composition and spinful systems

Core handles mode order, exchange signs, and parity strings over preceding factors. Primitives retain no Index, Site, or mode number. The internal spinful order is `(up,down)` and the double-occupancy phase is `|up down>=f_up† f_down†|0>`. Verification covers `f_up|up down>=|down>`, `f_down|up down>=−|up>`, and total parity `diag(1,−1,−1,1)`.

```math
\{f_\sigma,f_\tau\}=0,\qquad
\{f_\sigma,f_\tau^\dagger\}=\delta_{\sigma\tau}I,\qquad
P_i=(-1)^{n_{i,\mathrm{up}}+n_{i,\mathrm{down}}}.
```

Display order `[empty,up,down,double]` differs from the two-mode mixed-radix order `[empty,down,up,double]`. Merely placing operators in an ordinary tensor does not establish CAR between different modes; fermionic many-body systems use graded spaces.

### 8.6. Verification targets and tests

[contracts.rs](../../crates/syntaqs-fermion/tests/contracts.rs) checks local rules and bases against independent 2×2 matrices. [minimal_scalar.rs](../../crates/syntaqs-fermion/tests/minimal_scalar.rs) checks coefficient capabilities without Clone / Copy / Float. [local_expressions.rs](../../crates/syntaqs-fermion/tests/local_expressions.rs) checks expressions and every backend. [models.rs](../../crates/syntaqs-fermion/tests/models.rs) checks graded integration, CAR, and both Hubbard models against independent Fock-space references. Numerical tests cover both f32 and f64.

## 9. Spin-one local algebra

`syntaqs-spin-one` provides three spin-1 states, 15 operator primitives, two operator bases, and XYZ, Heisenberg, and BLBQ models. It permits std and forbids unsafe. Model definitions follow [SpinOne model construction](models.md#4-spinone-model-construction).

### 9.1. Primitives and conventions

```rust
enum SpinOneState { Plus, Zero, Minus }
enum SpinOneOperator {
    I, Sx, Sy, Sz, Sp, Sm, Dxy, Q0, Qxy, Qyz, Qzx,
    Q1p, Q1m, Q2p, Q2m,
}
```

State order is `(|+1>,|0>,|-1>)`, with Kronecker-delta inner products. Zero is the normalized state with magnetic quantum number 0; the zero vector is an empty linear combination. State-space dimension is 3 and operator-space dimension is 9.

Let J be physical spin-1 and `c=√(3/2)`. Normalized primitives are `S_alpha=c J_alpha`, where J is ordinary angular momentum with ℏ=1.

```math
S_x=\frac{\sqrt3}{2}\begin{pmatrix}0&1&0\\1&0&1\\0&1&0\end{pmatrix},\quad
S_y=\frac{\sqrt3}{2}\begin{pmatrix}0&-i&0\\i&0&-i\\0&i&0\end{pmatrix},\quad
S_z=c\operatorname{diag}(1,0,-1).
```

Quadrupoles are defined as follows.

| Primitive | Definition |
| --- | --- |
| `Dxy` | `c (Jx²−Jy²)=(Sx²−Sy²)/c` |
| `Q0` | `(3Jz²−2I)/√2=√2(Sz²−I)` |
| `Qxy/Qyz/Qzx` | `c {Jx,Jy}`, `c {Jy,Jz}`, and `c {Jz,Jx}`, respectively |
| `Sp/Sm` | `Sx±iSy` |
| `Q1p/Q1m` | `Qzx±iQyz={Sz,Sp/m}/c` |
| `Q2p/Q2m` | `Dxy±iQxy=(Sp/m)²/c` |

All nine Hermitian elements have ordinary squared HS norm 3 and are orthonormal under `Tr(A†B)/3`. The three spin components and five quadrupole components retain their respective rotation subspaces. Physical spin J uses existing scalar multiplication as `√(2/3) S`, without an additional helper API.

### 9.2. Bases and expansion

| Basis | Enumeration order |
| --- | --- |
| `SzBasis` | `[Plus,Zero,Minus]` |
| `CartesianBasis` | `[I,Sx,Sy,Sz,Dxy,Q0,Qxy,Qyz,Qzx]` |
| `LadderBasis` | `[I,Sp,Sm,Sz,Q0,Q1p,Q1m,Q2p,Q2m]` |

CartesianBasis is orthonormal under the dimension-normalized HS inner product. LadderBasis is orthogonal: I, Sz, and Q0 have squared norm 1, and the six ladder elements have squared norm 2. Ladder operators prioritize their direct connection to Cartesian components over uniform norms. Ordinary squared HS norms are respectively 3 and 6.

Each pair transforms as `L_±=A±iB`, `A=(L_++L_-)/2`, and `B=(L_+−L_-)/(2i)`. I, Sz, and Q0 are shared by both bases; nonmembers have index None. Nine operator coordinates are distinct from a 3×3 action matrix in a state basis.

### 9.3. Local rules and result representations

All 225 products and 45 actions directly return finite linear combinations. Mathematically zero terms are not generated; products with identity return the other operand as one term. Ordinary evaluation requires no basis.

| Input pair | Returned primitives |
| --- | --- |
| Product with I | The other operand, as one term |
| Both in the shared subset I, Sz, Q0 | Closed in `[I,Sz,Q0]` |
| Both CartesianBasis members | CartesianBasis members |
| Both LadderBasis members | LadderBasis members |
| Mixture of Cartesian-only and ladder-only elements | CartesianBasis members |

Results follow the target basis's enumeration order. Local products use no matrix-unit conversion, dense matrices, or symbolic computation. Mathematically simplified final coefficients are generated in the selected precision.

```math
[S_x,S_y]=icS_z,\quad \sum_\alpha S_\alpha^2=3I,\quad
S_z^2=I+Q_0/\sqrt2,\quad Q_0^2=I-Q_0/\sqrt2,
```

```math
S_\pm^2=cQ_{2\pm},\quad \{S_z,S_\pm\}=cQ_{1\pm},\quad S_\pm^3=0.
```

Nonzero transition amplitudes of Sp and Sm are √3; Sx amplitudes between adjacent states are √3/2. Sz eigenvalues are `c,0,−c`; magnetic quantum numbers in state names refer to physical Jz. Adjoint swaps Sp/Sm, Q1p/Q1m, and Q2p/Q2m, preserving the other nine elements. Ordinary trace is 3 for I and 0 for the other 14 elements.

### 9.4. Coefficient capabilities and shared APIs

| Capability | Required coefficient capabilities |
| --- | --- |
| Identity, Dimension, ExpressionFactory, storage | None |
| InnerProduct, Trace | `Zero + One` |
| Adjoint, SzBasis | `One` |
| CartesianBasis | `One + Neg + ImaginaryUnit` |
| LadderBasis | `One + Add + Div + Neg + ImaginaryUnit` |
| Product, Act | `Complex<R>`, `R: num_traits::Float` |

Arithmetic Output is C. Product also needs square-root coefficients, so it currently uses Float; no new trait for exact algebraic coefficients is introduced. Basis transformations use only rational coefficients and i. Storage, delayed construction, adjoints, and trace require no Float. Square roots are generated from R's One, arithmetic, and sqrt, without casting or parsing through f64.

This integrates with shared local and composite expressions, HS inner products, normalization, and export. HS inner products divide local traces in the ordinary representation by Tr(I) before contraction, without requiring an integer total dimension. Ordinary trace and state inner products retain their meanings. Integration does not change the local rules' Float or std requirements.

### 9.5. Composition, export, and grading

All states and operators are even; the parity operator is I. Ordinary and graded spaces yield identical values. Magnetic-quantum-number parity is not fermion parity. Two factors have 9 state-basis and 81 operator-basis elements, in rightmost-fastest mixed-radix order. Existing export generates action matrices column by column; primitives implement no backend-specific matrix conversion.

### 9.6. Verification targets and tests

[contracts.rs](../../crates/syntaqs-spin-one/tests/contracts.rs) builds references from independent physical J matrices and the chosen scale. It checks all 225 products, 45 actions, adjoints, trace, associativity, compatible action, both bases, and Gram matrices for f32 and f64.

[minimal_scalar.rs](../../crates/syntaqs-spin-one/tests/minimal_scalar.rs) checks minimal bounds for capabilities without square roots. [local_expressions.rs](../../crates/syntaqs-spin-one/tests/local_expressions.rs) checks local expressions and all backends. [tensor_products.rs](../../crates/syntaqs-spin-one/tests/tensor_products.rs) checks composite bases, ordinary / graded spaces, and all backends. [models.rs](../../crates/syntaqs-spin-one/tests/models.rs) checks the three physical models, energies, and construction capabilities.

Numerical comparisons use complex magnitude: `|a−b| <= atol + rtol max(|a|,|b|)`. Current small tests use atol=rtol=2e-5 for f32 and atol=rtol=1e-12 for f64. Verification tolerances are not reused for pruning ordinary evaluation terms. Exact zeros in local rules are distinct from cancellation residuals in general expressions.
