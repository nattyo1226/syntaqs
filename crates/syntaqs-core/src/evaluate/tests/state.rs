use crate::expression::{
    composite::{StateExpr, StateNode},
    local::StateNode as LocalNode,
    traversal::{LinearTerms, state_view},
};
use crate::{
    Act, Adjoint, HilbertSpace as H, HomogeneousParity, LinearCombination as Lc, LocalOperator,
    LocalState, Parity, ParityOperator, State,
};
use alloc::{vec, vec::Vec};
use core::cell::Cell;

// Neither payload nor index implements Clone/Ord/Hash; the operator has no Product.
struct Ket<'a> {
    value: f64,
    calls: &'a Cell<usize>,
}
impl PartialEq for Ket<'_> {
    fn eq(&self, rhs: &Self) -> bool {
        self.value == rhs.value
    }
}
struct Gate;
impl HomogeneousParity for Gate {
    fn parity(&self) -> Parity {
        Parity::Even
    }
}
impl ParityOperator for Gate {
    fn parity_operator() -> Self {
        Self
    }
}
impl Adjoint<f64> for Gate {
    type Term = Self;
    fn adjoint(&self) -> Lc<Self, f64> {
        Lc::primitive(Self)
    }
}
impl<'a> Act<f64> for Ket<'a> {
    type Operator = Gate;
    type Term = Self;
    fn act(&self, _: &Gate) -> Lc<Self, f64> {
        self.calls.set(self.calls.get() + 1);
        Lc::from_terms([
            crate::Term {
                coefficient: 0.5,
                operand: Self {
                    value: 0.0,
                    calls: self.calls,
                },
            },
            crate::Term {
                coefficient: 0.5,
                operand: Self {
                    value: 1.0,
                    calls: self.calls,
                },
            },
        ])
    }
}
#[derive(PartialEq, Eq)]
struct Index(u8);
type St<'a> = State<Index, Ket<'a>, Gate, f64>;
fn sum<'a>(parts: impl IntoIterator<Item = St<'a>>) -> Result<St<'a>, crate::ExpressionError> {
    let parts: Vec<_> = parts.into_iter().collect();
    let space = parts[0].space().clone();
    State::sum(&space, parts)
}
fn primitive(value: f64, calls: &Cell<usize>) -> LocalState<Ket<'_>, Gate, f64> {
    LocalState::primitive(Ket { value, calls })
}
fn leaves(state: &StateExpr<Index, Ket<'_>, Gate, f64>) -> usize {
    let mut pending = vec![state];
    let mut count = 0;
    while let Some(state) = pending.pop() {
        match state.node() {
            StateNode::Local(_, s) => count += LinearTerms::new(s, state_view).count(),
            StateNode::Sum(s) | StateNode::Tensor(s) => pending.extend(s.iter()),
            StateNode::Scale(s) => pending.push(&s.operand),
            StateNode::Unit => {}
            StateNode::Act(_) => panic!("pending action"),
        }
    }
    count
}
#[test]
fn local_actions_collect_before_the_next_action_and_shared_inputs_run_once() {
    let calls = Cell::new(0);
    let p = primitive(0.0, &calls);
    let op = LocalOperator::primitive(Gate);
    let mut state = p.clone();
    for _ in 0..100 {
        state = state.act(&op);
    }
    let result = state.evaluate();
    let terms: Vec<_> = LinearTerms::new(&result, state_view)
        .map(|(c, p)| (p.value, c.unwrap_or(1.0)))
        .collect();
    assert_eq!(terms, [(0.0, 0.5), (1.0, 0.5)]);
    assert_eq!(calls.get(), 199);
    calls.set(0);
    let shared = p.act(&op);
    let result = LocalState::sum([shared.clone(), shared]).evaluate();
    assert_eq!(calls.get(), 1);
    assert_eq!(LinearTerms::new(&result, state_view).count(), 2);
    assert_eq!(
        LinearTerms::new(&result, state_view)
            .map(|(c, _)| c.unwrap_or(1.0))
            .collect::<Vec<_>>(),
        [1.0, 1.0]
    );
    let again = result.evaluate();
    assert_eq!(LinearTerms::new(&again, state_view).count(), 2);
}
#[test]
fn composite_collection_normalizes_tensor_grouping_scales_units_and_single_factor_sums() {
    let calls = Cell::new(0);
    let a = H::factor(Index(2));
    let b = H::factor(Index(0));
    let c = H::factor(Index(1));
    let ab = a.tensor(&b).unwrap();
    let h = ab.tensor(&c).unwrap();
    let x = primitive(0.0, &calls).at(&a).unwrap();
    let y = primitive(1.0, &calls).at(&b).unwrap();
    let z = primitive(0.0, &calls).at(&c).unwrap();
    let left = St::tensor_in(
        &h,
        [
            St::tensor_in(&ab, [x.scaled(2.0), y.clone()]).unwrap(),
            z.clone(),
        ],
    )
    .unwrap();
    let right = St::tensor_in(&h, [St::unit(), x.clone(), y.clone(), z.clone()])
        .unwrap()
        .scaled(3.0);
    let result = sum([left, right]).unwrap().evaluate().unwrap();
    let StateNode::Scale(s) = result.expression.node() else {
        panic!("one retained term")
    };
    assert_eq!(s.coefficient, 5.0);
    let StateNode::Tensor(parts) = s.operand.node() else {
        panic!("flat tensor")
    };
    assert_eq!(parts.len(), 3);
    assert_eq!(leaves(&result.expression), 3);
    assert!(result.space().shares_node_with(&h));
    let again = result.evaluate().unwrap();
    assert_eq!(leaves(&again.expression), 3);
    let p = primitive(0.0, &calls);
    let q = primitive(1.0, &calls);
    let inside = LocalState::sum([p.clone(), q.clone()]).at(&a).unwrap();
    let outside = sum([p.at(&a).unwrap(), q.at(&a).unwrap()]).unwrap();
    let result = sum([inside, outside]).unwrap().evaluate().unwrap();
    let StateNode::Local(_, local) = result.expression.node() else {
        panic!("one local polynomial")
    };
    assert_eq!(
        LinearTerms::new(local, state_view)
            .map(|(c, _)| c.unwrap_or(1.0))
            .collect::<Vec<_>>(),
        [2.0, 2.0]
    );
}
#[test]
fn factored_sums_stay_factored_and_are_not_matched_by_distributive_search() {
    let calls = Cell::new(0);
    let factors: Vec<_> = (0..32).map(|i| H::factor(Index(i))).collect();
    let h = H::tensor_all(factors.iter().cloned()).unwrap();
    let state = St::tensor_in(
        &h,
        factors.iter().map(|f| {
            LocalState::sum([primitive(0.0, &calls), primitive(1.0, &calls)])
                .at(f)
                .unwrap()
        }),
    )
    .unwrap();
    let result = state.evaluate().unwrap();
    assert_eq!(leaves(&result.expression), 64);
    let pair = factors[0].tensor(&factors[1]).unwrap();
    let a = primitive(0.0, &calls).at(&factors[0]).unwrap();
    let b = primitive(1.0, &calls).at(&factors[0]).unwrap();
    let z = primitive(0.0, &calls).at(&factors[1]).unwrap();
    let factored = St::tensor_in(&pair, [sum([a.clone(), b.clone()]).unwrap(), z.clone()]).unwrap();
    let distributed = sum([
        St::tensor_in(&pair, [a, z.clone()]).unwrap(),
        St::tensor_in(&pair, [b, z]).unwrap(),
    ])
    .unwrap();
    let result = sum([factored, distributed]).unwrap().evaluate().unwrap();
    let StateNode::Sum(parts) = result.expression.node() else {
        panic!("distinct factored and distributed shapes")
    };
    assert_eq!(parts.len(), 3);
}
#[test]
fn exact_cancellation_tiny_terms_and_nonreflexive_values_follow_partial_eq() {
    let calls = Cell::new(0);
    let a = H::factor(Index(0));
    let b = H::factor(Index(1));
    let h = a.tensor(&b).unwrap();
    let p = primitive(0.0, &calls).at(&a).unwrap();
    let q = primitive(1.0, &calls).at(&b).unwrap();
    let tensor = St::tensor_in(&h, [p, q]).unwrap();
    let result = sum([tensor.clone(), tensor.scaled(-1.0)])
        .unwrap()
        .evaluate()
        .unwrap();
    assert!(matches!(result.expression.node(),StateNode::Sum(s) if s.is_empty()));
    assert!(result.space().shares_node_with(&h));
    let tiny = primitive(0.0, &calls).scaled(1e-200).evaluate();
    assert_eq!(
        LinearTerms::new(&tiny, state_view).next().unwrap().0,
        Some(1e-200)
    );
    // Sharing an input is not evidence that its primitive compares equal to itself.
    let nan = primitive(f64::NAN, &calls).at(&a).unwrap();
    let tensor = St::tensor_in(&h, [nan, primitive(0.0, &calls).at(&b).unwrap()]).unwrap();
    let result = sum([tensor.clone(), tensor]).unwrap().evaluate().unwrap();
    assert!(matches!(result.expression.node(),StateNode::Sum(s) if s.len()==2));
    // The same applies to a nonreflexive coefficient inside a retained polynomial.
    let polynomial = LocalState::sum([
        primitive(0.0, &calls).scaled(f64::NAN),
        primitive(1.0, &calls),
    ])
    .at(&a)
    .unwrap();
    let tensor = St::tensor_in(&h, [polynomial, primitive(0.0, &calls).at(&b).unwrap()]).unwrap();
    let result = sum([tensor.clone(), tensor]).unwrap().evaluate().unwrap();
    assert!(matches!(result.expression.node(),StateNode::Sum(s) if s.len()==2));
}
#[test]
fn local_first_occurrence_order_survives_cancellation_and_zero_tensor_keeps_space() {
    let calls = Cell::new(0);
    let p = primitive(1.0, &calls);
    let q = primitive(0.0, &calls);
    let result = LocalState::sum([p.clone(), q, p.scaled(-1.0), p.clone()]).evaluate();
    assert_eq!(
        LinearTerms::new(&result, state_view)
            .map(|(_, p)| p.value)
            .collect::<Vec<_>>(),
        [1.0, 0.0]
    );
    let a = H::factor(Index(0));
    let b = H::factor(Index(1));
    let h = a.tensor(&b).unwrap();
    let zero = LocalState::sum([p.clone(), p.scaled(-1.0)]).at(&a).unwrap();
    let state = St::tensor_in(&h, [zero, primitive(1.0, &calls).at(&b).unwrap()])
        .unwrap()
        .evaluate()
        .unwrap();
    assert!(matches!(state.expression.node(),StateNode::Sum(s) if s.is_empty()));
    assert!(state.space().shares_node_with(&h));
    assert!(matches!(result.node(), LocalNode::Sum(_)));
}

#[test]
fn shared_additive_dags_are_normalized_once_without_distributing_tensors() {
    let calls = Cell::new(0);
    let mut local = primitive(0.0, &calls);
    for _ in 0..64 {
        local = LocalState::sum([local.clone(), local]);
    }
    let ready = local.act(&LocalOperator::primitive(Gate)).evaluate();
    assert_eq!(calls.get(), 1);
    assert_eq!(
        LinearTerms::new(&ready, state_view)
            .map(|(c, _)| c.unwrap_or(1.0))
            .collect::<Vec<_>>(),
        [2f64.powi(63), 2f64.powi(63)]
    );
    calls.set(0);
    let a = H::factor(Index(0));
    let b = H::factor(Index(1));
    let h = a.tensor(&b).unwrap();
    let mut state = St::tensor_in(
        &h,
        [
            primitive(0.0, &calls).at(&a).unwrap(),
            primitive(1.0, &calls).at(&b).unwrap(),
        ],
    )
    .unwrap();
    for _ in 0..64 {
        state = State::sum(&h, [state.clone(), state]).unwrap();
    }
    let op = LocalOperator::primitive(Gate).at(&a).unwrap();
    let ready = state.act(&op).unwrap().evaluate().unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(leaves(&ready.expression), 3);
    let StateNode::Scale(s) = ready.expression.node() else {
        panic!("tensor outer coefficient")
    };
    assert_eq!(s.coefficient, 2f64.powi(64));
}
