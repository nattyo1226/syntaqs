# Index identity, issuance, and sharing

Updated: 2026-11-17

## 1. Scope and contract

An index identifies a Hilbert-space factor across spaces and placed expressions.
The caller supplies it before constructing the factor. Geometric sites are
positions, and basis-coordinate indices are numerical coordinates; neither is a
factor identity.

Core accepts a generic index type I and validates identity, placement, and space
consistency. Basic supplies UUID `Id128` and string `Tag` types. Geometry borrows
caller-owned spaces; Chain has neither an index type nor issuance helpers.

The core's semantic contract is stable identity comparison as an equivalence relation, requiring standard Eq where comparison occurs. UUID representation/generation, randomness, seeds, and ordering are outside core's contract. Hash, Ord, and issuance capabilities are not universal requirements. Space/expression handles share indices without requiring Clone on I itself.

## 2. Indices, local algebras, factors, and spaces

Spaces determine factor order and grouping; state/operator primitive types S/O
determine local algebra and dimensions; each operation selects its basis.
An index contributes only identity. Id128 displays its UUID and Tag displays its
caller-supplied string.

| Object | Stored information and role |
| --- | --- |
| Index | Factor identity. |
| Placed local state/operator | Primitive types S/O and an index identifying its factor. |
| Basic factor (Factor) | An indexed Hilbert-space leaf, without a separate Dof type. |
| Hilbert space | Basic indices, factor order, and ordinary/graded tensor structure. |

Construct a degree of freedom as an indexed HilbertSpace leaf. Tensor nodes
combine existing spaces without changing or issuing indices, and duplicate
checks cover all leaves. Callers retain spaces for later subsystem checks;
subsystem extraction is not provided. See [Hilbert spaces](hilbert-space.md).

Different indices identify independent factors. Indices do not define local algebra; S/O and capability implementations do. Space comparisons inspect indices and order, while primitive/coefficient types reject inappropriate expression combinations. See [primitive type contracts](local-structure.md#4-conditions-for-matching-local-structures).

One system uses the same S/O types at all factors. Separate systems may choose
different local algebras; the consistency rules in section 6 explain that boundary.

## 3. Issuance, construction, and placement

Issue indices independently without specifying local structure, then supply existing indices to factor construction.

```text
Issue an Index
    → Construct a HilbertSpace leaf with that Index
    → Use the leaf to compose spaces or place states and operators
```

Factor construction retains the supplied index. No constructor implicitly issues one from local structure alone.

Explicit placement assigns an existing index to a local state/operator. The expression retains S/O instead of obtaining them from I. An unplaced expression on one local factor receives one index throughout. Multi-factor expressions place local pieces before composition.

Placement is distinct from binding to an ambient Hilbert space including spectators. Placement does not determine that ambient space, and binding alone does not infer each local expression's factor. Neither factor construction nor placement implicitly issues indices.

Callers issue indices; Chain neither issues nor stores them. Id128::default() issues a fresh index, whereas Clone/Copy duplicates the same identity.

Callers pass site-ordered spaces by borrow to geometry and model construction. Space sharing and leaf enumeration need no index Clone. Model/space construction checks all leaf duplicates with Eq; see [geometry](geometry.md).

## 4. Uniqueness assumptions

Index providers must assign distinct identities to independent factors within
and across runtimes. Id128 uses seed-free UUID generation, providing practical
uniqueness rather than a mathematical collision guarantee.

Tag uses the whole string as identity, so independently constructed equal names
identify the same factor. Callers manage distinct names, including across
runtimes. Core compares identities, not generation methods or naming policies.

No global counter or registry of all issued indices is required. Standard Id128 uses UUID v4. UUID/randomness dependencies stay outside core, preserving its `no_std + alloc` support.

The approach follows UUIDs' practical uniqueness without central registration; see [RFC 9562 §6.8](https://www.rfc-editor.org/rfc/rfc9562.html#section-6.8).

## 5. Reuse and shared references

Reusing a factor retains its index. Cloning handles, reusing constructed factors, and placing expressions do not issue fresh identities. HilbertSpace and expression handles provide immutable sharing.

| Operation | Index handling |
| --- | --- |
| Create an independent factor | Issue an index, then construct the factor. |
| Clone an existing factor handle | Retain identity. |
| Reuse an existing factor | Retain identity. |
| Place an expression on a factor | Use that factor's index. |

Intentional reuse differs from accidental issuance collisions. Adding equal-index factors independently to one tensor product is a duplicate and is rejected. Multiple references to the same factor are allowed.

## 6. Local algebra correspondence and consistency

No global registry fixes index-to-algebra associations. Construction is not rejected by consulting whether an index was previously used with another structure.

Expressions determine local rules through S/O and their capabilities. Spaces
retain indices, order, and tensor structure. This separation lets the same space
structure be used independently by different primitive-typed systems.

Composition, placement, and binding check:

- Space compatibility/subsystems through indices, order, unit/singleton identifications, and associativity of the same product kind.
- Duplicate basic indices in space/state tensor products.
- Existence of expression references, with S/O/C and evaluation capabilities ensuring appropriate state/operator combinations.
- Operator products on the same factor remain allowed.

Spin-1/2 and custom-algebra expressions may independently use the same factor structure; they cannot thereby combine their different primitive types into one expression.

Validate as early as available information permits. Ambient-space/evaluation-dependent checks occur when that context is available, without global correspondence management.

## 7. Standard types and public APIs

Core provides generic identity comparison, basic provides Id128/Tag, and shared leaf/expression handles avoid unnecessary copies of I.

Comparison uses Eq, factor construction uses `HilbertSpace::factor(index)`, and placement uses `at(&factor)`. Opaque `syntaqs_basic::index::Id128` wraps UUID v4; `new()` and Default issue fresh identities without seeds. Clone/Copy preserve identity. Display, Debug, equality, ordering, and Hash are provided. Generation delegates to `uuid::Uuid::new_v4`, including behavior when randomness is unavailable. Chain does not depend on basic.

Use Id128 from `syntaqs-basic` and `syntaqs_core::space::HilbertSpace<Id128>` for a UUID-indexed space.

`syntaqs_basic::index::Tag { raw: String }` is a named index with private raw storage and `new(impl Into<String>)`, `as_str()`, `From<String>`, and `From<&str>`. Clone, Debug, Eq, Hash, and Ord follow the string. It has no Copy, hidden UUID, or rename API. Use `syntaqs_core::space::HilbertSpace<Tag>`.

Standalone Id128 Display/Debug retains the complete UUID; Tag Display writes the name unchanged. Expression Display alone limits any index to its first eight Rust chars. Debug retains full indices. Display collisions are allowed and never determine identity. Names are not inferred from tensor ordering and require no position lookup. See [formatting](expression-formatting.md).
