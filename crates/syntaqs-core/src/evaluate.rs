//! Evaluation and scalar contractions using primitive rules.

use alloc::vec::Vec;

use crate::expression::parts::Operands;

mod collect;
mod composite;
pub(crate) mod inner_product;
mod local;
mod trace;

mod state_terms;

// Arrange the remaining actions on a LIFO stack and return the first operand
// to apply to the current state. Adjoint action reverses the application order.
fn schedule_product_action<'a, E, T>(
    operands: &'a Operands<E>,
    adjoint: bool,
    tasks: &mut Vec<T>,
    then_apply: impl FnMut(&'a E) -> T,
) -> &'a E {
    let mut operands = operands.iter();
    let first = if adjoint {
        let first = operands.next();
        tasks.extend(operands.rev().map(then_apply));
        first
    } else {
        let first = operands.next_back();
        tasks.extend(operands.map(then_apply));
        first
    };
    first.expect("product operand")
}

#[cfg(test)]
mod tests;
