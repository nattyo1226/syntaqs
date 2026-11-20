//! Operator coordinates from sparse specified factors and shared spectators.
use alloc::{
    collections::{BTreeMap, btree_map::Entry},
    vec,
    vec::Vec,
};
use core::ops::Neg;
use num_traits::{One, Zero};

use super::{Expansion, accumulate, local::local_operator};
use crate::{
    Adjoint, Basis, BasisError, Conjugate, HomogeneousParity, Identity, Operator, Product,
    evaluate::operator::{EvaluatedOperator, OperatorTerm},
    expression::parts::Operands,
    space::FactorPositions,
};

type Choice<C> = (usize, C);
type Choices<C> = Operands<Choice<C>>;
struct Axis<'a, C> {
    position: usize,
    choices: &'a [Choice<C>],
}

// Multiplication stays in ambient factor order. Precomputing gamma^k and then
// multiplying by the term coefficient could overflow/underflow unnecessarily.
fn gap<C: Clone + Zero + One>(mut value: C, count: usize, gamma: Option<&C>) -> C {
    if let Some(gamma) = gamma {
        for _ in 0..count {
            if value.is_zero() {
                break;
            }
            value = value * gamma.clone();
        }
    }
    value
}

struct Frame<C> {
    next: usize,
    prefix: C,
}
fn enumerate<C: Clone + PartialEq + Zero + One>(
    axes: &[Axis<'_, C>],
    key: &mut [usize],
    coefficient: C,
    identity: Option<&Choice<C>>,
    mut emit: impl FnMut(&[usize], C),
) {
    if coefficient.is_zero() {
        return;
    }
    let gamma = identity.and_then(|(_, c)| (!c.is_one()).then_some(c));
    if axes.iter().all(|a| a.choices.len() == 1) {
        let mut value = coefficient;
        let mut previous = 0;
        for axis in axes {
            value = gap(value, axis.position - previous, gamma);
            if value.is_zero() {
                return;
            }
            let (digit, c) = &axis.choices[0];
            key[axis.position] = *digit;
            value = value * c.clone();
            if value.is_zero() {
                return;
            }
            previous = axis.position + 1;
        }
        value = gap(value, key.len() - previous, gamma);
        if !value.is_zero() {
            emit(key, value);
        }
        return;
    }
    let prefix = gap(coefficient, axes[0].position, gamma);
    if prefix.is_zero() {
        return;
    }
    let mut frames = vec![Frame { next: 0, prefix }];
    while !frames.is_empty() {
        let depth = frames.len() - 1;
        let frame = frames.last_mut().unwrap();
        let axis = &axes[depth];
        let Some((digit, c)) = axis.choices.get(frame.next) else {
            frames.pop();
            continue;
        };
        frame.next += 1;
        key[axis.position] = *digit;
        let value = frame.prefix.clone() * c.clone();
        if value.is_zero() {
            continue;
        }
        let next = axes.get(depth + 1).map_or(key.len(), |a| a.position);
        let value = gap(value, next - axis.position - 1, gamma);
        if value.is_zero() {
            continue;
        }
        if depth + 1 == axes.len() {
            emit(key, value);
        } else {
            frames.push(Frame {
                next: 0,
                prefix: value,
            });
        }
    }
}

pub(super) fn operator_expansion<I: Ord, O, C, B>(
    op: &Operator<I, O, C>,
    basis: &B,
    positions: &FactorPositions<'_, I>,
    dimension: Option<usize>,
) -> Result<Expansion<C>, BasisError>
where
    C: Clone + PartialEq + Zero + One + Conjugate<Output = C> + Neg<Output = C>,
    O: PartialEq + HomogeneousParity + Identity + Product<C, Term = O> + Adjoint<C, Term = O>,
    B: Basis<O, C>,
{
    let evaluated = EvaluatedOperator::new(&op.expression, positions);
    let mut result = Expansion::new();
    let mut identity: Option<Choices<C>> = None;
    let mut cache: BTreeMap<usize, Choices<C>> = BTreeMap::new();
    let factor_count = positions.leaves().len();
    let mut key = vec![0; factor_count];
    for OperatorTerm {
        coefficient,
        factors: locals,
    } in evaluated.terms()
    {
        let mut next_position = 0;
        let positions = locals.iter().map(|&(position, id)| (position, Some(id)));
        let mut empty = false;
        // Validate a spectator at the first gap, in order with the specified
        // factors. The right boundary accounts for the last spectator range.
        for (position, id) in positions.chain(core::iter::once((factor_count, None))) {
            if position > next_position && identity.is_none() {
                let choices: Choices<C> = local_operator(
                    evaluated.identity(),
                    basis,
                    dimension.expect("identity factor"),
                )?
                .into_iter()
                .collect();
                if let Operands::One((digit, _)) = &choices {
                    key.fill(*digit);
                }
                identity = Some(choices);
            }
            let Some(id) = id else { break };
            let choices = match cache.entry(id) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => entry.insert(
                    local_operator(
                        evaluated.primitive(id),
                        basis,
                        dimension.expect("operator factor"),
                    )?
                    .into_iter()
                    .collect(),
                ),
            };
            empty |= choices.is_empty();
            next_position = position + 1;
        }
        let spectators = locals.len() < factor_count;
        if empty || spectators && identity.as_ref().unwrap().is_empty() {
            continue;
        }
        let single = identity.as_ref().and_then(|v| match v {
            Operands::One(c) => Some(c),
            _ => None,
        });
        let mut axes = Vec::with_capacity(if spectators && single.is_none() {
            factor_count
        } else {
            locals.len()
        });
        let multi = spectators && single.is_none();
        let mut next_position = 0;
        for &(position, id) in &locals {
            if multi {
                let choices = identity.as_ref().unwrap().as_slice();
                axes.extend((next_position..position).map(|position| Axis { position, choices }));
            }
            axes.push(Axis {
                position,
                choices: cache[&id].as_slice(),
            });
            next_position = position + 1;
        }
        if multi {
            let choices = identity.as_ref().unwrap().as_slice();
            axes.extend((next_position..factor_count).map(|position| Axis { position, choices }));
        }
        enumerate(&axes, &mut key, coefficient, single, |digits, c| {
            accumulate(&mut result, digits.to_vec(), c)
        });
        if let Some((digit, _)) = single {
            for &(position, _) in &locals {
                key[position] = *digit;
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests;
