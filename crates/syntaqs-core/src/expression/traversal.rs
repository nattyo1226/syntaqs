//! Internal local linear-term traversal shared by evaluation and bases.

use super::{
    local::{LocalOperator, LocalState, OperatorNode, StateNode},
    parts::Operands,
};
use crate::coefficient::multiply;
use alloc::{vec, vec::Vec};
use core::ops::Mul;

// Both state and operator scalar reductions use this same linear walker.
// None as coefficient means an implicit one, so generating it needs no One.
pub(crate) enum LinearView<'a, E, P, C> {
    Leaf(&'a P),
    Scale(&'a C, &'a E),
    Sum(&'a Operands<E>),
}
pub(crate) struct LinearTerms<'a, E, P, C> {
    pending: Vec<(&'a E, Option<C>)>,
    view: fn(&'a E) -> LinearView<'a, E, P, C>,
}
impl<'a, E, P, C> LinearTerms<'a, E, P, C> {
    pub(crate) fn new(root: &'a E, view: fn(&'a E) -> LinearView<'a, E, P, C>) -> Self {
        Self {
            pending: vec![(root, None)],
            view,
        }
    }
}
impl<'a, E, P: 'a, C> Iterator for LinearTerms<'a, E, P, C>
where
    C: Clone + Mul<Output = C>,
{
    type Item = (Option<C>, &'a P);
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((expression, coefficient)) = self.pending.pop() {
            match (self.view)(expression) {
                LinearView::Leaf(leaf) => return Some((coefficient, leaf)),
                LinearView::Scale(c, operand) => {
                    let coefficient = Some(match coefficient {
                        None => c.clone(),
                        Some(a) => multiply(a, c.clone()),
                    });
                    self.pending.push((operand, coefficient));
                }
                LinearView::Sum(s) => {
                    for child in s.iter().rev() {
                        self.pending.push((child, coefficient.clone()));
                    }
                }
            }
        }
        None
    }
}
pub(crate) fn state_view<S, O, C>(
    state: &LocalState<S, O, C>,
) -> LinearView<'_, LocalState<S, O, C>, S, C> {
    match state.node() {
        StateNode::Primitive(p) => LinearView::Leaf(p),
        StateNode::Scale(s) => LinearView::Scale(&s.coefficient, &s.operand),
        StateNode::Sum(s) => LinearView::Sum(s),
        StateNode::Act(_) => unreachable!("evaluated state"),
    }
}
pub(crate) fn operator_view<O, C>(
    op: &LocalOperator<O, C>,
) -> LinearView<'_, LocalOperator<O, C>, O, C> {
    match op.node() {
        OperatorNode::Primitive(p) => LinearView::Leaf(p),
        OperatorNode::Scale(s) => LinearView::Scale(&s.coefficient, &s.operand),
        OperatorNode::Sum(s) => LinearView::Sum(s),
        _ => unreachable!("evaluated operator"),
    }
}

pub(crate) fn operator_handle_view<O, C>(
    operator: &LocalOperator<O, C>,
) -> LinearView<'_, LocalOperator<O, C>, LocalOperator<O, C>, C> {
    match operator_view(operator) {
        LinearView::Leaf(_) => LinearView::Leaf(operator),
        LinearView::Scale(c, op) => LinearView::Scale(c, op),
        LinearView::Sum(ops) => LinearView::Sum(ops),
    }
}
