//! Ordered, finite linear combinations returned by algebra rules.

use crate::{
    coefficient::multiply,
    expression::parts::{IntoOperands, Operands},
};
use core::{
    iter::FusedIterator,
    ops::{Add, Mul},
    slice,
};
use num_traits::One;

/// A coefficient and an operand; distinct from an operation's associated `Term`.
#[derive(Clone, Debug, PartialEq)]
pub struct Term<P, C> {
    /// The coefficient, including explicitly stored zero and one.
    pub coefficient: C,
    /// The operand multiplied by the coefficient.
    pub operand: P,
}

/// An ordered sum preserving duplicates, zero coefficients, and input order.
///
/// Empty and single-term containers do not allocate. Storage and element mapping
/// require no arithmetic, cloning, or comparison capabilities.
#[derive(Clone, Debug, PartialEq)]
pub struct LinearCombination<P, C> {
    terms: Operands<Term<P, C>>,
}

impl<P, C> LinearCombination<P, C> {
    /// Constructs the empty sum.
    pub fn zero() -> Self {
        Self {
            terms: Operands::Empty,
        }
    }

    /// Constructs one term, retaining even a zero coefficient.
    pub fn term(coefficient: C, operand: P) -> Self {
        Self {
            terms: Operands::One(Term {
                coefficient,
                operand,
            }),
        }
    }

    /// Collects terms in order, without allocation for at most one term.
    pub fn from_terms(terms: impl IntoIterator<Item = Term<P, C>>) -> Self {
        Self {
            terms: terms.into_iter().collect(),
        }
    }

    /// Borrows the stored terms in order.
    pub fn terms(&self) -> slice::Iter<'_, Term<P, C>> {
        self.terms.iter()
    }

    /// Moves the terms out, taking over any existing allocation.
    pub fn into_terms(self) -> IntoTerms<P, C> {
        IntoTerms {
            inner: self.terms.into_iter(),
        }
    }

    /// Returns the number of stored terms, including zero coefficients.
    pub fn len(&self) -> usize {
        self.terms.len()
    }

    /// Returns whether the term sequence is empty, not mathematical zero.
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    /// Transforms operands once in order, moving coefficients unchanged.
    pub fn map<Q>(self, mut f: impl FnMut(P) -> Q) -> LinearCombination<Q, C> {
        LinearCombination {
            terms: self.terms.map(|term| Term {
                coefficient: term.coefficient,
                operand: f(term.operand),
            }),
        }
    }

    pub(crate) fn try_map<Q, E>(
        self,
        mut f: impl FnMut(P) -> Result<Q, E>,
    ) -> Result<LinearCombination<Q, C>, E> {
        let terms = self
            .into_iter()
            .map(|term| {
                Ok(Term {
                    coefficient: term.coefficient,
                    operand: f(term.operand)?,
                })
            })
            .collect::<Result<_, E>>()?;
        Ok(LinearCombination { terms })
    }

    /// Constructs one operand with coefficient one.
    pub fn primitive(operand: P) -> Self
    where
        C: One,
    {
        Self::term(C::one(), operand)
    }

    /// Multiplies coefficients, without cloning operands or removing zero terms.
    /// Converts the input once even for an empty sum; the input need not be Clone.
    pub fn scaled(self, coefficient: impl Into<C>) -> Self
    where
        C: Clone + Mul<Output = C>,
    {
        let coefficient = coefficient.into();
        Self {
            terms: self.terms.map(|term| Term {
                coefficient: multiply(term.coefficient, coefficient.clone()),
                operand: term.operand,
            }),
        }
    }
}

impl<P, C> Default for LinearCombination<P, C> {
    fn default() -> Self {
        Self::zero()
    }
}
impl<P, C> Add for LinearCombination<P, C> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            terms: self.terms.concat(rhs.terms),
        }
    }
}
impl<P, C> FromIterator<Term<P, C>> for LinearCombination<P, C> {
    fn from_iter<I: IntoIterator<Item = Term<P, C>>>(iter: I) -> Self {
        Self::from_terms(iter)
    }
}
impl<P, C> IntoIterator for LinearCombination<P, C> {
    type Item = Term<P, C>;
    type IntoIter = IntoTerms<P, C>;
    fn into_iter(self) -> Self::IntoIter {
        self.into_terms()
    }
}
impl<'a, P, C> IntoIterator for &'a LinearCombination<P, C> {
    type Item = &'a Term<P, C>;
    type IntoIter = slice::Iter<'a, Term<P, C>>;
    fn into_iter(self) -> Self::IntoIter {
        self.terms()
    }
}

/// Owning, exact-size, double-ended iteration over linear-combination terms.
pub struct IntoTerms<P, C> {
    inner: IntoOperands<Term<P, C>>,
}

impl<P, C> Iterator for IntoTerms<P, C> {
    type Item = Term<P, C>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<P, C> DoubleEndedIterator for IntoTerms<P, C> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<P, C> ExactSizeIterator for IntoTerms<P, C> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<P, C> FusedIterator for IntoTerms<P, C> {}
