//! Evaluation and scalar contractions using primitive rules.

use crate::{
    LocalOperator,
    expression::{
        composite::{OpExpr, OpNode},
        local::OperatorNode,
        parts::Operands,
    },
};

mod collect;
mod composite;
pub(crate) mod inner_product;
mod local;
pub(crate) mod operator;
mod trace;

mod state_terms;

trait ProductOperands: Clone {
    fn operands(&self) -> &Operands<Self>;
}

impl<I, O, C> ProductOperands for OpExpr<I, O, C> {
    fn operands(&self) -> &Operands<Self> {
        let OpNode::Product(operands) = self.node() else {
            unreachable!("composite product action");
        };
        operands
    }
}

impl<O, C> ProductOperands for LocalOperator<O, C> {
    fn operands(&self) -> &Operands<Self> {
        let OperatorNode::Product(operands) = self.node() else {
            unreachable!("local product action");
        };
        operands
    }
}

// One continuation owns the original product. Advance sends just one operand
// to the evaluator. The continuation can be dropped on an exact-zero state.
struct ProductAction<E> {
    product: E,
    // One-based offset. Positive offsets start at the front for adjoints.
    // Negative offsets start at the back for ordinary actions.
    next: isize,
}

impl<E: ProductOperands> ProductAction<E> {
    fn new(product: E, adjoint: bool) -> Self {
        Self {
            product,
            next: if adjoint { 1 } else { -1 },
        }
    }

    fn advance(mut self) -> (E, bool, Option<Self>) {
        let operands = self.product.operands().as_slice();
        let adjoint = self.next > 0;
        let offset = self.next.unsigned_abs();
        let position = if adjoint {
            offset - 1
        } else {
            operands.len() - offset
        };
        let operand = operands[position].clone();
        let remaining = if offset < operands.len() {
            self.next += if adjoint { 1 } else { -1 };
            Some(self)
        } else {
            None
        };
        (operand, adjoint, remaining)
    }
}

fn add_pair<C>(sum: C, value: C, off_diagonal: bool) -> C
where
    C: num_traits::Zero + crate::Conjugate<Output = C>,
{
    if value.is_zero() {
        return sum;
    }
    if off_diagonal {
        let reflected = value.conjugate();
        sum + (value + reflected)
    } else {
        sum + value
    }
}

#[cfg(test)]
mod tests;
