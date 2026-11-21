//! Reusable syntax containers, independent of local structures and evaluation.
//!
//! Expression nodes and their operand sequences are internal implementation details:
//!
//! ```compile_fail
//! use syntaqs_core::expression::parts::Operands;
//! ```
//! ```compile_fail
//! use syntaqs_core::expression::parts::ScaleExpr;
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
mod operands;
mod operation;
mod scale;

pub(crate) use operands::{IntoOperands, Operands};
pub(crate) use operation::{ActExpr, AdjointExpr};
pub(crate) use scale::ScaleExpr;
