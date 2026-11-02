# Recursive Hilbert spaces

Updated: 2026-11-02

## 1. Scope and contract

This document defines Hilbert-space construction, grouping, and compatibility under [requirements](../requirements.md) and [architecture](../architecture.md).

Core owns generic identity comparison, Hilbert spaces, expressions, and evaluation. Basic issues concrete UUID indices; geometry handles site assignments. Space construction does not require Geometry. Section 9 lists public APIs.

A space recursively represents a basic factor or an ordered tensor product of child spaces, including nested products. Grouping is the tree's parenthesization. Operations include construction/composition, child/index enumeration, compatibility, and subsystem checks. Arbitrary factor-selection extraction and regrouping are not provided, nor a separate grouping type. Factor/Tensor are concepts for private nodes; HilbertSpace is public.

Spaces store no local algebra types. S/O/C on expressions enforce appropriate homogeneous composition; sharing space structure does not enable implicit mixing of primitive types.

## 2. Factors and tensors

`HilbertSpace` is the single public space type. A degree of freedom is its indexed Factor leaf, without a separate Dof object or conversion to a one-factor space. Retain/share leaves for tensor inputs and placement. Local placement requires a basic factor and does not infer one from a tensor. See the public API section for constructors and accessors.

| Object | Information and role |
| --- | --- |
| Factor | An indexed leaf; local dimensions come from query inputs. |
| Tensor | Ordinary/graded kind and ordered child spaces, which may be factors or tensors. |

```text
Hilbert space
├─ Factor(Index)
└─ Tensor(tensor kind, ordered child Hilbert spaces)
```

`dim::<P>()` applies P's span dimension to each leaf; `dim_with(dim_local)` supplies it explicitly. Tensor dimensions multiply, and an empty tensor has dimension one.

Total dimension need not fit usize for construction. Dimension queries use checked arithmetic and return `Result<usize, DimensionOverflow>`. Compatibility/subsystems and matrix-free actions, inner products, and traces do not require an integer total dimension.

```math
\dim \operatorname{Tensor}(k;\mathcal H_1,\ldots,\mathcal H_n)
=\prod_{j=1}^{n}\dim\mathcal H_j,
\qquad
\dim\mathcal H_{\varnothing}=1.
```

Bases are chosen separately. Building trees or enumerating factors does not expand all bases, expressions, or matrices.

## 3. Indices and factor order

Only factors carry degree-of-freedom indices. Tensor construction issues no extra identity. Supply previously issued indices under the [index contract](index.md); construction, cloning, and reuse do not issue them implicitly.

Factor order is the left-to-right leaf order, available as an ordered index sequence, independent of index value ordering. Binary tensor products place left-operand leaves before right-operand leaves.

Duplicate checks cover every basic leaf, including combining a parent with its internal factor or using a shared subspace twice. Retaining multiple descriptions of an existing factor is distinct from adding it independently twice.

## 4. Stored trees and compatibility

Retain caller-selected grouping without mandatory flattening at construction. Traverse factors when needed under these identifications:

- Empty tensors of either kind identify with the same one-dimensional unit space; its state 1 is even in graded composition.
- A singleton tensor identifies with its child.
- Products of the same kind are associative for compatibility; changing parentheses alone has no effect.
- An indexed one-dimensional factor remains distinct from an empty tensor; dimension alone does not remove identity.

Spaces are compatible when their basic indices, order, and effective tensor structures match under those rules. Expressions hold primitive types/rules; space comparison has no definition label. Grouping alone does not affect compatibility, and neither object identity nor dimension alone establishes it.

Reject a single space whose ordinary/graded mixture remains after unit/singleton identification. Compatibility does not change kinds. There is no public regrouping or factor permutation/reordering.

These rules concern mathematical compatibility, not mandatory deletion of stored/displayed grouping. Numeric precision, scalars, and grading conditions are also checked separately under [numeric conventions](numerics.md).

## 5. Grouping, geometry, and local composite algebras

Callers represent a site's multiple degrees of freedom as internal tensor factors. Core manages trees; Chain stores only site count and boundary. Callers manage site-ordered spaces, ambient spaces, and grouping without internal labels or mandatory geometry.

```text
Global space: Tensor
├─ Tensor                         ← Mapped to site 0 by the placement layer
│  ├─ Factor(Index u0, mode)      ← Factor 0 within the Site
│  └─ Factor(Index d0, mode)      ← Factor 1 within the Site
└─ Tensor                         ← Mapped to site 1 by the placement layer
   ├─ Factor(Index u1, mode)      ← Factor 0 within the Site
   └─ Factor(Index d1, mode)      ← Factor 1 within the Site
```

In this spinful example, multi-factor tensors are graded and leaf order is `u0, d0, u1, d1`. Placement/model construction maps site and internal order explicitly to that sequence; core uses the supplied child order.

Local composite algebras use the same recursive space structure. Shared composition/evaluation combines child products, actions, inner products, adjoints, traces, and grading under [local-expression responsibilities](primitive-expression.md). Grouping does not require new basic primitives or rules.

A composite space carries the conceptual responsibility for consistent composition from child structures, factor order, and product kind. Shared machinery preserves local contracts under associativity and unit/singleton identification without a separate composite-structure type. See [composite expressions](composite-expression.md) for capability and embedding contracts.

## 6. Subsystem checks

Subsystem here means a space made of tensor factors, not a general linear subspace or state embedding map.

For valid K and H, K is a subsystem of H when:

1. Every basic index of K exists in H.
2. K's factor sequence is a subsequence of H's, preserving relative order without permutation.
3. Effective product kinds agree after unit/singleton identification. A multi-factor K must have H's kind; empty/single-factor cases follow section 4.

K need not be a stored subtree or a contiguous factor range. Callers may construct K from retained factors. H is its own subsystem; the empty tensor is a subsystem of every space. An indexed one-dimensional factor requires its matching index.

Precision compatibility is separate and checked at operation boundaries under [numerics](numerics.md). Subsystem checks compare two supplied structures without extracting or regrouping spaces.

## 7. Enumeration and immutable sharing

Callers retain constructed leaf or tensor spaces for later action targets or prospective partial-trace selections. Three lazy enumerations inspect existing structure:

| Capability | Enumerates | Order and contract |
| --- | --- | --- |
| Tensor children | Immediate child spaces | Construction order, retaining nested grouping. |
| Leaves | Actual factor handles inside nesting | Borrowed in factor order through `leaves()`; a leaf yields itself, an empty tensor yields nothing. |
| Indices | Every factor index in the nested tree | Left-to-right factor order; one for a factor, none for an empty tensor. |

Iterators primarily borrow: `children()` reads direct children, `leaves()` all factor handles, and `indices()` all indices. Factors and empty tensors have no direct children. No eager collection is required. Enumeration reads stored grouping regardless of mathematical identifications, without modifying spaces, indices, local structure, or grouping and without expression/basis/matrix expansion. Concrete signatures belong to the implementation API.

This inspects existing structure rather than extracting a new subsystem. Callers borrow site-ordered spaces into geometry; connection checks lengths, while models/spaces check all leaf duplicates. See [geometry](geometry.md).

Embedding checks the specified index against ambient basic factors. No special object records subtree origins/positions. Core may traverse factors for validation/evaluation under [composite-expression](composite-expression.md) contracts.

HilbertSpace shares immutable nodes through Arc with private Factor/Tensor enums. Cloning preserves indices and avoids deep copies. Traversal and final-owner destruction use explicit work stacks rather than depth-proportional recursion. Ordinary compatibility compares leaf sequences; subsystems use ordered subsequences without changing stored trees.

## 8. Examples and verification

Let a, b, c be distinct compatible factors of one local structure; `T_o` and `T_g` denote ordinary/graded tensors, and E an empty tensor. Graded examples describe implemented behavior, not heterogeneous local algebras.

| Object / operation | Result |
| --- | --- |
| Compare `T_o(T_o(a,b),c)` and `T_o(a,T_o(b,c))` | Compatible: same factors, order, and kind. |
| Compare `T_o(a,b)` and `T_o(b,a)` | Incompatible factor order. |
| Construct `T_o(T_o(a,b),a)` | Reject duplicate leaf index. |
| Combine `T_o(a,b)` with itself | Reject duplicate basic factors. |
| Compare `T_o()` and `T_g()` | Compatible unit spaces. |
| Compare `T_o(E,a)`, `T_g(a)`, and a | Compatible after unit/singleton identification. |
| Construct `T_o(a,T_g(b,c))` | Reject remaining kind mixture. |
| Compare an indexed one-dimensional factor and E | Incompatible: the factor's index remains. |
| Reuse a constructed factor in placement/composition | Retain its index and grouping. |

For subsystem examples, take `H = T_o(T_o(a,b),c)`:

| Candidate K | Subsystem of H? |
| --- | --- |
| `T_o(a,b)` | Yes. |
| `T_o(a,c)` | Yes: preserves order without being a subtree. |
| `T_o(c,a)` | No: different relative order. |
| H | Yes. |
| Empty tensor E | Yes. |
| Factor with an index absent from H | No. |
| `T_g(a,b)` | No: incompatible multi-factor kind. |
| `T_g(a)` | Yes: identifies with a. |

Only spaces identified with an empty tensor are subsystems of an empty tensor. Even a one-dimensional indexed factor is not, since its index is absent.

Hierarchical evaluation and basic-factor evaluation must agree for the same order and kind. Enumerating factors must not generate full bases, state expressions, or matrices.

H's direct children are `[T_o(a,b),c]` and indices `[Index(a),Index(b),Index(c)]`. A singleton retains one child; an empty tensor has no children/indices. Verify that enumeration preserves identities, order, and grouping.

## 9. Public APIs

| API | Result and constraints |
| --- | --- |
| `HilbertSpace::factor(index)`, `unit()` | Construct an existing-index factor or empty tensor; no index Clone/Default. |
| `tensor_all_with(kind, children)`, `tensor_all(children)`, `tensor_all_g(children)` | Explicit, ordinary, or graded construction; Eq checks duplicates/kinds; returns `Result<Self, SpaceError>`. |
| `tensor_with(kind, &rhs)`, `tensor(&rhs)`, `tensor_g(&rhs)` | Corresponding binary forms, left factors before right. |
| `kind()`, `effective_kind()` | Stored tensor kind and effective `Option<TensorKind>` after unit/singleton identification. |
| `factor_index()`, `children()`, `indices()` | Borrow a factor's `Option<&I>`, immediate children, or all leaf indices. |
| `is_compatible_with(&rhs)`, `is_subsystem_of(&ambient)` | Eq-based factor/relative order and effective-kind checks. |
| `dim::<P>()` | Apply `P: Dimension` per factor; return `Result<usize, DimensionOverflow>`. |
| `dim_with(dim_local)` | Supply `NonZeroUsize` explicitly; return the same Result. |
| `clone()`, `shares_node_with(&rhs)` | Clone/compare shared handles without cloning indices. |

`dim::<S>()` queries state space and `dim::<O>()` operator space. Operator stores no S, so provide a local dimension through dim_with when its action-space dimension d is needed. Empty tensors have dimension one even if P's local dimension overflows. Comparison, placement, and binding register no primitive definitions; different primitive-typed expressions can use the same space structure.

Specify `TensorKind::{Ordinary, Graded}` at construction. tensor_all/tensor are ordinary and tensor_all_g/tensor_g graded conveniences over explicit-kind constructors. Kinds are not inferred from children/primitives or changed on existing spaces. kind() is None for a factor and the stored kind for tensors including empty/singleton wrappers. unit() stores an ordinary empty tensor and is compatible with `tensor_all_g([])`.

### 9.1. Effective kinds and mixture validation

Handles retain only a structural leaf-count class (zero, one, multiple) and effective kind, not local dimensions, full bases, or evaluation caches. Let m count nonempty children:

1. m=0 is the unit with effective kind None.
2. m=1 inherits its sole nonempty child's effective kind; the outer kind does not affect compatibility.
3. m≥2 uses the parent kind, which must match every child with an effective kind.

Thus `T_o(E,T_g(a,b))` is allowed while `T_o(a,T_g(b,c))` returns `SpaceError::MixedTensorKinds`. Zero/one-leaf spaces have no effective kind and both rules agree. Multi-factor compatibility, subsystem checks, and evaluation use effective_kind().

Retain trees and child grouping. Singleton wrappers inherit child metadata in constant time without rescanning descendants; duplicate checks add no Hash/Ord/Clone beyond Eq. Traversal and destruction remain iterative.
