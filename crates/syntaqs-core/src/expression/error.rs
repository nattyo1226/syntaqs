//! Errors from expression construction and evaluation.

use crate::space::SpaceError;

/// Errors connecting, placing or evaluating composite expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ExpressionError {
    /// A local expression must be placed on a leaf, not an arbitrary tensor.
    #[error("local placement requires a factor space")]
    NotAFactor,
    /// Operands have incompatible ordered factor spaces.
    #[error("incompatible Hilbert spaces")]
    IncompatibleSpaces,
    /// An operator refers to an index absent from its selected ambient space.
    #[error("referenced index is absent from the ambient space")]
    MissingIndex,
    /// No binding or explicit ambient space is available.
    #[error("operator ambient space is not specified")]
    UnboundOperator,
    /// An invalid tensor space was requested.
    #[error(transparent)]
    Space(#[from] SpaceError),
}
