//! The two supported core expression contexts; the contraction cannot be customized.

use crate::{FamilyExpression, HsOverlaps};

use ndarray_linalg::{Lapack, Scalar};
use num_complex::Complex;
use num_traits::Float;
use syntaqs_core::{
    algebra::Adjoint, algebra::HomogeneousParity, algebra::Identity, algebra::ParityOperator,
    algebra::Product, algebra::Trace, expression::composite::Operator,
    expression::error::ExpressionError, expression::local::LocalOperator, space::HilbertSpace,
};

impl<O, R> FamilyExpression for LocalOperator<O, Complex<R>>
where
    R: Float + Scalar<Real = R> + Lapack,
    O: PartialEq
        + Identity
        + Product<Complex<R>, Term = O>
        + Adjoint<Complex<R>, Term = O>
        + Trace<Complex<R>>,
{
    type Real = R;
    type Context = ();

    fn inferred_context(_: &[Self]) -> Result<(), ExpressionError> {
        Ok(())
    }
    fn in_context(&self, _: &()) -> Result<Self, ExpressionError> {
        Ok(self.clone())
    }
    fn sum_in(operators: Vec<Self>, _: &()) -> Result<Self, ExpressionError> {
        Ok(Self::sum(operators))
    }
    fn collected_in(&self, _: &()) -> Result<Self, ExpressionError> {
        Ok(self.collect_terms_compact())
    }
    fn scaled_real(&self, coefficient: R) -> Self {
        self.scaled(Complex::new(coefficient, R::zero()))
    }
    fn adjoint_expression(&self) -> Self {
        self.adjoint()
    }
    fn lie_bracket_expression(&self, rhs: &Self) -> Result<Self, ExpressionError> {
        Ok(self
            .commutator(rhs)
            .scaled(Complex::new(R::zero(), -R::one())))
    }
    fn hs_in(&self, rhs: &Self, _: &()) -> Result<Complex<R>, ExpressionError> {
        Ok(self.collect_terms().hs_inner_product(&rhs.collect_terms()))
    }
    fn prepare_hs_against<'a>(
        basis: &'a [Self],
        _: &'a (),
    ) -> Result<impl FnMut(&Self) -> HsOverlaps<R> + 'a, ExpressionError> {
        let basis: Vec<_> = basis.iter().map(Self::collect_terms).collect();
        Ok(move |target: &Self| {
            let target = target.collect_terms();
            Ok(basis
                .iter()
                .map(|op| op.hs_inner_product(&target))
                .collect())
        })
    }
}

impl<I, O, R> FamilyExpression for Operator<I, O, Complex<R>>
where
    I: Ord,
    R: Float + Scalar<Real = R> + Lapack,
    O: PartialEq
        + Identity
        + HomogeneousParity
        + ParityOperator
        + Product<Complex<R>, Term = O>
        + Adjoint<Complex<R>, Term = O>
        + Trace<Complex<R>>,
{
    type Real = R;
    type Context = HilbertSpace<I>;

    fn inferred_context(operators: &[Self]) -> Result<Self::Context, ExpressionError> {
        operators
            .iter()
            .find_map(Self::space)
            .cloned()
            .ok_or(ExpressionError::UnboundOperator)
    }
    fn in_context(&self, context: &Self::Context) -> Result<Self, ExpressionError> {
        // A family never silently overrides a conflicting binding, even on zero.
        if self
            .space()
            .is_some_and(|space| !space.is_compatible_with(context))
        {
            return Err(ExpressionError::IncompatibleSpaces);
        }
        self.bind(context)
    }
    fn sum_in(operators: Vec<Self>, context: &Self::Context) -> Result<Self, ExpressionError> {
        Self::sum(operators)?.bind(context)
    }
    fn collected_in(&self, context: &Self::Context) -> Result<Self, ExpressionError> {
        self.in_context(context)?.collect_terms_in(context)
    }
    fn scaled_real(&self, coefficient: R) -> Self {
        self.scaled(Complex::new(coefficient, R::zero()))
    }
    fn adjoint_expression(&self) -> Self {
        self.adjoint()
    }
    fn lie_bracket_expression(&self, rhs: &Self) -> Result<Self, ExpressionError> {
        Ok(self
            .commutator(rhs)?
            .scaled(Complex::new(R::zero(), -R::one())))
    }
    fn closure_bracket_in(
        &self,
        rhs: &Self,
        context: &Self::Context,
    ) -> Result<Self, ExpressionError> {
        if let Some(commutator) = self.try_collect_commutator_in(rhs, context)? {
            return Ok(commutator.scaled(Complex::new(R::zero(), -R::one())));
        }
        self.in_context(context)?
            .lie_bracket_expression(&rhs.in_context(context)?)
    }
    fn hs_in(&self, rhs: &Self, context: &Self::Context) -> Result<Complex<R>, ExpressionError> {
        self.hs_inner_product_in(rhs, context)
    }
    fn prepare_hs_against<'a>(
        basis: &'a [Self],
        context: &'a Self::Context,
    ) -> Result<impl FnMut(&Self) -> HsOverlaps<R> + 'a, ExpressionError> {
        Self::prepare_hs_against_in(basis, context)
    }
}
