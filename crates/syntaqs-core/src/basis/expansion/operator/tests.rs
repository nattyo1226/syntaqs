use super::*;
use crate::{Dimension, DimensionOverflow, LinearCombination as Lc, TensorKind};
use core::ops::{Add, Mul};
use std::{cell::RefCell, num::NonZeroUsize};

#[derive(Default)]
struct Counts {
    identities: usize,
    dimensions: usize,
    basis: Vec<u8>,
    payloads: usize,
}
std::thread_local! {static COUNTS:RefCell<Counts>=RefCell::new(Counts::default());}
fn reset() {
    COUNTS.with(|c| *c.borrow_mut() = Counts::default());
}

// No primitive Clone/PartialEq; basis data describes actual real 2x2 matrices.
enum P {
    I,
    HalfI,
    Z,
    Unit(u8),
    X,
    Zero,
    Bad(u8),
}
impl P {
    fn matrix(&self) -> [[f64; 2]; 2] {
        match self {
            Self::I => [[1.0, 0.0], [0.0, 1.0]],
            Self::HalfI => [[0.5, 0.0], [0.0, 0.5]],
            Self::Z => [[1.0, 0.0], [0.0, -1.0]],
            Self::X => [[0.0, 1.0], [1.0, 0.0]],
            Self::Unit(i) => core::array::from_fn(|r| {
                core::array::from_fn(|c| f64::from(*i as usize == 2 * r + c))
            }),
            Self::Zero => [[0.0; 2]; 2],
            Self::Bad(_) => panic!("invalid basis primitive"),
        }
    }
    fn terms<C: From<f64>>(matrix: [[f64; 2]; 2]) -> Lc<Self, C> {
        if matrix == [[1.0, 0.0], [0.0, 1.0]] {
            return Lc::term(C::from(1.0), Self::I);
        }
        Lc::from_terms((0..4).filter_map(|i| {
            let c = matrix[i / 2][i % 2];
            (c != 0.0).then(|| crate::Term {
                coefficient: C::from(c),
                operand: Self::Unit(i as u8),
            })
        }))
    }
}
impl Identity for P {
    fn identity() -> Self {
        COUNTS.with(|c| c.borrow_mut().identities += 1);
        Self::I
    }
}
impl Dimension for P {
    fn dim() -> Result<NonZeroUsize, DimensionOverflow> {
        COUNTS.with(|c| c.borrow_mut().dimensions += 1);
        Ok(NonZeroUsize::new(4).unwrap())
    }
}
impl HomogeneousParity for P {
    fn parity(&self) -> crate::Parity {
        crate::Parity::Even
    }
}
impl<C: From<f64>> Product<C> for P {
    type Term = Self;
    fn product(&self, rhs: &Self) -> Lc<Self, C> {
        let a = self.matrix();
        let b = rhs.matrix();
        Self::terms(core::array::from_fn(|i| {
            core::array::from_fn(|j| a[i][0] * b[0][j] + a[i][1] * b[1][j])
        }))
    }
}
impl<C: From<f64>> Adjoint<C> for P {
    type Term = Self;
    fn adjoint(&self) -> Lc<Self, C> {
        let a = self.matrix();
        Self::terms(core::array::from_fn(|i| core::array::from_fn(|j| a[j][i])))
    }
}
#[derive(Clone, Copy)]
enum Mode {
    Cartesian,
    Scaled,
    Units,
}
#[derive(Clone, Copy)]
enum IdentityRule {
    Normal,
    Empty,
    Bad(u8),
}
struct B {
    mode: Mode,
    identity: IdentityRule,
    bad: Option<u8>,
}
impl B {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            identity: IdentityRule::Normal,
            bad: None,
        }
    }
}
impl<C: From<f64> + 'static> Basis<P, C> for B {
    fn expand(&self, p: &P) -> Lc<P, C> {
        let tag = match p {
            P::I => 0,
            P::HalfI => 1,
            P::Z => 2,
            P::Unit(i) => 10 + i,
            P::X => 3,
            P::Zero => 4,
            P::Bad(_) => 99,
        };
        COUNTS.with(|c| c.borrow_mut().basis.push(tag));
        if matches!(p, P::I) {
            match self.identity {
                IdentityRule::Normal => {}
                IdentityRule::Empty => return Lc::zero(),
                IdentityRule::Bad(i) => return Lc::term(C::from(1.0), P::Bad(i)),
            }
        }
        if matches!(p,P::Unit(i) if Some(*i)==self.bad) {
            return Lc::term(C::from(1.0), P::Bad(0));
        }
        let matrix = p.matrix();
        let a = matrix[0][0];
        let b = matrix[0][1];
        let c = matrix[1][0];
        let d = matrix[1][1];
        let terms = match self.mode {
            Mode::Cartesian => vec![
                ((a + d) / 2.0, P::I),
                (b, P::Unit(1)),
                (c, P::Unit(2)),
                ((a - d) / 2.0, P::Z),
            ],
            Mode::Scaled => vec![
                (a - d, P::Unit(0)),
                (b, P::Unit(1)),
                (c, P::Unit(2)),
                (2.0 * d, P::HalfI),
            ],
            Mode::Units => vec![
                (a, P::Unit(0)),
                (b, P::Unit(1)),
                (c, P::Unit(2)),
                (d, P::Unit(3)),
            ],
        };
        Lc::from_terms(
            terms
                .into_iter()
                .filter(|(c, _)| *c != 0.0)
                .map(|(c, operand)| crate::Term {
                    coefficient: C::from(c),
                    operand,
                }),
        )
    }
    fn index(&self, p: &P) -> Option<usize> {
        match (self.mode, p) {
            (Mode::Cartesian, P::I) => Some(0),
            (Mode::Cartesian, P::Z) => Some(3),
            (Mode::Cartesian | Mode::Scaled, P::Unit(i @ 1..=2)) => Some(*i as usize),
            (Mode::Scaled, P::HalfI) => Some(3),
            (Mode::Scaled, P::Unit(0)) => Some(0),
            (Mode::Units, P::Unit(i @ 0..=3)) => Some(*i as usize),
            (_, P::Bad(0)) => None,
            (_, P::Bad(i)) => Some(*i as usize),
            _ => None,
        }
    }
    fn elements(&self) -> impl Iterator<Item = P> {
        match self.mode {
            Mode::Cartesian => vec![P::I, P::Unit(1), P::Unit(2), P::Z],
            Mode::Scaled => vec![P::Unit(0), P::Unit(1), P::Unit(2), P::HalfI],
            Mode::Units => (0..4).map(P::Unit).collect(),
        }
        .into_iter()
    }
}
#[derive(PartialEq, Eq)]
struct Index(u8); // No Clone/Hash/Ord/Debug.
type Op<C = f64> = Operator<Index, P, C>;
fn sites(n: u8) -> Vec<HilbertSpace<Index>> {
    (0..n)
        .rev()
        .map(|i| HilbertSpace::factor(Index(i)))
        .collect()
}
fn placed<C>(p: P, f: &HilbertSpace<Index>) -> Op<C> {
    LocalOperator::primitive(p).at(f).unwrap()
}
fn full<C>(f: &[HilbertSpace<Index>]) -> Op<C> {
    Op::try_product(f.iter().map(|f| LocalOperator::primitive(P::Unit(1)).at(f))).unwrap()
}
fn entries(
    op: &Op,
    b: &B,
    h: &HilbertSpace<Index>,
) -> alloc::collections::BTreeMap<Vec<usize>, f64> {
    op.coordinates_in(b, h).unwrap().into_entries().collect()
}

#[test]
fn identity_expansion_is_shared_once_per_query_and_unused_when_fully_specified() {
    let f = sites(5);
    let h = HilbertSpace::tensor_all(f.iter().cloned()).unwrap();
    let b = B::new(Mode::Cartesian);
    let op = Op::sum(f.iter().map(|f| placed::<f64>(P::Unit(1), f))).unwrap();
    let expected: alloc::collections::BTreeMap<_, _> = (0..5)
        .map(|i| {
            let mut key = vec![0; 5];
            key[i] = 1;
            (key, 1.0)
        })
        .collect();
    reset();
    assert_eq!(entries(&op, &b, &h), expected);
    COUNTS.with(|c| {
        let c = c.borrow();
        assert_eq!(c.identities, 1);
        assert_eq!(c.basis.iter().filter(|t| **t == 0).count(), 1);
        assert_eq!(c.basis.len(), 6);
    });
    reset();
    let expanded = op.expand_in(&b, &h).unwrap();
    COUNTS.with(|c| {
        let c = c.borrow();
        assert_eq!(c.identities, 1);
        assert_eq!(c.basis.len(), 6);
    });
    assert_eq!(entries(&expanded, &b, &h), expected);
    reset();
    assert_eq!(entries(&full(&f), &b, &h).len(), 1);
    COUNTS.with(|c| {
        let c = c.borrow();
        assert_eq!(c.identities, 0);
        assert_eq!(c.basis.len(), 5);
    });
    let reverse = Op::<f64>::try_product(
        f.iter()
            .rev()
            .map(|f| LocalOperator::primitive(P::Unit(1)).at(f)),
    )
    .unwrap();
    assert_eq!(
        entries(&reverse, &b, &h),
        alloc::collections::BTreeMap::from([(vec![1; 5], 1.0)])
    );
    let unit = HilbertSpace::<Index>::unit();
    reset();
    assert_eq!(entries(&Op::identity(), &b, &unit).get(&vec![]), Some(&1.0));
    COUNTS.with(|c| {
        let c = c.borrow();
        assert_eq!((c.identities, c.dimensions, c.basis.len()), (0, 0, 0));
    });
    reset();
    assert!(entries(&Op::zero(), &b, &h).is_empty());
    COUNTS.with(|c| assert!(c.borrow().basis.is_empty()));
    let mut empty = B::new(Mode::Cartesian);
    empty.identity = IdentityRule::Empty;
    let mixed = Op::sum([placed(P::Unit(1), &f[0]), full(&f)]).unwrap();
    assert_eq!(
        entries(&mixed, &empty, &h),
        alloc::collections::BTreeMap::from([(vec![1; 5], 1.0)])
    );
}

#[test]
fn single_component_nonzero_index_and_nonunit_weight_restore_working_digits() {
    let f = sites(3);
    let h = HilbertSpace::tensor_all(f.iter().cloned()).unwrap();
    let b = B::new(Mode::Scaled);
    let all = full(&f);
    let op = Op::sum([
        all.clone(),
        placed(P::Unit(1), &f[1]).scaled(3.0),
        placed(P::Unit(0), &f[0]).scaled(-2.0),
        all.scaled(-1.0),
    ])
    .unwrap();
    let expected =
        alloc::collections::BTreeMap::from([(vec![3, 1, 3], 12.0), (vec![0, 3, 3], -8.0)]);
    reset();
    assert_eq!(entries(&op, &b, &h), expected);
    COUNTS.with(|c| assert_eq!(c.borrow().identities, 1));
    assert_eq!(entries(&op.expand_in(&b, &h).unwrap(), &b, &h), expected);
    let op = placed(P::Unit(3), &f[1]);
    assert_eq!(
        entries(&op, &b, &h),
        alloc::collections::BTreeMap::from([(vec![3, 0, 3], -4.0), (vec![3, 3, 3], 8.0)])
    );
    let b = B::new(Mode::Cartesian);
    assert_eq!(
        entries(&placed(P::Unit(0), &f[1]), &b, &h),
        alloc::collections::BTreeMap::from([(vec![0, 0, 0], 0.5), (vec![0, 3, 0], 0.5)])
    );
}

#[test]
fn multicomponent_identity_and_local_choices_enumerate_complete_keys() {
    let f = sites(3);
    let h = HilbertSpace::tensor_all(f.iter().cloned()).unwrap();
    let b = B::new(Mode::Units);
    let sparse = placed(P::X, &f[1]);
    let dense = Op::try_product(f.iter().map(|f| LocalOperator::primitive(P::X).at(f))).unwrap();
    let op = Op::sum([sparse, dense]).unwrap();
    let expected: alloc::collections::BTreeMap<_, _> = (0..2)
        .flat_map(|a| (1..=2).flat_map(move |i| (0..2).map(move |c| (vec![3 * a, i, 3 * c], 1.0))))
        .chain(
            (1..=2).flat_map(|a| {
                (1..=2).flat_map(move |b| (1..=2).map(move |c| (vec![a, b, c], 1.0)))
            }),
        )
        .collect();
    reset();
    assert_eq!(entries(&op, &b, &h), expected);
    COUNTS.with(|c| assert_eq!(c.borrow().identities, 1));
    assert_eq!(entries(&op.expand_in(&b, &h).unwrap(), &b, &h), expected);
}

#[test]
fn zero_or_empty_contributions_do_not_hide_later_basis_errors() {
    let f = sites(3);
    let h = HilbertSpace::tensor_all(f.iter().cloned()).unwrap();
    for expand in [false, true] {
        let query = |op: &Op, b: &B| {
            if expand {
                op.expand_in(b, &h).map(|_| ())
            } else {
                op.coordinates_in(b, &h).map(|_| ())
            }
        };
        let mut b = B::new(Mode::Cartesian);
        b.bad = Some(2);
        let bad = placed(P::Unit(2), &f[2]).scaled(0.0);
        assert!(matches!(query(&bad, &b), Err(BasisError::NonBasisElement)));
        let empty = Op::try_product([
            LocalOperator::primitive(P::Zero).at(&f[0]),
            LocalOperator::primitive(P::Unit(2)).at(&f[2]),
        ])
        .unwrap();
        assert!(matches!(
            query(&empty, &b),
            Err(BasisError::NonBasisElement)
        ));
        b.identity = IdentityRule::Empty;
        reset();
        assert!(matches!(query(&bad, &b), Err(BasisError::NonBasisElement)));
        COUNTS.with(|c| assert_eq!(c.borrow().basis, [0, 12]));
        b.identity = IdentityRule::Bad(4);
        reset();
        assert!(matches!(
            query(&bad, &b),
            Err(BasisError::IndexOutOfBounds {
                index: 4,
                dimension: 4
            })
        ));
        COUNTS.with(|c| assert_eq!(c.borrow().basis, [0]));
        b.identity = IdentityRule::Normal;
        let sum = Op::sum([placed(P::Unit(1), &f[0]), bad]).unwrap();
        reset();
        assert!(matches!(query(&sum, &b), Err(BasisError::NonBasisElement)));
        COUNTS.with(|c| assert_eq!(c.borrow().basis.iter().filter(|t| **t == 0).count(), 1));
        b.identity = IdentityRule::Bad(0);
        let outside = HilbertSpace::factor(Index(99));
        reset();
        let invalid = placed(P::Unit(2), &outside).scaled(0.0);
        assert!(matches!(
            query(&invalid, &b),
            Err(BasisError::Expression(crate::ExpressionError::MissingIndex))
        ));
        COUNTS.with(|c| assert!(c.borrow().basis.is_empty()));
    }
}

// Neither PartialEq, division nor Float; all public operator-coordinate bounds
// remain satisfied with equality-free coefficient and non-Clone primitives/index.
#[derive(Clone)]
struct Scalar(f64);
impl From<f64> for Scalar {
    fn from(c: f64) -> Self {
        Self(c)
    }
}
impl Add for Scalar {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}
impl Mul for Scalar {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}
impl Neg for Scalar {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0)
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
        self.clone()
    }
}
#[test]
fn queries_keep_capabilities_and_original_grading_path() {
    let f = sites(3);
    let h = HilbertSpace::tensor_all_with(TensorKind::Graded, f.iter().cloned()).unwrap();
    let b = B::new(Mode::Scaled);
    let op = placed::<Scalar>(P::Unit(1), &f[1]).scaled(Scalar(3.0));
    let c = op.coordinates_in(&b, &h).unwrap();
    let (key, value) = c.entries().next().unwrap();
    assert_eq!(key, [3, 1, 3]);
    assert_eq!(value.0, 12.0);
    let e = op
        .expand_in(&b, &h)
        .unwrap()
        .coordinates_in(&b, &h)
        .unwrap();
    assert_eq!(e.entries().next().unwrap().1.0, 12.0);
}

#[test]
fn expanded_payloads_share_identity_handles_and_join_only_for_new_keys() {
    use crate::expression::{composite::OpNode, local::OperatorNode};
    let f = sites(3);
    let h = HilbertSpace::tensor_all(f.iter().cloned()).unwrap();
    let b = B::new(Mode::Cartesian);
    let term = (placed::<f64>(P::Unit(1), &f[0]) * placed(P::Unit(1), &f[1])).unwrap();
    let op = Op::sum([term.clone(), term]).unwrap();
    #[derive(Clone)]
    struct Payload(usize);
    impl OperatorValue for Payload {
        fn product<'a>(values: impl Iterator<Item = &'a Self>) -> Self
        where
            Self: 'a,
        {
            COUNTS.with(|c| c.borrow_mut().payloads += 1);
            Self(values.map(|v| v.0).sum())
        }
    }
    let factors: Vec<_> = h.leaves().collect();
    reset();
    let result = operator_expansion(&op, &b, &h, &factors, Some(4), |_| Payload(1)).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result.values().next().unwrap().0, 2.0);
    assert_eq!(result.values().next().unwrap().1.0, 3);
    COUNTS.with(|c| assert_eq!(c.borrow().payloads, 1));
    let op = Op::sum(f.iter().map(|f| placed::<f64>(P::Unit(1), f))).unwrap();
    let expanded = op.expand_in(&b, &h).unwrap();
    let mut pending = vec![&expanded.expression];
    let mut identities = Vec::new();
    while let Some(op) = pending.pop() {
        match op.node() {
            OpNode::Local(_, local) => {
                if matches!(local.node(), OperatorNode::Primitive(P::I)) {
                    identities.push(local.node());
                }
            }
            OpNode::Scale(s) => pending.push(&s.operand),
            OpNode::Sum(s) => pending.extend(s.iter()),
            OpNode::Product(p) => {
                pending.push(&p.lhs);
                pending.push(&p.rhs);
            }
            _ => panic!("expanded syntax"),
        }
    }
    assert_eq!(identities.len(), 6);
    assert!(identities.iter().all(|p| core::ptr::eq(*p, identities[0])));
}

#[test]
fn spectator_coefficient_multiplication_keeps_ambient_order_without_division() {
    let choices = [(0, 1.0_f64, ()), (1, 2.0, ())];
    let axes = [Axis {
        position: 1,
        choices: &choices,
    }];
    for (coefficient, gamma) in [(1e300, 1e-200), (1e-300, 1e200)] {
        let identity = (3, gamma, ());
        let mut key = vec![3; 3];
        let mut values = Vec::new();
        enumerate(
            &axes,
            &mut key,
            coefficient,
            Some(&identity),
            |key, c, _| values.push((key.to_vec(), c)),
        );
        assert_eq!(
            values,
            [
                (vec![3, 0, 3], ((coefficient * gamma) * 1.0) * gamma),
                (vec![3, 1, 3], ((coefficient * gamma) * 2.0) * gamma)
            ]
        );
        assert!(values.iter().all(|(_, c)| *c != 0.0 && c.is_finite()));
    }
}

#[test]
fn deep_choice_enumeration_uses_a_work_stack() {
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| {
            let one = [(0, 1.0, ())];
            let two = [(0, 1.0, ()), (1, 1.0, ())];
            let axes: Vec<_> = (0..20_000)
                .map(|position| Axis {
                    position,
                    choices: if position == 19_999 {
                        &two[..]
                    } else {
                        &one[..]
                    },
                })
                .collect();
            let mut key = vec![0; axes.len()];
            let mut count = 0;
            enumerate(&axes, &mut key, 1.0, None, |key, c, _| {
                assert_eq!(key.len(), 20_000);
                assert_eq!(c, 1.0);
                count += 1;
            });
            assert_eq!(count, 2);
        })
        .unwrap()
        .join()
        .unwrap();
}
