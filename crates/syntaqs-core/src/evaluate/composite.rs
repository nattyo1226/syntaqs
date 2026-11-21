use alloc::{collections::BTreeMap, vec, vec::Vec};
use core::{marker::PhantomData, ptr};
use num_traits::{One, Zero};

use super::{
    ProductAction,
    local::Evaluator as LocalEvaluator,
    state_terms::{Key, Ready, Sum, Terms},
};
use crate::expression::composite::{OpExpr, OpNode, State, StateExpr, StateNode};
use crate::expression::error::ExpressionError;
use crate::{
    Act, Adjoint, Conjugate, HilbertSpace, HomogeneousParity, LocalOperator, Parity,
    ParityOperator, TensorKind, coefficient::conjugate,
};

enum Task<'a, I, S, O, C> {
    Evaluate(&'a StateExpr<I, S, O, C>),
    Remember(&'a StateExpr<I, S, O, C>),
    Apply(Ready<C>, OpExpr<I, O, C>, bool, HilbertSpace<I>),
    ThenApply(OpExpr<I, O, C>, HilbertSpace<I>),
    ContinueProduct(ProductAction<OpExpr<I, O, C>>, HilbertSpace<I>),
    ContinuePlacedProduct(
        ProductAction<LocalOperator<O, C>>,
        HilbertSpace<I>,
        HilbertSpace<I>,
    ),
    Local(
        Ready<C>,
        HilbertSpace<I>,
        LocalOperator<O, C>,
        bool,
        HilbertSpace<I>,
    ),
    VisitLocal(Ready<C>, usize),
    FinishLocalTensor {
        shape: usize,
        first: usize,
        count: usize,
    },
    FinishLocalSum {
        remaining: alloc::vec::IntoIter<Ready<C>>,
        start: usize,
    },
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

fn apply_product<I, S, O, C>(
    tasks: &mut Vec<Task<'_, I, S, O, C>>,
    state: Ready<C>,
    product: ProductAction<OpExpr<I, O, C>>,
    scope: HilbertSpace<I>,
) {
    let (operand, adjoint, remaining) = product.advance();
    if let Some(remaining) = remaining {
        tasks.push(Task::ContinueProduct(remaining, scope.clone()));
    }
    tasks.push(Task::Apply(state, operand, adjoint, scope));
}

fn apply_placed_product<I, S, O, C>(
    tasks: &mut Vec<Task<'_, I, S, O, C>>,
    state: Ready<C>,
    product: ProductAction<LocalOperator<O, C>>,
    factor: HilbertSpace<I>,
    scope: HilbertSpace<I>,
) {
    let (operand, adjoint, remaining) = product.advance();
    if let Some(remaining) = remaining {
        tasks.push(Task::ContinuePlacedProduct(
            remaining,
            factor.clone(),
            scope.clone(),
        ));
    }
    tasks.push(Task::Placed(state, factor, operand, adjoint, scope));
}

// A placement traverses action-free shapes at positions relative to its Act scope.
struct Placement<O, C> {
    target: usize,
    operator: LocalOperator<O, C>,
    odd: bool,
}
fn schedule_sum<'a, I, S, O, C>(
    tasks: &mut Vec<Task<'a, I, S, O, C>>,
    children: impl DoubleEndedIterator<Item = Task<'a, I, S, O, C>>,
) {
    tasks.push(Task::SumEnd);
    for child in children.rev() {
        tasks.push(Task::SumAdd);
        tasks.push(child);
    }
    tasks.push(Task::SumStart);
}
impl<I: Ord, S, O, C> State<I, S, O, C> {
    /// Evaluates delayed actions and collects matching natural-state terms
    /// between actions. Action-free inputs are normalized too; tensor products
    /// of sums stay factored. No basis or primitive operator Product is required.
    pub fn evaluate(&self) -> Result<Self, ExpressionError>
    where
        C: Clone + PartialEq + Zero + One + Conjugate<Output = C>,
        S: PartialEq + Act<C, Operator = O, Term = S>,
        O: Adjoint<C, Term = O> + HomogeneousParity + ParityOperator,
    {
        let mut evaluator = Evaluator::new();
        Ok(Self {
            space: self.space.clone(),
            expression: evaluator.evaluate(&self.expression),
        })
    }
}

type Memo<I, S, O, C> = BTreeMap<*const StateNode<I, S, O, C>, Ready<C>>;

/// The operands of one query share the shape table and the normalization memo.
pub(super) struct Evaluator<'a, I: Ord, S, O, C> {
    // Input roots outlive the pointer-keyed memo of their descendants.
    sources: PhantomData<&'a StateExpr<I, S, O, C>>,
    terms: Option<Terms<I, S, O, C>>,
    memo: Memo<I, S, O, C>,
    locals: LocalEvaluator<S, O, C>,
}

impl<'a, I: Ord, S, O, C> Evaluator<'a, I, S, O, C>
where
    C: Clone + PartialEq + Zero + One + Conjugate<Output = C>,
    S: PartialEq + Act<C, Operator = O, Term = S>,
    O: Adjoint<C, Term = O> + HomogeneousParity + ParityOperator,
{
    pub(super) fn new() -> Self {
        Self {
            sources: PhantomData,
            terms: None,
            memo: BTreeMap::new(),
            locals: LocalEvaluator::new(),
        }
    }

    pub(super) fn evaluate(
        &mut self,
        expression: &'a StateExpr<I, S, O, C>,
    ) -> StateExpr<I, S, O, C> {
        // These shapes are intrinsically normalized.
        use crate::expression::local::StateNode as LocalNode;
        let primitive = |state: &StateExpr<I, S, O, C>| matches!(state.node(), StateNode::Local(_, local) if matches!(local.node(), LocalNode::Primitive(_)));
        if matches!(expression.node(), StateNode::Unit)
            || primitive(expression)
            || matches!(expression.node(), StateNode::Tensor(parts) if parts.len() > 1 && parts.iter().all(primitive))
        {
            return expression.clone();
        }
        let Self {
            terms,
            memo,
            locals,
            ..
        } = self;
        let terms = terms.get_or_insert_with(Terms::new);
        let mut tasks = vec![Task::Evaluate(expression)];
        let mut values: Vec<Ready<C>> = Vec::new();
        let mut sums = Vec::new();
        let mut placement = None;
        while let Some(task) = tasks.pop() {
            match task {
                Task::Evaluate(state) => {
                    let key = ptr::from_ref(state.node());
                    if let Some(ready) = memo.get(&key) {
                        values.push(Ready::clone(ready));
                        continue;
                    }
                    tasks.push(Task::Remember(state));
                    match state.node() {
                        StateNode::Local(factor, local) => {
                            let local = locals.evaluate(local);
                            values.push(terms.local(factor.clone(), local));
                        }
                        StateNode::Unit => values.push(terms.unit()),
                        StateNode::Scale(s) => {
                            tasks.push(Task::Scale(s.coefficient.clone()));
                            tasks.push(Task::Evaluate(&s.operand));
                        }
                        StateNode::Sum(s) | StateNode::Tensor(s) => {
                            if matches!(state.node(), StateNode::Tensor(_)) {
                                tasks.push(Task::Tensor(s.len()));
                                for child in s.iter().rev() {
                                    tasks.push(Task::Evaluate(child));
                                }
                            } else {
                                schedule_sum(&mut tasks, s.iter().map(Task::Evaluate));
                            }
                        }
                        StateNode::Act(a) => {
                            tasks.push(Task::ThenApply(a.operator.clone(), a.space.clone()));
                            tasks.push(Task::Evaluate(&a.state));
                        }
                    }
                }
                Task::Remember(source) => {
                    terms.reuse(values.last().unwrap(), source);
                    memo.insert(ptr::from_ref(source.node()), values.last().unwrap().clone());
                }
                Task::ThenApply(op, scope) => {
                    if values.last().unwrap().shape != 0 {
                        tasks.push(Task::Apply(values.pop().unwrap(), op, false, scope));
                    }
                }
                Task::ContinueProduct(product, scope) => {
                    if values.last().unwrap().shape != 0 {
                        apply_product(&mut tasks, values.pop().unwrap(), product, scope);
                    }
                }
                Task::ContinuePlacedProduct(product, factor, scope) => {
                    if values.last().unwrap().shape != 0 {
                        apply_placed_product(
                            &mut tasks,
                            values.pop().unwrap(),
                            product,
                            factor,
                            scope,
                        );
                    }
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
                    OpNode::Product(_) => {
                        apply_product(&mut tasks, state, ProductAction::new(op, adjoint), scope);
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
                        tasks.push(Task::Local(state, factor, operator, false, scope));
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
                        OperatorNode::Product(_) => {
                            apply_placed_product(
                                &mut tasks,
                                state,
                                ProductAction::new(operator, adjoint),
                                factor,
                                scope,
                            );
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
                            let odd = p.parity() == Parity::Odd;
                            tasks.push(Task::Local(state, factor, operator, odd, scope));
                        }
                    }
                }
                Task::Local(state, factor, operator, odd, scope) => {
                    let target = scope
                        .factor_position(factor.factor_index().expect("placed factor"))
                        .expect("validated placement");
                    placement = Some(Placement {
                        target,
                        operator,
                        odd,
                    });
                    tasks.push(Task::VisitLocal(state, 0));
                }
                Task::VisitLocal(state, start) => {
                    let placement = placement.as_ref().expect("local action");
                    if state.shape == 0 {
                        values.push(state);
                        continue;
                    }
                    match terms.key(state.shape) {
                        Key::Local(..) => {
                            let (factor, local) = terms.local_parts(state.shape).unwrap();
                            let op = if start == placement.target {
                                placement.operator.clone()
                            } else {
                                debug_assert!(placement.odd && start < placement.target);
                                LocalOperator::primitive(O::parity_operator())
                            };
                            let local = locals.apply(local, op);
                            let mut ready = terms.local(factor, local);
                            if let Some(c) = state.coefficient {
                                ready = terms.scaled(ready, c);
                            }
                            values.push(ready);
                        }
                        Key::Unit => values.push(state),
                        Key::Tensor(children) => {
                            if let Some(c) = state.coefficient {
                                tasks.push(Task::Scale(c));
                            }
                            let width = terms.factor_count(state.shape);
                            // Normalized tensors have no unit children.
                            let single_factors = width == children.len();
                            let (end, mut position) = if placement.target >= start + width {
                                debug_assert!(placement.odd);
                                (children.len(), start + width)
                            } else if single_factors {
                                (placement.target - start + 1, placement.target + 1)
                            } else {
                                let mut position = start;
                                let child = children
                                    .iter()
                                    .position(|&shape| {
                                        position += terms.factor_count(shape);
                                        placement.target < position
                                    })
                                    .expect("target covered by tensor");
                                (child + 1, position)
                            };
                            let first = if placement.odd { 0 } else { end - 1 };
                            tasks.push(Task::FinishLocalTensor {
                                shape: state.shape,
                                first,
                                count: end - first,
                            });
                            for &shape in children[first..end].iter().rev() {
                                position -= if single_factors {
                                    1
                                } else {
                                    terms.factor_count(shape)
                                };
                                tasks.push(Task::VisitLocal(
                                    Ready {
                                        coefficient: None,
                                        shape,
                                    },
                                    position,
                                ));
                            }
                        }
                        Key::Sum(_) => {
                            if let Some(c) = state.coefficient {
                                tasks.push(Task::Scale(c));
                            }
                            let mut remaining = terms.sum_children(state.shape).into_iter();
                            let first = remaining.next().expect("nonempty sum");
                            sums.push(Sum::new());
                            tasks.push(Task::FinishLocalSum { remaining, start });
                            tasks.push(Task::VisitLocal(first, start));
                        }
                    }
                }
                Task::FinishLocalTensor {
                    shape,
                    first,
                    count,
                } => {
                    let updated =
                        terms.update_tensor(shape, first, values.drain(values.len() - count..));
                    values.push(updated);
                }
                Task::FinishLocalSum {
                    mut remaining,
                    start,
                } => {
                    terms.add(sums.last_mut().unwrap(), values.pop().unwrap());
                    if let Some(next) = remaining.next() {
                        tasks.push(Task::FinishLocalSum { remaining, start });
                        tasks.push(Task::VisitLocal(next, start));
                    } else {
                        values.push(terms.finish_sum(sums.pop().unwrap()));
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
        terms.expression(values.pop().unwrap())
    }

    pub(super) fn reflexive_sums(&self) -> impl Iterator<Item = &StateExpr<I, S, O, C>> {
        self.terms.iter().flat_map(Terms::reflexive_sums)
    }
}
