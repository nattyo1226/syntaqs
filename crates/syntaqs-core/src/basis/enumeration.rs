//! Local basis sizes and lazy tensor-product enumeration.

use crate::{
    Basis, BasisError, Dimension, HilbertSpace, LocalOperator, LocalState, Operator, State,
    expression::{
        composite::{OpExpr, OpNode, StateExpr, StateNode},
        parts::Operands,
    },
};
use alloc::{vec, vec::Vec};

type BasisElement<T> = Result<T, BasisError>;

pub(super) fn collect_leaves<I>(space: &HilbertSpace<I>) -> Vec<&HilbertSpace<I>> {
    let mut pending = vec![space];
    let mut result = Vec::new();
    while let Some(space) = pending.pop() {
        if space.factor_index().is_some() {
            result.push(space);
        } else {
            pending.extend(space.children().rev());
        }
    }
    result
}

pub(super) fn local_size<P: Dimension>(count: usize) -> Result<Option<usize>, BasisError> {
    if count == 0 {
        Ok(None)
    } else {
        Ok(Some(P::dim()?.get()))
    }
}

// An odometer, independent of the representability of the global dimension.
fn basis_indices(count: usize, dimension: Option<usize>) -> impl Iterator<Item = Vec<usize>> {
    let mut digits = vec![0; count];
    let mut done = false;
    core::iter::from_fn(move || {
        if done {
            return None;
        }
        let current = digits.clone();
        done = true;
        for digit in digits.iter_mut().rev() {
            *digit += 1;
            if *digit < dimension.expect("nonempty basis index") {
                done = false;
                break;
            }
            *digit = 0;
        }
        Some(current)
    })
}

pub(super) fn basis_element<P, C, B: Basis<P, C>>(
    basis: &B,
    index: usize,
    dimension: usize,
) -> Result<P, BasisError> {
    let primitive = basis.element(index).ok_or(BasisError::InvalidEnumeration)?;
    let actual = basis.index(&primitive).ok_or(BasisError::NonBasisElement)?;
    if actual >= dimension {
        return Err(BasisError::IndexOutOfBounds {
            index: actual,
            dimension,
        });
    }
    if actual != index {
        return Err(BasisError::InvalidEnumeration);
    }
    Ok(primitive)
}

fn enumerated_element<P, C, B: Basis<P, C>>(
    basis: &B,
    index: usize,
    dimension: usize,
) -> Result<P, BasisError> {
    let primitive = basis_element(basis, index, dimension)?;
    if index == dimension - 1 && basis.element(dimension).is_some() {
        return Err(BasisError::InvalidEnumeration);
    }
    Ok(primitive)
}

impl<I> HilbertSpace<I> {
    /// Lazily enumerates tensor-product basis states in mixed-radix order.
    ///
    /// The same local basis is used on every factor. Enumeration needs neither
    /// index comparison, algebraic evaluation, nor cloning primitives/indices,
    /// and does not require a representable global dimension. The unit space
    /// yields one unit state.
    /// Invalid enumeration is reported when the corresponding element is read.
    pub fn basis_states<S, O, C, B>(
        &self,
        basis: &B,
    ) -> Result<impl Iterator<Item = BasisElement<State<I, S, O, C>>>, BasisError>
    where
        S: Dimension,
        B: Basis<S, C>,
    {
        let factors = collect_leaves(self);
        let dimension = local_size::<S>(factors.len())?;
        Ok(basis_indices(factors.len(), dimension).map(move |digits| {
            let nodes = digits
                .into_iter()
                .zip(&factors)
                .map(|(index, factor)| {
                    let primitive =
                        enumerated_element(basis, index, dimension.expect("basis factor"))?;
                    Ok(StateExpr::new(StateNode::Local(
                        LocalState::primitive(primitive),
                        (*factor).clone(),
                    )))
                })
                .collect::<Result<Operands<_>, BasisError>>()?;
            Ok(State {
                space: self.clone(),
                expression: StateExpr::tensor(nodes),
            })
        }))
    }

    /// Lazily enumerates operator-basis products, bound to this space.
    ///
    /// These are basis operators, not columns of an action matrix. No index
    /// comparison, product, adjoint, identity, coefficient arithmetic, or
    /// primitive clone is needed.
    pub fn basis_operators<O, C, B>(
        &self,
        basis: &B,
    ) -> Result<impl Iterator<Item = Result<Operator<I, O, C>, BasisError>>, BasisError>
    where
        O: Dimension,
        B: Basis<O, C>,
    {
        let factors = collect_leaves(self);
        let dimension = local_size::<O>(factors.len())?;
        Ok(basis_indices(factors.len(), dimension).map(move |digits| {
            let mut nodes = Vec::with_capacity(factors.len());
            for (index, factor) in digits.into_iter().zip(&factors) {
                let primitive = enumerated_element(basis, index, dimension.expect("basis factor"))?;
                nodes.push(OpExpr::new(OpNode::Local(
                    LocalOperator::primitive(primitive),
                    (*factor).clone(),
                )));
            }
            Ok(Operator {
                ambient: Some(self.clone()),
                expression: OpExpr::product(nodes.into()),
            })
        }))
    }
}
