//! Tensor-product basis expansion and sparse coordinates.

use super::{Expansion, accumulate, local::evaluated_local_state, operator::operator_expansion};
use crate::basis::enumeration::{collect_leaves, local_size};
use crate::{
    Act, Adjoint, Basis, BasisError, Conjugate, Coordinates, Dimension, HilbertSpace, Identity,
    LocalOperator, LocalState, Operator, Product, State,
    expression::{
        composite::{OpExpr, OpNode, StateExpr, StateNode},
        parts::ScaleExpr,
    },
    space::FactorPositions,
};
use alloc::{collections::BTreeMap, vec, vec::Vec};
use core::ops::Mul;
use num_traits::{One, Zero};

// Coordinates carry no primitive payload. Expansion carries only cheap local
// expression handles, so neither path needs Clone/Eq/Ord on primitives.
trait TensorValue: Clone {
    fn unit() -> Self;
    fn tensor(&self, rhs: &Self) -> Self;
}
impl TensorValue for () {
    fn unit() -> Self {}
    fn tensor(&self, _: &Self) -> Self {}
}
impl<E: Clone> TensorValue for Vec<E> {
    fn unit() -> Self {
        Vec::new()
    }
    fn tensor(&self, rhs: &Self) -> Self {
        self.iter().chain(rhs).cloned().collect()
    }
}

fn tensor<V: TensorValue, C: Clone + Zero + Mul<Output = C>>(
    lhs: Expansion<V, C>,
    rhs: Expansion<V, C>,
) -> Expansion<V, C> {
    let mut result = BTreeMap::new();
    for (a, (c, x)) in lhs {
        for (b, (d, y)) in &rhs {
            let mut digits = a.clone();
            digits.extend(b);
            accumulate(&mut result, digits, c.clone() * d.clone(), x.tensor(y));
        }
    }
    result
}

enum StateTask<'a, I, S, O, C> {
    Visit(&'a StateExpr<I, S, O, C>),
    Scale(&'a C),
    Sum(usize),
    Tensor(usize),
}

fn state_expansion<I: Ord, S, O, C, B, V: TensorValue>(
    state: &State<I, S, O, C>,
    basis: &B,
    dimension: Option<usize>,
    make: impl Fn(S) -> V,
) -> Result<Expansion<V, C>, BasisError>
where
    C: PartialEq + Clone + Zero + One + Conjugate<Output = C>,
    S: PartialEq + Act<C, Operator = O, Term = S>,
    O: Adjoint<C, Term = O> + crate::HomogeneousParity + crate::ParityOperator,
    B: Basis<S, C>,
{
    let state = state.evaluate()?;
    let mut tasks = vec![StateTask::Visit(&state.expression)];
    let mut values: Vec<Expansion<V, C>> = Vec::new();
    while let Some(task) = tasks.pop() {
        match task {
            StateTask::Visit(node) => match node.node() {
                StateNode::Local(_, local) => {
                    let terms = evaluated_local_state(
                        local,
                        basis,
                        dimension.expect("local factor"),
                        &make,
                    )?;
                    values.push(terms.into_iter().map(|(i, v)| (vec![i], v)).collect());
                }
                StateNode::Unit => {
                    values.push(BTreeMap::from([(Vec::new(), (C::one(), V::unit()))]))
                }
                StateNode::Scale(s) => {
                    tasks.push(StateTask::Scale(&s.coefficient));
                    tasks.push(StateTask::Visit(&s.operand));
                }
                StateNode::Sum(s) | StateNode::Tensor(s) => {
                    tasks.push(if matches!(node.node(), StateNode::Sum(_)) {
                        StateTask::Sum(s.len())
                    } else {
                        StateTask::Tensor(s.len())
                    });
                    tasks.extend(s.iter().rev().map(StateTask::Visit));
                }
                StateNode::Act(_) => unreachable!("evaluated state"),
            },
            StateTask::Scale(c) => {
                let value = values.last_mut().expect("scale operand");
                for (d, _) in value.values_mut() {
                    *d = c.clone() * core::mem::replace(d, C::zero());
                }
                value.retain(|_, (d, _)| !d.is_zero());
            }
            StateTask::Sum(count) => {
                let mut sum = BTreeMap::new();
                for term in values.drain(values.len() - count..) {
                    for (digits, (c, v)) in term {
                        accumulate(&mut sum, digits, c, v);
                    }
                }
                values.push(sum);
            }
            StateTask::Tensor(count) => {
                let factors = values.split_off(values.len() - count);
                let mut product = BTreeMap::from([(Vec::new(), (C::one(), V::unit()))]);
                for factor in factors {
                    product = tensor(product, factor);
                }
                values.push(product);
            }
        }
    }
    let mut result = values.pop().expect("one expansion");
    result.retain(|_, (c, _)| !c.is_zero());
    Ok(result)
}

impl<I: Ord, S, O, C> State<I, S, O, C> {
    /// Expands in one local basis applied uniformly in the space's factor order.
    ///
    /// Returns state syntax preserving the tensor kind on the original shared space. No global
    /// dimension, primitive clone, or primitive operator product is required.
    /// Evaluates/collects first; removed zero states do not run basis rules.
    pub fn expand<B>(&self, basis: &B) -> Result<Self, BasisError>
    where
        C: PartialEq + Clone + Zero + One + Conjugate<Output = C>,
        S: PartialEq + Dimension + Act<C, Operator = O, Term = S>,
        O: Adjoint<C, Term = O> + crate::HomogeneousParity + crate::ParityOperator,
        B: Basis<S, C>,
    {
        let factors = collect_leaves(&self.space);
        let dimension = local_size::<S>(factors.len())?;
        let terms = state_expansion(self, basis, dimension, |p| vec![LocalState::primitive(p)])?;
        let expressions = terms
            .into_values()
            .map(|(c, locals)| {
                let nodes = locals
                    .into_iter()
                    .zip(&factors)
                    .map(|(local, factor)| {
                        StateExpr::new(StateNode::Local((*factor).clone(), local))
                    })
                    .collect();
                StateExpr::tensor(nodes).scaled(c)
            })
            .collect();
        Ok(Self {
            space: self.space.clone(),
            expression: StateExpr::sum(expressions),
        })
    }

    /// Obtains sparse coordinates with the original space and borrowed local basis.
    pub fn coordinates<'b, B>(
        &self,
        basis: &'b B,
    ) -> Result<Coordinates<'b, B, C, HilbertSpace<I>>, BasisError>
    where
        C: PartialEq + Clone + Zero + One + Conjugate<Output = C>,
        S: PartialEq + Dimension + Act<C, Operator = O, Term = S>,
        O: Adjoint<C, Term = O> + crate::HomogeneousParity + crate::ParityOperator,
        B: Basis<S, C>,
    {
        let count = self.space.indices().count();
        let dimension = local_size::<S>(count)?;
        let terms = state_expansion(self, basis, dimension, |_| ())?;
        Ok(Coordinates {
            basis,
            space: self.space.clone(),
            factor_count: count,
            dim_local: dimension,
            entries: terms.into_iter().map(|(k, (c, ()))| (k, c)).collect(),
        })
    }
}

impl<I: Ord, O, C> Operator<I, O, C> {
    /// Expands on the existing binding; an unbound operator returns an error.
    pub fn expand<B>(&self, basis: &B) -> Result<Self, BasisError>
    where
        C: Clone + PartialEq + Zero + One + Conjugate<Output = C> + core::ops::Neg<Output = C>,
        O: PartialEq
            + crate::HomogeneousParity
            + Dimension
            + Identity
            + Product<C, Term = O>
            + Adjoint<C, Term = O>,
        B: Basis<O, C>,
    {
        self.expand_in(basis, self.bound_space()?)
    }

    /// Expands on an explicit space, overriding any previous binding.
    ///
    /// Spectators are expanded as local identities in the same basis, even when
    /// identity itself is not a basis primitive. Its spectator expansion is shared
    /// lazily within this query. Zero retains the selected space.
    pub fn expand_in<B>(&self, basis: &B, ambient: &HilbertSpace<I>) -> Result<Self, BasisError>
    where
        C: Clone + PartialEq + Zero + One + Conjugate<Output = C> + core::ops::Neg<Output = C>,
        O: PartialEq
            + crate::HomogeneousParity
            + Dimension
            + Identity
            + Product<C, Term = O>
            + Adjoint<C, Term = O>,
        B: Basis<O, C>,
    {
        let positions = FactorPositions::new(ambient);
        self.validate(&positions)?;
        let factors = positions.leaves();
        let dimension = local_size::<O>(factors.len())?;
        let terms = operator_expansion(self, basis, &positions, dimension, |p| {
            vec![LocalOperator::primitive(p)]
        })?;
        let expressions = terms
            .into_values()
            .map(|(c, locals)| {
                let operands = locals
                    .into_iter()
                    .zip(factors)
                    .map(|(local, factor)| OpExpr::new(OpNode::Local((*factor).clone(), local)));
                OpExpr::new(OpNode::Scale(ScaleExpr {
                    operand: OpExpr::product(operands.collect()),
                    coefficient: c,
                }))
            })
            .collect();
        Ok(Self {
            ambient: Some(ambient.clone()),
            expression: OpExpr::sum(expressions),
        })
    }

    /// Obtains operator-basis coordinates using the existing binding.
    pub fn coordinates<'b, B>(
        &self,
        basis: &'b B,
    ) -> Result<Coordinates<'b, B, C, HilbertSpace<I>>, BasisError>
    where
        C: Clone + PartialEq + Zero + One + Conjugate<Output = C> + core::ops::Neg<Output = C>,
        O: PartialEq
            + crate::HomogeneousParity
            + Dimension
            + Identity
            + Product<C, Term = O>
            + Adjoint<C, Term = O>,
        B: Basis<O, C>,
    {
        self.coordinates_in(basis, self.bound_space()?)
    }

    /// Obtains operator-basis coordinates on an explicit space, overriding a binding.
    /// Shares the spectator identity expansion within this query and validates
    /// local expansions for the terms that survive evaluation and collection.
    pub fn coordinates_in<'b, B>(
        &self,
        basis: &'b B,
        ambient: &HilbertSpace<I>,
    ) -> Result<Coordinates<'b, B, C, HilbertSpace<I>>, BasisError>
    where
        C: Clone + PartialEq + Zero + One + Conjugate<Output = C> + core::ops::Neg<Output = C>,
        O: PartialEq
            + crate::HomogeneousParity
            + Dimension
            + Identity
            + Product<C, Term = O>
            + Adjoint<C, Term = O>,
        B: Basis<O, C>,
    {
        let positions = FactorPositions::new(ambient);
        self.validate(&positions)?;
        let factors = positions.leaves();
        let dimension = local_size::<O>(factors.len())?;
        let terms = operator_expansion(self, basis, &positions, dimension, |_| ())?;
        Ok(Coordinates {
            basis,
            space: ambient.clone(),
            factor_count: factors.len(),
            dim_local: dimension,
            entries: terms.into_iter().map(|(k, (c, ()))| (k, c)).collect(),
        })
    }
}
