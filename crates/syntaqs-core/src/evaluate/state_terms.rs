//! Query-local natural-state shapes. IDs describe syntax, never physical bases.
use alloc::{collections::BTreeMap, vec::Vec};
use num_traits::{One, Zero};

use super::local::{Ready as LocalReady, times};
use crate::{
    HilbertSpace, LocalState,
    expr::{
        composite::{StateExpr, StateNode},
        local::StateNode as LocalNode,
    },
};

pub(super) struct Ready<C> {
    pub coefficient: Option<C>,
    pub shape: usize,
}
impl<C: Clone> Clone for Ready<C> {
    fn clone(&self) -> Self {
        Self {
            coefficient: self.coefficient.clone(),
            shape: self.shape,
        }
    }
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Key {
    Unit,
    Local(usize, Vec<(Option<usize>, usize)>),
    Tensor(Vec<usize>),
    Sum(Vec<(Option<usize>, usize)>),
}
struct Entry<I, S, O, C> {
    key: Key,
    expression: StateExpr<I, S, O, C>,
    local: Option<(HilbertSpace<I>, LocalReady<S, O, C>)>,
    reflexive: bool,
}
pub(super) struct Terms<I, S, O, C> {
    entries: Vec<Entry<I, S, O, C>>,
    index: BTreeMap<Key, usize>,
    factors: Vec<HilbertSpace<I>>,
    primitives: Vec<LocalState<S, O, C>>,
    coefficients: Vec<C>,
}
// A separate output order preserves first occurrence independently of lookup.
#[derive(Default)]
pub(super) struct Sum<C> {
    parts: Vec<Ready<C>>,
    positions: BTreeMap<usize, usize>,
}
impl<C> Sum<C> {
    pub fn new() -> Self {
        Self {
            parts: Vec::new(),
            positions: BTreeMap::new(),
        }
    }
}
impl<I: Eq, S: PartialEq, O, C: Clone + PartialEq + Zero + One> Terms<I, S, O, C> {
    pub fn new() -> Self {
        let mut result = Self {
            entries: Vec::new(),
            index: BTreeMap::new(),
            factors: Vec::new(),
            primitives: Vec::new(),
            coefficients: Vec::new(),
        };
        result.intern(
            Key::Sum(Vec::new()),
            StateExpr::sum(Default::default()),
            None,
            true,
        );
        result.intern(Key::Unit, StateExpr::new(StateNode::Unit), None, true);
        result
    }
    pub fn zero(&self) -> Ready<C> {
        Ready {
            coefficient: None,
            shape: 0,
        }
    }
    pub fn unit(&self) -> Ready<C> {
        Ready {
            coefficient: None,
            shape: 1,
        }
    }
    pub fn key(&self, id: usize) -> &Key {
        &self.entries[id].key
    }
    pub fn local_parts(&self, id: usize) -> Option<(HilbertSpace<I>, LocalReady<S, O, C>)> {
        self.entries[id]
            .local
            .as_ref()
            .map(|(f, p)| (f.clone(), p.clone()))
    }
    pub fn expression(&self, ready: &Ready<C>) -> StateExpr<I, S, O, C> {
        let expression = self.entries[ready.shape].expression.clone();
        match &ready.coefficient {
            Some(c) => expression.scaled(c.clone()),
            None => expression,
        }
    }
    // Reuse an original node only when its children already are the normalized
    // handles. This is sharing, not a shortcut for user-value equality.
    pub fn reuse(&mut self, ready: &Ready<C>, source: &StateExpr<I, S, O, C>) {
        if ready.coefficient.is_some() {
            return;
        }
        let entry = &mut self.entries[ready.shape];
        let unchanged = match (source.node(), entry.expression.node()) {
            (StateNode::Unit, StateNode::Unit) => true,
            (StateNode::Local(a, x), StateNode::Local(b, y)) => {
                a.factor_index() == b.factor_index() && core::ptr::eq(x.node(), y.node())
            }
            (StateNode::Tensor(a), StateNode::Tensor(b))
            | (StateNode::Sum(a), StateNode::Sum(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| a.shares_node_with(b))
            }
            _ => false,
        };
        if unchanged {
            entry.expression = source.clone();
        }
    }
    pub fn scaled(&mut self, ready: Ready<C>, c: C) -> Ready<C> {
        self.normalize_coefficient(ready.shape, times(Some(c), ready.coefficient))
    }
    fn normalize_coefficient(&mut self, shape: usize, c: Option<C>) -> Ready<C> {
        if let Some(c) = c {
            if let Some((factor, poly)) = self.local_parts(shape) {
                if poly.0.len() > 1 {
                    return self.local(factor, poly.scaled(c));
                }
            }
            self.with_coefficient(shape, Some(c))
        } else {
            self.with_coefficient(shape, None)
        }
    }
    fn with_coefficient(&self, shape: usize, c: Option<C>) -> Ready<C> {
        if shape == 0 || c.as_ref().is_some_and(Zero::is_zero) {
            return self.zero();
        }
        Ready {
            shape,
            coefficient: c.filter(|c| *c != C::one()),
        }
    }
    fn coefficient_id(&mut self, c: &C) -> usize {
        if let Some(i) = self.coefficients.iter().position(|other| other == c) {
            i
        } else {
            let i = self.coefficients.len();
            self.coefficients.push(c.clone());
            i
        }
    }
    fn intern(
        &mut self,
        key: Key,
        expression: StateExpr<I, S, O, C>,
        local: Option<(HilbertSpace<I>, LocalReady<S, O, C>)>,
        reflexive: bool,
    ) -> usize {
        if reflexive {
            if let Some(id) = self.index.get(&key) {
                return *id;
            }
        }
        let id = self.entries.len();
        if reflexive {
            self.index.insert(key.clone(), id);
        }
        self.entries.push(Entry {
            key,
            expression,
            local,
            reflexive,
        });
        id
    }
    pub fn local(&mut self, factor: HilbertSpace<I>, poly: LocalReady<S, O, C>) -> Ready<C> {
        if poly.0.is_empty() {
            return self.zero();
        }
        let (outer, poly) = if poly.0.len() == 1 {
            let (c, p) = &poly.0[0];
            (c.clone(), LocalReady::collect([(None, p.clone())]))
        } else {
            (None, poly)
        };
        let factor_id = match self
            .factors
            .iter()
            .position(|f| f.factor_index() == factor.factor_index())
        {
            Some(id) => id,
            None => {
                self.factors.push(factor.clone());
                self.factors.len() - 1
            }
        };
        let mut reflexive = true;
        let mut terms = Vec::new();
        for (c, p) in poly.0.iter() {
            let LocalNode::Primitive(value) = p.node() else {
                unreachable!()
            };
            reflexive &= PartialEq::eq(value, value);
            let primitive = match self.primitives.iter().position(|other| {
                let LocalNode::Primitive(other) = other.node() else {
                    unreachable!()
                };
                other == value
            }) {
                Some(id) => id,
                None => {
                    self.primitives.push(p.clone());
                    self.primitives.len() - 1
                }
            };
            let c = c.as_ref().filter(|c| **c != C::one());
            if let Some(c) = c {
                reflexive &= PartialEq::eq(c, c);
            }
            terms.push((c.map(|c| self.coefficient_id(c)), primitive));
        }
        let expression = StateExpr::new(StateNode::Local(factor.clone(), poly.expression()));
        let id = self.intern(
            Key::Local(factor_id, terms),
            expression,
            Some((factor, poly)),
            reflexive,
        );
        self.with_coefficient(id, outer)
    }
    pub fn children(&self, id: usize) -> Vec<Ready<C>> {
        match self.key(id) {
            Key::Tensor(parts) => parts
                .iter()
                .map(|id| Ready {
                    shape: *id,
                    coefficient: None,
                })
                .collect(),
            Key::Sum(parts) => parts
                .iter()
                .map(|(c, id)| Ready {
                    shape: *id,
                    coefficient: c.map(|i| self.coefficients[i].clone()),
                })
                .collect(),
            _ => unreachable!("container shape"),
        }
    }
    pub fn tensor(&mut self, children: Vec<Ready<C>>) -> Ready<C> {
        let mut pending: Vec<_> = children.into_iter().rev().collect();
        let mut parts = Vec::new();
        let mut coefficient = None;
        while let Some(child) = pending.pop() {
            if child.shape == 0 {
                return self.zero();
            }
            coefficient = times(coefficient, child.coefficient);
            match self.key(child.shape) {
                Key::Unit => {}
                Key::Tensor(ids) => pending.extend(ids.iter().rev().map(|id| Ready {
                    shape: *id,
                    coefficient: None,
                })),
                _ => parts.push(child.shape),
            }
        }
        let id = match parts.as_slice() {
            [] => 1,
            [id] => *id,
            _ => {
                let expression = StateExpr::new(StateNode::Tensor(
                    parts
                        .iter()
                        .map(|id| self.entries[*id].expression.clone())
                        .collect(),
                ));
                let reflexive = parts.iter().all(|id| self.entries[*id].reflexive);
                self.intern(Key::Tensor(parts), expression, None, reflexive)
            }
        };
        self.normalize_coefficient(id, coefficient)
    }
    pub fn add(&self, sum: &mut Sum<C>, part: Ready<C>) {
        let mut pending = alloc::vec![part];
        while let Some(part) = pending.pop() {
            if part.shape == 0 || part.coefficient.as_ref().is_some_and(Zero::is_zero) {
                continue;
            }
            if matches!(self.key(part.shape), Key::Sum(_)) {
                let mut children = self.children(part.shape);
                for child in &mut children {
                    child.coefficient = times(part.coefficient.clone(), child.coefficient.take());
                }
                pending.extend(children.into_iter().rev());
                continue;
            }
            let slot = if self.entries[part.shape].reflexive {
                sum.positions.get(&part.shape).copied()
            } else {
                None
            };
            if let Some(slot) = slot {
                let previous = &mut sum.parts[slot];
                previous.coefficient = Some(
                    previous.coefficient.take().unwrap_or_else(C::one)
                        + part.coefficient.unwrap_or_else(C::one),
                );
            } else {
                if self.entries[part.shape].reflexive {
                    sum.positions.insert(part.shape, sum.parts.len());
                }
                sum.parts.push(part);
            }
        }
    }
    pub fn finish_sum(&mut self, sum: Sum<C>) -> Ready<C> {
        let mut parts = sum.parts;
        parts.retain(|p| !p.coefficient.as_ref().is_some_and(Zero::is_zero));
        if parts.is_empty() {
            return self.zero();
        }
        if parts.len() == 1 {
            let p = parts.pop().unwrap();
            return self.normalize_coefficient(p.shape, p.coefficient);
        }
        // A placed local sum and a sum of placed locals share one polynomial.
        if let Some(first) = parts.first() {
            if let Key::Local(factor, _) = self.key(first.shape) {
                let factor = *factor;
                if parts
                    .iter()
                    .all(|p| matches!(self.key(p.shape),Key::Local(f,_) if *f==factor))
                {
                    let mut terms = Vec::new();
                    for part in parts {
                        let (_, poly) = self.local_parts(part.shape).unwrap();
                        terms.extend(
                            poly.0.iter().map(|(c, p)| {
                                (times(part.coefficient.clone(), c.clone()), p.clone())
                            }),
                        );
                    }
                    return self.local(self.factors[factor].clone(), LocalReady::collect(terms));
                }
            }
        }
        let collected = parts;
        let expression = StateExpr::sum(collected.iter().map(|p| self.expression(p)).collect());
        let mut reflexive = true;
        let mut key = Vec::new();
        for p in collected {
            reflexive &= self.entries[p.shape].reflexive;
            let c = p.coefficient.filter(|c| *c != C::one());
            if let Some(c) = &c {
                reflexive &= PartialEq::eq(c, c);
            }
            key.push((c.as_ref().map(|c| self.coefficient_id(c)), p.shape));
        }
        let shape = self.intern(Key::Sum(key), expression, None, reflexive);
        Ready {
            shape,
            coefficient: None,
        }
    }
}
