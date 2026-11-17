//! Reusable syntax containers, independent of local structures and evaluation.
//!
//! Expression nodes are internal implementation details:
//!
//! ```compile_fail
//! use syntaqs_core::expression::parts::SumExpr;
//! ```
//! ```compile_fail
//! use syntaqs_core::expression::parts::ScaleExpr;
//! ```
//! ```compile_fail
//! use syntaqs_core::expression::parts::ProductExpr;
//! ```
//! ```compile_fail
//! use syntaqs_core::expression::parts::ActExpr;
//! ```
//! ```compile_fail
//! use syntaqs_core::expression::parts::AdjointExpr;
//! ```
//! ```compile_fail
//! use syntaqs_core::expression::parts::IntoOperands;
//! ```

pub mod linear_combination;
mod operation;
mod scale;
mod sum;

pub(crate) use operation::{ActExpr, AdjointExpr, ProductExpr};
pub(crate) use scale::ScaleExpr;
pub(crate) use sum::{IntoOperands, SumExpr};
