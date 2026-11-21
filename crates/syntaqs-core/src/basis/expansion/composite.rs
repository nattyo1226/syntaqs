//! Tensor-product basis expansion and sparse coordinates.

use super::{
    Expansion, accumulate,
    local::{indexed_terms, local_expansion},
    operator::operator_expansion,
    reconstruct,
};
use crate::basis::enumeration::{collect_leaves, local_size};
use crate::{
    Act, Adjoint, Basis, BasisError, Conjugate, Coordinates, Dimension, HilbertSpace, Identity,
    LocalOperator, LocalState, Operator, Product, State,
    expression::{
        composite::{OpExpr, OpNode, StateExpr, StateNode},
        traversal::{LinearTerms, state_view},
    },
};
use alloc::{
    collections::{BTreeMap, btree_map},
    vec,
    vec::Vec,
};
use core::ops::Range;
use num_traits::{One, Zero};

struct TensorAxis<'a, C> {
    positions: Range<usize>,
    choices: &'a Expansion<C>,
}

struct TensorFrame<'a, C> {
    choices: btree_map::Iter<'a, Vec<usize>, C>,
    prefix: C,
}

fn tensor<C: Clone + Zero + One>(factors: &[Expansion<C>]) -> Expansion<C> {
    let mut axes = Vec::with_capacity(factors.len());
    let mut width = 0;
    for choices in factors {
        let Some((digits, _)) = choices.first_key_value() else {
            return Expansion::new();
        };
        let end = width + digits.len();
        axes.push(TensorAxis {
            positions: width..end,
            choices,
        });
        width = end;
    }
    let Some(first) = axes.first() else {
        return BTreeMap::from([(Vec::new(), C::one())]);
    };
    let mut key = vec![0; width];
    let mut result = Expansion::new();
    let mut frames = Vec::with_capacity(axes.len());
    frames.push(TensorFrame {
        choices: first.choices.iter(),
        prefix: C::one(),
    });
    while !frames.is_empty() {
        let depth = frames.len() - 1;
        let frame = frames.last_mut().unwrap();
        let Some((digits, coefficient)) = frame.choices.next() else {
            frames.pop();
            continue;
        };
        let value = frame.prefix.clone() * coefficient.clone();
        if value.is_zero() {
            continue;
        }
        key[axes[depth].positions.clone()].copy_from_slice(digits);
        if let Some(next) = axes.get(depth + 1) {
            frames.push(TensorFrame {
                choices: next.choices.iter(),
                prefix: value,
            });
        } else {
            // Child tuples are unique and occupy disjoint slices, so each
            // complete coordinate appears exactly once.
            result.insert(key.clone(), value);
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

fn state_expansion<I: Ord, S, O, C, B>(
    state: &State<I, S, O, C>,
    basis: &B,
    dimension: Option<usize>,
) -> Result<Expansion<C>, BasisError>
where
    C: PartialEq + Clone + Zero + One + Conjugate<Output = C>,
    S: PartialEq + Act<C, Operator = O, Term = S>,
    O: Adjoint<C, Term = O> + crate::HomogeneousParity + crate::ParityOperator,
    B: Basis<S, C>,
{
    let state = state.evaluate()?;
    let mut primitive_expansions: Vec<(&S, crate::LinearCombination<usize, C>)> = Vec::new();
    let mut tasks = vec![StateTask::Visit(&state.expression)];
    let mut values: Vec<Expansion<C>> = Vec::new();
    while let Some(task) = tasks.pop() {
        match task {
            StateTask::Visit(node) => match node.node() {
                StateNode::Local(local, _) => {
                    let terms = local_expansion(LinearTerms::new(local, state_view), |p| {
                        if let Some((_, terms)) = primitive_expansions.iter().find(|(x, _)| *x == p)
                        {
                            return Ok(terms.clone());
                        }
                        let terms = indexed_terms(p, basis, dimension.expect("local factor"))?;
                        primitive_expansions.push((p, terms.clone()));
                        Ok(terms)
                    })?;
                    values.push(terms.into_iter().map(|(i, v)| (vec![i], v)).collect());
                }
                StateNode::Unit => values.push(BTreeMap::from([(Vec::new(), C::one())])),
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
                for d in value.values_mut() {
                    *d = c.clone() * core::mem::replace(d, C::zero());
                }
                value.retain(|_, d| !d.is_zero());
            }
            StateTask::Sum(count) => {
                let mut sum = BTreeMap::new();
                for term in values.drain(values.len() - count..) {
                    for (digits, c) in term {
                        accumulate(&mut sum, digits, c);
                    }
                }
                values.push(sum);
            }
            StateTask::Tensor(count) => {
                let start = values.len() - count;
                let product = tensor(&values[start..]);
                values.truncate(start);
                values.push(product);
            }
        }
    }
    let mut result = values.pop().expect("one expansion");
    result.retain(|_, c| !c.is_zero());
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
        let terms = state_expansion(self, basis, dimension)?;
        let expressions = reconstruct(
            terms,
            basis,
            dimension,
            LocalState::primitive,
            |local, position| {
                StateExpr::new(StateNode::Local(local.clone(), factors[position].clone()))
            },
            |factors, c| StateExpr::tensor(factors).scaled(c),
        )?;
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
        let terms = state_expansion(self, basis, dimension)?;
        Ok(Coordinates {
            basis,
            space: self.space.clone(),
            factor_count: count,
            dim_local: dimension,
            entries: terms,
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
        self.validate(ambient)?;
        let dimension = local_size::<O>(ambient.factor_count())?;
        let terms = operator_expansion(self, basis, ambient, dimension)?;
        let factors: Vec<_> = ambient.leaves().collect();
        let expressions = reconstruct(
            terms,
            basis,
            dimension,
            LocalOperator::primitive,
            |local, position| OpExpr::new(OpNode::Local(local.clone(), factors[position].clone())),
            |factors, c| OpExpr::product(factors).scaled(c),
        )?;
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
        self.validate(ambient)?;
        let dimension = local_size::<O>(ambient.factor_count())?;
        let terms = operator_expansion(self, basis, ambient, dimension)?;
        Ok(Coordinates {
            basis,
            space: ambient.clone(),
            factor_count: ambient.factor_count(),
            dim_local: dimension,
            entries: terms,
        })
    }
}
