//! Standard UUID and named indices. Randomness stays outside core.
//!
//! ```
//! use qmb_basic::Id128;
//! let index = Id128::new();
//! let same = index;
//! assert_eq!(index, same);
//! assert_ne!(index, Id128::new());
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use core::fmt;

/// An opaque, randomly generated version 4 UUID identifying one basic factor.
///
/// Copies preserve identity. Only explicit generation issues a new index.
/// The UUID library obtains randomness from the operating system; callers do
/// not supply a seed. Generation follows `uuid::Uuid::new_v4`'s platform and
/// failure behavior (it can panic if randomness is unavailable).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id128(uuid::Uuid);

impl Id128 {
    /// Issues a fresh UUID without a local-structure argument or seed.
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for Id128 {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Id128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A user-chosen name identifying a basic factor.
///
/// Equality, hashing and ordering use the complete string: independently
/// constructed tags with the same name refer to the same factor. Choosing
/// distinct names for distinct factors is the caller's responsibility.
/// Display writes the name unchanged; expression Display shows its first
/// eight Unicode scalar values. Debug retains the complete name.
///
/// ```
/// use qmb_basic::Tag;
/// use qmb_core::{HilbertSpace, LocalState};
/// let factor = HilbertSpace::factor(Tag::new("spin-left"));
/// let state = LocalState::<_, &str, i32>::primitive("up").at(&factor)?;
/// assert_eq!(state.to_string(), "|up[spin-lef]>");
/// assert_eq!(Tag::from("spin-left").to_string(), "spin-left");
/// # Ok::<(), qmb_core::ExpressionError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Tag {
    raw: String,
}

impl Tag {
    /// Creates a named index without generating a UUID or normalizing the name.
    pub fn new(raw: impl Into<String>) -> Self {
        Self { raw: raw.into() }
    }

    /// Borrows the complete name.
    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

impl From<String> for Tag {
    fn from(raw: String) -> Self {
        Self { raw }
    }
}

impl From<&str> for Tag {
    fn from(raw: &str) -> Self {
        Self::new(raw)
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.raw, f)
    }
}

/// A core space using the standard UUID index.
pub type HilbertSpace = qmb_core::HilbertSpace<Id128>;
