//! Contraction of evaluated states without distributing independent factors.

use alloc::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
    vec,
    vec::Vec,
};
use core::{
    ops::{Mul, Range},
    ptr, slice,
};
use num_traits::{One, Zero};

use super::{add_pair, composite::Evaluator};
use crate::expression::{
    composite::{StateExpr, StateNode},
    traversal::{LinearTerms, state_view},
};
use crate::{
    Act, Adjoint, Conjugate, ExpressionError, HilbertSpace, InnerProduct, LocalOperator,
    LocalState, Operator, RealPart, State,
    coefficient::{add, conjugate, multiply, zero},
};

type Pair<'a, I, S, O, C> = (Factors<'a, I, S, O, C>, Factors<'a, I, S, O, C>);

enum Factors<'a, I, S, O, C> {
    Slice(&'a [StateExpr<I, S, O, C>]),
    Joined(Rc<[&'a StateExpr<I, S, O, C>]>, Range<usize>),
}

impl<'a, I, S, O, C> Factors<'a, I, S, O, C> {
    fn len(&self) -> usize {
        match self {
            Self::Slice(states) => states.len(),
            Self::Joined(_, positions) => positions.len(),
        }
    }

    fn get(&self, index: usize) -> Option<&'a StateExpr<I, S, O, C>> {
        match self {
            Self::Slice(states) => states.get(index),
            Self::Joined(states, positions) => states[positions.clone()].get(index).copied(),
        }
    }

    fn only(&self) -> Option<&'a StateExpr<I, S, O, C>> {
        self.get(0).filter(|_| self.len() == 1)
    }

    fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = &'a StateExpr<I, S, O, C>> + use<'_, 'a, I, S, O, C> {
        (0..self.len()).map(|index| self.get(index).expect("factor index"))
    }

    fn range(&self, range: Range<usize>) -> Self {
        match self {
            Self::Slice(states) => Self::Slice(&states[range]),
            Self::Joined(_, _) if range.is_empty() => Self::Slice(&[]),
            Self::Joined(_, _) if range.len() == 1 => single(self.get(range.start).unwrap()),
            Self::Joined(states, positions) => Self::Joined(
                states.clone(),
                positions.start + range.start..positions.start + range.end,
            ),
        }
    }

    fn joined(states: Vec<&'a StateExpr<I, S, O, C>>) -> Self {
        match states.as_slice() {
            [] => Self::Slice(&[]),
            [state] => single(state),
            _ => {
                let size = states.len();
                Self::Joined(states.into(), 0..size)
            }
        }
    }

    fn replace(&self, state: &'a StateExpr<I, S, O, C>, index: usize) -> Self {
        if self.len() == 1 {
            return single(state);
        }
        let mut states: Vec<_> = self.iter().collect();
        states[index] = state;
        Self::joined(states)
    }
}

// None counts an empty sum, including tensors annihilated by an empty sum.
// A flat tensor can lend its factors directly to the alignment cursor.
struct Layout {
    count: Option<usize>,
    flat: bool,
}

struct FactorCounts<I, S, O, C>(BTreeMap<*const StateNode<I, S, O, C>, Layout>);

impl<I, S, O, C> FactorCounts<I, S, O, C> {
    fn new(lhs: &StateExpr<I, S, O, C>, rhs: &StateExpr<I, S, O, C>) -> Self {
        let mut counts = Self(BTreeMap::new());
        let mut pending = vec![(lhs, false), (rhs, false)];
        while let Some((state, visited)) = pending.pop() {
            if matches!(state.node(), StateNode::Local(_, _) | StateNode::Unit) {
                continue;
            }
            let key = ptr::from_ref(state.node());
            if counts.0.contains_key(&key) {
                continue;
            }
            if !visited {
                pending.push((state, true));
                match state.node() {
                    StateNode::Scale(s) => pending.push((&s.operand, false)),
                    StateNode::Sum(s) | StateNode::Tensor(s) => {
                        pending.extend(s.iter().map(|child| (child, false)));
                    }
                    _ => unreachable!("evaluated container"),
                }
                continue;
            }
            let mut flat = false;
            let count = match state.node() {
                StateNode::Scale(s) => counts.get(&s.operand),
                // All nonempty alternatives have the same ordered factor space.
                StateNode::Sum(s) => s.iter().find_map(|child| counts.get(child)),
                StateNode::Tensor(s) => {
                    flat = true;
                    s.iter().try_fold(0, |n, child| {
                        let count = counts.get(child)?;
                        flat &= count != 0
                            && matches!(child.node(), StateNode::Local(_, _) | StateNode::Sum(_));
                        Some(n + count)
                    })
                }
                _ => unreachable!("evaluated container"),
            };
            counts.0.insert(key, Layout { count, flat });
        }
        counts
    }

    fn get(&self, state: &StateExpr<I, S, O, C>) -> Option<usize> {
        match state.node() {
            StateNode::Local(_, _) => Some(1),
            StateNode::Unit => Some(0),
            node => self.0[&ptr::from_ref(node)].count,
        }
    }
}

enum Task<'a, I, S, O, C> {
    Contract(Pair<'a, I, S, O, C>),
    Block(Pair<'a, I, S, O, C>),
    Scale(&'a C, bool),
    Sum(usize),
    MatchingSum {
        terms: &'a [StateExpr<I, S, O, C>],
        row: usize,
        column: usize,
        accumulated: C,
    },
    Product {
        remaining: Alignment<'a, I, S, O, C>,
        accumulated: C,
    },
}

fn single<I, S, O, C>(state: &StateExpr<I, S, O, C>) -> Factors<'_, I, S, O, C> {
    Factors::Slice(slice::from_ref(state))
}

// Expose tensor factors without distributing sums. Normalized tensors borrow
// their operand slices. Newly joined branches can contain scales or tensors.
fn tensor_factors<'a, I, S, O, C: Zero>(
    states: Factors<'a, I, S, O, C>,
    counts: &FactorCounts<I, S, O, C>,
    tasks: &mut Vec<Task<'a, I, S, O, C>>,
    conjugated: bool,
) -> Option<Factors<'a, I, S, O, C>> {
    let states = match states.only() {
        Some(mut state) => {
            while let StateNode::Scale(s) = state.node() {
                if s.coefficient.is_zero() {
                    return None;
                }
                tasks.push(Task::Scale(&s.coefficient, conjugated));
                state = &s.operand;
            }
            match state.node() {
                StateNode::Unit => return Some(Factors::Slice(&[])),
                StateNode::Local(_, _) | StateNode::Sum(_) => return Some(single(state)),
                StateNode::Tensor(parts) if counts.0[&ptr::from_ref(state.node())].flat => {
                    return Some(Factors::Slice(parts.as_slice()));
                }
                _ => single(state),
            }
        }
        None => states,
    };
    let mut pending: Vec<_> = states.iter().rev().collect();
    let mut factors = Vec::new();
    let mut scalars = Vec::new();
    while let Some(state) = pending.pop() {
        match state.node() {
            StateNode::Tensor(s) => pending.extend(s.iter().rev()),
            StateNode::Unit => {}
            StateNode::Scale(s) => {
                if s.coefficient.is_zero() {
                    return None;
                }
                tasks.push(Task::Scale(&s.coefficient, conjugated));
                pending.push(&s.operand);
            }
            StateNode::Local(_, _) | StateNode::Sum(_) => {
                if counts.get(state) == Some(0) {
                    scalars.push(state);
                } else {
                    factors.push(state);
                }
            }
            StateNode::Act(_) => unreachable!("evaluated state"),
        }
    }
    scalars.extend(factors);
    Some(Factors::joined(scalars))
}

// Compatible root spaces establish the common factor order. Equal prefix
// lengths therefore identify the next shared non-scalar subsystem.
struct Alignment<'a, I, S, O, C> {
    lhs: Factors<'a, I, S, O, C>,
    rhs: Factors<'a, I, S, O, C>,
    left: usize,
    right: usize,
}

impl<'a, I, S, O, C> Alignment<'a, I, S, O, C> {
    fn new(lhs: Factors<'a, I, S, O, C>, rhs: Factors<'a, I, S, O, C>) -> Self {
        Self {
            lhs,
            rhs,
            left: 0,
            right: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.left == self.lhs.len() && self.right == self.rhs.len()
    }

    fn next(&mut self, counts: &FactorCounts<I, S, O, C>) -> Option<Pair<'a, I, S, O, C>> {
        if self.is_empty() {
            return None;
        }
        let span = |state: &StateExpr<I, S, O, C>| counts.get(state).expect("nonempty factor");
        let i = self.left;
        let j = self.right;
        // Unit-space sums precede indexed groups, left side first.
        if self.lhs.get(i).is_some_and(|state| span(state) == 0) {
            self.left += 1;
        } else if self.rhs.get(j).is_some_and(|state| span(state) == 0) {
            self.right += 1;
        } else {
            let mut left = span(self.lhs.get(i).expect("compatible factor spaces"));
            let mut right = span(self.rhs.get(j).expect("compatible factor spaces"));
            self.left += 1;
            self.right += 1;
            while left != right {
                if left < right {
                    left += span(self.lhs.get(self.left).expect("compatible factor spaces"));
                    self.left += 1;
                } else {
                    right += span(self.rhs.get(self.right).expect("compatible factor spaces"));
                    self.right += 1;
                }
            }
        }
        Some((self.lhs.range(i..self.left), self.rhs.range(j..self.right)))
    }
}

#[cfg(test)]
pub(crate) fn contract<I, S, O, C>(lhs: &StateExpr<I, S, O, C>, rhs: &StateExpr<I, S, O, C>) -> C
where
    C: Clone + Zero + One + Conjugate<Output = C>,
    S: InnerProduct<C>,
{
    contract_evaluated(lhs, rhs, |_| false)
}

fn contract_evaluated<I, S, O, C>(
    lhs: &StateExpr<I, S, O, C>,
    rhs: &StateExpr<I, S, O, C>,
    is_reflexive_sum: impl Fn(&StateExpr<I, S, O, C>) -> bool,
) -> C
where
    C: Clone + Zero + One + Conjugate<Output = C>,
    S: InnerProduct<C>,
{
    let counts = FactorCounts::new(lhs, rhs);
    let mut tasks = vec![Task::Contract((single(lhs), single(rhs)))];
    let mut values = Vec::new();
    while let Some(task) = tasks.pop() {
        match task {
            Task::Contract((lhs, rhs)) => {
                if lhs
                    .iter()
                    .chain(rhs.iter())
                    .any(|s| counts.get(s).is_none())
                {
                    values.push(zero());
                    continue;
                }
                let Some(lhs) = tensor_factors(lhs, &counts, &mut tasks, true) else {
                    values.push(zero());
                    continue;
                };
                let Some(rhs) = tensor_factors(rhs, &counts, &mut tasks, false) else {
                    values.push(zero());
                    continue;
                };
                let mut groups = Alignment::new(lhs, rhs);
                let Some((lhs, rhs)) = groups.next(&counts) else {
                    values.push(C::one());
                    continue;
                };
                if !groups.is_empty() {
                    tasks.push(Task::Product {
                        remaining: groups,
                        accumulated: C::one(),
                    });
                }
                tasks.push(Task::Block((lhs, rhs)));
            }
            Task::Block((lhs, rhs)) => {
                if let (Some(a), Some(b)) = (lhs.only(), rhs.only()) {
                    // Outer scales have been scheduled separately. A shared
                    // node certifies equal summands only for a reflexive shape
                    // identified by the common normalization context.
                    let matching = a.shares_node_with(b) && is_reflexive_sum(a);
                    match (a.node(), b.node()) {
                        (StateNode::Local(a, _), StateNode::Local(b, _)) => {
                            values.push(if matching {
                                a.matching_inner_product()
                            } else {
                                a.evaluated_inner_product(b)
                            });
                            continue;
                        }
                        (StateNode::Sum(terms), StateNode::Sum(_)) if matching => {
                            let terms = terms.as_slice();
                            tasks.push(Task::MatchingSum {
                                terms,
                                row: 0,
                                column: 0,
                                accumulated: zero(),
                            });
                            tasks.push(Task::Contract((single(&terms[0]), single(&terms[0]))));
                            continue;
                        }
                        _ => {}
                    }
                }
                // No further boundary is shared. Distribute one sum within
                // this block, then align each resulting branch again.
                let left_sum = lhs
                    .iter()
                    .position(|s| matches!(s.node(), StateNode::Sum(_)));
                let (side, conjugated, position) = match left_sum {
                    Some(position) => (&lhs, true, position),
                    None => {
                        let position = rhs
                            .iter()
                            .position(|s| matches!(s.node(), StateNode::Sum(_)))
                            .expect("inseparable subsystem contains a sum");
                        (&rhs, false, position)
                    }
                };
                let StateNode::Sum(children) = side.get(position).unwrap().node() else {
                    unreachable!()
                };
                tasks.push(Task::Sum(children.len()));
                for child in children.iter().rev() {
                    let branch = side.replace(child, position);
                    let pair = if conjugated {
                        (branch, rhs.range(0..rhs.len()))
                    } else {
                        (lhs.range(0..lhs.len()), branch)
                    };
                    tasks.push(Task::Contract(pair));
                }
            }
            Task::Scale(coefficient, conjugated) => {
                let value = values.pop().expect("scaled contraction");
                if value.is_zero() {
                    values.push(value);
                    continue;
                }
                let c = coefficient.clone();
                values.push(multiply(if conjugated { conjugate(c) } else { c }, value));
            }
            Task::Sum(count) => {
                let parts = values.split_off(values.len() - count);
                values.push(
                    parts
                        .into_iter()
                        .filter(|value| !value.is_zero())
                        .fold(zero(), add),
                );
            }
            Task::MatchingSum {
                terms,
                row,
                column,
                accumulated,
            } => {
                let value = values.pop().expect("next summand pair");
                let accumulated = add_pair(accumulated, value, row != column);
                let (row, column) = if column + 1 == terms.len() {
                    (row + 1, row + 1)
                } else {
                    (row, column + 1)
                };
                if row == terms.len() {
                    values.push(accumulated);
                } else {
                    tasks.push(Task::MatchingSum {
                        terms,
                        row,
                        column,
                        accumulated,
                    });
                    tasks.push(Task::Contract((
                        single(&terms[row]),
                        single(&terms[column]),
                    )));
                }
            }
            Task::Product {
                mut remaining,
                accumulated,
            } => {
                let value = values.pop().expect("next subsystem contraction");
                // Stop this product only; enclosing sum tasks remain scheduled.
                if value.is_zero() {
                    values.push(value);
                    continue;
                }
                let accumulated = multiply(accumulated, value);
                if accumulated.is_zero() {
                    values.push(accumulated);
                    continue;
                }
                if let Some(next) = remaining.next(&counts) {
                    tasks.push(Task::Product {
                        remaining,
                        accumulated,
                    });
                    tasks.push(Task::Block(next));
                } else {
                    values.push(accumulated);
                }
            }
        }
    }
    values.pop().expect("one contraction result")
}

fn primitive_overlap<S, C>(left: &S, right: &S, a: Option<&C>, b: Option<C>) -> Option<C>
where
    S: InnerProduct<C>,
    C: Clone + Zero + Mul<Output = C> + Conjugate<Output = C>,
{
    if b.as_ref().is_some_and(Zero::is_zero) {
        return None;
    }
    let mut value = left.inner_product(right);
    if value.is_zero() {
        return None;
    }
    if let Some(b) = b {
        value = multiply(value, b);
        if value.is_zero() {
            return None;
        }
    }
    if let Some(a) = a {
        value = multiply(conjugate(a.clone()), value);
        if value.is_zero() {
            return None;
        }
    }
    Some(value)
}

impl<S, O, C> LocalState<S, O, C> {
    /// Computes the inner product, conjugate-linear on the left.
    ///
    /// Actions are evaluated internally; no basis or explicit prior evaluation
    /// is needed. Coefficients require arithmetic but no division or remainder.
    pub fn inner_product(&self, rhs: &Self) -> C
    where
        C: One + Clone + Zero + Mul<Output = C> + Conjugate<Output = C>,
        S: PartialEq + Act<C, Operator = O, Term = S> + InnerProduct<C>,
        O: Adjoint<C, Term = O>,
    {
        let lhs = self.evaluate();
        let rhs = rhs.evaluate();
        lhs.evaluated_inner_product(&rhs)
    }

    // Both operands must be action-free. Separating this reduction avoids
    // requiring action/adjoint capabilities again in composite contraction.
    pub(crate) fn evaluated_inner_product(&self, rhs: &Self) -> C
    where
        C: Clone + Zero + Mul<Output = C> + Conjugate<Output = C>,
        S: InnerProduct<C>,
    {
        let mut result = zero();
        for (a, left) in LinearTerms::new(self, state_view) {
            if a.as_ref().is_some_and(Zero::is_zero) {
                continue;
            }
            for (b, right) in LinearTerms::new(rhs, state_view) {
                if let Some(value) = primitive_overlap(left, right, a.as_ref(), b) {
                    result = add(result, value);
                }
            }
        }
        result
    }

    // The caller has proved that the evaluated local polynomials coincide.
    fn matching_inner_product(&self) -> C
    where
        C: Clone + Zero + Mul<Output = C> + Conjugate<Output = C>,
        S: InnerProduct<C>,
    {
        let terms: Vec<_> = LinearTerms::new(self, state_view).collect();
        let mut result = zero();
        for (row, (a, left)) in terms.iter().enumerate() {
            for (column, (b, right)) in terms.iter().enumerate().skip(row) {
                if let Some(value) = primitive_overlap(*left, *right, a.as_ref(), b.clone()) {
                    result = add_pair(result, value, row != column);
                }
            }
        }
        result
    }

    /// Returns the real part of the self-inner-product, without clamping.
    pub fn norm_squared(&self) -> C::Real
    where
        C: One + Clone + Zero + Mul<Output = C> + Conjugate<Output = C> + RealPart,
        S: PartialEq + Act<C, Operator = O, Term = S> + InnerProduct<C>,
        O: Adjoint<C, Term = O>,
    {
        self.inner_product(self).real_part()
    }

    /// Computes the unnormalized expectation `self† operator self`.
    pub fn expectation(&self, operator: &LocalOperator<O, C>) -> C
    where
        C: One + Clone + Zero + Mul<Output = C> + Conjugate<Output = C>,
        S: PartialEq + Act<C, Operator = O, Term = S> + InnerProduct<C>,
        O: Adjoint<C, Term = O>,
    {
        self.inner_product(&self.act(operator))
    }
}

impl<I: Ord, S, O, C> State<I, S, O, C> {
    /// Computes the conjugate-linear/linear inner product on compatible spaces.
    ///
    /// Contracts matching tensor factors before distributing sums. Tensor
    /// grouping need not agree, and no global product-of-sums expansion is built.
    /// Contracts one independent group at a time; an exact-zero product skips
    /// remaining groups, while every summand is still considered.
    pub fn inner_product(&self, rhs: &Self) -> Result<C, ExpressionError>
    where
        C: PartialEq + Clone + Zero + One + Conjugate<Output = C>,
        S: PartialEq + Act<C, Operator = O, Term = S> + InnerProduct<C>,
        O: Adjoint<C, Term = O> + crate::HomogeneousParity + crate::ParityOperator,
    {
        if !self.space.is_compatible_with(&rhs.space) {
            return Err(ExpressionError::IncompatibleSpaces);
        }
        let mut evaluator = Evaluator::new();
        let lhs = evaluator.evaluate(&self.expression);
        let rhs = evaluator.evaluate(&rhs.expression);
        let sums: BTreeSet<_> = evaluator
            .reflexive_sums()
            .map(|sum| ptr::from_ref(sum.node()))
            .collect();
        Ok(contract_evaluated(&lhs, &rhs, |sum| {
            sums.contains(&ptr::from_ref(sum.node()))
        }))
    }

    /// Returns the real part of the self-inner-product, without clamping.
    pub fn norm_squared(&self) -> Result<C::Real, ExpressionError>
    where
        C: PartialEq + Clone + Zero + One + Conjugate<Output = C> + RealPart,
        S: PartialEq + Act<C, Operator = O, Term = S> + InnerProduct<C>,
        O: Adjoint<C, Term = O> + crate::HomogeneousParity + crate::ParityOperator,
    {
        Ok(self.inner_product(self)?.real_part())
    }

    /// Computes the unnormalized expectation without requiring prior evaluation.
    pub fn expectation(&self, operator: &Operator<I, O, C>) -> Result<C, ExpressionError>
    where
        C: PartialEq + Clone + Zero + One + Conjugate<Output = C>,
        S: PartialEq + Act<C, Operator = O, Term = S> + InnerProduct<C>,
        O: Adjoint<C, Term = O> + crate::HomogeneousParity + crate::ParityOperator,
    {
        self.inner_product(&self.act(operator)?)
    }

    /// Computes an expectation using an explicit ambient space over any binding.
    pub fn expectation_in(
        &self,
        operator: &Operator<I, O, C>,
        ambient: &HilbertSpace<I>,
    ) -> Result<C, ExpressionError>
    where
        C: PartialEq + Clone + Zero + One + Conjugate<Output = C>,
        S: PartialEq + Act<C, Operator = O, Term = S> + InnerProduct<C>,
        O: Adjoint<C, Term = O> + crate::HomogeneousParity + crate::ParityOperator,
    {
        self.inner_product(&self.act_in(operator, ambient)?)
    }
}
