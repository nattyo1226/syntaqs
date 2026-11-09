//! Explicit exact collection, separate from ordinary expression construction/evaluation.

use alloc::vec::Vec;
use core::ops::{Mul, Neg};
use num_traits::{One, Zero};

use crate::{
    Adjoint, Conjugate, ExpressionError, HilbertSpace, HomogeneousParity, Identity, LocalOperator,
    Operator, Product,
    expr::{
        local::OperatorNode,
        traversal::{IntrinsicOpTerms, LinearTerms, LinearView},
    },
};

fn handle_view<O, C>(
    op: &LocalOperator<O, C>,
) -> LinearView<'_, LocalOperator<O, C>, LocalOperator<O, C>, C> {
    match op.node() {
        OperatorNode::Primitive(_) => LinearView::Leaf(op),
        OperatorNode::Scale(scale) => LinearView::Scale(&scale.coefficient, &scale.operand),
        OperatorNode::Sum(sum) => LinearView::Sum(sum),
        _ => unreachable!("evaluated local operator"),
    }
}

fn equal_primitive<O: PartialEq, C>(
    left: &LocalOperator<O, C>,
    right: &LocalOperator<O, C>,
) -> bool {
    match (left.node(), right.node()) {
        (OperatorNode::Primitive(a), OperatorNode::Primitive(b)) => a == b,
        _ => unreachable!("collected primitive handles"),
    }
}

fn local_terms<O, C>(op: &LocalOperator<O, C>) -> Vec<(C, LocalOperator<O, C>)>
where
    C: Clone + Zero + One + Mul<Output = C> + Conjugate<Output = C>,
    O: PartialEq + Product<C, Term = O> + Adjoint<C, Term = O>,
{
    let evaluated = op.evaluate();
    let mut terms: Vec<(C, LocalOperator<O, C>)> = Vec::new();
    for (coefficient, primitive) in LinearTerms::new(&evaluated, handle_view) {
        let coefficient = coefficient.unwrap_or_else(C::one);
        if let Some((previous, _)) = terms
            .iter_mut()
            .find(|(_, op)| equal_primitive(op, primitive))
        {
            *previous = previous.clone() + coefficient;
        } else {
            terms.push((coefficient, primitive.clone()));
        }
    }
    terms.retain(|(coefficient, _)| !coefficient.is_zero());
    terms
}

type LocalTerms<O, C> = Vec<(C, LocalOperator<O, C>)>;
type TensorTerms<O, C> = (C, Vec<LocalTerms<O, C>>);

fn equal_local<O: PartialEq, C: PartialEq>(
    left: &LocalTerms<O, C>,
    right: &LocalTerms<O, C>,
) -> bool {
    left.len() == right.len()
        && left.iter().all(|(c, op)| {
            right
                .iter()
                .any(|(d, other)| c == d && equal_primitive(op, other))
        })
}

// Merge terms with equal factors except possibly one. Keep local polynomials
// factored: expanding every factor would turn product_i(X_i+Z_i) into 2^n terms.
fn merge_tensor<O, C>(terms: &mut Vec<TensorTerms<O, C>>, mut term: TensorTerms<O, C>)
where
    C: Clone + PartialEq + Zero + One + Mul<Output = C> + Conjugate<Output = C>,
    O: PartialEq + Product<C, Term = O> + Adjoint<C, Term = O>,
{
    loop {
        if term.0.is_zero() || term.1.iter().any(Vec::is_empty) {
            return;
        }
        let matching = terms.iter().enumerate().find_map(|(i, (_, factors))| {
            let mut differing = factors
                .iter()
                .zip(&term.1)
                .enumerate()
                .filter(|(_, (a, b))| !equal_local(a, b))
                .map(|(j, _)| j);
            let first = differing.next();
            differing.next().is_none().then_some((i, first))
        });
        let Some((index, factor)) = matching else {
            terms.push(term);
            return;
        };
        let (previous, mut factors) = terms.remove(index);
        match factor {
            None => term.0 = previous + term.0,
            Some(factor) => {
                let a = factors.remove(factor);
                let b = core::mem::take(&mut term.1[factor]);
                let sum = LocalOperator::sum(
                    a.into_iter()
                        .map(|(c, op)| op.scaled(previous.clone() * c))
                        .chain(b.into_iter().map(|(c, op)| op.scaled(term.0.clone() * c))),
                );
                term.1[factor] = local_terms(&sum);
                term.0 = C::one();
            }
        }
    }
}

impl<O, C> LocalOperator<O, C> {
    /// Evaluates products/adjoints and combines equal natural primitives.
    ///
    /// Equality is primitive equality, not expansion in a linearly independent
    /// basis. Only exactly zero coefficients are removed; there is no tolerance.
    /// The input is unchanged and primitive handles are shared without cloning O.
    pub fn collect_terms(&self) -> Self
    where
        C: Clone + Zero + One + Mul<Output = C> + Conjugate<Output = C>,
        O: PartialEq + Product<C, Term = O> + Adjoint<C, Term = O>,
    {
        Self::sum(local_terms(self).into_iter().map(|(c, op)| op.scaled(c)))
    }
}

impl<I: Eq, O, C> Operator<I, O, C> {
    /// Collects local primitives and merges compatible intrinsic tensor terms.
    ///
    /// Uses the existing ordinary/graded traversal, including Koszul signs, and
    /// retains the selected space even for zero. Products/adjoints are evaluated;
    /// spectators use identity. Local sums remain factored; terms differing in
    /// at most one factor are merged without Cartesian expansion. No state
    /// basis, dimension, or primitive clone is needed. It removes exact zeros only and does not
    /// identify linear relations between distinct natural primitives.
    /// As in core's other explicit-space queries, `ambient` overrides old bindings.
    pub fn collect_terms_in(&self, ambient: &HilbertSpace<I>) -> Result<Self, ExpressionError>
    where
        C: Clone
            + PartialEq
            + Zero
            + One
            + Mul<Output = C>
            + Conjugate<Output = C>
            + Neg<Output = C>,
        O: PartialEq + Identity + HomogeneousParity + Product<C, Term = O> + Adjoint<C, Term = O>,
    {
        let ambient = self.select_space(Some(ambient), None)?;
        let factors: Vec<_> = ambient.leaves().collect();
        let mut terms = Vec::new();
        for (coefficient, locals) in IntrinsicOpTerms::new(&self.expression, ambient) {
            let mut product = Vec::with_capacity(factors.len());
            for factor in &factors {
                let identity;
                let local = match locals
                    .iter()
                    .find(|(f, _)| f.factor_index() == factor.factor_index())
                {
                    Some((_, op)) => op,
                    None => {
                        identity = LocalOperator::identity();
                        &identity
                    }
                };
                product.push(local_terms(local));
            }
            merge_tensor(&mut terms, (coefficient.unwrap_or_else(C::one), product));
        }
        let operators = terms
            .into_iter()
            .filter(|(c, _)| !c.is_zero())
            .map(|(c, operands)| {
                Self::try_product(operands.iter().zip(&factors).map(|(terms, factor)| {
                    LocalOperator::sum(terms.iter().map(|(c, op)| op.scaled(c.clone()))).at(factor)
                }))
                .map(|op| op.scaled(c))
            });
        Self::try_sum(operators)?.bind(ambient)
    }
}
