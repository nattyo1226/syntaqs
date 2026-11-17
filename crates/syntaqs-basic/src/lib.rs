//! Standard UUID and named indices. Randomness stays outside core.
//!
//! ```
//! use syntaqs_basic::index::Id128;
//! let index = Id128::new();
//! let same = index;
//! assert_eq!(index, same);
//! assert_ne!(index, Id128::new());
//! ```
//!
//! Identifiers are exposed through `index` and `prelude`:
//!
//! ```compile_fail
//! use syntaqs_basic::Id128;
//! ```
//! ```compile_fail
//! use syntaqs_basic::Tag;
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Standard identifiers for individual factors.
pub mod index;

/// Commonly used factor identifiers.
pub mod prelude {
    pub use crate::index::{Id128, Tag};
}
