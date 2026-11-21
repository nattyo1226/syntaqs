//! Structural local grouping, without primitive evaluation or factor exchange.

use alloc::{collections::BTreeMap, vec};
use core::{iter, ptr, slice};

use super::{
    composite::{OpExpr, OpNode, Operator},
    parts::{AdjointExpr, Operands, ScaleExpr},
};
use crate::{HilbertSpace, Identity, LocalOperator};

fn same_factor<I: Eq>(a: &HilbertSpace<I>, b: &HilbertSpace<I>) -> bool {
    a.factor_index() == b.factor_index()
}

fn local_prefix<I: Eq, O, C>(operands: &[OpExpr<I, O, C>]) -> Option<(&HilbertSpace<I>, usize)> {
    let (first, rest) = operands.split_first()?;
    let OpNode::Local(factor, _) = first.node() else {
        return None;
    };
    let count = 1 + rest
        .iter()
        .take_while(|op| matches!(op.node(), OpNode::Local(other, _) if same_factor(factor, other)))
        .count();
    (count > 1).then_some((factor, count))
}

fn local_operand<I, O, C>(op: OpExpr<I, O, C>) -> LocalOperator<O, C> {
    let OpNode::Local(_, local) = op.node() else {
        unreachable!("checked local operand");
    };
    local.clone()
}

// Construction and full grouping use the same immediate-child rules.
pub(super) fn sum<I: Eq, O, C>(operands: Operands<OpExpr<I, O, C>>) -> OpExpr<I, O, C> {
    if let Some((factor, count)) = local_prefix(operands.as_slice()) {
        if count == operands.len() {
            return OpExpr::new(OpNode::Local(
                factor.clone(),
                LocalOperator::sum_collected(operands.map(local_operand)),
            ));
        }
    }
    OpExpr::sum(operands)
}

pub(super) fn product<I: Eq, O, C>(operands: Operands<OpExpr<I, O, C>>) -> OpExpr<I, O, C> {
    let Some((factor, count)) = local_prefix(operands.as_slice()) else {
        return OpExpr::product(operands);
    };
    // The leading same-factor operands are the local part of the implicit
    // left fold. Later operands keep their order and their existing subtrees.
    let factor = factor.clone();
    let (locals, remaining) = if count == operands.len() {
        (operands.map(local_operand), Operands::Empty.into_iter())
    } else {
        let mut operands = operands.into_iter();
        let locals = operands.by_ref().take(count).map(local_operand).collect();
        (locals, operands)
    };
    let local = LocalOperator::from_product(locals).expect("nonempty local operands");
    let grouped = OpExpr::new(OpNode::Local(factor, local));
    OpExpr::product(iter::once(grouped).chain(remaining).collect())
}

enum Support<I> {
    Scalar,
    One(HilbertSpace<I>),
    Many,
}

impl<I: Eq> Support<I> {
    fn include_factor(&mut self, factor: &HilbertSpace<I>) -> bool {
        match self {
            Self::Scalar => {
                *self = Self::One(factor.clone());
                true
            }
            Self::One(previous) => same_factor(previous, factor),
            Self::Many => false,
        }
    }

    fn include(&mut self, other: &Self) -> bool {
        match other {
            Self::Scalar => true,
            Self::One(factor) => self.include_factor(factor),
            Self::Many => false,
        }
    }
}

struct Grouped<I, O, C> {
    support: Support<I>,
    replacement: Option<OpExpr<I, O, C>>,
    // Scalar conversion is independent of a factor, and is performed only if
    // this scalar subtree becomes part of a single-factor expression.
    scalar_local: Option<LocalOperator<O, C>>,
}
type Memo<I, O, C> = BTreeMap<*const OpNode<I, O, C>, Grouped<I, O, C>>;

fn prepared<'a, I, O, C>(op: &'a OpExpr<I, O, C>, memo: &'a Memo<I, O, C>) -> &'a OpExpr<I, O, C> {
    memo.get(&ptr::from_ref(op.node()))
        .and_then(|entry| entry.replacement.as_ref())
        .unwrap_or(op)
}

fn children<I, O, C>(op: &OpExpr<I, O, C>) -> &[OpExpr<I, O, C>] {
    match op.node() {
        OpNode::Scale(s) => slice::from_ref(&s.operand),
        OpNode::Sum(s) | OpNode::Product(s) => s.as_slice(),
        OpNode::Adjoint(a) => slice::from_ref(&a.operand),
        OpNode::Identity | OpNode::Local(_, _) => &[],
    }
}

// Avoid a full memo for small expressions that cannot contain a rewrite.
// The cap keeps this non-memoized preflight bounded even for a highly shared
// DAG. Reaching it means "possibly groupable", never "already grouped".
fn may_group<I: Eq, O, C>(root: &OpExpr<I, O, C>) -> bool {
    const LIMIT: usize = 256;
    let mut pending = vec![root];
    for _ in 0..LIMIT {
        let Some(op) = pending.pop() else {
            return false;
        };
        match op.node() {
            OpNode::Scale(_) | OpNode::Sum(_) => return true,
            OpNode::Adjoint(a)
                if matches!(a.operand.node(), OpNode::Local(_, _) | OpNode::Identity) =>
            {
                return true;
            }
            OpNode::Product(p)
                if p.iter()
                    .any(|child| matches!(child.node(), OpNode::Identity)) =>
            {
                return true;
            }
            _ => {}
        }
        pending.extend(children(op));
    }
    true
}

fn scalar_local<I, O: Identity, C: Clone>(
    root: &OpExpr<I, O, C>,
    memo: &mut Memo<I, O, C>,
) -> LocalOperator<O, C> {
    let mut pending = vec![(root, false)];
    while let Some((op, visited)) = pending.pop() {
        let key = ptr::from_ref(op.node());
        if memo
            .get(&key)
            .is_some_and(|entry| entry.scalar_local.is_some())
        {
            continue;
        }
        if !visited {
            pending.push((op, true));
            pending.extend(children(op).iter().rev().map(|child| (child, false)));
            continue;
        }
        let get = |child: &OpExpr<I, O, C>| {
            memo[&ptr::from_ref(child.node())]
                .scalar_local
                .as_ref()
                .expect("scalar child converted")
                .clone()
        };
        let local = match op.node() {
            OpNode::Identity => LocalOperator::identity(),
            OpNode::Scale(s) => get(&s.operand).scaled(s.coefficient.clone()),
            OpNode::Sum(s) => LocalOperator::sum(s.iter().map(get)),
            OpNode::Product(p) => LocalOperator::product(p.iter().map(get)),
            OpNode::Adjoint(a) => get(&a.operand).adjoint(),
            OpNode::Local(_, _) => unreachable!("scalar-only subtree"),
        };
        memo.entry(key)
            .or_insert_with(|| Grouped {
                support: Support::Scalar,
                replacement: None,
                scalar_local: None,
            })
            .scalar_local = Some(local);
    }
    memo[&ptr::from_ref(root.node())]
        .scalar_local
        .as_ref()
        .expect("converted scalar")
        .clone()
}

fn prepared_on_factor<I, O: Identity, C: Clone>(
    child: &OpExpr<I, O, C>,
    factor: &HilbertSpace<I>,
    memo: &mut Memo<I, O, C>,
) -> OpExpr<I, O, C> {
    let scalar = match child.node() {
        OpNode::Identity => true,
        OpNode::Local(_, _) => false,
        _ => matches!(memo[&ptr::from_ref(child.node())].support, Support::Scalar),
    };
    if scalar {
        OpExpr::new(OpNode::Local(factor.clone(), scalar_local(child, memo)))
    } else {
        prepared(child, memo).clone()
    }
}

impl<I: Eq, O: Identity, C: Clone> OpExpr<I, O, C> {
    pub(crate) fn group_local(&self) -> Self {
        if matches!(self.node(), OpNode::Local(_, _) | OpNode::Identity) {
            return self.clone();
        }
        if !may_group(self) {
            return self.clone();
        }
        let mut memo: Memo<I, O, C> = BTreeMap::new();
        let mut pending = vec![(self, false)];
        while let Some((op, visited)) = pending.pop() {
            // Leaves already carry their classification; memoize containers only.
            if matches!(op.node(), OpNode::Local(_, _) | OpNode::Identity) {
                continue;
            }
            let key = ptr::from_ref(op.node());
            if memo.contains_key(&key) {
                continue;
            }
            let operands = children(op);
            if !visited && !operands.is_empty() {
                pending.push((op, true));
                pending.extend(operands.iter().rev().map(|child| (child, false)));
                continue;
            }
            let mut support = Support::Scalar;
            let prefix_len = operands
                .iter()
                .take_while(|child| match child.node() {
                    OpNode::Local(factor, _) => support.include_factor(factor),
                    OpNode::Identity => true,
                    _ => support.include(&memo[&ptr::from_ref(child.node())].support),
                })
                .count();
            let mut prefix = None;
            let mut remainder = operands;
            if prefix_len != operands.len() {
                // An ordered product uses an implicit left fold. Keep the local
                // prefix that could be grouped before the first distinct factor.
                if let (OpNode::Product(_), Support::One(factor)) = (op.node(), &support) {
                    if prefix_len > 1 {
                        let (leading, rest) = operands.split_at(prefix_len);
                        prefix = Some(product(
                            leading
                                .iter()
                                .map(|child| prepared_on_factor(child, factor, &mut memo))
                                .collect(),
                        ));
                        remainder = rest;
                    }
                }
                support = Support::Many;
            }
            let replacement = match &support {
                Support::Scalar => None,
                Support::One(factor) => {
                    let mut parts = operands
                        .iter()
                        .map(|child| prepared_on_factor(child, factor, &mut memo));
                    Some(match op.node() {
                        OpNode::Sum(_) => sum(parts.collect()),
                        OpNode::Product(_) => product(parts.collect()),
                        OpNode::Scale(s) => {
                            let child = parts.next().expect("scale operand");
                            let OpNode::Local(_, local) = child.node() else {
                                unreachable!("single-factor child")
                            };
                            Self::new(OpNode::Local(
                                factor.clone(),
                                local.scaled(s.coefficient.clone()),
                            ))
                        }
                        OpNode::Adjoint(_) => {
                            let child = parts.next().expect("adjoint operand");
                            let OpNode::Local(_, local) = child.node() else {
                                unreachable!("single-factor child")
                            };
                            Self::new(OpNode::Local(factor.clone(), local.adjoint()))
                        }
                        OpNode::Identity | OpNode::Local(_, _) => {
                            unreachable!("container expected")
                        }
                    })
                }
                Support::Many => {
                    if prefix.is_none()
                        && operands
                            .iter()
                            .all(|before| ptr::eq(before.node(), prepared(before, &memo).node()))
                    {
                        None
                    } else {
                        let mut parts = prefix
                            .into_iter()
                            .chain(remainder.iter().map(|child| prepared(child, &memo).clone()));
                        Some(match op.node() {
                            OpNode::Sum(_) => sum(parts.collect()),
                            OpNode::Product(_) => product(parts.collect()),
                            OpNode::Scale(s) => Self::new(OpNode::Scale(ScaleExpr {
                                coefficient: s.coefficient.clone(),
                                operand: parts.next().unwrap(),
                            })),
                            OpNode::Adjoint(_) => Self::new(OpNode::Adjoint(AdjointExpr {
                                operand: parts.next().unwrap(),
                            })),
                            _ => unreachable!("multi-factor container"),
                        })
                    }
                }
            };
            memo.insert(
                key,
                Grouped {
                    support,
                    replacement,
                    scalar_local: None,
                },
            );
        }
        memo.remove(&ptr::from_ref(self.node()))
            .expect("grouped root")
            .replacement
            .unwrap_or_else(|| self.clone())
    }
}

impl<I: Eq, O: Identity, C: Clone> Operator<I, O, C> {
    /// Groups single-factor subtrees into local operator syntax.
    ///
    /// Preserves the binding, factor references, and operand order without
    /// evaluating primitives, coefficients, or graded signs. Unbound operators
    /// are accepted; ambient-space validation remains at binding/evaluation.
    /// Scalar-only roots stay scalar. No distributive factoring or exchange of
    /// different factors is performed. The input is unchanged, and grouping an
    /// already grouped expression makes no further structural changes.
    ///
    /// ```
    /// use syntaqs_core::{space::HilbertSpace, algebra::Identity, expression::local::LocalOperator, expression::composite::Operator};
    /// struct Gate;
    /// impl Identity for Gate {
    ///     fn identity() -> Self { Self }
    /// }
    /// let site = HilbertSpace::factor(0u8);
    /// let gate = LocalOperator::<Gate, f64>::primitive(Gate).at(&site)?;
    /// let expression = (Operator::scalar(2.0) + gate.scaled(3.0))?;
    /// let grouped = expression.group_local();
    /// assert!(grouped.space().is_none());
    /// // No primitive product or adjoint implementation is needed.
    /// # Ok::<(), syntaqs_core::expression::error::ExpressionError>(())
    /// ```
    pub fn group_local(&self) -> Self {
        Self {
            expression: self.expression.group_local(),
            ambient: self.ambient.clone(),
        }
    }
}

#[cfg(test)]
mod tests;
