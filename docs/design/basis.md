# Basis expansion, coordinates, and enumeration

Updated: 2026-11-02

## 1. Scope and contract

Under [requirements](../requirements.md) and [architecture](../architecture.md), this document specifies local state/operator bases, tensor-product bases, and expression expansion, detailing the basis connection of [local expressions](primitive-expression.md).

Core owns basis capabilities, whole-expression expansion, composite bases, and coordinates. Concrete algebras implement local bases and primitive expansion rules; export stores the resulting coefficients in numerical containers.

A struct such as CartesianBasis defines a basis through `Basis<P,C>`, selecting primitive members and expansion rules. State expansion returns `LinearCombination<S,C>`, operator expansion `LinearCombination<O,C>`; expressions map these elements into ordinary syntax. Callers need not supply arrays/sets of basis elements.

One Basis implementation provides expansion, indexing, and enumeration. The trait has no numeric bounds; concrete implementations request their rule's capabilities, which consequently apply to index/elements too. Use a qualified call such as `<B as Basis<P,C>>::elements` when precision cannot be inferred.

State/operator processing uses the same trait:

| Processing | Bound |
| --- | --- |
| Local state bases | `B: Basis<S, C>` |
| Local operator bases | `B: Basis<O, C>` |

Local basis elements are a linearly independent, complete subset of the corresponding primitives, not arbitrary expressions. Orthonormality is not required, and the full primitive set may be dependent.

Users determine the [local algebra](local-structure.md), its definition parameters, and whether the target is a state/operator. Basis<S,C>/Basis<O,C> establishes that correspondence without a separate membership declaration. Basis selection does not define algebra or factor identity.

Composite bases currently apply one local basis to all factors of the same S/O types, without per-factor basis/parameter choices. This is an API/storage scope decision rather than a mathematical consequence of homogeneous primitive types. A custom algebra's basis can be used in a separate homogeneous system.

## 2. Three Basis capabilities

| Capability | Input | Result and contract |
| --- | --- | --- |
| Primitive expansion | A primitive in the target space | Finite combination of selected basis primitives. |
| Basis index | A primitive | Its index if it is itself a basis member, otherwise none. |
| Ordered enumeration | Basis definition/parameters | Basis primitives generated as needed in index order. |

For `B={b_0,…,b_(n−1)}` and every target primitive p:

```math
p=\sum_{\mu=0}^{n-1}c_\mu(p)b_\mu.
```

Independence makes these coefficients unique. Expansion represents the whole object without projection/pruning. Nonmembers expand but have no index identifying themselves.

| Input | Result |
| --- | --- |
| State primitive S | `LinearCombination<S, C>` |
| Operator primitive O | `LinearCombination<O, C>` |

Results contain only selected basis primitives. A coefficient/order-preserving map and common constructors produce ordinary Primitive/Scale/Sum syntax. Neither combinations nor expressions retain basis metadata or a special expanded state; they remain usable for ordinary operations and re-expansion. Verify membership and conversion consistency.

Indices are zero-based basis coordinates, not implicit label/value order or factor identity. `index(&self, primitive: &P) -> Option<usize>` returns None for nonmembers.

`elements(&self) -> impl Iterator<Item=P> + '_` lazily yields owned elements without materializing all of them. Index-to-element lookup can be derived from enumeration and is not a separate mandatory capability. Representation and shared primitive storage are implementation details.

## 3. Deriving basis sizes

Derive sizes from local structure and target kind. For local Hilbert dimension d:

| Space | Basis size |
| --- | --- |
| State space | d |
| Full operator space | d² |

Basis has no independent size capability. Convenience queries also derive these values, rather than defining size by primitive variant count or a completed iterator count.

This contract assumes complete bases, excluding subspace/projection and fermion-even-only bases. `dim_state<S>()`/`dim_op<O>()` query Dimension and return `Result<usize, DimensionOverflow>`. O already has span dimension d² and is not squared again. Representability errors occur at queries, not storage/construction.

## 4. Capability consistency

- Enumerate each basis element once; element k has index k.
- Enumeration and indexing agree for the same definition/parameters.
- Enumeration count equals the derived dimension.
- A basis member expands to itself with coefficient 1.
- Every output primitive is a selected basis member.
- Preserve primitive linear relations, giving mathematically identical coordinates for equivalent expressions.

These laws, independence, and completeness are implementation contracts and verification obligations, not runtime automatic proofs over every possible primitive. Precision, special values, and comparisons follow [numerics](numerics.md).

## 5. Expressions, coordinates, and numerical conversion

`Basis::expand` expands one primitive; core extends this linearly to expressions, evaluating local/composition rules as needed for products/actions. Basis indices associate coefficients with coordinates and aggregate equal members.

Core provides explicit expansion/coordinates separately from local-rule evaluate. Coordinates are mathematically unique without requiring a generally optimal simplification normal form. Inputs remain unchanged.

expand returns the ordinary expression type. Coordinates are a separate public operation on an expression and basis, without prior expand or a dedicated expanded type. Expanded trees gain no coordinate containers, basis labels, markers, or caches. Separate single-component access and unaggregated contribution enumeration are not required APIs. Section 7 defines numbering/results.

Inner products, Gram matrices, and dual bases may implement expansion, but are not mandatory Basis capabilities. Direct rules suffice even for non-orthonormal bases, without a shared generic Gram solver or extra numeric dependency. Operator inner products use [dimension-normalized HS](primitive-expression.md#41-dimension-normalized-hilbertschmidt-inner-products), `Tr(A†B)/D`, where D is the action-space dimension.

For a complete Hermitian basis with `Tr(B_mu B_nu)=D delta_mu,nu`, coefficients are `a_mu=Tr(B_mu A)/D`. Tensor products of normalized orthonormal local operator bases stay orthonormal under total-dimension-normalized HS. Verify graded orthogonality also using ordinary action matrices with parity strings.

Ordinary evaluation/coordinates do not approximately discard components. Explicit pruning remains [future expression work](../future-work.md#22-expression-and-space-operations).

Expansion/coordinates are separate from ordinary product/action/inner-product/trace paths, which need no basis. Expand necessary parts when explicitly requested, without complete numerical matrices as intermediates. Aggregate by basis index rather than structural expression comparison.

Distribution, local multiplication, and coefficient handling are internal expansion steps, not independently selectable simplification APIs. Follow [numerics](numerics.md); comparison tolerances never silently discard components.

### 5.1. Composite tensor-product bases

Apply one local basis uniformly in Hilbert-space factor order, using state bases for states and operator bases for operators. Core derives the composite basis from the local basis and space; callers construct no separate object. Custom multi-factor or entangled bases are outside this contract, without limiting representable objects because each local basis is complete.

Use mixed-radix order with the rightmost factor varying fastest. For local indices s_i and sizes d_i, flatten as follows; current uniform basis arguments use the same basis/dimension throughout.

```math
\operatorname{index}(s_0,\ldots,s_{N-1})
=\sum_{i=0}^{N-1}s_i\prod_{j=i+1}^{N-1}d_j.
```

Two spin-1/2 SzBasis factors enumerate `|up,up>`, `|up,down>`, `|down,up>`, `|down,down>` at indices 0,1,2,3, shared by dense/sparse output.

Sizes multiply. Grouping changes parentheses only, not coordinates for equal factors/bases/order. The empty tensor basis is `{1}` and a singleton uses its local basis.

Graded operator bases follow [composite composition](composite-expression.md). Operator tensor notation expresses mathematical rules, not another constructor/node. Do not materialize all elements; process needed elements/components only. Choosing bases does not change factor order or compatibility.

### 5.2. Expansion returns ordinary expression trees

The basis is an operation input. Expansion returns ordinary syntax in selected members; local LinearCombination results transfer data for constructing that syntax. No retained basis identity or special expanded state is required. Inputs are unchanged; indices, local algebra, and definite state/ambient spaces remain. Original sources are not automatically retained.

Expanded expressions support products, actions, inner products, traces, and expansion again. Re-expanding in a different basis needs neither previous basis metadata nor a separate tree basis-change mechanism. Direct expansion and re-expansion have equal mathematical coordinates, subject to ordinary floating-point ordering differences.

Numerical vectors/matrices require a definite basis/order for interpretation, distinct from storing basis metadata on expression trees.

## 6. CartesianBasis example and verification

Spin-half CartesianBasis implements Basis for operators in order `{I,Sx,Sy,Sz}` under [local conventions](local-structure.md#6-spin-12-conventions).

| Primitive | Expansion | Index |
| --- | --- | --- |
| `I` | `I` | `0` |
| `Sx` | `Sx` | `1` |
| `Sy` | `Sy` | `2` |
| `Sz` | `Sz` | `3` |
| `Sp` | `(Sx + i Sy)/2` | None |

Enumeration is I,Sx,Sy,Sz. State dimension d=2 gives operator size d²=4. Verify enumeration/index/expansion consistency and equal coordinates for Sp and its Cartesian expression.

Verify that SzBasis returns state combinations, CartesianBasis/LadderBasis operator combinations, and map/common constructors accept them into local syntax. LadderBasis enumerates `{I,Sp,Sm,Sz}`. Check member-only outputs and rules/order at both precisions, plus custom local algebras and non-orthonormal bases.

Whole-expression examples are `Sp+Sm` expanding to Sx and `Sx Sx` to I: the first aggregates coefficients, the second evaluates local multiplication. Verify unchanged inputs, retained space, and basis-independent ordinary evaluation.

Expanded syntax works without metadata: expand Sx into Sp+Sm, then back into CartesianBasis for Sx coordinates. Composite tests cover grouping-independent elements/coordinates, rightmost-fastest order, and empty/singleton cases.

## 7. Public APIs

### 7.1. Expansion and aggregated coordinates

| Target | API | Result |
| --- | --- | --- |
| LocalState / LocalOperator | `expand(&basis)` | Same local expression type, aggregated in the basis. |
| State | `expand(&basis)` | Ordinary state sharing its original space. |
| Operator | `expand(&basis)` | Ordinary operator on its binding; unbound is an error. |
| Operator | `expand_in(&basis, &ambient)` | Ordinary operator bound to the explicit, preferred space. |
| Local expressions / State / Operator | `coordinates(&basis)` | Independent aggregated sparse Coordinates. |
| Operator | `coordinates_in(&basis, &ambient)` | Coordinates using the preferred explicit space. |

All return `Result<_,BasisError>`, applying the same local basis to all factors and aggregating indices after evaluating local rules. States use their own space; operators prefer explicit space, then binding. A local basis never implies an unbound operator's ambient space. No primitive comparison/Clone or index Clone/Hash/Ord is required. Expand identity spectators too, even when identity is not itself a basis member. Zero retains its selected space as an empty sum; zero-factor states/operators use unit/scalar identity syntax.

State evaluation reuses existing action without primitive Product. Operator expansion requires Identity/Product/Adjoint. Coefficients need operation-specific Clone/Zero/multiplication/conjugation, and composite unit coefficients use One. Shared processing needs neither Float nor division, though concrete rules/bases may. Ordinary action/inner-product/trace paths gain no basis dependency.

### 7.2. Coordinate interpretation and indices

`Coordinates<'b,B,C,H=()>` borrows the basis and owns coefficients. H is () locally or a shared HilbertSpace<I> for composites. It does not borrow its source expression and remains usable after that expression is dropped while the basis lives.

Coordinates does not retain a primitive type parameter. If B implements Basis for multiple primitives, callers determine interpretation from the source type/context; expression export has that information.

- basis(), space(), factor_count(), and dim_local() expose basis/space/order interpretation.
- entries() returns unique `(&[usize],&C)` in basis order, one local basis digit per ordered factor. Local expressions have one digit; zero-factor spaces an empty sequence.
- flat_entries() yields `Result<(usize,&C),DimensionOverflow>` using rightmost-fastest mixed radix and per-entry overflow checks. The empty index is zero.
- dim() checks total coordinate dimension. Retaining/enumerating entries or expanding does not need that dimension to fit; small flat indices can remain available despite global overflow.
- len()/is_empty() describe stored entries; into_entries() moves digit sequences and coefficients out in basis order.

For `2(Up⊗Down)+3(Up⊗Down)−(Down⊗Up)`, SzBasis coordinates are 5 at `[0,1]` and −1 at `[1,0]`, with flat indices 1 and 2.

Internally, BTreeMap aggregates sparse digit keys. Remove exact zeros during aggregation and composite state scaling before later tensor products, without approximate pruning or stopping later basis rules because a coefficient is zero. LinearCombination's order/duplicate/zero-preserving contract stays unchanged. Aggregated sparse coefficients are stored, but no full basis, dense vector, or matrix is materialized.

Operator coordinates are operator-basis coefficients, distinct from action matrices. [Export](export.md) stores coordinates numerically and generates action-matrix columns by applying operators to state-basis inputs.

### 7.3. Lazy composite enumeration

`HilbertSpace::basis_states::<S,O,C,B>(&basis)` and `basis_operators::<O,C,B>(&basis)` lazily enumerate ordinary states and bound operators from one local basis, returning `Result<impl Iterator<Item=Result<Expression,BasisError>>,BasisError>`. No separate composite basis object is needed.

Generation needs Dimension/Basis, not index Eq/Clone, primitive Clone, local action/product/adjoint/identity rules, or coefficient arithmetic. Operations later comparing indices still need Eq. Increment digit sequences without an integer total count, allowing initial elements of huge spaces. Grouping does not change order; the unit space yields one unit element.

### 7.4. Errors, re-exports, and examples

BasisError distinguishes non-basis outputs (NonBasisElement), out-of-range local indices (IndexOutOfBounds), inconsistent enumeration/counts (InvalidEnumeration), dimension overflow (Dimension), and incompatible/unbound spaces (Expression). Check available information without automatically proving full independence/completeness. Zero coefficients do not hide invalid references or expansions.

Basis, BasisError, and Coordinates are re-exported at core's root. See [basis_expansion.rs](../../examples/src/bin/basis_expansion.rs) and section 6 for verification.

## 8. Graded bases

Tensor state bases add no sign and preserve space kind in enumeration/expansion/coordinates. Operator bases are factor-ordered placement products. expand/coordinates and explicit-space variants use intrinsic sorted terms, not ordinary matrices containing parity strings. Reconstruct placements bound to the original space, preserving action/coordinates under re-expansion. For second-mode annihilation c_1, intrinsic `I ⊗g a` has coefficient 1 while its ordinary state-basis matrix is `P⊗a`. Reconstructing ordinary `P_0 a_1` placements would insert P twice and is wrong for operator coordinates. basis_operators enumerates ordered placements without local Product evaluation or full matrices.

The empty basis `{1}`, rightmost-fastest order, and non-orthonormal state bases retain the same contracts. See [common evaluation bounds](composite-expression.md#112-shared-apis-and-minimal-capabilities) and [intrinsic/ordinary representations](composite-expression.md#114-intrinsic-terms-and-ordinary-representations).
