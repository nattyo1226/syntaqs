# syntaqs-geometry

Describe sites and interactions independently of quantum states and operators.
A `Geometry` gives the site count, a `Relation` selects ordered site groups, and
a `SiteMapper` connects those groups to caller-owned data.

The built-in geometry is an open or periodic one-dimensional `Chain`. Custom
geometries and relations use the same interfaces; the
[square-lattice example](../../examples/src/bin/geometry_relations.rs) shows how.

Follow the [tagged installation guide](../../docs/installation.md) and use
release-matched documentation. The example below uses current-checkout paths.
See the [API overview](src/lib.rs), [geometry contracts](../../docs/design/geometry.md),
and [model construction](../../docs/design/models.md).


## Enumerating chain edges

This example lists the two edges of a three-site open chain:

```rust
use syntaqs_geometry::{
    geometry::{Boundary, chain::Chain},
    relation::{Relation, nearest_neighbor::NearestNeighbors},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chain = Chain::new(3, Boundary::Open);
    let edges: Vec<_> = NearestNeighbors
        .groups(&chain)?
        .map(|edge| edge.as_ref().to_vec())
        .collect();
    assert_eq!(edges, vec![vec![0, 1], vec![1, 2]]);
    Ok(())
}
```

`NearestNeighbors` yields each undirected edge once, including one edge for a
two-site periodic chain. `Consecutive<K>` instead preserves ordered roles and
periodic rotations. Choose the relation that matches the interaction's meaning.

## Connecting sites to quantum factors

`SiteMapper` borrows a site-ordered slice. For Hilbert spaces, use
`leaves_cartesian()`, `leaves_zip()`, or `combine_leaves()` to select factors
within each site. `MappedGroup` retains the original relation group;
`SiteGroup<CAP>` supplies a bounded-capacity group representation.

The caller owns identifiers, spaces, and data. `Chain` stores site count and
boundary conditions; model helpers in the concrete algebra crates turn selected
groups into expressions. Geometry uses `no_std + alloc` and depends on core for
Hilbert-space connections.

## Integration and references