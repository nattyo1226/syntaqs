//! Ordinary traces and Hilbert–Schmidt contractions.

use super::operator::EvaluatedOperator;
use crate::{
    Adjoint, Conjugate, Divide, ExpressionError, HilbertSpace, Identity, LocalOperator, Operator,
    Product, Trace,
    coefficient::{add, multiply, zero},
    expression::traversal::{LinearTerms, operator_view},
    space::FactorPositions,
};
use core::ops::{Mul, Neg};
use num_traits::{One, Zero};

impl<O, C> LocalOperator<O, C> {
    /// Computes the ordinary, unnormalized local trace without a chosen basis.
    pub fn trace(&self) -> C
    where
        C: Clone + Zero + Mul<Output = C> + Conjugate<Output = C>,
        O: Identity,
        O: Product<C, Term = O> + Adjoint<C, Term = O> + Trace<C>,
    {
        self.trace_with(|value| value)
    }

    // Normalize primitive traces before multiplying stored coefficients or
    // adding terms, so a representable normalized trace need not pass through
    // an overflowing unnormalized intermediate.
    fn trace_with(&self, primitive_trace: impl Fn(C) -> C) -> C
    where
        C: Clone + Zero + Mul<Output = C> + Conjugate<Output = C>,
        O: Product<C, Term = O> + Adjoint<C, Term = O> + Trace<C>,
    {
        let evaluated = self.evaluate();
        let mut result = zero();
        for (coefficient, primitive) in LinearTerms::new(&evaluated, operator_view) {
            if coefficient.as_ref().is_some_and(Zero::is_zero) {
                continue;
            }
            let value = primitive.trace();
            if value.is_zero() {
                continue;
            }
            let mut value = primitive_trace(value);
            if value.is_zero() {
                continue;
            }
            if let Some(c) = coefficient {
                value = multiply(c, value);
                if value.is_zero() {
                    continue;
                }
            }
            result = add(result, value);
        }
        result
    }

    /// Computes the normalized Hilbert–Schmidt inner product `Tr(self† rhs)/d`.
    ///
    /// The local dimension is obtained as `Tr(I)` in the coefficient type. It
    /// must be nonzero and division by it must represent mathematical division.
    pub fn hs_inner_product(&self, rhs: &Self) -> C
    where
        C: Clone + Zero + Mul<Output = C> + Conjugate<Output = C> + Divide<Output = C>,
        O: Identity,
        O: Product<C, Term = O> + Adjoint<C, Term = O> + Trace<C>,
    {
        let dimension = O::identity().trace();
        (self.adjoint() * rhs).trace_with(|value| value.divide(dimension.clone()))
    }
}

impl<I: Ord, O, C> Operator<I, O, C> {
    /// Computes the ordinary full trace using the existing binding.
    pub fn trace(&self) -> Result<C, ExpressionError>
    where
        C: Clone + PartialEq + Zero + One + Conjugate<Output = C> + Neg<Output = C>,
        O: PartialEq
            + Identity
            + crate::HomogeneousParity
            + Product<C, Term = O>
            + Adjoint<C, Term = O>
            + Trace<C>,
    {
        self.trace_in(self.bound_space()?)
    }

    /// Computes full trace on an explicit space, overriding an old binding.
    /// On a graded space, any odd local factor annihilates its tensor term.
    pub fn trace_in(&self, ambient: &HilbertSpace<I>) -> Result<C, ExpressionError>
    where
        C: Clone + PartialEq + Zero + One + Conjugate<Output = C> + Neg<Output = C>,
        O: PartialEq
            + Identity
            + crate::HomogeneousParity
            + Product<C, Term = O>
            + Adjoint<C, Term = O>
            + Trace<C>,
    {
        let positions = FactorPositions::new(ambient);
        self.validate(&positions)?;
        let operator = EvaluatedOperator::new(&self.expression, &positions);
        Ok(operator.full_trace(positions.leaves().len()))
    }

    /// Computes the normalized Hilbert–Schmidt inner product `Tr(self† rhs)/D`.
    ///
    /// Contracts independent blocks and divides each primitive local trace
    /// by `Tr(I)` before scaling and summation. The local dimension must be
    /// nonzero and support mathematical division.
    pub fn hs_inner_product(&self, rhs: &Self) -> Result<C, ExpressionError>
    where
        C: Clone
            + PartialEq
            + Zero
            + One
            + Conjugate<Output = C>
            + Neg<Output = C>
            + Divide<Output = C>,
        O: PartialEq
            + Identity
            + crate::HomogeneousParity
            + Product<C, Term = O>
            + Adjoint<C, Term = O>
            + Trace<C>,
    {
        let ambient = self.bound_space()?;
        if !ambient.is_compatible_with(rhs.bound_space()?) {
            return Err(ExpressionError::IncompatibleSpaces);
        }
        self.hs_inner_product_in(rhs, ambient)
    }

    /// Computes `Tr(self† rhs)/D` on an explicit ambient space.
    /// The explicit space overrides old bindings.
    pub fn hs_inner_product_in(
        &self,
        rhs: &Self,
        ambient: &HilbertSpace<I>,
    ) -> Result<C, ExpressionError>
    where
        C: Clone
            + PartialEq
            + Zero
            + One
            + Conjugate<Output = C>
            + Neg<Output = C>
            + Divide<Output = C>,
        O: PartialEq
            + Identity
            + crate::HomogeneousParity
            + Product<C, Term = O>
            + Adjoint<C, Term = O>
            + Trace<C>,
    {
        let positions = FactorPositions::new(ambient);
        self.validate(&positions)?;
        rhs.validate(&positions)?;
        Ok(EvaluatedOperator::hs_inner_product(
            &self.expression,
            &rhs.expression,
            &positions,
        ))
    }
}
