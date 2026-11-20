# Basis expansion, coordinates, and enumeration

Updated: 2026-11-20

## 1. Scope and contract

`Basis<P,C>` expresses a primitive P as coefficients C in a selected complete
basis. Core extends those rules to whole expressions and tensor spaces, returning
either an expression in basis members or aggregated sparse coordinates.

Core owns basis capabilities, whole-expression expansion, composite bases, and coordinates. Concrete algebras implement local bases and primitive expansion rules; export stores the resulting coefficients in numerical containers.

A struct such as `CartesianBasis` defines a basis through `Basis<P,C>`.
It selects primitive members and defines the rule that returns their indices
and coefficients for a given primitive.

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
| `expand_indices(&self, primitive: &P)` | A primitive in the target space | `LinearCombination<usize, C>` with zero-based indices and coefficients. |
| `element(&self, index: usize)` | A zero-based index | `Some(P)` for a basis element within range, `None` outside the basis. |
| `index(&self, primitive: &P)` | A primitive | `Some(usize)` for a basis member, `None` for a nonmember. |

For `B={b_0,…,b_(n−1)}` and every target primitive p:

```math
p=\sum_{\mu=0}^{n-1}c_\mu(p)b_\mu.
```

Independence makes these coefficients unique. Expansion represents the whole object without projection/pruning. Nonmembers expand but have no index identifying themselves.

`expand(&self, primitive: &P)` returns `Result<LinearCombination<P, C>, BasisError>`.
It maps each index to an owned basis element and preserves the coefficients and
term order. A missing element returns `InvalidEnumeration`. `elements()`
lazily yields owned elements in index order by calling `element()` by default.
Indices represent coordinates in the selected basis independently of factor
identity and primitive ordering.

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
- A basis member expands to its index with coefficient 1.
- Every index returned by `expand_indices()` is in the local basis range.
- Preserve primitive linear relations, giving mathematically identical coordinates for equivalent expressions.

These laws, independence, and completeness are implementation contracts and verification obligations, not runtime automatic proofs over every possible primitive. Precision, special values, and comparisons follow [numerics](numerics.md).

## 5. Expressions, coordinates, and numerical conversion

Core extends `Basis::expand_indices()` linearly to expressions. It evaluates
local and composition rules and aggregates contributions with equal basis
indices. Both expression queries use the resulting indexed coefficients.

Choose the result according to how it will be used:

| Operation | Result and purpose |
| --- | --- |
| `expand(&basis)` | An ordinary expression in selected basis members, usable in further algebra. |
| `coordinates(&basis)` | Aggregated sparse coefficients with basis/space interpretation, suitable for numerical conversion. |
| Ordinary evaluation | Actions or scalar contractions from local rules, without selecting a basis. |

Both basis operations preserve the input. Coordinates are mathematically unique
without requiring an optimal expression normal form or a prior call to expand.
The API does not require separate single-component access or unaggregated
contribution enumeration. Section 7 defines the result interfaces and numbering.

Inner products, Gram matrices, and dual bases may implement expansion, but are not mandatory Basis capabilities. Direct rules suffice even for non-orthonormal bases, without a shared generic Gram solver or extra numeric dependency. Operator inner products use [dimension-normalized HS](primitive-expression.md#41-dimension-normalized-hilbertschmidt-inner-products), `Tr(A†B)/D`, where D is the action-space dimension.

For a complete Hermitian basis with `Tr(B_mu B_nu)=D delta_mu,nu`, coefficients are `a_mu=Tr(B_mu A)/D`. Tensor products of normalized orthonormal local operator bases stay orthonormal under total-dimension-normalized HS. Verify graded orthogonality also using ordinary action matrices with parity strings.

Ordinary evaluation/coordinates do not approximately discard components. Explicit pruning remains [future expression work](../future-work.md#22-expression-and-space-operations).

Basis processing expands only the parts required by the explicit request and
aggregates by basis index, without a complete numerical matrix intermediate.

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

The basis is an operation input, not expression metadata. Expansion transfers
local LinearCombination results into ordinary syntax in selected members, with
no basis labels, coordinate containers, expanded markers, or caches on the tree.
It preserves indices, local algebra, and definite state/ambient spaces. Original
sources are not automatically retained.

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

All return `Result<_,BasisError>`, applying the same local basis to all factors after evaluating local rules. States use their own space. Operators prefer explicit space, then binding. Composite operator queries require `I: Ord` and share a factor position lookup across membership checks, graded ordering, and coordinate placement. They use primitive `PartialEq` to collect terms and coefficient `PartialEq` to match common blocks. The selected basis applies to identity spectators too. Zero retains its selected space as an empty sum. Zero-factor states and operators use unit and scalar identity syntax respectively.

State evaluation collects natural-state terms using PartialEq before basis
expansion, without primitive Product. Local state queries require coefficient
One. Composite state queries additionally require `I: Ord` and coefficient
`PartialEq`. Operator expansion requires Identity/Product/Adjoint. Coefficients
need operation-specific Clone/Zero/multiplication/conjugation, and composite unit
coefficients use One. Shared processing needs neither Float nor division, though
concrete rules and bases may. Ordinary action, inner product, and trace operate
independently of a basis.

Composite operator queries first validate the original expression, then use
[operator block evaluation](composite-expression.md#117-operator-block-evaluation)
to resolve products, adjoints, graded signs, and exact cancellation. Operator
generation shares spectator identity expansion and emits complete contributions
for the surviving terms.

### 7.2. Coordinate interpretation and indices

`Coordinates<'b,B,C,H=()>` borrows the basis and owns coefficients. H is () locally or a shared HilbertSpace<I> for composites. It does not borrow its source expression and remains usable after that expression is dropped while the basis lives.

Coordinates does not retain a primitive type parameter. If B implements Basis for multiple primitives, callers determine interpretation from the source type/context; expression export has that information.

- basis(), space(), factor_count(), and dim_local() expose basis/space/order interpretation.
- entries() returns unique `(&[usize],&C)` in basis order, one local basis digit per ordered factor. Local expressions have one digit; zero-factor spaces an empty sequence.
- flat_entries() yields `Result<(usize,&C),DimensionOverflow>` using rightmost-fastest mixed radix and per-entry overflow checks. The empty index is zero.
- dim() checks total coordinate dimension. Retaining/enumerating entries or expanding does not need that dimension to fit; small flat indices can remain available despite global overflow.
- len()/is_empty() describe stored entries; into_entries() moves digit sequences and coefficients out in basis order.

For `2(Up⊗Down)+3(Up⊗Down)−(Down⊗Up)`, SzBasis coordinates are 5 at `[0,1]` and −1 at `[1,0]`, with flat indices 1 and 2.

Internally, `BTreeMap` aggregates sparse digit keys. Exact zeros are removed
during aggregation and composite state scaling. Every index returned by a
reached basis rule is checked against the local dimension, including indices
with zero coefficients. `LinearCombination` preserves order, duplicates, and
zero coefficients. Coordinates store the aggregated sparse coefficients.

#### State coordinate generation

A composite state query reuses validated basis conversions for equal local
primitives. Each tensor child contributes a choice of an entire digit tuple,
preserving correlations inside that block. A reusable key holds the selected
tuples in factor order. The tensor traversal writes completed coordinates into
the result map. Sums aggregate overlapping coordinates before their maps are
used as tensor children.

#### Operator coordinate generation

The generator receives specified primitives in ambient position order with
graded signs resolved. It reuses the indexed conversion of each specified
primitive and expands identity at the first spectator gap in the query.
A local product evaluated to the natural identity uses the same spectator
expansion.

For a single-component identity `I = gamma B_q`, a reusable working key starts
with digit q at every factor. Only specified positions are overwritten, then
restored for the next term. Coefficient multiplication follows ambient order,
including gamma at each spectator when gamma differs from one. Multiplying
gamma in sequence avoids the additional overflow or underflow that can arise
from computing powers first.
For a multi-component identity, all spectator
axes borrow the same choices and an iterative traversal enumerates required
combinations. Single-choice products use a direct path; general enumeration uses
an explicit work stack. No spectator tensor map or factor-by-factor coordinate
prefix maps are constructed.

Every complete contribution enters one final sparse accumulator. Full-length
keys and genuinely large basis output retain their storage and enumeration
costs.

For each surviving term, validate the specified factors and the first needed
spectator identity in ambient order. An empty basis conversion at one factor
still leaves the other conversions subject to validation.

Operator coordinates are operator-basis coefficients, distinct from action matrices. [Export](export.md) stores coordinates numerically and generates action-matrix columns by applying operators to state-basis inputs.

#### Reconstructing expanded expressions

`expand()` resolves only the basis indices that remain after aggregation. It
shares a local basis-expression node for each index and a placed node for each
pair of factor position and index. Each term uses those shared nodes in a
state tensor or ordered operator product on the selected space.

### 7.3. Lazy composite enumeration

`HilbertSpace::basis_states::<S,O,C,B>(&basis)` and `basis_operators::<O,C,B>(&basis)` lazily enumerate ordinary states and bound operators from one local basis, returning `Result<impl Iterator<Item=Result<Expression,BasisError>>,BasisError>`. No separate composite basis object is needed.

Generation needs Dimension/Basis, not index Eq/Clone, primitive Clone, local action/product/adjoint/identity rules, or coefficient arithmetic. Operations later comparing indices still need Eq. Increment digit sequences without an integer total count, allowing initial elements of huge spaces. Grouping does not change order; the unit space yields one unit element.

### 7.4. Errors and examples

`BasisError` distinguishes missing membership (`NonBasisElement`), out-of-range
indices (`IndexOutOfBounds`), missing elements or inconsistent ordering and
counts (`InvalidEnumeration`), dimension overflow (`Dimension`), and invalid
expression spaces (`Expression`). Core validates placements before applying
basis rules to terms that survive evaluation and collection. `coordinates()`
checks the local index bounds. Expression expansion and lazy composite
enumeration also check the retrieved element against `index()` when they
materialize it.

`Basis`, `BasisError`, and `Coordinates` are defined in `syntaqs_core::basis`; `Basis` is also available in the core prelude. See [basis_expansion.rs](../../examples/src/bin/basis_expansion.rs) and section 6 for verification.

## 8. Graded bases

Tensor state bases add no sign and preserve space kind in enumeration/expansion/coordinates. Operator bases are factor-ordered placement products. expand/coordinates and explicit-space variants use intrinsic sorted terms, not ordinary matrices containing parity strings. Reconstruct placements bound to the original space, preserving action/coordinates under re-expansion. For second-mode annihilation c_1, intrinsic `I ⊗g a` has coefficient 1 while its ordinary state-basis matrix is `P⊗a`. Reconstructing ordinary `P_0 a_1` placements would insert P twice and is wrong for operator coordinates. basis_operators enumerates ordered placements without local Product evaluation or full matrices.

The empty basis `{1}`, rightmost-fastest order, and non-orthonormal state bases retain the same contracts. See [common evaluation bounds](composite-expression.md#42-shared-apis-and-minimal-capabilities) and [intrinsic/ordinary representations](composite-expression.md#111-intrinsic-terms-and-ordinary-representations).
