//! Operator coordinates from sparse specified factors and shared spectators.
use alloc::{vec, vec::Vec};
use core::ops::Neg;
use num_traits::{One, Zero};

use super::{Expansion, accumulate_with, local::local_operator};
use crate::{
    Adjoint, Basis, BasisError, Conjugate, HilbertSpace, HomogeneousParity, Identity,
    LocalOperator, Operator, Product,
    expr::{parts::SumExpr, traversal::IntrinsicOpTerms},
};

// Coordinates need no payload. Expanded syntax joins cheap local handles only
// when a complete key first enters the result; no primitive Clone is needed.
pub(super) trait OperatorValue: Clone {
    fn product<'a>(values: impl Iterator<Item = &'a Self>) -> Self
    where
        Self: 'a;
}
impl OperatorValue for () {
    fn product<'a>(_: impl Iterator<Item = &'a Self>)
    where
        Self: 'a,
    {
    }
}
impl<E: Clone> OperatorValue for Vec<E> {
    fn product<'a>(values: impl Iterator<Item = &'a Self>) -> Self
    where
        Self: 'a,
    {
        values.flat_map(|v| v.iter().cloned()).collect()
    }
}
type Choice<V, C> = (usize, C, V);
type Choices<V, C> = SumExpr<Choice<V, C>>;
struct Specified<V, C> {
    position: usize,
    choices: Choices<V, C>,
}
struct Axis<'a, V, C> {
    position: usize,
    choices: &'a [Choice<V, C>],
}

// Multiplication stays in ambient factor order. Precomputing gamma^k and then
// multiplying by the term coefficient could overflow/underflow unnecessarily.
fn gap<C: Clone + Zero + One>(
    mut value: C,
    count: usize,
    identity: Option<&Choice<impl OperatorValue, C>>,
) -> C {
    if let Some((_, gamma, _)) = identity {
        for _ in 0..count {
            if value.is_zero() {
                break;
            }
            value = value * gamma.clone();
        }
    } else {
        debug_assert_eq!(count, 0);
    }
    value
}

struct Frame<C> {
    next: usize,
    prefix: C,
}
fn enumerate<V: OperatorValue, C: Clone + Zero + One>(
    axes: &[Axis<'_, V, C>],
    key: &mut [usize],
    coefficient: C,
    identity: Option<&Choice<V, C>>,
    mut emit: impl FnMut(&[usize], C, Option<&[usize]>),
) {
    if coefficient.is_zero() {
        return;
    }
    if axes.iter().all(|a| a.choices.len() == 1) {
        let mut value = coefficient;
        let mut previous = 0;
        for axis in axes {
            value = gap(value, axis.position - previous, identity);
            if value.is_zero() {
                return;
            }
            let (digit, c, _) = &axis.choices[0];
            key[axis.position] = *digit;
            value = value * c.clone();
            if value.is_zero() {
                return;
            }
            previous = axis.position + 1;
        }
        value = gap(value, key.len() - previous, identity);
        if !value.is_zero() {
            emit(key, value, None);
        }
        return;
    }
    let prefix = gap(coefficient, axes[0].position, identity);
    if prefix.is_zero() {
        return;
    }
    let mut frames = vec![Frame { next: 0, prefix }];
    let mut selected = vec![0; axes.len()];
    while !frames.is_empty() {
        let depth = frames.len() - 1;
        let frame = frames.last_mut().unwrap();
        let axis = &axes[depth];
        let Some((digit, c, _)) = axis.choices.get(frame.next) else {
            frames.pop();
            continue;
        };
        selected[depth] = frame.next;
        frame.next += 1;
        key[axis.position] = *digit;
        let value = frame.prefix.clone() * c.clone();
        if value.is_zero() {
            continue;
        }
        let next = axes.get(depth + 1).map_or(key.len(), |a| a.position);
        let value = gap(value, next - axis.position - 1, identity);
        if value.is_zero() {
            continue;
        }
        if depth + 1 == axes.len() {
            emit(key, value, Some(&selected));
        } else {
            frames.push(Frame {
                next: 0,
                prefix: value,
            });
        }
    }
}

pub(super) fn operator_expansion<I: Eq, O, C, B, V: OperatorValue>(
    op: &Operator<I, O, C>,
    basis: &B,
    ambient: &HilbertSpace<I>,
    factors: &[&HilbertSpace<I>],
    dimension: Option<usize>,
    make: impl Fn(O) -> V,
) -> Result<Expansion<V, C>, BasisError>
where
    C: Clone + Zero + One + Conjugate<Output = C> + Neg<Output = C>,
    O: HomogeneousParity + Identity + Product<C, Term = O> + Adjoint<C, Term = O>,
    B: Basis<O, C>,
{
    let expression = op.expression.group_local();
    let mut result = Expansion::new();
    let mut identity: Option<Choices<V, C>> = None;
    let mut key = vec![0; factors.len()];
    for (coefficient, locals) in IntrinsicOpTerms::new(&expression, ambient) {
        // Ordinary intrinsic terms retain encounter order; graded terms have
        // already resolved crossing signs. Position sorting here exchanges no
        // local algebra and never recomputes a graded sign.
        let mut locals: Vec<_> = locals
            .into_iter()
            .map(|(factor, local)| {
                let position = factors
                    .iter()
                    .position(|f| f.factor_index() == factor.factor_index())
                    .expect("validated factor");
                (position, local)
            })
            .collect();
        locals.sort_unstable_by_key(|(position, _)| *position);
        let mut specified = Vec::with_capacity(locals.len());
        let mut locals = locals.into_iter().peekable();
        let mut spectators = false;
        // Validate every reached local expansion, even when another is zero.
        for position in 0..factors.len() {
            if locals.peek().is_some_and(|(p, _)| *p == position) {
                let (_, local) = locals.next().unwrap();
                let choices =
                    local_operator(&local, basis, dimension.expect("operator factor"), &make)?
                        .into_iter()
                        .map(|(i, (c, v))| (i, c, v))
                        .collect();
                specified.push(Specified { position, choices });
            } else {
                spectators = true;
                if identity.is_none() {
                    let choices: Choices<V, C> = local_operator(
                        &LocalOperator::identity(),
                        basis,
                        dimension.expect("identity factor"),
                        &make,
                    )?
                    .into_iter()
                    .map(|(i, (c, v))| (i, c, v))
                    .collect();
                    if let SumExpr::One((digit, _, _)) = &choices {
                        key.fill(*digit);
                    }
                    identity = Some(choices);
                }
            }
        }
        debug_assert!(
            locals.next().is_none(),
            "validated intrinsic factor membership"
        );
        if specified.iter().any(|f| f.choices.is_empty())
            || spectators && identity.as_ref().unwrap().is_empty()
        {
            continue;
        }
        let single = identity.as_ref().and_then(|v| match v {
            SumExpr::One(c) => Some(c),
            _ => None,
        });
        let axes: Vec<_> = if spectators && single.is_none() {
            let mut next = 0;
            (0..factors.len())
                .map(|position| {
                    let choices = if specified.get(next).is_some_and(|f| f.position == position) {
                        let choices = specified[next].choices.as_slice();
                        next += 1;
                        choices
                    } else {
                        identity.as_ref().unwrap().as_slice()
                    };
                    Axis { position, choices }
                })
                .collect()
        } else {
            specified
                .iter()
                .map(|f| Axis {
                    position: f.position,
                    choices: f.choices.as_slice(),
                })
                .collect()
        };
        enumerate(
            &axes,
            &mut key,
            coefficient.unwrap_or_else(C::one),
            single,
            |digits, c, selected| {
                accumulate_with(&mut result, digits.to_vec(), c, || {
                    let mut next = 0;
                    V::product((0..factors.len()).map(|position| {
                        if axes.get(next).is_some_and(|a| a.position == position) {
                            let i = next;
                            next += 1;
                            &axes[i].choices[selected.map_or(0, |s| s[i])].2
                        } else {
                            &single.expect("single-component spectator").2
                        }
                    }))
                });
            },
        );
        if let Some((digit, _, _)) = single {
            for f in &specified {
                key[f.position] = *digit;
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests;
