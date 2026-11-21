//! Explicit exact collection, separate from ordinary expression construction/evaluation.

use alloc::vec::Vec;
use core::ops::{Mul, Neg};
use num_complex::Complex;
use num_traits::{Num, NumCast, One, Zero};

use crate::{
    Adjoint, Conjugate, ExpressionError, HilbertSpace, HomogeneousParity, Identity, LocalOperator,
    Operator, Product,
    expression::{
        local::OperatorNode,
        traversal::{LinearTerms, operator_handle_view},
    },
};

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
    for (coefficient, primitive) in LinearTerms::new(&evaluated, operator_handle_view) {
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

impl<I: Ord, O, C> Operator<I, O, C> {
    /// Evaluates products and adjoints and collects equal natural operator terms.
    ///
    /// Disjoint factor blocks remain factored. Within each block, coefficients
    /// of equal ordered primitive products are added and exact zeros are removed.
    /// Coefficient equality compares the internal weights of common blocks
    /// before factoring them out of a sum.
    /// Graded products use homogeneous primitive parities and the ambient order.
    /// The explicit space overrides any old binding and is validated before
    /// evaluation. The result is bound to that space, including for zero.
    pub fn collect_terms_in(&self, ambient: &HilbertSpace<I>) -> Result<Self, ExpressionError>
    where
        C: Clone + PartialEq + Zero + One + Conjugate<Output = C> + Neg<Output = C>,
        O: PartialEq + Identity + HomogeneousParity + Product<C, Term = O> + Adjoint<C, Term = O>,
    {
        self.validate(ambient)?;
        let evaluated = super::operator::EvaluatedOperator::new(&self.expression, ambient);
        let factors: Vec<_> = ambient.leaves().collect();
        Ok(Self {
            expression: evaluated.expression(&factors),
            ambient: Some(ambient.clone()),
        })
    }
}

impl<I, O, R> Operator<I, O, Complex<R>>
where
    I: Ord,
    R: Copy + Num + NumCast + Neg<Output = R>,
    O: PartialEq
        + Identity
        + HomogeneousParity
        + Product<Complex<R>, Term = O>
        + Adjoint<Complex<R>, Term = O>,
{
    /// Collects the ordinary commutator when both operands evaluate to single
    /// overlapping sparse blocks. Uses the validated explicit ambient space.
    /// Returns `None` when the caller should evaluate its symbolic commutator.
    /// Disjoint commuting term pairs cancel before coefficient accumulation.
    #[doc(hidden)]
    pub fn try_collect_commutator_in(
        &self,
        rhs: &Self,
        ambient: &HilbertSpace<I>,
    ) -> Result<Option<Self>, ExpressionError> {
        self.validate(ambient)?;
        rhs.validate(ambient)?;
        Ok(super::operator::EvaluatedOperator::commutator(
            &self.expression,
            &rhs.expression,
            ambient,
        )
        .map(|evaluated| {
            let factors: Vec<_> = ambient.leaves().collect();
            Self {
                expression: evaluated.expression(&factors),
                ambient: Some(ambient.clone()),
            }
        }))
    }
}
