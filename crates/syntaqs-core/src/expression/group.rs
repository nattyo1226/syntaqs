//! Structural local grouping, without primitive evaluation or factor exchange.

use alloc::{collections::BTreeMap, vec};
use core::ptr;

use super::{
    composite::{OpExpr, OpNode, Operator},
    parts::{AdjointExpr, ProductExpr, ScaleExpr, SumExpr},
};
use crate::{HilbertSpace, Identity, LocalOperator};

fn same_factor<I: Eq>(a: &HilbertSpace<I>, b: &HilbertSpace<I>) -> bool {
    a.factor_index() == b.factor_index()
}

// These two constructors are shared by cheap construction and full grouping.
// They inspect only immediate children and add no bounds on O or C.
pub(super) fn sum<I: Eq, O, C>(parts: SumExpr<OpExpr<I, O, C>>) -> OpExpr<I, O, C> {
    if let SumExpr::One(op) = parts {
        return op;
    }
    if let Some(OpNode::Local(factor, _)) = parts.iter().next().map(OpExpr::node) {
        if parts
            .iter()
            .all(|op| matches!(op.node(), OpNode::Local(other, _) if same_factor(factor, other)))
        {
            let local = LocalOperator::sum(parts.iter().map(|op| {
                let OpNode::Local(_, local) = op.node() else {
                    unreachable!("checked local operands")
                };
                local.clone()
            }));
            return OpExpr::new(OpNode::Local(factor.clone(), local));
        }
    }
    OpExpr::new(OpNode::Sum(parts))
}

pub(super) fn product<I: Eq, O, C>(lhs: OpExpr<I, O, C>, rhs: OpExpr<I, O, C>) -> OpExpr<I, O, C> {
    if let (OpNode::Local(a, x), OpNode::Local(b, y)) = (lhs.node(), rhs.node()) {
        if same_factor(a, b) {
            return OpExpr::new(OpNode::Local(a.clone(), x * y));
        }
    }
    OpExpr::new(OpNode::Product(ProductExpr { lhs, rhs }))
}

enum Support<I> {
    Scalar,
    One(HilbertSpace<I>),
    Many,
}

impl<I: Eq> Support<I> {
    fn include_factor(&mut self, factor: &HilbertSpace<I>) {
        match self {
            Self::Scalar => *self = Self::One(factor.clone()),
            Self::One(previous) if !same_factor(previous, factor) => *self = Self::Many,
            _ => {}
        }
    }

    fn include(&mut self, other: &Self) {
        match other {
            Self::Scalar => {}
            Self::One(factor) => self.include_factor(factor),
            Self::Many => *self = Self::Many,
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

fn children<I, O, C>(
    op: &OpExpr<I, O, C>,
) -> impl DoubleEndedIterator<Item = &OpExpr<I, O, C>> + Clone {
    let (first, second, rest) = match op.node() {
        OpNode::Scale(s) => (Some(&s.operand), None, &[][..]),
        OpNode::Sum(s) => (None, None, s.as_slice()),
        OpNode::Product(p) => (Some(&p.lhs), Some(&p.rhs), &[][..]),
        OpNode::Adjoint(a) => (Some(&a.operand), None, &[][..]),
        OpNode::Identity | OpNode::Local(_, _) => (None, None, &[][..]),
    };
    first.into_iter().chain(second).chain(rest.iter())
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
            OpNode::Product(p) => match (p.lhs.node(), p.rhs.node()) {
                (OpNode::Identity, _) | (_, OpNode::Identity) => return true,
                (OpNode::Local(a, _), OpNode::Local(b, _)) if same_factor(a, b) => {
                    return true;
                }
                _ => {}
            },
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
            pending.extend(children(op).rev().map(|child| (child, false)));
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
            OpNode::Product(p) => get(&p.lhs) * get(&p.rhs),
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
            if !visited && operands.clone().next().is_some() {
                pending.push((op, true));
                pending.extend(operands.clone().rev().map(|child| (child, false)));
                continue;
            }
            let mut support = Support::Scalar;
            for child in operands.clone() {
                match child.node() {
                    OpNode::Local(factor, _) => support.include_factor(factor),
                    OpNode::Identity => {}
                    _ => support.include(&memo[&ptr::from_ref(child.node())].support),
                }
                if matches!(support, Support::Many) {
                    break;
                }
            }
            let replacement = match &support {
                Support::Scalar => None,
                Support::One(factor) => {
                    let mut parts = operands.map(|child| {
                        let child_key = ptr::from_ref(child.node());
                        let scalar = match child.node() {
                            OpNode::Identity => true,
                            OpNode::Local(_, _) => false,
                            _ => matches!(memo[&child_key].support, Support::Scalar),
                        };
                        if scalar {
                            let local = scalar_local(child, &mut memo);
                            OpExpr::new(OpNode::Local(factor.clone(), local))
                        } else {
                            prepared(child, &memo).clone()
                        }
                    });
                    Some(match op.node() {
                        OpNode::Sum(_) => sum(parts.collect()),
                        OpNode::Product(_) => {
                            let lhs = parts.next().expect("left operand");
                            product(lhs, parts.next().expect("right operand"))
                        }
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
                    if operands
                        .clone()
                        .all(|before| ptr::eq(before.node(), prepared(before, &memo).node()))
                    {
                        None
                    } else {
                        let mut parts = operands.map(|child| prepared(child, &memo).clone());
                        Some(match op.node() {
                            OpNode::Sum(_) => sum(parts.collect()),
                            OpNode::Product(_) => {
                                product(parts.next().unwrap(), parts.next().unwrap())
                            }
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
    /// use syntaqs_core::{HilbertSpace, Identity, LocalOperator, Operator};
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
    /// # Ok::<(), syntaqs_core::ExpressionError>(())
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
