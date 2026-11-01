//! Shared syntax parts and immutable local and composite expressions.
//!
//! # Formatting
//!
//! [`LocalState`], [`LocalOperator`], [`State`] and [`Operator`] implement
//! [`core::fmt::Display`] when their stored primitives, coefficients and indices
//! implement Display. Formatting borrows the stored syntax without evaluating,
//! simplifying, sorting or computing coefficients. No Clone or algebra abilities
//! are needed. Ordinary and graded expressions keep their stored order.
//!
//! `{}` writes mathematical notation without inserting newlines; `{:#}` breaks
//! sums and tensor kets across lines. Bracketed multiline operands use four-space
//! indentation. Leading `+ `
//! and `⊗ ` hang two columns left of aligned operands; root sequences reserve
//! two initial spaces. Pretty kets use `| up[a],` followed by aligned factors
//! and close with ` >`; a single factor is `| up[a] >`. Coefficients always
//! appear as `(c) * E`, action as `A psi`, and adjoints as `(A)†`.
//! Primitive state Display supplies a label; the expression adds ket markers.
//! Placed labels include their index, as in `|up[a],down[b]>`. Tensor subtrees
//! containing sums, scales, actions or explicit units use `⊗` instead.
//!
//! ```
//! use qmb_core::{HilbertSpace, LocalState, State};
//! let a = HilbertSpace::factor("a");
//! let b = HilbertSpace::factor("b");
//! let h = a.tensor(&b)?;
//! let up = LocalState::<_, &str, i32>::primitive("up");
//! let down = LocalState::<_, &str, i32>::primitive("down");
//! let state = State::tensor_in(&h, [up.at(&a)?, down.at(&b)?])?;
//! assert_eq!(state.to_string(), "|up[a],down[b]>");
//! assert_eq!(format!("{state:#}"), "| up[a],\n  down[b] >");
//! # Ok::<(), qmb_core::ExpressionError>(())
//! ```
//!
//! In expression Display only, index labels are limited to their first eight
//! Unicode scalar values, without an ellipsis or collision detection. Labels
//! are not escaped or parsed. The standalone index Display remains unchanged.
//! Width and precision do not specify wrapping or numeric precision, and outer
//! formatting flags are not forwarded to leaves. Display is iterative even for
//! deeply nested syntax, but custom leaf formatters control their own behavior.
//!
//! `{:?}` and `{:#?}` use derived Debug, including full indices, spaces and
//! management fields. Debug is recursive, and its textual representation is not
//! stable. Display may omit tensor grouping and ambient spaces; it is neither
//! an identity test nor a serialization format. Formatting traits are not
//! required for constructing or evaluating expressions.

pub mod parts;

mod arithmetic;
pub mod composite;
mod format;
pub(crate) mod local;
pub(crate) mod traversal;

pub use composite::{ExpressionError, Operator, State};
pub use local::{LocalOperator, LocalState};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod composite_tests;
