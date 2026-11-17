//! Geometric site relations and borrowing connections to caller-owned data.
//!
//! Geometry retains site counts and boundary conditions; [`relation::Relation`] selects
//! ordered groups of site numbers. [`mapping::SiteMapper`] connects those groups to
//! a caller-owned slice without cloning its elements. For Hilbert spaces,
//! [`mapping::MappedGroup::leaves_cartesian`] and [`mapping::MappedGroup::leaves_zip`] combine
//! local factors while preserving site and tensor order.
//!
//! ```
//! use syntaqs_geometry::{geometry::Boundary, geometry::chain::Chain, relation::consecutive::Consecutive, mapping::SiteMapper};
//! let chain = Chain::new(4, Boundary::Open);
//! let values = ["a", "b", "c", "d"];
//! let mapper = SiteMapper::new(&chain, &values)?;
//! let triples = mapper.groups(&Consecutive::<3>)?
//!     .map(|group| group.map(|group| group.iter().copied().collect::<Vec<_>>()))
//!     .collect::<Result<Vec<_>, _>>()?;
//! assert_eq!(triples, [["a", "b", "c"], ["b", "c", "d"]]);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! The previous root facades are not public:
//!
//! ```compile_fail
//! use syntaqs_geometry::Boundary;
//! ```
//! ```compile_fail
//! use syntaqs_geometry::Consecutive;
//! ```
//! ```compile_fail
//! use syntaqs_geometry::SiteMapper;
//! ```

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;

pub mod geometry;
pub mod mapping;
pub mod relation;

pub(crate) use geometry::{Boundary, Geometry, chain::Chain};
pub(crate) use relation::Relation;

/// Common geometry, relation and mapping types.
pub mod prelude {
    pub use crate::geometry::{Boundary, Geometry, chain::Chain};
    pub use crate::mapping::SiteMapper;
    pub use crate::relation::{
        Relation, consecutive::Consecutive, nearest_neighbor::NearestNeighbors,
    };
}
