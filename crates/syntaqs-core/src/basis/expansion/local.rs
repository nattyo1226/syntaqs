//! Local basis expansion and sparse coordinate extraction.

use super::{accumulate, basis_element};
use crate::{
    Act, Adjoint, Basis, BasisError, Conjugate, Coordinates, Dimension, Identity,
    LinearCombination, LocalOperator, LocalState, Product,
    expression::traversal::{LinearTerms, operator_view, state_view},
};
use alloc::{collections::BTreeMap, vec};
use core::ops::Mul;
use num_traits::{One, Zero};

type LocalExpansion<C> = BTreeMap<usize, C>;

pub(super) fn indexed_terms<P, C, B: Basis<P, C>>(
    primitive: &P,
    basis: &B,
    dimension: usize,
) -> Result<LinearCombination<usize, C>, BasisError> {
    let terms = basis.expand_indices(primitive);
    for term in terms.terms() {
        if term.operand >= dimension {
            return Err(BasisError::IndexOutOfBounds {
                index: term.operand,
                dimension,
            });
        }
    }
    Ok(terms)
}

pub(super) fn local_expansion<'a, P: 'a, C>(
    terms: impl Iterator<Item = (Option<C>, &'a P)>,
    mut expand: impl FnMut(&'a P) -> Result<LinearCombination<usize, C>, BasisError>,
) -> Result<LocalExpansion<C>, BasisError>
where
    C: Clone + Zero + Mul<Output = C>,
{
    let mut result = BTreeMap::new();
    for (coefficient, primitive) in terms {
        for term in expand(primitive)? {
            let c = match &coefficient {
                None => term.coefficient,
                Some(c) => c.clone() * term.coefficient,
            };
            accumulate(&mut result, term.operand, c);
        }
    }
    Ok(result)
}

pub(super) fn local_operator<O, C, B>(
    op: &LocalOperator<O, C>,
    basis: &B,
    dimension: usize,
) -> Result<LocalExpansion<C>, BasisError>
where
    C: Clone + Zero + Mul<Output = C> + Conjugate<Output = C>,
    O: Identity + Product<C, Term = O> + Adjoint<C, Term = O>,
    B: Basis<O, C>,
{
    let op = op.evaluate();
    local_expansion(LinearTerms::new(&op, operator_view), |p| {
        indexed_terms(p, basis, dimension)
    })
}

fn local_coordinates<B, C>(
    basis: &B,
    dimension: usize,
    terms: LocalExpansion<C>,
) -> Coordinates<'_, B, C> {
    Coordinates {
        basis,
        space: (),
        factor_count: 1,
        dim_local: Some(dimension),
        entries: terms.into_iter().map(|(i, c)| (vec![i], c)).collect(),
    }
}

impl<S, O, C> LocalState<S, O, C> {
    /// Expands and aggregates into ordinary local state syntax in `basis`.
    ///
    /// The input is unchanged. No basis metadata, dense coordinates, operator
    /// multiplication capability, or primitive cloning is required.
    /// Evaluates/collects first; removed zero terms do not run basis rules.
    pub fn expand<B>(&self, basis: &B) -> Result<Self, BasisError>
    where
        C: One + Clone + Zero + Mul<Output = C> + Conjugate<Output = C>,
        S: PartialEq + Dimension + Act<C, Operator = O, Term = S>,
        O: Adjoint<C, Term = O>,
        B: Basis<S, C>,
    {
        let dimension = S::dim()?.get();
        let state = self.evaluate();
        let terms = local_expansion(LinearTerms::new(&state, state_view), |p| {
            indexed_terms(p, basis, dimension)
        })?;
        let terms = terms
            .into_iter()
            .map(|(index, c)| {
                Ok(Self::primitive(basis_element(basis, index, dimension)?).into_scaled(c))
            })
            .collect::<Result<crate::expression::parts::Operands<_>, BasisError>>()?;
        Ok(Self::sum(terms))
    }

    /// Obtains aggregated sparse basis coordinates, without building an expanded tree.
    /// State terms removed by evaluation do not run basis rules.
    pub fn coordinates<'b, B>(&self, basis: &'b B) -> Result<Coordinates<'b, B, C>, BasisError>
    where
        C: One + Clone + Zero + Mul<Output = C> + Conjugate<Output = C>,
        S: PartialEq + Dimension + Act<C, Operator = O, Term = S>,
        O: Adjoint<C, Term = O>,
        B: Basis<S, C>,
    {
        let dimension = S::dim()?.get();
        Ok(local_coordinates(
            basis,
            dimension,
            local_expansion(LinearTerms::new(&self.evaluate(), state_view), |p| {
                indexed_terms(p, basis, dimension)
            })?,
        ))
    }
}

impl<O, C> LocalOperator<O, C> {
    /// Evaluates local products and adjoints, then expands into ordinary syntax.
    ///
    /// Equal basis indices are aggregated without comparing primitives. Exact
    /// zeros are removed; no tolerance-based pruning is performed.
    pub fn expand<B>(&self, basis: &B) -> Result<Self, BasisError>
    where
        C: Clone + Zero + Mul<Output = C> + Conjugate<Output = C>,
        O: Dimension + Identity + Product<C, Term = O> + Adjoint<C, Term = O>,
        B: Basis<O, C>,
    {
        let dimension = O::dim()?.get();
        let terms = local_operator(self, basis, dimension)?;
        let terms = terms
            .into_iter()
            .map(|(index, c)| {
                Ok(Self::primitive(basis_element(basis, index, dimension)?).into_scaled(c))
            })
            .collect::<Result<crate::expression::parts::Operands<_>, BasisError>>()?;
        Ok(Self::sum(terms))
    }

    /// Obtains coordinates in an operator basis, not a matrix acting on states.
    pub fn coordinates<'b, B>(&self, basis: &'b B) -> Result<Coordinates<'b, B, C>, BasisError>
    where
        C: Clone + Zero + Mul<Output = C> + Conjugate<Output = C>,
        O: Dimension + Identity + Product<C, Term = O> + Adjoint<C, Term = O>,
        B: Basis<O, C>,
    {
        let dimension = O::dim()?.get();
        Ok(local_coordinates(
            basis,
            dimension,
            local_operator(self, basis, dimension)?,
        ))
    }
}
