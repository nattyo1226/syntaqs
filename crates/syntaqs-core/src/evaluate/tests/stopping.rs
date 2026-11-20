use alloc::vec::Vec;
use core::ops::{Add, Mul};
use num_complex::Complex64;
use num_traits::{One, Zero};
use std::cell::RefCell;

use super::super::inner_product::contract;
use crate::{
    Act, Adjoint, Conjugate, Divide, ExpressionError, HilbertSpace as H, HomogeneousParity,
    Identity, InnerProduct, LinearCombination as Lc, LocalOperator, LocalState, Operator, Parity,
    ParityOperator, State, Trace,
};

#[derive(Default)]
struct Counts {
    inner: Vec<usize>,
    traces: Vec<Option<usize>>,
    add: usize,
    multiply: usize,
    conjugate: usize,
    divide: usize,
}
std::thread_local! { static COUNTS: RefCell<Counts> = RefCell::new(Counts::default()); }
fn reset() {
    COUNTS.with(|c| *c.borrow_mut() = Counts::default());
}

// No PartialEq, ordering, Copy, Num, or Float; contraction needs only exact Zero.
#[derive(Clone)]
struct Scalar(f64);
impl From<f64> for Scalar {
    fn from(v: f64) -> Self {
        Self(v)
    }
}
impl Add for Scalar {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        COUNTS.with(|c| c.borrow_mut().add += 1);
        Self(self.0 + rhs.0)
    }
}
impl Mul for Scalar {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        COUNTS.with(|c| {
            let mut c = c.borrow_mut();
            c.multiply = c.multiply.saturating_add(1);
        });
        Self(self.0 * rhs.0)
    }
}
impl Zero for Scalar {
    fn zero() -> Self {
        Self(0.0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
}
impl One for Scalar {
    fn one() -> Self {
        Self(1.0)
    }
}
impl Conjugate for Scalar {
    type Output = Self;
    fn conjugate(&self) -> Self {
        COUNTS.with(|c| c.borrow_mut().conjugate += 1);
        self.clone()
    }
}
impl Divide for Scalar {
    type Output = Self;
    fn divide(self, rhs: Self) -> Self {
        COUNTS.with(|c| c.borrow_mut().divide += 1);
        Self(self.0 / rhs.0)
    }
}

// Distinct natural primitives can describe overlapping states. No Clone or Eq.
struct Ket {
    label: u8,
    site: usize,
}
impl PartialEq for Ket {
    fn eq(&self, rhs: &Self) -> bool {
        self.label != 6 && self.label == rhs.label && self.site == rhs.site
    }
}
impl Ket {
    fn amplitudes(&self) -> [f64; 2] {
        match self.label {
            0 => [1.0, 0.0],
            1 => [0.0, 1.0],
            2 | 6 => [1.0, 1.0],
            3 => [1.0, -1.0],
            4 => [1e-100, 0.0],
            _ => [f64::NAN, 0.0],
        }
    }
}
impl<C: From<f64>> InnerProduct<C> for Ket {
    fn inner_product(&self, rhs: &Self) -> C {
        COUNTS.with(|c| c.borrow_mut().inner.push(self.site));
        let a = self.amplitudes();
        let b = rhs.amplitudes();
        C::from(a[0] * b[0] + a[1] * b[1])
    }
}
impl<C: From<f64>> Act<C> for Ket {
    type Operator = Op;
    type Term = Self;
    fn act(&self, op: &Op) -> Lc<Self, C> {
        let a = self.amplitudes();
        Lc::from_terms((0..2).map(|i| crate::Term {
            coefficient: C::from(op.matrix[i][0] * a[0] + op.matrix[i][1] * a[1]),
            operand: Self {
                label: i as u8,
                site: self.site,
            },
        }))
    }
}

// A homogeneous real 2x2 algebra; no Clone, PartialEq, state, or Dimension bound.
struct Op {
    matrix: [[f64; 2]; 2],
    site: Option<usize>,
}
impl Identity for Op {
    fn identity() -> Self {
        Self {
            matrix: [[1.0, 0.0], [0.0, 1.0]],
            site: None,
        }
    }
}
impl HomogeneousParity for Op {
    fn parity(&self) -> Parity {
        if self.matrix[0][0] == 0.0 && self.matrix[1][1] == 0.0 {
            Parity::Odd
        } else {
            Parity::Even
        }
    }
}
impl ParityOperator for Op {
    fn parity_operator() -> Self {
        Self {
            matrix: [[1.0, 0.0], [0.0, -1.0]],
            site: None,
        }
    }
}
impl<C: From<f64>> Adjoint<C> for Op {
    type Term = Self;
    fn adjoint(&self) -> Lc<Self, C> {
        Lc::term(
            C::from(1.0),
            Self {
                matrix: core::array::from_fn(|i| core::array::from_fn(|j| self.matrix[j][i])),
                site: self.site,
            },
        )
    }
}
impl<C: From<f64>> crate::Product<C> for Op {
    type Term = Self;
    fn product(&self, rhs: &Self) -> Lc<Self, C> {
        Lc::term(
            C::from(1.0),
            Self {
                matrix: core::array::from_fn(|i| {
                    core::array::from_fn(|j| {
                        self.matrix[i][0] * rhs.matrix[0][j] + self.matrix[i][1] * rhs.matrix[1][j]
                    })
                }),
                site: self.site.or(rhs.site),
            },
        )
    }
}
impl<C: From<f64>> Trace<C> for Op {
    fn trace(&self) -> C {
        COUNTS.with(|c| c.borrow_mut().traces.push(self.site));
        C::from(self.matrix[0][0] + self.matrix[1][1])
    }
}
fn sites() -> Vec<H<u8>> {
    [9, 3, 7, 1].into_iter().map(H::factor).collect()
}
fn product<C>(space: &H<u8>, sites: &[H<u8>], labels: &[u8]) -> State<u8, Ket, Op, C> {
    State::tensor_in(
        space,
        sites
            .iter()
            .zip(labels)
            .enumerate()
            .map(|(site, (factor, label))| {
                LocalState::primitive(Ket {
                    label: *label,
                    site,
                })
                .at(factor)
                .unwrap()
            }),
    )
    .unwrap()
}
fn operators<C>(space: &H<u8>, sites: &[H<u8>], zero: Option<usize>) -> Operator<u8, Op, C> {
    Operator::try_product(sites.iter().enumerate().map(|(i, f)| {
        LocalOperator::primitive(Op {
            matrix: if zero == Some(i) {
                [[0.0, 1.0], [-1.0, 0.0]]
            } else {
                [[0.0, 1.0], [1.0, 0.0]]
            },
            site: Some(i),
        })
        .at(f)
    }))
    .unwrap()
    .bind(space)
    .unwrap()
}

#[test]
fn state_products_stop_at_first_exact_zero_and_keep_nonzero_factors() {
    let factors = sites();
    let space = H::tensor_all(factors.iter().cloned()).unwrap();
    let a = product::<f64>(&space, &factors, &[0, 0, 0, 0]);
    for zero in [None, Some(0), Some(2), Some(3)] {
        let labels: Vec<_> = (0..4).map(|i| u8::from(zero == Some(i))).collect();
        let b = product::<f64>(&space, &factors, &labels);
        reset();
        assert_eq!(
            a.inner_product(&b).unwrap(),
            if zero.is_some() { 0.0 } else { 1.0 }
        );
        COUNTS.with(|c| {
            assert_eq!(
                c.borrow().inner,
                (0..zero.map_or(4, |i| i + 1)).collect::<Vec<_>>()
            )
        });
    }
    // An underflowed running product stops too; a tiny nonzero one does not.
    for (labels, expected, calls) in [([4, 4, 0, 0], 0.0, 2), ([4, 0, 0, 0], 1e-200, 4)] {
        let b = product::<f64>(&space, &factors, &labels);
        reset();
        let value = b.inner_product(&b).unwrap();
        assert_eq!(value, expected);
        COUNTS.with(|c| assert_eq!(c.borrow().inner.len(), calls));
    }
    let nan = product::<f64>(&space, &factors, &[5, 0, 0, 0]);
    reset();
    assert!(nan.inner_product(&a).unwrap().is_nan());
    COUNTS.with(|c| assert_eq!(c.borrow().inner.len(), 4));
}

#[test]
fn sum_branches_continue_through_zero_terms_and_zero_partial_sums() {
    let f = sites();
    let pair = f[0].tensor(&f[1]).unwrap();
    let space = pair.tensor(&f[2]).unwrap();
    let rhs = product::<f64>(&space, &f[..3], &[0, 0, 0]);
    let tail = LocalState::<Ket, Op, f64>::primitive(Ket { label: 0, site: 2 })
        .at(&f[2])
        .unwrap();
    // Overlaps with |00> are 1,-1,1. All shapes survive state collection.
    let parts = [(2, 1.0), (0, -1.0), (3, 1.0)]
        .map(|(label, c)| product::<f64>(&pair, &f[..2], &[label, 0]).scaled(c));
    let sum = State::sum(&pair, parts).unwrap();
    let lhs = State::tensor_in(&space, [sum, tail.clone()]).unwrap();
    reset();
    assert_eq!(lhs.inner_product(&rhs).unwrap(), 1.0);
    COUNTS.with(|c| assert_eq!(c.borrow().inner, [0, 1, 0, 1, 0, 1, 2]));
    let sum = State::sum(
        &pair,
        [
            product::<f64>(&pair, &f[..2], &[2, 0]),
            product::<f64>(&pair, &f[..2], &[0, 0]).scaled(-1.0),
        ],
    )
    .unwrap();
    let lhs = State::tensor_in(&space, [sum, tail.clone()]).unwrap();
    reset();
    assert_eq!(lhs.inner_product(&rhs).unwrap(), 0.0);
    COUNTS.with(|c| assert_eq!(c.borrow().inner, [0, 1, 0, 1]));
    // The first pair vanishes at its first local factor, but the next pair runs.
    let lhs = State::sum(
        &space,
        [product::<f64>(&space, &f[..3], &[1, 0, 0]), rhs.clone()],
    )
    .unwrap();
    reset();
    assert_eq!(lhs.inner_product(&rhs).unwrap(), 1.0);
    COUNTS.with(|c| assert_eq!(c.borrow().inner, [0, 0, 1, 2]));
}

#[test]
fn matching_nested_sums_use_symmetry_with_distinct_outer_scales() {
    let f = sites();
    let pair = f[0].tensor(&f[1]).unwrap();
    let space = pair.tensor(&f[2]).unwrap();
    let state = |c: Complex64, tail, scale| {
        let block = State::sum(
            &pair,
            [
                product::<Complex64>(&pair, &f[..2], &[0, 0]),
                product::<Complex64>(&pair, &f[..2], &[2, 0]).scaled(c),
            ],
        )
        .unwrap()
        .scaled(scale);
        let tail = LocalState::primitive(Ket {
            label: tail,
            site: 2,
        })
        .at(&f[2])
        .unwrap();
        State::tensor_in(&space, [block, tail]).unwrap()
    };
    let c = Complex64::new(0.5, 1.0);
    let lhs = state(c, 0, Complex64::new(2.0, 1.0));
    let rhs = state(c, 2, Complex64::new(-1.0, 2.0));

    reset();
    // The two-site block has amplitudes (1+c, c) on its nonzero coordinates.
    // The tail overlap is 1 and the outer scalar contribution is 5i.
    assert_eq!(lhs.inner_product(&rhs).unwrap(), Complex64::new(0.0, 22.5));
    COUNTS.with(|c| assert_eq!(c.borrow().inner, [0, 1, 0, 1, 0, 1, 2]));

    reset();
    let rhs = state(c.conj(), 2, Complex64::new(-1.0, 2.0));
    assert_eq!(lhs.inner_product(&rhs).unwrap(), Complex64::new(20.0, 2.5));
    COUNTS.with(|c| assert_eq!(c.borrow().inner, [0, 1, 0, 1, 0, 1, 0, 1, 2]));
}

#[test]
fn shared_nonreflexive_shapes_contract_all_pairs() {
    let f = sites();
    let space = f[0].tensor(&f[1]).unwrap();
    let run = |label, coefficient| {
        let sum = State::sum(
            &space,
            [
                product::<f64>(&space, &f[..2], &[0, 0]),
                product::<f64>(&space, &f[..2], &[label, 0]).scaled(coefficient),
            ],
        )
        .unwrap();
        reset();
        let result = sum.inner_product(&sum).unwrap();
        COUNTS.with(|c| assert_eq!(c.borrow().inner, [0, 1, 0, 1, 0, 1, 0, 1]));
        result
    };
    assert_eq!(run(6, 1.0), 5.0);
    assert!(run(2, f64::NAN).is_nan());
}

#[test]
fn zero_contractions_skip_remaining_arithmetic_without_coefficient_equality() {
    let f = sites();
    let space = H::tensor_all(f.iter().cloned()).unwrap();
    let a = product::<Scalar>(&space, &f, &[0, 0, 0, 0]).scaled(Scalar(42.0));
    let b = product::<Scalar>(&space, &f, &[1, 0, 0, 0]);
    reset();
    assert_eq!(contract(&a.expression, &b.expression).0, 0.0);
    COUNTS.with(|c| {
        let c = c.borrow();
        assert_eq!(c.inner, [0]);
        assert_eq!((c.multiply, c.conjugate), (0, 0));
    });
    let a =
        LocalState::<Ket, Op, Scalar>::primitive(Ket { label: 0, site: 0 }).scaled(Scalar(42.0));
    let b = LocalState::primitive(Ket { label: 1, site: 0 }).scaled(Scalar(17.0));
    reset();
    assert_eq!(a.evaluated_inner_product(&b).0, 0.0);
    COUNTS.with(|c| {
        let c = c.borrow();
        assert_eq!(c.inner, [0]);
        assert_eq!((c.multiply, c.conjugate), (0, 0));
    });
    let b = LocalState::primitive(Ket { label: 4, site: 0 }).scaled(Scalar(1e-300));
    reset();
    assert_eq!(a.evaluated_inner_product(&b).0, 0.0);
    COUNTS.with(|c| assert_eq!(c.borrow().conjugate, 0));
    let a = product::<Scalar>(&f[0], &f[..1], &[0]);
    let sum = State::sum(&f[0], [a.clone(), a]).unwrap();
    let b = product::<Scalar>(&f[0], &f[..1], &[1]);
    reset();
    assert_eq!(contract(&sum.expression, &b.expression).0, 0.0);
    COUNTS.with(|c| {
        let c = c.borrow();
        assert_eq!(c.inner, [0, 0]);
        assert_eq!(c.add, 0);
    });
    let a = product::<Scalar>(&space, &f, &[0, 0, 0, 0]).scaled(Scalar(0.0));
    reset();
    assert_eq!(contract(&a.expression, &a.expression).0, 0.0);
    COUNTS.with(|c| assert!(c.borrow().inner.is_empty()));
}

#[test]
fn hs_stops_local_traces_and_does_not_divide_zero_traces() {
    let f = sites();
    for kind in [crate::TensorKind::Ordinary, crate::TensorKind::Graded] {
        let space = H::tensor_all_with(kind, f.iter().cloned()).unwrap();
        let a = operators::<Scalar>(&space, &f, None);
        for zero in [None, Some(0), Some(2), Some(3)] {
            let b = operators::<Scalar>(&space, &f, zero);
            reset();
            let value = a.hs_inner_product(&b).unwrap();
            assert_eq!(value.0, if zero.is_some() { 0.0 } else { 1.0 });
            COUNTS.with(|c| {
                let c = c.borrow();
                assert_eq!(
                    c.traces.iter().filter_map(|i| *i).collect::<Vec<_>>(),
                    (0..zero.map_or(4, |i| i + 1)).collect::<Vec<_>>()
                );
                assert_eq!(c.divide, zero.unwrap_or(4));
            });
        }
    }
}

#[test]
fn trace_and_hs_sums_keep_other_branches_and_original_space_checks() {
    let f = sites();
    let space = H::tensor_all(f.iter().cloned()).unwrap();
    let zero = operators::<f64>(&space, &f, None); // Each X has zero trace.
    let identity = Operator::<u8, Op, f64>::identity().bind(&space).unwrap();
    let sum = Operator::sum([
        zero,
        identity.clone(),
        identity.scaled(-1.0),
        identity.clone(),
    ])
    .unwrap();
    reset();
    assert_eq!(sum.trace().unwrap(), 16.0);
    COUNTS.with(|c| assert_eq!(c.borrow().traces.len(), 1 + 3 * 4));
    assert_eq!(identity.hs_inner_product(&sum).unwrap(), 1.0);
    let local = LocalOperator::<Op, Scalar>::sum([
        LocalOperator::primitive(Op {
            matrix: [[0.0, 1.0], [1.0, 0.0]],
            site: Some(0),
        }),
        LocalOperator::identity(),
    ]);
    reset();
    assert_eq!(local.trace().0, 2.0);
    let outside = H::factor(99);
    let invalid = (Operator::<u8, Op, f64>::zero()
        * LocalOperator::identity().at(&outside).unwrap())
    .unwrap();
    reset();
    assert!(matches!(
        invalid.trace_in(&space),
        Err(ExpressionError::MissingIndex)
    ));
    assert!(identity.hs_inner_product_in(&invalid, &space).is_err());
    COUNTS.with(|c| assert!(c.borrow().traces.is_empty()));
    let state = product::<f64>(&space, &f, &[0, 0, 0, 0]);
    let other = State::<u8, Ket, Op, f64>::zero(&outside);
    assert!(state.inner_product(&other).is_err());
}

#[test]
fn deep_action_free_contraction_and_destruction_use_work_stacks() {
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| {
            use crate::expression::composite::{StateExpr, StateNode};
            let factor = H::factor(0u8);
            let local = LocalState::<Ket, Op, f64>::primitive(Ket { label: 0, site: 0 })
                .at(&factor)
                .unwrap();
            let unit = State::<u8, Ket, Op, f64>::unit().expression.scaled(0.5);
            let scalar = StateExpr::sum([unit.clone(), unit].into_iter().collect());
            // Retain action-free source syntax here: public evaluate would collect
            // these unit-space sums. This directly stresses the contraction stack.
            let mut expression = local.expression.clone();
            for _ in 0..20_000 {
                expression = StateExpr::new(StateNode::Tensor(
                    [expression, scalar.clone()].into_iter().collect(),
                ));
            }
            assert_eq!(contract(&expression, &expression), 1.0);
            drop(expression);
        })
        .unwrap()
        .join()
        .unwrap();
}
