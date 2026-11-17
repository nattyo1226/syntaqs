//! Basis-aware exports into owned vectors, ndarray arrays and sprs containers.
//!
//! Dense data always has a public `Vec` export path. Optional `ndarray` exports
//! reuse that path; optional `sprs` exports use sparse coordinates directly.
//! Results retain the basis and selected factor space needed to interpret them.
//!
//! ```
//! use num_complex::Complex;
//! use syntaqs_core::expression::local::LocalState;
//! use syntaqs_export::{Export, Exported};
//! use syntaqs_spin_half::{state::SpinHalfState as S, operator::SpinHalfOperator as O, basis::SzBasis};
//! let psi = LocalState::<S,O,Complex<f64>>::primitive(S::Down);
//! let output: Exported<Vec<Complex<f64>>, _> = (&psi).export(&SzBasis)?;
//! assert_eq!(output.metadata().shape(), [2]);
//! assert_eq!(output.data().as_slice(), &[Complex::new(0.,0.),Complex::new(1.,0.)]);
//! # #[cfg(feature="ndarray")] {
//! let array: Exported<ndarray::Array1<_>, _> = output.export(())?;
//! assert_eq!(array.data().len(),2);
//! # }
//! # Ok::<(), syntaqs_export::ExportError>(())
//! ```
//!
//! Representation-specific items and external container crates are not
//! re-exported at the crate root:
//!
//! ```compile_fail
//! use syntaqs_export::MatrixArgs;
//! ```
//! ```compile_fail
//! use syntaqs_export::MatrixMetadata;
//! ```
//! ```compile_fail
//! use syntaqs_export::VectorMetadata;
//! ```
//! ```compile_fail
//! use syntaqs_export::Layout;
//! ```
//! ```compile_fail
//! use syntaqs_export::ndarray;
//! ```
//! ```compile_fail
//! use syntaqs_export::sprs;
//! ```
//!
//! Coefficient precision cannot change implicitly during export:
//!
//! ```compile_fail
//! use num_complex::Complex;
//! use syntaqs_core::expression::local::LocalState;
//! use syntaqs_export::{Export, Exported};
//! use syntaqs_spin_half::{state::SpinHalfState as S, operator::SpinHalfOperator as O, basis::SzBasis};
//! let psi = LocalState::<S,O,Complex<f32>>::primitive(S::Up);
//! let output: Exported<Vec<Complex<f64>>, _> = (&psi).export(&SzBasis).unwrap();
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod backend;
mod coordinates;
pub mod matrix;
pub mod vector;

use matrix::{args::MatrixArgs, metadata::MatrixMetadata};
use vector::metadata::VectorMetadata;

use std::collections::TryReserveError;
use syntaqs_core::{
    basis::BasisError, expression::error::ExpressionError, space::DimensionOverflow,
    space::SpaceError,
};

/// Failure to construct or interpret a numerical export.
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    /// Invalid basis expansion or enumeration.
    #[error("{0}")]
    Basis(#[from] BasisError),
    /// Incompatible, unbound, or invalid expression space.
    #[error("{0}")]
    Expression(#[from] ExpressionError),
    /// Invalid tensor factor space.
    #[error("{0}")]
    Space(#[from] SpaceError),
    /// An axis dimension or flat coordinate overflows usize.
    #[error("{0}")]
    Dimension(#[from] DimensionOverflow),
    /// The output element count, byte size or compressed pointer length overflows.
    #[error("numerical export storage size overflows")]
    SizeOverflow,
    /// An allocation owned by this export could not be reserved.
    #[error("{0}")]
    Allocation(#[from] TryReserveError),
    /// The container shape or entry positions disagree with their metadata.
    #[error("numerical export shape disagrees with its data")]
    InvalidShape,
    /// An ndarray constructor rejected the shape.
    #[cfg(feature = "ndarray")]
    #[error("{0}")]
    NdarrayShape(#[from] ndarray::ShapeError),
    /// A sprs constructor rejected the compressed structure.
    #[cfg(feature = "sprs")]
    #[error("{0}")]
    SparseStructure(#[from] sprs::errors::StructureError),
}

/// Numerical data paired with the metadata needed to interpret its coordinates.
pub struct Exported<D, M> {
    pub(crate) data: D,
    pub(crate) metadata: M,
}

impl<D, M> Exported<D, M> {
    /// Borrows the numerical container.
    pub fn data(&self) -> &D {
        &self.data
    }
    /// Borrows its basis, shape and factor-space interpretation.
    pub fn metadata(&self) -> &M {
        &self.metadata
    }
    /// Takes the numerical container, discarding the metadata.
    pub fn into_data(self) -> D {
        self.data
    }
    /// Takes both the numerical container and its interpretation.
    pub fn into_parts(self) -> (D, M) {
        (self.data, self.metadata)
    }
}

/// Common export trait, result type and matrix arguments.
pub mod prelude {
    pub use crate::{Export, Exported, matrix::args::MatrixArgs};
}

/// Ability to export into `T` with the operation-specific arguments `Args`.
///
/// Expressions implement this for shared references. Owned coordinate and Vec
/// results can instead move their data into the target, without coefficient clones.
pub trait Export<T, Args = ()> {
    /// Failure of basis interpretation, space selection, sizing or construction.
    type Error;
    /// Converts this source without implicitly changing coefficient precision.
    fn export(self, args: Args) -> Result<T, Self::Error>;
}
