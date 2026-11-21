//! Natural operator terms in products of blocks with disjoint factor support.

use alloc::{collections::BTreeMap, vec, vec::Vec};
use num_traits::{One, Zero};

use crate::{
    Adjoint, Conjugate, HilbertSpace, HomogeneousParity, Identity, LocalOperator, Parity, Product,
    TensorKind,
    expression::{
        composite::{OpExpr, OpNode},
        local::OperatorNode,
        traversal::{LinearTerms, operator_handle_view},
    },
};
use core::{borrow::Borrow, cell::OnceCell, ops::Neg};

mod commutator;
mod contraction;

// Factor positions are in ambient order. Primitive numbers are valid only in
// the evaluation that produced the terms.
type Factors = Vec<(usize, usize)>;
#[derive(Clone, PartialEq)]
pub(crate) struct OperatorTerm<C> {
    pub(crate) coefficient: C,
    pub(crate) factors: Factors,
}

struct Primitive<O, C> {
    local: LocalOperator<O, C>,
    odd: bool,
    reflexive: bool,
}

// A block is a sum of ordered primitive products. Different blocks in a
// product have disjoint supports, but their factor positions may interleave.
struct Block<C> {
    terms: Vec<OperatorTerm<C>>,
    support: Vec<usize>,
    odd: Option<bool>,
}

struct Factored<C> {
    coefficient: C,
    blocks: Vec<Block<C>>,
}

#[derive(Clone, Copy)]
enum ProductEnd {
    Front,
    Back,
}

impl<C> Factored<C> {
    fn scalar(coefficient: C) -> Self {
        Self {
            coefficient,
            blocks: Vec::new(),
        }
    }
}

struct Sum<C> {
    terms: Vec<OperatorTerm<C>>,
    positions: BTreeMap<Factors, usize>,
}

impl<C: Clone + Zero> Sum<C> {
    fn new() -> Self {
        Self {
            terms: Vec::new(),
            positions: BTreeMap::new(),
        }
    }
    fn add<O>(&mut self, term: OperatorTerm<C>, primitives: &[Primitive<O, C>]) {
        if term.coefficient.is_zero() {
            return;
        }
        if term.factors.iter().all(|&(_, id)| primitives[id].reflexive) {
            if let Some(&index) = self.positions.get(&term.factors) {
                let c = &mut self.terms[index].coefficient;
                *c = c.clone() + term.coefficient;
                return;
            }
            self.positions
                .insert(term.factors.clone(), self.terms.len());
        }
        self.terms.push(term);
    }

    fn finish<O>(mut self, primitives: &[Primitive<O, C>]) -> Block<C> {
        self.terms.retain(|t| !t.coefficient.is_zero());
        let mut support: Vec<_> = self
            .terms
            .iter()
            .flat_map(|t| t.factors.iter().map(|&(position, _)| position))
            .collect();
        support.sort_unstable();
        support.dedup();

        let mut grades = self.terms.iter().map(|t| {
            t.factors
                .iter()
                .fold(false, |odd, &(_, id)| odd ^ primitives[id].odd)
        });
        let odd = grades
            .next()
            .filter(|&first| grades.all(|odd| odd == first));
        Block {
            terms: self.terms,
            support,
            odd,
        }
    }
}

impl<C: Zero + One> Factored<C> {
    fn from_block(mut block: Block<C>) -> Self {
        if block.support.is_empty() {
            return Self::scalar(block.terms.pop().map_or_else(C::zero, |t| t.coefficient));
        }
        Self {
            coefficient: C::one(),
            blocks: vec![block],
        }
    }
}

// None means the exchange would depend on the selected pair of summands.
fn exchange(left: Option<bool>, right: Option<bool>) -> Option<bool> {
    match (left, right) {
        (Some(false), _) | (_, Some(false)) => Some(false),
        (Some(true), Some(true)) => Some(true),
        _ => None,
    }
}

fn overlaps(left: &[usize], right: &[usize]) -> bool {
    let mut right = right.iter().peekable();
    for position in left {
        while right.peek().is_some_and(|p| *p < position) {
            right.next();
        }
        if right.peek() == Some(&position) {
            return true;
        }
    }
    false
}

fn primitive_value<O, C>(local: &LocalOperator<O, C>) -> &O {
    match local.node() {
        OperatorNode::Primitive(value) => value,
        _ => unreachable!("evaluated primitive"),
    }
}

/// Basis-independent evaluation of a validated operator on its selected space.
pub(crate) struct EvaluatedOperator<O, C> {
    product: Factored<C>,
    primitives: Vec<Primitive<O, C>>,
    // Identity is represented by the empty product, including when returned by
    // a local multiplication. Share its handle with the spectator expansion.
    identity: OnceCell<LocalOperator<O, C>>,
    graded: bool,
}

enum Task<'a, I, O, C> {
    Visit(&'a OpExpr<I, O, C>, bool),
    Scale(C),
    Sum(usize),
    Product(usize),
}

impl<O, C> EvaluatedOperator<O, C>
where
    O: PartialEq + Identity + HomogeneousParity + Product<C, Term = O> + Adjoint<C, Term = O>,
    C: Clone + PartialEq + Zero + One + Conjugate<Output = C> + Neg<Output = C>,
{
    pub(crate) fn new<I: Ord>(root: &OpExpr<I, O, C>, space: &HilbertSpace<I>) -> Self {
        let mut result = Self::empty(space);
        result.product = result.evaluate(root, space);
        result
    }

    fn empty<I: Ord>(space: &HilbertSpace<I>) -> Self {
        Self {
            product: Factored::scalar(C::zero()),
            primitives: Vec::new(),
            identity: OnceCell::new(),
            graded: space.effective_kind() == Some(TensorKind::Graded),
        }
    }

    fn evaluate<I: Ord>(&mut self, root: &OpExpr<I, O, C>, space: &HilbertSpace<I>) -> Factored<C> {
        // Collapse purely local syntax through the existing local algebra.
        let root = root.group_local();
        let mut tasks = vec![Task::Visit(&root, false)];
        let mut values: Vec<Factored<C>> = Vec::new();
        while let Some(task) = tasks.pop() {
            match task {
                Task::Visit(op, adjoint) => match op.node() {
                    OpNode::Identity => values.push(Factored::scalar(C::one())),
                    OpNode::Local(factor, local) => {
                        let position = space
                            .factor_position(factor.factor_index().expect("placed factor"))
                            .expect("validated factor");
                        let local = if adjoint {
                            local.adjoint()
                        } else {
                            local.clone()
                        };
                        values.push(self.evaluate_local(position, &local));
                    }
                    OpNode::Scale(s) => {
                        if s.coefficient.is_zero() {
                            values.push(Factored::scalar(C::zero()));
                            continue;
                        }
                        let c = if adjoint {
                            s.coefficient.conjugate()
                        } else {
                            s.coefficient.clone()
                        };
                        tasks.push(Task::Scale(c));
                        tasks.push(Task::Visit(&s.operand, adjoint));
                    }
                    OpNode::Sum(children) => {
                        tasks.push(Task::Sum(children.len()));
                        tasks.extend(children.iter().rev().map(|op| Task::Visit(op, adjoint)));
                    }
                    OpNode::Product(children) => {
                        tasks.push(Task::Product(children.len()));
                        if adjoint {
                            tasks.extend(children.iter().map(|op| Task::Visit(op, true)));
                        } else {
                            tasks.extend(children.iter().rev().map(|op| Task::Visit(op, false)));
                        }
                    }
                    OpNode::Adjoint(a) => tasks.push(Task::Visit(&a.operand, !adjoint)),
                },
                Task::Scale(c) => {
                    let value = values.last_mut().expect("scaled operator");
                    if !value.coefficient.is_zero() {
                        value.coefficient = c * value.coefficient.clone();
                    }
                }
                Task::Sum(count) => {
                    let parts = values.split_off(values.len() - count);
                    values.push(self.add(parts));
                }
                Task::Product(count) => {
                    let product = values
                        .drain(values.len() - count..)
                        .fold(Factored::scalar(C::one()), |left, right| {
                            self.multiply(left, right)
                        });
                    values.push(product);
                }
            }
        }
        values.pop().expect("evaluated operator")
    }

    pub(crate) fn identity(&self) -> &LocalOperator<O, C> {
        self.identity.get_or_init(LocalOperator::identity)
    }

    fn register(&mut self, local: &LocalOperator<O, C>) -> Option<usize> {
        let p = primitive_value(local);
        if primitive_value(self.identity()) == p {
            return None;
        }
        if let Some(id) = self
            .primitives
            .iter()
            .position(|other| primitive_value(&other.local) == p)
        {
            return Some(id);
        }
        let id = self.primitives.len();
        self.primitives.push(Primitive {
            local: local.clone(),
            odd: self.graded && p.parity() == Parity::Odd,
            reflexive: PartialEq::eq(p, p),
        });
        Some(id)
    }

    fn evaluate_local(&mut self, position: usize, local: &LocalOperator<O, C>) -> Factored<C> {
        let local = local.evaluate();
        let mut sum = Sum::new();
        for (coefficient, local) in LinearTerms::new(&local, operator_handle_view) {
            let coefficient = coefficient.unwrap_or_else(C::one);
            if coefficient.is_zero() {
                continue;
            }
            let factors = self
                .register(local)
                .map(|id| (position, id))
                .into_iter()
                .collect();
            sum.add(
                OperatorTerm {
                    coefficient,
                    factors,
                },
                &self.primitives,
            );
        }
        Factored::from_block(sum.finish(&self.primitives))
    }

    fn same_terms(&self, lhs: &[OperatorTerm<C>], rhs: &[OperatorTerm<C>]) -> bool {
        lhs == rhs
            && lhs.iter().all(|term| {
                term.factors
                    .iter()
                    .all(|&(_, id)| self.primitives[id].reflexive)
            })
    }

    fn take_common_blocks(&self, parts: &mut [Factored<C>]) -> (Vec<Block<C>>, Vec<Block<C>>) {
        if parts.iter().any(|part| part.blocks.is_empty()) {
            return (Vec::new(), Vec::new());
        }
        let first = &parts[0].blocks;
        // The first support position uniquely identifies a block within a product.
        let mut common: BTreeMap<_, _> = first
            .iter()
            .enumerate()
            .map(|(index, block)| (block.support[0], (index, Vec::new())))
            .collect();
        for (part_index, part) in parts.iter().enumerate().skip(1) {
            for (index, block) in part.blocks.iter().enumerate() {
                if let Some((first_index, matches)) = common.get_mut(&block.support[0]) {
                    if self.same_terms(&first[*first_index].terms, &block.terms) {
                        matches.push(index);
                    }
                }
            }
            common.retain(|_, (_, matches)| matches.len() == part_index);
            if common.is_empty() {
                return (Vec::new(), Vec::new());
            }
        }
        let mut common: Vec<_> = common.into_values().collect();
        common.sort_unstable_by_key(|(index, _)| *index);
        let mut taken: Vec<_> = parts
            .iter()
            .map(|part| vec![None; part.blocks.len()])
            .collect();
        let candidates = common
            .iter()
            .map(|candidate| (ProductEnd::Front, candidate))
            .chain(
                common
                    .iter()
                    .rev()
                    .map(|candidate| (ProductEnd::Back, candidate)),
            );
        let mut signs = Vec::with_capacity(parts.len());
        for (end, (first_index, matches)) in candidates {
            if taken[0][*first_index].is_some() {
                continue;
            }
            let indices = || core::iter::once(*first_index).chain(matches.iter().copied());
            let odd = parts[0].blocks[*first_index].odd;
            signs.clear();
            signs.extend(parts.iter().zip(&taken).zip(indices()).map_while(
                |((part, taken), index)| {
                    if !self.graded || odd == Some(false) {
                        return Some(false);
                    }
                    let crossed = match end {
                        ProductEnd::Front => 0..index,
                        ProductEnd::Back => index + 1..part.blocks.len(),
                    };
                    // Extracted blocks are already outside the remaining product.
                    crossed
                        .filter(|&i| taken[i].is_none())
                        .try_fold(false, |sign, i| {
                            exchange(odd, part.blocks[i].odd).map(|negative| sign ^ negative)
                        })
                },
            ));
            if signs.len() == parts.len() {
                for (((part, taken), index), &negative) in
                    parts.iter_mut().zip(&mut taken).zip(indices()).zip(&signs)
                {
                    taken[index] = Some(end);
                    if negative {
                        part.coefficient = -part.coefficient.clone();
                    }
                }
            }
        }

        let mut leading = Vec::new();
        let mut trailing = Vec::new();
        for (index, (part, taken)) in parts.iter_mut().zip(taken).enumerate() {
            if taken.iter().all(Option::is_none) {
                continue;
            }
            for (block, end) in core::mem::take(&mut part.blocks).into_iter().zip(taken) {
                match end {
                    None => part.blocks.push(block),
                    Some(ProductEnd::Front) if index == 0 => leading.push(block),
                    Some(ProductEnd::Back) if index == 0 => trailing.push(block),
                    Some(_) => {}
                }
            }
        }
        (leading, trailing)
    }

    fn add(&self, mut parts: Vec<Factored<C>>) -> Factored<C> {
        parts.retain(|part| !part.coefficient.is_zero());
        match parts.len() {
            0 => return Factored::scalar(C::zero()),
            1 => return parts.pop().unwrap(),
            _ => {}
        }
        // Adjoint reversal can change the order of independent blocks.
        // Put commuting blocks in the same order before comparing them.
        for part in &mut parts {
            if !self.graded || part.blocks.iter().all(|block| block.odd == Some(false)) {
                part.blocks.sort_unstable_by_key(|block| block.support[0]);
            }
        }
        let first = &parts[0];
        // Ordered common prefixes and suffixes factor by distributivity in
        // either tensor kind. Every remaining block has disjoint support from
        // these common factors, and no exchange is needed to extract them.
        let shortest = parts.iter().map(|part| part.blocks.len()).min().unwrap();
        let same = |a: &Block<C>, b: &Block<C>| self.same_terms(&a.terms, &b.terms);
        let prefix = (0..shortest)
            .take_while(|&i| {
                parts[1..]
                    .iter()
                    .all(|part| same(&first.blocks[i], &part.blocks[i]))
            })
            .count();
        let suffix = (1..=shortest - prefix)
            .take_while(|&i| {
                parts[1..].iter().all(|part| {
                    same(
                        &first.blocks[first.blocks.len() - i],
                        &part.blocks[part.blocks.len() - i],
                    )
                })
            })
            .count();

        let first = &mut parts[0].blocks;
        let trailing = first.split_off(first.len() - suffix);
        let mut leading: Vec<_> = first.drain(..prefix).collect();
        for part in &mut parts[1..] {
            part.blocks.truncate(part.blocks.len() - suffix);
            part.blocks.drain(..prefix);
        }
        let (common_leading, common_trailing) = self.take_common_blocks(&mut parts);

        let mut sum = Sum::new();
        for part in parts {
            for term in self.product_terms(part.coefficient, &part.blocks) {
                sum.add(term, &self.primitives);
            }
        }
        let mut middle = Factored::from_block(sum.finish(&self.primitives));
        if middle.coefficient.is_zero() {
            return middle;
        }
        leading.extend(common_leading);
        leading.append(&mut middle.blocks);
        leading.extend(common_trailing);
        leading.extend(trailing);
        middle.blocks = leading;
        middle
    }

    fn multiply(&mut self, mut left: Factored<C>, right: Factored<C>) -> Factored<C> {
        if left.coefficient.is_zero() || right.coefficient.is_zero() {
            return Factored::scalar(C::zero());
        }
        left.coefficient = left.coefficient * right.coefficient;
        if left.coefficient.is_zero() {
            return Factored::scalar(left.coefficient);
        }
        for mut carry in right.blocks {
            let first = left
                .blocks
                .iter()
                .position(|b| overlaps(&b.support, &carry.support));
            let Some(first) = first else {
                left.blocks.push(carry);
                continue;
            };
            // Bring the incoming block up to its first overlap, retaining any
            // intervening independent blocks that exchange with a uniform sign.
            for index in (first..left.blocks.len()).rev() {
                let block = &left.blocks[index];
                let sign = if overlaps(&block.support, &carry.support) {
                    None
                } else {
                    exchange(block.odd, carry.odd)
                };
                if let Some(negative) = sign {
                    if negative {
                        left.coefficient = -left.coefficient;
                    }
                } else {
                    carry = self.multiply_blocks(left.blocks.remove(index), &carry);
                    if carry.terms.is_empty() {
                        return Factored::scalar(C::zero());
                    }
                }
            }
            if carry.support.is_empty() {
                let constant = carry.terms.pop().expect("scalar term").coefficient;
                left.coefficient = left.coefficient * constant;
                if left.coefficient.is_zero() {
                    return Factored::scalar(left.coefficient);
                }
            } else {
                left.blocks.insert(first, carry);
            }
        }
        left
    }

    fn multiply_blocks(&mut self, left: Block<C>, right: &Block<C>) -> Block<C> {
        let mut sum = Sum::new();
        for a in &left.terms {
            for b in &right.terms {
                for term in self.multiply_terms(a, b) {
                    sum.add(term, &self.primitives);
                }
            }
        }
        sum.finish(&self.primitives)
    }

    // Merge ordered placements. At an equal position maintain the local
    // left-to-right product. A right odd factor crosses only the greater left
    // positions, including when the two factors at the current position match.
    fn multiply_terms(
        &mut self,
        left: &OperatorTerm<C>,
        right: &OperatorTerm<C>,
    ) -> Vec<OperatorTerm<C>> {
        let mut coefficient = left.coefficient.clone() * right.coefficient.clone();
        if coefficient.is_zero() {
            return Vec::new();
        }
        let mut a = left.factors.iter().copied().peekable();
        let mut odd_suffix = left
            .factors
            .iter()
            .fold(false, |odd, &(_, id)| odd ^ self.primitives[id].odd);
        let mut factors = Vec::with_capacity(left.factors.len() + right.factors.len());
        let mut local_products = Vec::new();
        for &(position, rhs) in &right.factors {
            while a.peek().is_some_and(|&(p, _)| p < position) {
                let factor = a.next().unwrap();
                odd_suffix ^= self.primitives[factor.1].odd;
                factors.push(factor);
            }
            let same = a.next_if(|&(p, _)| p == position).map(|(_, id)| id);
            if let Some(lhs) = same {
                odd_suffix ^= self.primitives[lhs].odd;
            }
            if odd_suffix && self.primitives[rhs].odd {
                coefficient = -coefficient;
            }
            if let Some(lhs) = same {
                let local = primitive_value(self.primitive(lhs))
                    .product(primitive_value(self.primitive(rhs)));
                let choices: Vec<_> = local
                    .into_terms()
                    .filter_map(|term| {
                        if term.coefficient.is_zero() {
                            return None;
                        }
                        let id = self.register(&LocalOperator::primitive(term.operand));
                        Some((id, term.coefficient))
                    })
                    .collect();
                if choices.is_empty() {
                    return Vec::new();
                }
                local_products.push((position, choices));
            } else {
                factors.push((position, rhs));
            }
        }
        factors.extend(a);
        let mut terms = vec![OperatorTerm {
            coefficient,
            factors,
        }];
        for (position, choices) in local_products {
            terms = terms
                .into_iter()
                .flat_map(|term| {
                    choices.iter().filter_map(move |(id, c)| {
                        let coefficient = term.coefficient.clone() * c.clone();
                        if coefficient.is_zero() {
                            return None;
                        }
                        let mut factors = term.factors.clone();
                        if let Some(id) = *id {
                            let index = factors.partition_point(|&(p, _)| p < position);
                            factors.insert(index, (position, id));
                        }
                        Some(OperatorTerm {
                            coefficient,
                            factors,
                        })
                    })
                })
                .collect();
        }
        terms
    }

    pub(crate) fn primitive(&self, id: usize) -> &LocalOperator<O, C> {
        &self.primitives[id].local
    }

    pub(crate) fn terms(&self) -> impl Iterator<Item = OperatorTerm<C>> + '_ {
        self.product_terms(self.product.coefficient.clone(), &self.product.blocks)
    }

    fn product_terms<'a, B: Borrow<Block<C>> + 'a>(
        &'a self,
        coefficient: C,
        blocks: &'a [B],
    ) -> impl Iterator<Item = OperatorTerm<C>> + 'a {
        let mut next = (!coefficient.is_zero()).then(|| vec![0; blocks.len()]);
        core::iter::from_fn(move || {
            while let Some(indices) = &mut next {
                let mut coefficient = coefficient.clone();
                let mut factors: Factors = Vec::new();
                let mut negative = false;
                for (block, &index) in blocks.iter().zip(indices.iter()) {
                    let term = &block.borrow().terms[index];
                    coefficient = coefficient * term.coefficient.clone();
                    for &(position, id) in &term.factors {
                        if self.primitives[id].odd {
                            for &(p, previous) in &factors {
                                if p > position && self.primitives[previous].odd {
                                    negative = !negative;
                                }
                            }
                        }
                        factors.push((position, id));
                    }
                }

                let mut carry = true;
                for (index, block) in indices.iter_mut().zip(blocks).rev() {
                    *index += 1;
                    if *index < block.borrow().terms.len() {
                        carry = false;
                        break;
                    }
                    *index = 0;
                }
                if carry {
                    next = None;
                }
                if !coefficient.is_zero() {
                    factors.sort_unstable_by_key(|&(position, _)| position);
                    return Some(OperatorTerm {
                        coefficient: if negative { -coefficient } else { coefficient },
                        factors,
                    });
                }
            }
            None
        })
    }

    pub(crate) fn expression<I>(&self, factors: &[&crate::HilbertSpace<I>]) -> OpExpr<I, O, C> {
        if self.product.coefficient.is_zero() {
            return OpExpr::sum(Default::default());
        }
        let blocks = self.product.blocks.iter().map(|block| {
            if let &[position] = block.support.as_slice() {
                let local = LocalOperator::sum(block.terms.iter().map(|term| {
                    let primitive = match term.factors.as_slice() {
                        [] => self.identity(),
                        &[(_, id)] => self.primitive(id),
                        _ => unreachable!("one-factor block"),
                    };
                    primitive.scaled(term.coefficient.clone())
                }));
                return OpExpr::new(OpNode::Local(factors[position].clone(), local));
            }
            OpExpr::sum(
                block
                    .terms
                    .iter()
                    .map(|term| {
                        let operands = term.factors.iter().map(|&(position, id)| {
                            OpExpr::new(OpNode::Local(
                                factors[position].clone(),
                                self.primitive(id).clone(),
                            ))
                        });
                        OpExpr::product(operands.collect()).scaled(term.coefficient.clone())
                    })
                    .collect(),
            )
        });
        OpExpr::product(blocks.collect()).scaled(self.product.coefficient.clone())
    }
}
