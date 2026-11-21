use alloc::{vec, vec::Vec};
use core::{iter::FusedIterator, slice};

/// An ordered sequence with inline storage for zero or one operand.
///
/// Constructors store sequences of length two or greater in `Many`.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum Operands<E> {
    /// The empty sequence.
    #[default]
    Empty,
    /// One operand, stored inline.
    One(E),
    /// An owned sequence of at least two operands.
    Many(Vec<E>),
}

impl<E> Operands<E> {
    /// Borrows operands in their stored order.
    pub fn iter(&self) -> slice::Iter<'_, E> {
        self.as_slice().iter()
    }

    /// Borrows the contiguous stored sequence.
    pub fn as_slice(&self) -> &[E] {
        match self {
            Self::Empty => &[],
            Self::One(value) => slice::from_ref(value),
            Self::Many(values) => values,
        }
    }

    /// Returns the number of operands.
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    /// Returns whether there are no operands.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Reuses a single operand directly. The caller handles an empty or multi-operand sequence.
    pub fn collapse_singleton(self, build: impl FnOnce(Self) -> E) -> E {
        match self {
            Self::One(operand) => operand,
            operands => build(operands),
        }
    }

    /// Transforms each operand once, in order, without cloning.
    pub fn map<F>(self, mut f: impl FnMut(E) -> F) -> Operands<F> {
        match self {
            Self::Empty => Operands::Empty,
            Self::One(operand) => Operands::One(f(operand)),
            Self::Many(operands) => Operands::Many(operands.into_iter().map(f).collect()),
        }
    }

    /// Concatenates sequences without arithmetic or cloning.
    pub fn concat(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Empty, rhs) => rhs,
            (lhs, Self::Empty) => lhs,
            (Self::One(lhs), Self::One(rhs)) => Self::Many(vec![lhs, rhs]),
            (Self::Many(mut lhs), Self::One(rhs)) => {
                lhs.push(rhs);
                Self::Many(lhs)
            }
            (Self::One(lhs), Self::Many(mut rhs)) => {
                rhs.insert(0, lhs);
                Self::Many(rhs)
            }
            (Self::Many(mut lhs), Self::Many(rhs)) => {
                lhs.extend(rhs);
                Self::Many(lhs)
            }
        }
    }
}

impl<E> FromIterator<E> for Operands<E> {
    fn from_iter<I: IntoIterator<Item = E>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        let Some(first) = iter.next() else {
            return Self::Empty;
        };
        let Some(second) = iter.next() else {
            return Self::One(first);
        };
        let mut values = Vec::with_capacity(2 + iter.size_hint().0);
        values.extend([first, second]);
        values.extend(iter);
        Self::Many(values)
    }
}

impl<E> From<Vec<E>> for Operands<E> {
    fn from(mut operands: Vec<E>) -> Self {
        match operands.len() {
            0 => Self::Empty,
            1 => Self::One(operands.pop().expect("one operand")),
            _ => Self::Many(operands),
        }
    }
}

impl<E> AsRef<[E]> for Operands<E> {
    fn as_ref(&self) -> &[E] {
        self.as_slice()
    }
}

/// An owning iterator that preserves inline empty and single-element storage.
pub enum IntoOperands<E> {
    /// Zero or one operand.
    Inline(core::option::IntoIter<E>),
    /// Operands from an existing allocation.
    Many(alloc::vec::IntoIter<E>),
}

impl<E> IntoIterator for Operands<E> {
    type Item = E;
    type IntoIter = IntoOperands<E>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Empty => IntoOperands::Inline(None.into_iter()),
            Self::One(value) => IntoOperands::Inline(Some(value).into_iter()),
            Self::Many(values) => IntoOperands::Many(values.into_iter()),
        }
    }
}

impl<'a, E> IntoIterator for &'a Operands<E> {
    type Item = &'a E;
    type IntoIter = slice::Iter<'a, E>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<E> Iterator for IntoOperands<E> {
    type Item = E;

    fn next(&mut self) -> Option<E> {
        match self {
            Self::Inline(iter) => iter.next(),
            Self::Many(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<E> DoubleEndedIterator for IntoOperands<E> {
    fn next_back(&mut self) -> Option<E> {
        match self {
            Self::Inline(iter) => iter.next_back(),
            Self::Many(iter) => iter.next_back(),
        }
    }
}

impl<E> ExactSizeIterator for IntoOperands<E> {
    fn len(&self) -> usize {
        match self {
            Self::Inline(iter) => iter.len(),
            Self::Many(iter) => iter.len(),
        }
    }
}

impl<E> FusedIterator for IntoOperands<E> {}

#[cfg(test)]
mod tests {
    use super::Operands;
    use alloc::vec::Vec;

    #[test]
    fn operands_support_borrowed_and_owned_mixed_end_iteration() {
        assert!(Operands::<u8>::default().is_empty());
        for n in 0..5 {
            let operands: Operands<_> = (0..n).collect();
            assert_eq!(operands.len(), n);
            assert_eq!(
                operands.iter().copied().collect::<Vec<_>>(),
                (0..n).collect::<Vec<_>>()
            );
            let mut iter = operands.into_iter();
            if n > 0 {
                assert_eq!(iter.next_back(), Some(n - 1));
            }
            if n > 1 {
                assert_eq!(iter.next(), Some(0));
            }
            assert_eq!(
                iter.collect::<Vec<_>>(),
                (1..n.saturating_sub(1)).collect::<Vec<_>>()
            );
        }
    }
}
