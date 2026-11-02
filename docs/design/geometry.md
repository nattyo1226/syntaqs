# Geometry, site relations, and borrowing connections

Updated: 2026-11-02

## 1. Scope and responsibilities

`syntaqs-geometry` separates geometry, site-selection rules, and borrowing connections to external sequences. Public types are available at the crate root. Core does not depend on geometry; geometry depends on core for HilbertSpace connections, maintaining `no_std + alloc`.

| Type / trait | Responsibility |
| --- | --- |
| `Geometry` | Minimal common `num_sites()` contract. |
| `Chain` | Immutable site count and boundary. |
| `Relation<G>` | Enumerate groups of site numbers from adjacency, directions, boundaries, or other rules on G. |
| `SiteGroup<CAP>` | Fixed-capacity array with a used length; AsRef exposes only used entries. |
| `SiteMapper<G,T>` | Borrow geometry and site-ordered `&[T]`, validating their connection. |
| `MappedGroup<T,Group>` | Retain the original group and external sequence, enumerating `&T` in group order. |

`Chain::new(num_sites, boundary)` returns Self without validation errors, including zero sites. num_sites() and boundary() expose its data. It neither retains nor issues indices, Hilbert spaces, local algebras, or coefficients. Sites are positions in `0..num_sites()` without SiteId or ownership tokens.

Multiple relations may apply to one geometry, and a rule may support multiple geometries. The only common group bound is `AsRef<[usize]>`, allowing arrays, variable-length groups, stored slices, and custom center/direction metadata. Each relation defines order/repetition semantics; SiteGroup does not sort or deduplicate.

CAP is storage capacity; used length is the actual site count. new rejects overflow as `SiteGroupError::CapacityExceeded { capacity, len }` without exposing unused slots. Groups need no heap allocation, but storage/move cost scales with capacity.

Standard implementations provide the common traits/connections and chain nearest-neighbor/consecutive selections. A square lattice exists only in [geometry_relations.rs](../../examples/src/bin/geometry_relations.rs); arbitrary support-set combinations are not standard relations. See [models](models.md) for model construction.

## 2. Relation enumeration and borrowing

`Relation<G>::groups(&geometry)` returns its associated Error or a lazy group iterator. Validate settings before returning the iterator without universally requiring Debug, Display, or Error on that error type. GATs permit borrowing stored data from rules/geometries, but ordinary iterators cannot lend a reusable mutable generation buffer.

| Chain relation | Enumeration |
| --- | --- |
| `NearestNeighbors` | Unique undirected `[i,j]` with i<j, lexicographically ordered. No edges for zero/one site; one edge for two sites with either boundary. |
| `Consecutive<K>` | Ordered K-site placements preserving roles; open starts `0..=N-K`, periodic starts `0..N` and wraps. |

`Consecutive<0>` returns `ConsecutiveError::ZeroLength`; K>N yields no placements, K=1 yields every site once. Do not sort a placement or merge rotations sharing a support. With two periodic sites, `Consecutive<2>` returns `[0,1]` and `[1,0]`, unlike the single nearest-neighbor edge. NearestNeighbors uses Infallible on Chain.

N=0 with K≥1 yields nothing, and K>N never repeats sites to fill a placement. Avoid arithmetic overflow and modulo zero. For N=5, K=3:

```text
open:     [0,1,2], [1,2,3], [2,3,4]
periodic: [0,1,2], [1,2,3], [2,3,4], [3,4,0], [4,0,1]
```

Positions correspond to local-operator roles. Do not merge equal rotated supports, since `X_i Z_j X_k` has a distinct middle role and graded products may change signs. Models decide duplication based on interaction symmetries.

```rust,ignore
use syntaqs_geometry::{Boundary, Chain, Consecutive, NearestNeighbors, SiteMapper};
let chain = Chain::new(4, Boundary::Open);
let mapper = SiteMapper::new(&chain, &site_spaces)?;
for group in mapper.groups(&Consecutive::<3>)? {
    let group = group?;
    for leaves in group.leaves_cartesian() {
        // Select one leaf per Site and use existing core placement and products.
    }
}
for group in mapper.groups(&NearestNeighbors)? {
    let group = group?;
    // Apply another Relation to the same geometry and external sequence.
}
```

SiteMapper::new checks exact sequence length before applying relations, even for empty enumeration. Mismatch is `MappingError::SiteCountMismatch { expected, actual }`. The outer groups() Result has `R::Error`, while each mapped group is `Result<MappedGroup<...>, MappingError>`. Check site ranges when a group is consumed, returning `SiteOutOfRange { site, num_sites }`; do not eagerly collect all groups or roll back earlier results.

Geometry site counts/meanings must remain stable while borrowed. Equal lengths cannot verify that caller-intended coordinates match sequence order.

MappedGroup iter() borrows in group order, get(position) uses group position, and source() retains the typed source group, including center/direction information accessible through custom extensions. Connection needs no element/index Clone or Eq; spaces/models validate duplicate indices.

## 3. Spaces and local factors

Each element of `&[HilbertSpace<I>]` is one site, whether a leaf, multiple leaves, nesting, singleton tensor, or empty space. Callers can share existing handles without index cloning or reissuance.

SiteMapper::from_space reads the stored top-level partition: a leaf is one site, an empty tensor zero sites, `Tensor([unit])` one empty site, and a singleton tensor one site. Site count does not come from total leaves; local grouping remains intact.

MappedGroup for HilbertSpace provides:

| Method | Result for `[A⊗B,C⊗D]` |
| --- | --- |
| `leaves_cartesian()` | `[A,C], [A,D], [B,C], [B,D]` |
| `leaves_zip()` | `[A,C], [B,D]` |
| `combine_leaves(FnOnce)` | Custom combination of each site's `Vec<Leaves>` into an iterator. |

Use the same API for arbitrary group arity without per-arity tuple traits. The standard methods use combine_leaves. Cartesian order varies the first site slowest and caches read leaf references for replay without eagerly generating results or cloning Leaves/indices. Zip stops at the shortest stream; both preserve within-site tensor order.

combine_leaves passes `Vec<syntaqs_core::space::Leaves<'s,I>>` once to FnOnce and returns its iterator unchanged. Item types are unrestricted, without a selection trait. Fully consuming a Cartesian product caches references proportional to the sum of input leaf counts.

An empty site produces no rows without closing gaps. A zero-site group yields one empty row for both methods. One site yields its leaves individually. Custom combiners own their ordering/mapping contract.

Site order and an independently existing ambient factor order are distinct. Connections neither reorder spaces/states/matrices nor construct tensors. Callers select ambient spaces, axes, and normalization; core handles placement, multiplication, and binding. Selecting one leaf per site differs from enumerating all Pauli assignments across leaves.

Existing `ExpressionFactory::operator_product_at` and `try_operator_product` accept arbitrary-length placements/products. No per-arity construction API or geometry-specific evaluator is added, and enumeration/connections require no evaluation capabilities.

[pauli_strings.rs](../../examples/src/bin/pauli_strings.rs) demonstrates all supports up to K sites with one spin per site: ordinary iterators select each size-r support for `r=0..=min(K,N)` and assign `3^r` X/Y/Z combinations. The total is `Σ(r=0..min(K,N)) binomial(N,r) × 3^r`, giving 37 terms including identity on three sites up to two-site support. Empty support is one ambient identity, unlike Consecutive<0>. Sx/Sy/Sz are directly Pauli matrices, requiring no extra scale before binding. Construction needs only `I: Eq`, `Group: AsRef<[usize]>`, and iteration over the group collection.

K counts sites, not acting leaves. A site containing m spins has `4^m−1` nonidentity Pauli strings. Exhaustive multi-leaf assignments use caller iterators/custom combiners rather than additional standard geometry APIs; this is not included in the current example.

## 4. Verification

Test zero/one/two/larger open/periodic chains, length mismatches, non-Clone/non-Eq elements, borrowing identity, nested/empty/variable-length spaces, Cartesian/zip order, top-level partitions and new/from_space agreement, three-/four-site placements, custom errors/role metadata, and lazy range checks.

Numerical model conventions and verification belong in [models](models.md).

### 4.1. Tests and examples

| Coverage | Test / example |
| --- | --- |
| Neighbor/many-site enumeration, capacities, borrowed stored groups, custom errors, lazy ranges | [geometry.rs](../../crates/syntaqs-geometry/tests/geometry.rs) |
| Wrapping arithmetic at maximum site count | [consecutive.rs unit tests](../../crates/syntaqs-geometry/src/relation/consecutive.rs) |
| Nested/multiple leaves, variable groups, empty/exhausted streams, borrowing, FnOnce, top-level interpretation | [space.rs](../../crates/syntaqs-geometry/tests/space.rs) |
| Three-/four-site Pauli placements, 37 strings, periodic fermion products versus manual/independent matrices | [geometry_relations.rs](../../examples/tests/geometry_relations.rs) |
| Multiple relations/geometries, directional groups, center metadata, variable neighborhoods, minimal bounds | [geometry_relations example](../../examples/src/bin/geometry_relations.rs) |
| Support iterators, Pauli normalization, independent matrices | [pauli_strings example](../../examples/src/bin/pauli_strings.rs) |

Examples are workspace binaries; see their [README](../../examples/README.md) for commands and features.
