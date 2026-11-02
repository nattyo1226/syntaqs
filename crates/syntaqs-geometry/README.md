# syntaqs-geometry

Site geometries, relations, and borrowing connections to caller-owned data for [syntaqs](../../README.md).

## Responsibilities

- Describe site counts through `Geometry` and provide one-dimensional open/periodic chains through `Chain` and `Boundary`.
- Enumerate site groups through `Relation<G>`, including `NearestNeighbors` and `Consecutive<K>` for chains.
- Represent bounded-capacity groups with `SiteGroup<CAP>`.
- Connect groups to an external site-ordered slice through `SiteMapper` and `MappedGroup`.
- Combine the leaves of mapped Hilbert spaces with `leaves_cartesian`, `leaves_zip`, or a custom `combine_leaves` iterator.

`NearestNeighbors` enumerates each undirected edge once, including a single edge for a two-site periodic chain. `Consecutive<K>` preserves role order and periodic rotations, so its groups have different semantics from undirected edges.

## Integration

The caller owns site data, factor identifiers, and Hilbert spaces. `Chain` stores only a site count and boundary condition. Hamiltonian and state helpers live in the concrete algebra crates, which use these geometric relations to place core expressions.

This crate uses `no_std + alloc` and depends on [syntaqs-core](../syntaqs-core/README.md) for Hilbert-space connections. Custom geometries and relations can implement the public traits; a square-lattice implementation is shown in the examples rather than provided as a built-in geometry.

See the [crate API overview](src/lib.rs), [geometry design](../../docs/design/geometry.md), and [custom relation example](../../examples/src/bin/geometry_relations.rs). Installation is covered by the [project README](../../README.md).
