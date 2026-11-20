use alloc::{collections::BTreeMap, vec, vec::Vec};
use core::ptr;
use num_traits::{One, Zero};

use super::{
    local::Evaluator as LocalEvaluator,
    schedule_product_action,
    state_terms::{Key, Ready, Sum, Terms},
};
use crate::expression::composite::{OpExpr, OpNode, State, StateExpr, StateNode};
use crate::expression::error::ExpressionError;
use crate::{
    Act, Adjoint, Conjugate, HilbertSpace, HomogeneousParity, LocalOperator, Parity,
    ParityOperator, TensorKind, coefficient::conjugate,
};

enum Task<I, S, O, C> {
    Evaluate(StateExpr<I, S, O, C>),
    Remember(StateExpr<I, S, O, C>),
    Apply(Ready<C>, OpExpr<I, O, C>, bool, HilbertSpace<I>),
    ThenApply(OpExpr<I, O, C>, bool, HilbertSpace<I>),
    Local(
        Ready<C>,
        HilbertSpace<I>,
        LocalOperator<O, C>,
        Option<HilbertSpace<I>>,
    ),
    Placed(
        Ready<C>,
        HilbertSpace<I>,
        LocalOperator<O, C>,
        bool,
        HilbertSpace<I>,
    ),
    Scale(C),
    SumStart,
    SumAdd,
    SumEnd,
    Tensor(usize),
}
fn schedule_sum<I, S, O, C>(
    tasks: &mut Vec<Task<I, S, O, C>>,
    children: impl DoubleEndedIterator<Item = Task<I, S, O, C>>,
) {
    tasks.push(Task::SumEnd);
    for child in children.rev() {
        tasks.push(Task::SumAdd);
        tasks.push(child);
    }
    tasks.push(Task::SumStart);
}
impl<I: Eq, S, O, C> State<I, S, O, C> {
    /// Evaluates delayed actions and collects matching natural-state terms
    /// between actions. Action-free inputs are normalized too; tensor products
    /// of sums stay factored. No basis or primitive operator Product is required.
    pub fn evaluate(&self) -> Result<Self, ExpressionError>
    where
        C: Clone + PartialEq + Zero + One + Conjugate<Output = C>,
        S: PartialEq + Act<C, Operator = O, Term = S>,
        O: Adjoint<C, Term = O> + HomogeneousParity + ParityOperator,
    {
        // These shapes are intrinsically normalized; inspecting direct leaves
        // needs no persistent certificate or equality shortcut between terms.
        let primitive = |state: &StateExpr<I, S, O, C>| matches!(state.node(),StateNode::Local(_,local) if matches!(local.node(),crate::expression::local::StateNode::Primitive(_)));
        if matches!(self.expression.node(), StateNode::Unit)
            || primitive(&self.expression)
            || matches!(self.expression.node(),StateNode::Tensor(parts) if parts.len()>1 && parts.iter().all(primitive))
        {
            return Ok(self.clone());
        }
        let mut tasks = vec![Task::Evaluate(self.expression.clone())];
        let mut values: Vec<Ready<C>> = Vec::new();
        let mut memo = BTreeMap::new();
        let mut sums = Vec::new();
        let mut terms = Terms::new();
        let mut locals = LocalEvaluator::new();
        while let Some(task) = tasks.pop() {
            match task {
                Task::Evaluate(state) => {
                    let key = ptr::from_ref(state.node());
                    if let Some(ready) = memo.get(&key) {
                        values.push(Ready::clone(ready));
                        continue;
                    }
                    tasks.push(Task::Remember(state.clone()));
                    match state.node() {
                        StateNode::Local(factor, local) => {
                            let local = locals.evaluate(local);
                            values.push(terms.local(factor.clone(), local));
                        }
                        StateNode::Unit => values.push(terms.unit()),
                        StateNode::Scale(s) => {
                            tasks.push(Task::Scale(s.coefficient.clone()));
                            tasks.push(Task::Evaluate(s.operand.clone()));
                        }
                        StateNode::Sum(s) | StateNode::Tensor(s) => {
                            if matches!(state.node(), StateNode::Tensor(_)) {
                                tasks.push(Task::Tensor(s.len()));
                                for child in s.iter().rev() {
                                    tasks.push(Task::Evaluate(child.clone()));
                                }
                            } else {
                                schedule_sum(
                                    &mut tasks,
                                    s.iter().map(|child| Task::Evaluate(child.clone())),
                                );
                            }
                        }
                        StateNode::Act(a) => {
                            tasks.push(Task::ThenApply(a.operator.clone(), false, a.space.clone()));
                            tasks.push(Task::Evaluate(a.state.clone()));
                        }
                    }
                }
                Task::Remember(source) => {
                    terms.reuse(values.last().unwrap(), &source);
                    memo.insert(ptr::from_ref(source.node()), values.last().unwrap().clone());
                }
                Task::ThenApply(op, adjoint, scope) => {
                    tasks.push(Task::Apply(values.pop().unwrap(), op, adjoint, scope))
                }
                Task::Apply(state, op, adjoint, scope) => match op.node() {
                    OpNode::Identity => values.push(state),
                    OpNode::Local(factor, local) => tasks.push(Task::Placed(
                        state,
                        factor.clone(),
                        local.clone(),
                        adjoint,
                        scope,
                    )),
                    OpNode::Scale(s) => {
                        tasks.push(Task::Scale(if adjoint {
                            conjugate(s.coefficient.clone())
                        } else {
                            s.coefficient.clone()
                        }));
                        tasks.push(Task::Apply(state, s.operand.clone(), adjoint, scope));
                    }
                    OpNode::Sum(s) => {
                        tasks.push(Task::SumEnd);
                        for child in s.iter().rev() {
                            tasks.push(Task::SumAdd);
                            tasks.push(Task::Apply(
                                state.clone(),
                                child.clone(),
                                adjoint,
                                scope.clone(),
                            ));
                        }
                        tasks.push(Task::SumStart);
                    }
                    OpNode::Product(p) => {
                        let first = schedule_product_action(p, adjoint, &mut tasks, |operand| {
                            Task::ThenApply(operand.clone(), adjoint, scope.clone())
                        });
                        tasks.push(Task::Apply(state, first.clone(), adjoint, scope));
                    }
                    OpNode::Adjoint(a) => {
                        tasks.push(Task::Apply(state, a.operand.clone(), !adjoint, scope))
                    }
                },
                Task::Placed(state, factor, operator, adjoint, scope) => {
                    // Ordinary (including unit/single-factor) action can retain
                    // a whole local expression without distributing its sums.
                    if scope.effective_kind() != Some(TensorKind::Graded) {
                        let operator = if adjoint {
                            operator.adjoint()
                        } else {
                            operator
                        };
                        tasks.push(Task::Local(state, factor, operator, None));
                        continue;
                    }
                    use crate::expression::local::OperatorNode;
                    match operator.node() {
                        OperatorNode::Scale(s) => {
                            tasks.push(Task::Scale(if adjoint {
                                conjugate(s.coefficient.clone())
                            } else {
                                s.coefficient.clone()
                            }));
                            tasks.push(Task::Placed(
                                state,
                                factor,
                                s.operand.clone(),
                                adjoint,
                                scope,
                            ));
                        }
                        OperatorNode::Sum(s) => {
                            tasks.push(Task::SumEnd);
                            for child in s.iter().rev() {
                                tasks.push(Task::SumAdd);
                                tasks.push(Task::Placed(
                                    state.clone(),
                                    factor.clone(),
                                    child.clone(),
                                    adjoint,
                                    scope.clone(),
                                ));
                            }
                            tasks.push(Task::SumStart);
                        }
                        OperatorNode::Product(p) => {
                            let first =
                                schedule_product_action(p, adjoint, &mut tasks, |operand| {
                                    let placed =
                                        OpExpr::new(OpNode::Local(factor.clone(), operand.clone()));
                                    Task::ThenApply(placed, adjoint, scope.clone())
                                });
                            tasks.push(Task::Placed(state, factor, first.clone(), adjoint, scope));
                        }
                        OperatorNode::Adjoint(a) => tasks.push(Task::Placed(
                            state,
                            factor,
                            a.operand.clone(),
                            !adjoint,
                            scope,
                        )),
                        OperatorNode::Primitive(p) if adjoint => {
                            let local = LocalOperator::from_terms(
                                p.adjoint().map(LocalOperator::primitive),
                            );
                            tasks.push(Task::Placed(state, factor, local, false, scope));
                        }
                        OperatorNode::Primitive(p) => {
                            let prefix = if scope.effective_kind() == Some(TensorKind::Graded)
                                && p.parity() == Parity::Odd
                            {
                                Some(scope)
                            } else {
                                None
                            };
                            tasks.push(Task::Local(state, factor, operator, prefix));
                        }
                    }
                }
                Task::Local(state, factor, operator, prefix) => {
                    if state.shape == 0 {
                        values.push(state);
                        continue;
                    }
                    if let Some((target, local)) = terms.local_parts(state.shape) {
                        let op = if target.factor_index() == factor.factor_index() {
                            Some(operator)
                        } else if prefix.as_ref().is_some_and(|space| {
                            space
                                .indices()
                                .take_while(|i| Some(*i) != factor.factor_index())
                                .any(|i| Some(i) == target.factor_index())
                        }) {
                            Some(LocalOperator::primitive(O::parity_operator()))
                        } else {
                            None
                        };
                        if let Some(op) = op {
                            let local = locals.apply(local, op);
                            let mut ready = terms.local(target, local);
                            if let Some(c) = state.coefficient {
                                ready = terms.scaled(ready, c);
                            }
                            values.push(ready);
                        } else {
                            values.push(state);
                        }
                    } else if matches!(terms.key(state.shape), Key::Unit) {
                        values.push(state);
                    } else {
                        let children = terms.children(state.shape);
                        if let Some(c) = state.coefficient {
                            tasks.push(Task::Scale(c));
                        }
                        let branches = children.into_iter().map(|child| {
                            Task::Local(child, factor.clone(), operator.clone(), prefix.clone())
                        });
                        if matches!(terms.key(state.shape), Key::Tensor(_)) {
                            tasks.push(Task::Tensor(branches.len()));
                            tasks.extend(branches.rev());
                        } else {
                            schedule_sum(&mut tasks, branches);
                        }
                    }
                }
                Task::Scale(c) => {
                    let ready = values.pop().unwrap();
                    values.push(terms.scaled(ready, c));
                }
                Task::SumStart => sums.push(Sum::new()),
                Task::SumAdd => terms.add(sums.last_mut().unwrap(), values.pop().unwrap()),
                Task::SumEnd => values.push(terms.finish_sum(sums.pop().unwrap())),
                Task::Tensor(count) => {
                    let children = values.split_off(values.len() - count);
                    values.push(terms.tensor(children));
                }
            }
        }
        Ok(Self {
            space: self.space.clone(),
            expression: terms.expression(&values.pop().unwrap()),
        })
    }
}
