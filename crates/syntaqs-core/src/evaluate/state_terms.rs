//! Query-local natural-state shapes. IDs describe syntax, never physical bases.
use alloc::{collections::BTreeMap, vec::Vec};
use core::cmp::Ordering;
use num_traits::{One, Zero};

use super::local::{Ready as LocalReady, times};
use crate::{
    HilbertSpace, LocalState,
    expression::{
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
pub(super) struct Factor<I>(HilbertSpace<I>);

impl<I: Eq> PartialEq for Factor<I> {
    fn eq(&self, rhs: &Self) -> bool {
        self.0.factor_index() == rhs.0.factor_index()
    }
}
impl<I: Eq> Eq for Factor<I> {}
impl<I: Ord> PartialOrd for Factor<I> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}
impl<I: Ord> Ord for Factor<I> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.0.factor_index().cmp(&rhs.0.factor_index())
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Key<I: Ord> {
    Unit,
    Local(Factor<I>, Vec<(Option<usize>, usize)>),
    Tensor(Vec<usize>),
    Sum(Vec<(Option<usize>, usize)>),
}
impl<I: Ord> Clone for Key<I> {
    fn clone(&self) -> Self {
        match self {
            Self::Unit => Self::Unit,
            Self::Local(factor, terms) => Self::Local(Factor(factor.0.clone()), terms.clone()),
            Self::Tensor(children) => Self::Tensor(children.clone()),
            Self::Sum(children) => Self::Sum(children.clone()),
        }
    }
}
struct Entry<I: Ord, S, O, C> {
    key: Key<I>,
    expression: Option<StateExpr<I, S, O, C>>,
    payload: Payload<S, O, C>,
    reflexive: bool,
}
enum Payload<S, O, C> {
    // A tensor or unit needs only the shape key.
    Structure,
    Local(LocalReady<S, O, C>),
    // The key canonicalizes coefficients for matching. Keep the values from
    // the first interned sum for materializing its coefficients.
    Sum(Vec<Ready<C>>),
}
pub(super) struct Terms<I: Ord, S, O, C> {
    entries: Vec<Entry<I, S, O, C>>,
    index: BTreeMap<Key<I>, usize>,
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
impl<I: Ord, S: PartialEq, O, C: Clone + PartialEq + Zero + One> Terms<I, S, O, C> {
    pub fn new() -> Self {
        let mut result = Self {
            entries: Vec::new(),
            index: BTreeMap::new(),
            primitives: Vec::new(),
            coefficients: Vec::new(),
        };
        result.intern(Key::Sum(Vec::new()), Payload::Sum(Vec::new()), true);
        result.intern(Key::Unit, Payload::Structure, true);
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
    pub fn key(&self, id: usize) -> &Key<I> {
        &self.entries[id].key
    }
    pub fn local_parts(&self, id: usize) -> Option<(HilbertSpace<I>, LocalReady<S, O, C>)> {
        let entry = &self.entries[id];
        match (&entry.key, &entry.payload) {
            (Key::Local(factor, _), Payload::Local(local)) => {
                Some((factor.0.clone(), local.clone()))
            }
            _ => None,
        }
    }
    pub fn into_expression(mut self, ready: Ready<C>) -> StateExpr<I, S, O, C> {
        let mut pending = Vec::new();
        if self.entries[ready.shape].expression.is_none() {
            pending.push((ready.shape, false));
        }
        while let Some((id, children_ready)) = pending.pop() {
            let entry = &self.entries[id];
            if entry.expression.is_some() {
                continue;
            }
            if !children_ready {
                match (&entry.key, &entry.payload) {
                    (Key::Tensor(parts), _) => {
                        pending.push((id, true));
                        pending.extend(parts.iter().rev().map(|&shape| (shape, false)));
                        continue;
                    }
                    (_, Payload::Sum(parts)) if !parts.is_empty() => {
                        pending.push((id, true));
                        pending.extend(parts.iter().rev().map(|part| (part.shape, false)));
                        continue;
                    }
                    _ => {}
                }
            }
            let expression = match (&entry.key, &entry.payload) {
                (Key::Unit, _) => StateExpr::new(StateNode::Unit),
                (Key::Local(factor, _), Payload::Local(local)) => {
                    StateExpr::new(StateNode::Local(factor.0.clone(), local.expression()))
                }
                (Key::Tensor(parts), _) => StateExpr::tensor(
                    parts
                        .iter()
                        .map(|&part| self.materialized(part).clone())
                        .collect(),
                ),
                (_, Payload::Sum(parts)) => StateExpr::sum(
                    parts
                        .iter()
                        .map(|part| {
                            let expression = self.materialized(part.shape).clone();
                            match &part.coefficient {
                                Some(c) => expression.scaled(c.clone()),
                                None => expression,
                            }
                        })
                        .collect(),
                ),
                _ => unreachable!("state shape and payload match"),
            };
            self.entries[id].expression = Some(expression);
        }
        let expression = self.entries[ready.shape]
            .expression
            .take()
            .expect("materialized root");
        match ready.coefficient {
            Some(c) => expression.scaled(c),
            None => expression,
        }
    }

    fn materialized(&self, shape: usize) -> &StateExpr<I, S, O, C> {
        self.entries[shape]
            .expression
            .as_ref()
            .expect("materialized child")
    }

    // Save an original node if it represents the normalized shape. Compare
    // shape data and child handles without creating new expression nodes.
    pub fn reuse(&mut self, ready: &Ready<C>, source: &StateExpr<I, S, O, C>) {
        if ready.coefficient.is_some() {
            return;
        }
        let entry = &self.entries[ready.shape];
        let shares = |node: &StateExpr<I, S, O, C>, shape: usize| {
            self.entries[shape]
                .expression
                .as_ref()
                .is_some_and(|expression| node.shares_node_with(expression))
        };
        let unchanged = match (source.node(), &entry.key, &entry.payload) {
            (StateNode::Unit, Key::Unit, _) => true,
            (StateNode::Local(a, x), Key::Local(b, _), Payload::Local(local)) => {
                a.factor_index() == b.0.factor_index()
                    && matches!(local.0.as_slice(), [(None, y)] if core::ptr::eq(x.node(), y.node()))
            }
            (StateNode::Tensor(a), Key::Tensor(b), _) => {
                a.len() == b.len() && a.iter().zip(b).all(|(node, &shape)| shares(node, shape))
            }
            (StateNode::Sum(a), Key::Sum(_), Payload::Sum(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .zip(b)
                        .all(|(node, part)| part.coefficient.is_none() && shares(node, part.shape))
            }
            _ => false,
        };
        if unchanged {
            self.entries[ready.shape].expression = Some(source.clone());
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
    fn intern(&mut self, key: Key<I>, payload: Payload<S, O, C>, reflexive: bool) -> usize {
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
            expression: None,
            payload,
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
        let id = self.intern(
            Key::Local(Factor(factor), terms),
            Payload::Local(poly),
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
                let reflexive = parts.iter().all(|id| self.entries[*id].reflexive);
                self.intern(Key::Tensor(parts), Payload::Structure, reflexive)
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
                if parts
                    .iter()
                    .all(|p| matches!(self.key(p.shape), Key::Local(f, _) if f == factor))
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
                    return self.local(factor.0.clone(), LocalReady::collect(terms));
                }
            }
        }
        let mut reflexive = true;
        let mut key = Vec::with_capacity(parts.len());
        for p in &parts {
            reflexive &= self.entries[p.shape].reflexive;
            let c = p.coefficient.as_ref().filter(|c| **c != C::one());
            if let Some(c) = c {
                reflexive &= PartialEq::eq(c, c);
            }
            key.push((c.map(|c| self.coefficient_id(c)), p.shape));
        }
        let shape = self.intern(Key::Sum(key), Payload::Sum(parts), reflexive);
        Ready {
            shape,
            coefficient: None,
        }
    }
}
