//! Scalar contractions of evaluated intrinsic operator blocks.

use alloc::{borrow::Cow, collections::BTreeMap, vec::Vec};
use core::{cell::OnceCell, ops::Neg};
use num_traits::{One, Zero};

use super::{Block, EvaluatedOperator, OperatorTerm, primitive_value};
use crate::{
    Adjoint, Conjugate, Divide, HilbertSpace, HomogeneousParity, Identity, Product, Trace,
    evaluate::add_pair, expression::composite::OpExpr,
};

// A block's support belongs to one component. Overlap on either side joins
// supports, so distinct components can be contracted independently.
struct Components(Vec<usize>);

impl Components {
    fn new(count: usize) -> Self {
        Self((0..count).collect())
    }

    fn root(&mut self, mut position: usize) -> usize {
        while self.0[position] != position {
            self.0[position] = self.0[self.0[position]];
            position = self.0[position];
        }
        position
    }

    fn join(&mut self, a: usize, b: usize) {
        let a = self.root(a);
        let b = self.root(b);
        self.0[a.max(b)] = a.min(b);
    }
}

struct Group<'a, C> {
    left: Vec<&'a Block<C>>,
    right: Vec<&'a Block<C>>,
}

impl<C> Default for Group<'_, C> {
    fn default() -> Self {
        Self {
            left: Vec::new(),
            right: Vec::new(),
        }
    }
}

// Even blocks commute with any disjoint block. Two homogeneous odd
// blocks cross with a fixed sign. An inversion involving a mixed block and
// another potentially odd block needs to stay inside a common component.
fn join_mixed_inversion<C>(components: &mut Components, blocks: &[Block<C>]) -> bool {
    let mut highest = None;
    let mut highest_mixed = None;
    for block in blocks.iter().filter(|block| block.odd != Some(false)) {
        let current = components.root(block.support[0]);
        let earlier = if block.odd.is_none() {
            highest
        } else {
            highest_mixed
        };
        if let Some(earlier) = earlier.filter(|&position| position > current) {
            components.join(earlier, current);
            return true;
        }
        highest = highest.max(Some(current));
        if block.odd.is_none() {
            highest_mixed = highest_mixed.max(Some(current));
        }
    }
    false
}

fn odd_exchange_sign<C>(components: &mut Components, blocks: &[Block<C>]) -> bool {
    let mut negative = false;
    let mut earlier = Vec::new();
    let mut highest = 0;
    for block in blocks.iter().filter(|block| block.odd == Some(true)) {
        let current = components.root(block.support[0]);
        if current < highest {
            negative ^= earlier
                .iter()
                .filter(|&&position| position > current)
                .count()
                % 2
                != 0;
        }
        highest = highest.max(current);
        earlier.push(current);
    }
    negative
}

type Groups<'a, C> = BTreeMap<usize, Group<'a, C>>;

fn aligned_groups<'a, C>(
    left: &'a [Block<C>],
    right: &'a [Block<C>],
    factor_count: usize,
    graded: bool,
) -> (Groups<'a, C>, bool) {
    let mut components = Components::new(factor_count);
    for block in left.iter().chain(right) {
        let first = block.support[0];
        for &position in &block.support[1..] {
            components.join(first, position);
        }
    }

    while graded
        && (join_mixed_inversion(&mut components, left)
            || join_mixed_inversion(&mut components, right))
    {}
    let negative = graded
        && (odd_exchange_sign(&mut components, left) ^ odd_exchange_sign(&mut components, right));

    let mut groups: BTreeMap<_, Group<'a, C>> = BTreeMap::new();
    for block in left {
        groups
            .entry(components.root(block.support[0]))
            .or_default()
            .left
            .push(block);
    }
    for block in right {
        groups
            .entry(components.root(block.support[0]))
            .or_default()
            .right
            .push(block);
    }
    (groups, negative)
}

impl<O, C> EvaluatedOperator<O, C>
where
    O: PartialEq
        + Identity
        + HomogeneousParity
        + Product<C, Term = O>
        + Adjoint<C, Term = O>
        + Trace<C>,
    C: Clone + PartialEq + Zero + One + Conjugate<Output = C> + Neg<Output = C>,
{
    pub(crate) fn full_trace(&self, factor_count: usize) -> C {
        if self.product.coefficient.is_zero() {
            return C::zero();
        }
        let identity = OnceCell::new();
        let dimension = || {
            identity
                .get_or_init(|| primitive_value(self.identity()).trace())
                .clone()
        };
        let mut local_traces = Vec::new();
        local_traces.resize_with(self.primitives.len(), || None);
        let mut blocks: Vec<_> = self.product.blocks.iter().collect();
        blocks.sort_unstable_by_key(|block| block.support[0]);
        let mut spectators = factor_count;
        let mut value = C::one();
        for block in blocks {
            spectators -= block.support.len();
            let mut sum = C::zero();
            for term in &block.terms {
                // A local odd factor has zero trace even after multiplication
                // by any parity operators from factors to its right.
                if self.graded && term.factors.iter().any(|&(_, id)| self.is_odd(Some(id))) {
                    continue;
                }
                let mut factors = term.factors.iter().copied().peekable();
                let mut product = C::one();
                for &position in &block.support {
                    let local = match factors.next_if(|&(p, _)| p == position) {
                        Some((_, id)) => local_traces[id]
                            .get_or_insert_with(|| primitive_value(self.primitive(id)).trace())
                            .clone(),
                        None => dimension(),
                    };
                    if local.is_zero() {
                        product = local;
                        break;
                    }
                    product = product * local;
                    if product.is_zero() {
                        break;
                    }
                }
                if !product.is_zero() {
                    sum = sum + term.coefficient.clone() * product;
                }
            }
            if sum.is_zero() {
                return sum;
            }
            value = value * sum;
            if value.is_zero() {
                return value;
            }
        }
        for _ in 0..spectators {
            value = value * dimension();
            if value.is_zero() {
                return value;
            }
        }
        self.product.coefficient.clone() * value
    }

    fn is_odd(&self, primitive: Option<usize>) -> bool {
        primitive.is_some_and(|id| self.primitives[id].odd)
    }
}

impl<O, C> EvaluatedOperator<O, C>
where
    O: PartialEq
        + Identity
        + HomogeneousParity
        + Product<C, Term = O>
        + Adjoint<C, Term = O>
        + Trace<C>,
    C: Clone
        + PartialEq
        + Zero
        + One
        + Conjugate<Output = C>
        + Neg<Output = C>
        + Divide<Output = C>,
{
    pub(crate) fn hs_inner_product<I: Ord>(
        lhs: &OpExpr<I, O, C>,
        rhs: &OpExpr<I, O, C>,
        space: &HilbertSpace<I>,
    ) -> C {
        let mut evaluated = Self::new(lhs, space);
        let other =
            (!core::ptr::eq(lhs.node(), rhs.node())).then(|| evaluated.evaluate(rhs, space));
        let left = &evaluated.product;
        let right = other.as_ref().unwrap_or(left);
        if left.coefficient.is_zero() || right.coefficient.is_zero() {
            return C::zero();
        }
        let (groups, negative) = aligned_groups(
            &left.blocks,
            &right.blocks,
            space.factor_count(),
            evaluated.graded,
        );
        let mut contraction = HilbertSchmidt {
            operator: &evaluated,
            dimension: primitive_value(evaluated.identity()).trace(),
            overlaps: BTreeMap::new(),
        };
        let mut value = C::one();
        for group in groups.into_values() {
            let lhs = evaluated.group_terms(&group.left);
            let rhs = evaluated.group_terms(&group.right);
            let part = contraction.sum(&lhs, &rhs);
            if part.is_zero() {
                return part;
            }
            value = value * part;
            if value.is_zero() {
                return value;
            }
        }
        if negative {
            value = -value;
        }
        scale_overlap(value, &left.coefficient, &right.coefficient)
    }

    fn group_terms<'a>(&'a self, blocks: &[&'a Block<C>]) -> Cow<'a, [OperatorTerm<C>]> {
        if let [block] = blocks {
            Cow::Borrowed(&block.terms)
        } else {
            Cow::Owned(self.product_terms(C::one(), blocks).collect())
        }
    }
}

// Conjugate the outer left scalar only after the right scalar has been
// applied to the normalized overlap.
fn scale_overlap<C>(value: C, left: &C, right: &C) -> C
where
    C: Clone + Zero + Conjugate<Output = C> + core::ops::Mul<Output = C>,
{
    let value = value * right.clone();
    if value.is_zero() {
        value
    } else {
        left.conjugate() * value
    }
}

type PrimitivePair = (Option<usize>, Option<usize>);

struct HilbertSchmidt<'a, O, C> {
    operator: &'a EvaluatedOperator<O, C>,
    dimension: C,
    overlaps: BTreeMap<PrimitivePair, C>,
}

impl<O, C> HilbertSchmidt<'_, O, C>
where
    O: PartialEq
        + Identity
        + HomogeneousParity
        + Product<C, Term = O>
        + Adjoint<C, Term = O>
        + Trace<C>,
    C: Clone
        + PartialEq
        + Zero
        + One
        + Conjugate<Output = C>
        + Neg<Output = C>
        + Divide<Output = C>,
{
    fn sum(&mut self, lhs: &[OperatorTerm<C>], rhs: &[OperatorTerm<C>]) -> C {
        let matching = lhs.len() > 1 && self.operator.same_terms(lhs, rhs);
        let mut sum = C::zero();
        for (row, left) in lhs.iter().enumerate() {
            let start = if matching { row } else { 0 };
            for (offset, right) in rhs[start..].iter().enumerate() {
                let value = self.term_pair(left, right);
                sum = add_pair(sum, value, matching && offset != 0);
            }
        }
        sum
    }

    fn term_pair(&mut self, lhs: &OperatorTerm<C>, rhs: &OperatorTerm<C>) -> C {
        let mut left = lhs.factors.iter().copied().peekable();
        let mut right = rhs.factors.iter().copied().peekable();
        let mut value = C::one();
        while left.peek().is_some() || right.peek().is_some() {
            let a = left.peek().map_or(usize::MAX, |&(position, _)| position);
            let b = right.peek().map_or(usize::MAX, |&(position, _)| position);
            let x = (a <= b).then(|| left.next().unwrap().1);
            let y = (b <= a).then(|| right.next().unwrap().1);
            if self.operator.graded && self.operator.is_odd(x) != self.operator.is_odd(y) {
                return C::zero();
            }
            let overlap = self.overlap(x, y);
            if overlap.is_zero() {
                return overlap;
            }
            value = value * overlap;
            if value.is_zero() {
                return value;
            }
        }
        scale_overlap(value, &lhs.coefficient, &rhs.coefficient)
    }

    fn overlap(&mut self, left: Option<usize>, right: Option<usize>) -> C {
        let operator = self.operator;
        let d = &self.dimension;
        self.overlaps
            .entry((left, right))
            .or_insert_with(|| {
                let primitive = |id| primitive_value(operator.primitive(id));
                let lhs = left.map(primitive);
                let rhs = right.map(primitive);
                let trace = |p: &O| {
                    let value = p.trace();
                    if value.is_zero() {
                        value
                    } else {
                        value.divide(d.clone())
                    }
                };
                match (lhs, rhs) {
                    (None, None) => C::one(),
                    (Some(left), None) => trace(left).conjugate(),
                    (None, Some(right)) => trace(right),
                    (Some(left), Some(right)) => {
                        let mut sum = C::zero();
                        for adjoint in left.adjoint() {
                            if adjoint.coefficient.is_zero() {
                                continue;
                            }
                            for product in adjoint.operand.product(right) {
                                if product.coefficient.is_zero() {
                                    continue;
                                }
                                let value = trace(&product.operand);
                                if !value.is_zero() {
                                    sum = sum
                                        + adjoint.coefficient.clone()
                                            * (product.coefficient * value);
                                }
                            }
                        }
                        sum
                    }
                }
            })
            .clone()
    }
}
