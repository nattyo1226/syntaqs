//! Baseline-compatible semantic references, not tests of unimplemented rewrites.
use num_complex::Complex64 as C;
use std::{cell::Cell, collections::BTreeMap, num::NonZeroUsize};
use syntaqs_core::{
    Adjoint, Basis, Dimension, DimensionOverflow, HilbertSpace as H, HomogeneousParity, Identity,
    LinearCombination as LC, LocalOperator, Operator, Parity, Product, State, TensorKind,
};
use syntaqs_spin_half::{SpinHalf, SpinHalfState as S, SzBasis};
type Q = SpinHalf<C>;

fn close(a: C, b: C) {
    assert!((a - b).norm() < 1e-12, "{a} != {b}");
}

#[test]
fn action_free_duplicates_tensor_grouping_and_scale_positions() {
    for kind in [TensorKind::Ordinary, TensorKind::Graded] {
        let sites: Vec<_> = (0..3).map(H::factor).collect();
        let h = H::tensor_all_with(kind, sites.clone()).unwrap();
        let pair = H::tensor_all_with(kind, sites[..2].iter().cloned()).unwrap();
        let a = Q::state(S::Up).at(&sites[0]).unwrap();
        let b = Q::state(S::Down).at(&sites[1]).unwrap();
        let c = Q::state(S::Up).at(&sites[2]).unwrap();
        let ab = State::tensor_in(&pair, [a.clone(), b.clone()]).unwrap();
        let grouped = State::tensor_in(&h, [ab, c.clone()])
            .unwrap()
            .scaled(C::new(2., 0.));
        let flat = State::tensor_in(&h, [a.scaled(C::new(3., 0.)), b, c]).unwrap();
        let sum = State::sum(&h, [grouped, flat]).unwrap();
        let evaluated = sum.evaluate().unwrap();
        for candidate in [sum, evaluated.clone(), evaluated.evaluate().unwrap()] {
            let entries: Vec<_> = candidate
                .coordinates(&SzBasis)
                .unwrap()
                .into_entries()
                .collect();
            assert_eq!(entries, [(vec![0, 1, 0], C::new(5., 0.))]);
        }
    }
    let tiny = C::new(1e-30, 0.);
    let local = Q::state(S::Up).scaled(C::new(2., 0.))
        + Q::state(S::Up).scaled(C::new(-2., 0.))
        + Q::state(S::Down).scaled(tiny);
    assert_eq!(
        local
            .evaluate()
            .coordinates(&SzBasis)
            .unwrap()
            .into_entries()
            .collect::<Vec<_>>(),
        [(vec![1], tiny)]
    );
}

#[test]
fn heisenberg_powers_match_independent_bit_state_action() {
    use syntaqs_geometry::{Boundary, Chain};
    type Models = syntaqs_spin_half::models::SpinHalfModels<Q>;
    let n = 6;
    let sites: Vec<_> = (0..n).map(H::factor).collect();
    let chain = Chain::new(n, Boundary::Open);
    let h = Models::heisenberg(&chain, &sites, 1.).unwrap();
    let mut psi = Models::neel(&chain, &sites, S::Up).unwrap();
    let mut reference = vec![0.; 1 << n];
    reference[0b010101] = 1.;
    for _ in 0..4 {
        let mut next = vec![0.; reference.len()];
        for (bits, amplitude) in reference.iter().copied().enumerate() {
            for i in 0..n - 1 {
                let left = 1 << (n - 1 - i);
                let right = left >> 1;
                let z = |mask| if bits & mask == 0 { 1. } else { -1. };
                next[bits] -= 0.25 * z(left) * z(right) * amplitude;
                if z(left) != z(right) {
                    next[bits ^ left ^ right] -= 0.5 * amplitude;
                }
            }
        }
        psi = psi.act(&h).unwrap();
        let coords = psi.coordinates(&SzBasis).unwrap();
        let actual: BTreeMap<_, _> = coords
            .flat_entries()
            .map(Result::unwrap)
            .map(|(i, c)| (i, *c))
            .collect();
        for (i, value) in next.iter().copied().enumerate() {
            close(
                actual.get(&i).copied().unwrap_or_default(),
                C::new(value, 0.),
            );
        }
        reference = next;
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Matrix {
    I,
    HalfI,
    Unit(usize),
}
fn entries(p: &Matrix) -> Vec<(usize, f64)> {
    match p {
        Matrix::I => vec![(0, 1.), (3, 1.)],
        Matrix::HalfI => vec![(0, 0.5), (3, 0.5)],
        Matrix::Unit(i) => vec![(*i, 1.)],
    }
}
impl Identity for Matrix {
    fn identity() -> Self {
        Self::I
    }
}
impl Dimension for Matrix {
    fn dim() -> Result<NonZeroUsize, DimensionOverflow> {
        Ok(NonZeroUsize::new(4).unwrap())
    }
}
impl HomogeneousParity for Matrix {
    fn parity(&self) -> Parity {
        Parity::Even
    }
}
impl Adjoint<C> for Matrix {
    type Term = Self;
    fn adjoint(&self) -> LC<Self, C> {
        LC::from_terms(entries(self).into_iter().map(|(i, c)| syntaqs_core::Term {
            coefficient: C::new(c, 0.),
            operand: Self::Unit((i % 2) * 2 + i / 2),
        }))
    }
}
impl Product<C> for Matrix {
    type Term = Self;
    fn product(&self, rhs: &Self) -> LC<Self, C> {
        LC::from_terms(entries(self).into_iter().flat_map(|(a, c)| {
            entries(rhs).into_iter().filter_map(move |(b, d)| {
                (a % 2 == b / 2).then_some(syntaqs_core::Term {
                    coefficient: C::new(c * d, 0.),
                    operand: Self::Unit((a / 2) * 2 + b % 2),
                })
            })
        }))
    }
}
struct MatrixBasis {
    scaled_identity: bool,
    identity_calls: Cell<usize>,
}
impl Basis<Matrix, C> for MatrixBasis {
    fn expand(&self, p: &Matrix) -> LC<Matrix, C> {
        if *p == Matrix::I {
            self.identity_calls.set(self.identity_calls.get() + 1);
        }
        let mut result = Vec::new();
        for (i, c) in entries(p) {
            if self.scaled_identity && i == 3 {
                result.push(syntaqs_core::Term {
                    coefficient: C::new(2. * c, 0.),
                    operand: Matrix::HalfI,
                });
                result.push(syntaqs_core::Term {
                    coefficient: C::new(-c, 0.),
                    operand: Matrix::Unit(0),
                });
            } else {
                result.push(syntaqs_core::Term {
                    coefficient: C::new(c, 0.),
                    operand: Matrix::Unit(i),
                });
            }
        }
        LC::from_terms(result)
    }
    fn index(&self, p: &Matrix) -> Option<usize> {
        match p {
            Matrix::HalfI if self.scaled_identity => Some(3),
            Matrix::Unit(i) if *i < 4 && (!self.scaled_identity || *i != 3) => Some(*i),
            _ => None,
        }
    }
    fn elements(&self) -> impl Iterator<Item = Matrix> + '_ {
        (0..4).map(|i| {
            if i == 3 && self.scaled_identity {
                Matrix::HalfI
            } else {
                Matrix::Unit(i)
            }
        })
    }
}

#[test]
fn spectators_support_multicomponent_and_scaled_nonzero_index_identity() {
    let sites: Vec<_> = (0..3).map(H::factor).collect();
    let h = H::tensor_all(sites.clone()).unwrap();
    let op = LocalOperator::primitive(Matrix::Unit(0))
        .at(&sites[1])
        .unwrap()
        .scaled(C::new(3., 0.));
    for scaled_identity in [false, true] {
        let basis = MatrixBasis {
            scaled_identity,
            identity_calls: Cell::new(0),
        };
        let result = op.coordinates_in(&basis, &h).unwrap();
        let expected = if scaled_identity {
            BTreeMap::from([(vec![3, 0, 3], C::new(12., 0.))])
        } else {
            BTreeMap::from([
                (vec![0, 0, 0], C::new(3., 0.)),
                (vec![0, 0, 3], C::new(3., 0.)),
                (vec![3, 0, 0], C::new(3., 0.)),
                (vec![3, 0, 3], C::new(3., 0.)),
            ])
        };
        assert_eq!(result.into_entries().collect::<BTreeMap<_, _>>(), expected);
        let roundtrip = op
            .expand_in(&basis, &h)
            .unwrap()
            .coordinates_in(&basis, &h)
            .unwrap();
        assert_eq!(
            roundtrip.into_entries().collect::<BTreeMap<_, _>>(),
            expected
        );
        let fully_specified = Operator::product(sites.iter().map(|s| {
            LocalOperator::<_, C>::primitive(Matrix::Unit(0))
                .at(s)
                .unwrap()
        }))
        .unwrap();
        basis.identity_calls.set(0);
        assert_eq!(fully_specified.coordinates_in(&basis, &h).unwrap().len(), 1);
        assert_eq!(basis.identity_calls.get(), 0);
        let unit = H::<usize>::unit();
        assert_eq!(
            Operator::<usize, Matrix, C>::identity()
                .coordinates_in(&basis, &unit)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(basis.identity_calls.get(), 0);
    }
}
