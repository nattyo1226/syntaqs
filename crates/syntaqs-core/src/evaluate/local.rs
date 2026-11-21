use alloc::{collections::BTreeMap, sync::Arc, vec, vec::Vec};
use core::{ops::Mul, ptr};
use num_traits::{One, Zero};

use super::ProductAction;
use crate::expression::local::{LocalOperator, LocalState, OperatorNode, StateNode};
use crate::{
    Act, Adjoint, Conjugate, Product,
    coefficient::{conjugate, multiply},
};

// Only evaluator tasks construct this wrapper. Each operand is a primitive;
// coefficients are collected, but implicit one needs no coefficient equality.
type Polynomial<S, O, C> = Vec<(Option<C>, LocalState<S, O, C>)>;
pub(super) struct Ready<S, O, C>(pub(super) Arc<Polynomial<S, O, C>>);
impl<S, O, C> Clone for Ready<S, O, C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub(super) fn times<C: Mul<Output = C>>(a: Option<C>, b: Option<C>) -> Option<C> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a * b),
        (a, None) => a,
        (None, b) => b,
    }
}

impl<S: PartialEq, O, C: Clone + Zero + One> Ready<S, O, C> {
    pub(super) fn expression(&self) -> LocalState<S, O, C> {
        LocalState::sum(self.0.iter().map(|(c, p)| match c {
            Some(c) => p.scaled(c.clone()),
            None => p.clone(),
        }))
    }
    pub(super) fn scaled(&self, c: C) -> Self {
        Self::finish(
            self.0
                .iter()
                .map(|(inner, p)| (times(Some(c.clone()), inner.clone()), p.clone()))
                .collect(),
        )
    }
    pub(super) fn collect(
        terms: impl IntoIterator<Item = (Option<C>, LocalState<S, O, C>)>,
    ) -> Self {
        let mut collected = Vec::new();
        for (c, p) in terms {
            Self::insert(&mut collected, c, p);
        }
        Self::finish(collected)
    }
    fn insert(terms: &mut Polynomial<S, O, C>, c: Option<C>, p: LocalState<S, O, C>) {
        if c.as_ref().is_some_and(Zero::is_zero) {
            return;
        }
        let StateNode::Primitive(value) = p.node() else {
            unreachable!("normalized primitive")
        };
        if let Some((previous, _)) = terms.iter_mut().find(|(_, other)| {
            let StateNode::Primitive(other) = other.node() else {
                unreachable!()
            };
            other == value
        }) {
            *previous = Some(previous.take().unwrap_or_else(C::one) + c.unwrap_or_else(C::one));
        } else {
            terms.push((c, p));
        }
    }
    fn finish(mut terms: Vec<(Option<C>, LocalState<S, O, C>)>) -> Self {
        terms.retain(|(c, _)| !c.as_ref().is_some_and(Zero::is_zero));
        Self(Arc::new(terms))
    }
}

enum StateTask<S, O, C> {
    Evaluate(LocalState<S, O, C>),
    Remember(LocalState<S, O, C>),
    Apply(Ready<S, O, C>, LocalOperator<O, C>, bool),
    ThenApply(LocalOperator<O, C>),
    ContinueProduct(ProductAction<LocalOperator<O, C>>),
    Scale(C),
    SumStart,
    SumAdd,
    SumEnd,
}

fn apply_product<S, O, C>(
    tasks: &mut Vec<StateTask<S, O, C>>,
    state: Ready<S, O, C>,
    product: ProductAction<LocalOperator<O, C>>,
) {
    let (operand, adjoint, remaining) = product.advance();
    if let Some(remaining) = remaining {
        tasks.push(StateTask::ContinueProduct(remaining));
    }
    tasks.push(StateTask::Apply(state, operand, adjoint));
}

// Keeping the source handle in the memo prevents address reuse, even when this
// context is reused for several placed local actions in one composite query.
type Memo<S, O, C> = BTreeMap<*const StateNode<S, O, C>, (LocalState<S, O, C>, Ready<S, O, C>)>;
pub(super) struct Evaluator<S, O, C> {
    memo: Memo<S, O, C>,
}
impl<
    S: PartialEq + Act<C, Operator = O, Term = S>,
    O: Adjoint<C, Term = O>,
    C: Clone + Zero + One + Conjugate<Output = C>,
> Evaluator<S, O, C>
{
    pub(super) fn new() -> Self {
        Self {
            memo: BTreeMap::new(),
        }
    }
    pub(super) fn evaluate(&mut self, state: &LocalState<S, O, C>) -> Ready<S, O, C> {
        self.run(StateTask::Evaluate(state.clone()))
    }
    pub(super) fn apply(
        &mut self,
        state: Ready<S, O, C>,
        op: LocalOperator<O, C>,
    ) -> Ready<S, O, C> {
        if state.0.is_empty() {
            state
        } else {
            self.run(StateTask::Apply(state, op, false))
        }
    }
    fn run(&mut self, first: StateTask<S, O, C>) -> Ready<S, O, C> {
        let mut tasks = vec![first];
        let mut values: Vec<Ready<S, O, C>> = Vec::new();
        let mut sums: Vec<Polynomial<S, O, C>> = Vec::new();
        while let Some(task) = tasks.pop() {
            match task {
                StateTask::Evaluate(state) => {
                    let key = ptr::from_ref(state.node());
                    if let Some((_, ready)) = self.memo.get(&key) {
                        values.push(ready.clone());
                        continue;
                    }
                    tasks.push(StateTask::Remember(state.clone()));
                    match state.node() {
                        StateNode::Primitive(_) => {
                            values.push(Ready(Arc::new(vec![(None, state)])))
                        }
                        StateNode::Scale(s) => {
                            tasks.push(StateTask::Scale(s.coefficient.clone()));
                            tasks.push(StateTask::Evaluate(s.operand.clone()));
                        }
                        StateNode::Sum(s) => {
                            tasks.push(StateTask::SumEnd);
                            for child in s.iter().rev() {
                                tasks.push(StateTask::SumAdd);
                                tasks.push(StateTask::Evaluate(child.clone()));
                            }
                            tasks.push(StateTask::SumStart);
                        }
                        StateNode::Act(a) => {
                            tasks.push(StateTask::ThenApply(a.operator.clone()));
                            tasks.push(StateTask::Evaluate(a.state.clone()));
                        }
                    }
                }
                StateTask::Remember(source) => {
                    self.memo.insert(
                        ptr::from_ref(source.node()),
                        (source, values.last().unwrap().clone()),
                    );
                }
                StateTask::ThenApply(op) => {
                    if !values.last().unwrap().0.is_empty() {
                        tasks.push(StateTask::Apply(values.pop().unwrap(), op, false));
                    }
                }
                StateTask::ContinueProduct(product) => {
                    if !values.last().unwrap().0.is_empty() {
                        apply_product(&mut tasks, values.pop().unwrap(), product);
                    }
                }
                StateTask::Apply(state, op, adjoint) => match op.node() {
                    OperatorNode::Scale(s) => {
                        let c = s.coefficient.clone();
                        tasks.push(StateTask::Scale(if adjoint { conjugate(c) } else { c }));
                        tasks.push(StateTask::Apply(state, s.operand.clone(), adjoint));
                    }
                    OperatorNode::Sum(s) => {
                        tasks.push(StateTask::SumEnd);
                        for child in s.iter().rev() {
                            tasks.push(StateTask::SumAdd);
                            tasks.push(StateTask::Apply(state.clone(), child.clone(), adjoint));
                        }
                        tasks.push(StateTask::SumStart);
                    }
                    OperatorNode::Product(_) => {
                        apply_product(&mut tasks, state, ProductAction::new(op, adjoint));
                    }
                    OperatorNode::Adjoint(a) => {
                        tasks.push(StateTask::Apply(state, a.operand.clone(), !adjoint))
                    }
                    OperatorNode::Primitive(p) if adjoint => tasks.push(StateTask::Apply(
                        state,
                        LocalOperator::from_terms(p.adjoint().map(LocalOperator::primitive)),
                        false,
                    )),
                    OperatorNode::Primitive(op) => {
                        let mut terms = Vec::new();
                        for (outer, p) in state.0.iter() {
                            let StateNode::Primitive(p) = p.node() else {
                                unreachable!()
                            };
                            for term in p.act(op) {
                                Ready::insert(
                                    &mut terms,
                                    times(outer.clone(), Some(term.coefficient)),
                                    LocalState::primitive(term.operand),
                                );
                            }
                        }
                        values.push(Ready::finish(terms));
                    }
                },
                StateTask::Scale(c) => {
                    let state = values.pop().unwrap();
                    values.push(state.scaled(c));
                }
                StateTask::SumStart => sums.push(Vec::new()),
                StateTask::SumAdd => {
                    let state = values.pop().unwrap();
                    for (c, p) in state.0.iter() {
                        Ready::insert(sums.last_mut().unwrap(), c.clone(), p.clone());
                    }
                }
                StateTask::SumEnd => values.push(Ready::finish(sums.pop().unwrap())),
            }
        }
        values.pop().expect("one normalized result")
    }
}

impl<S, O, C> LocalState<S, O, C> {
    /// Evaluates actions and collects PartialEq-matching primitive terms after
    /// every action, without choosing a basis or requiring operator Product.
    /// Action-free inputs are collected too. Only exact-zero coefficients are
    /// removed; primitive linear dependence is left to basis expansion.
    pub fn evaluate(&self) -> Self
    where
        C: Clone + Zero + One + Conjugate<Output = C>,
        S: PartialEq + Act<C, Operator = O, Term = S>,
        O: Adjoint<C, Term = O>,
    {
        if matches!(self.node(), StateNode::Primitive(_)) {
            return self.clone();
        }
        Evaluator::new().evaluate(self).expression()
    }
}

enum OperatorTask<O, C> {
    Visit(LocalOperator<O, C>, bool),
    Multiply(LocalOperator<O, C>, LocalOperator<O, C>),
    Product,
    Scale(C),
    Sum(usize),
}

impl<O, C> LocalOperator<O, C> {
    pub(crate) fn evaluate(&self) -> Self
    where
        C: Clone + Mul<Output = C> + Conjugate<Output = C>,
        O: Product<C, Term = O> + Adjoint<C, Term = O>,
    {
        let mut tasks = vec![OperatorTask::Visit(self.clone(), false)];
        let mut values: Vec<Self> = Vec::new();
        while let Some(task) = tasks.pop() {
            match task {
                OperatorTask::Visit(op, adjoint) => match op.node() {
                    OperatorNode::Primitive(p) => {
                        if adjoint {
                            values.push(Self::from_terms(p.adjoint().map(Self::primitive)));
                        } else {
                            values.push(op);
                        }
                    }
                    OperatorNode::Scale(s) => {
                        let c = s.coefficient.clone();
                        tasks.push(OperatorTask::Scale(if adjoint { conjugate(c) } else { c }));
                        tasks.push(OperatorTask::Visit(s.operand.clone(), adjoint));
                    }
                    OperatorNode::Sum(s) => {
                        tasks.push(OperatorTask::Sum(s.len()));
                        for child in s.iter().rev() {
                            tasks.push(OperatorTask::Visit(child.clone(), adjoint));
                        }
                    }
                    OperatorNode::Adjoint(a) => {
                        tasks.push(OperatorTask::Visit(a.operand.clone(), !adjoint))
                    }
                    OperatorNode::Product(p) => {
                        if adjoint {
                            // Reverse the operand order and the left-associated
                            // grouping together, as in (AB)† = B†A†.
                            tasks.extend((1..p.len()).map(|_| OperatorTask::Product));
                            tasks.extend(
                                p.iter()
                                    .map(|child| OperatorTask::Visit(child.clone(), true)),
                            );
                        } else {
                            let (first, rest) =
                                p.as_slice().split_first().expect("product operand");
                            for child in rest.iter().rev() {
                                tasks.push(OperatorTask::Product);
                                tasks.push(OperatorTask::Visit(child.clone(), false));
                            }
                            tasks.push(OperatorTask::Visit(first.clone(), false));
                        }
                    }
                },
                OperatorTask::Product => {
                    let rhs = values.pop().expect("right product operand");
                    let lhs = values.pop().expect("left product operand");
                    tasks.push(OperatorTask::Multiply(lhs, rhs));
                }
                OperatorTask::Multiply(lhs, rhs) => match (lhs.node(), rhs.node()) {
                    (OperatorNode::Scale(s), _) => {
                        tasks.push(OperatorTask::Scale(s.coefficient.clone()));
                        tasks.push(OperatorTask::Multiply(s.operand.clone(), rhs));
                    }
                    (OperatorNode::Sum(s), _) => {
                        tasks.push(OperatorTask::Sum(s.len()));
                        for child in s.iter().rev() {
                            tasks.push(OperatorTask::Multiply(child.clone(), rhs.clone()));
                        }
                    }
                    (_, OperatorNode::Scale(s)) => {
                        tasks.push(OperatorTask::Scale(s.coefficient.clone()));
                        tasks.push(OperatorTask::Multiply(lhs, s.operand.clone()));
                    }
                    (_, OperatorNode::Sum(s)) => {
                        tasks.push(OperatorTask::Sum(s.len()));
                        for child in s.iter().rev() {
                            tasks.push(OperatorTask::Multiply(lhs.clone(), child.clone()));
                        }
                    }
                    (OperatorNode::Primitive(a), OperatorNode::Primitive(b)) => {
                        values.push(Self::from_terms(a.product(b).map(Self::primitive)));
                    }
                    _ => unreachable!("linear operator operands"),
                },
                OperatorTask::Scale(coefficient) => {
                    let operand = values.pop().expect("scale operand");
                    // Combining adjacent evaluated scales avoids quadratic
                    // rewalking of coefficients in long single-term products.
                    let value = if let OperatorNode::Scale(s) = operand.node() {
                        s.operand
                            .scaled(multiply(coefficient, s.coefficient.clone()))
                    } else {
                        operand.into_scaled(coefficient)
                    };
                    values.push(value);
                }
                OperatorTask::Sum(count) => {
                    let children = values.split_off(values.len() - count);
                    values.push(Self::sum(children));
                }
            }
        }
        values.pop().expect("one operator result")
    }
}
