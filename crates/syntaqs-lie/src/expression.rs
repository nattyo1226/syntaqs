//! The two supported core expression contexts; the contraction cannot be customized.

use ndarray_linalg::{Lapack, Scalar};
use num_complex::Complex;
use num_traits::Float;
use syntaqs_core::{
    Adjoint, ExpressionError, HilbertSpace, HomogeneousParity, Identity, LocalOperator, Operator,
    ParityOperator, Product, Trace,
};

mod private {
    pub trait Sealed {}
    impl<O, R> Sealed for syntaqs_core::LocalOperator<O, num_complex::Complex<R>> {}
    impl<I, O, R> Sealed for syntaqs_core::Operator<I, O, num_complex::Complex<R>> {}
}

/// A supported local or placed core operator with complex f32/f64 coefficients.
///
/// This sealed trait fixes the normalized Hilbert–Schmidt contraction. Users
/// supply primitive algebra rules through core traits, not a custom metric.
/// Primitives also need `PartialEq` for exact term collection before contraction.
/// Local context is `()`; placed context is `HilbertSpace<I>`.
pub trait FamilyExpression: private::Sealed + Clone {
    /// Real precision of the expression's complex coefficients.
    type Real: Float + Scalar<Real = Self::Real> + Lapack;
    /// Common context retained by a family.
    type Context: Clone;

    #[doc(hidden)]
    fn inferred_context(operators: &[Self]) -> Result<Self::Context, ExpressionError>;
    #[doc(hidden)]
    fn in_context(&self, context: &Self::Context) -> Result<Self, ExpressionError>;
    #[doc(hidden)]
    fn sum_in(operators: Vec<Self>, context: &Self::Context) -> Result<Self, ExpressionError>;
    #[doc(hidden)]
    fn scaled_real(&self, coefficient: Self::Real) -> Self;
    #[doc(hidden)]
    fn adjoint_expression(&self) -> Self;
    #[doc(hidden)]
    fn hs_in(
        &self,
        rhs: &Self,
        context: &Self::Context,
    ) -> Result<Complex<Self::Real>, ExpressionError>;
}

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
    fn scaled_real(&self, coefficient: R) -> Self {
        self.scaled(Complex::new(coefficient, R::zero()))
    }
    fn adjoint_expression(&self) -> Self {
        self.adjoint()
    }
    fn hs_in(&self, rhs: &Self, _: &()) -> Result<Complex<R>, ExpressionError> {
        Ok(self.collect_terms().hs_inner_product(&rhs.collect_terms()))
    }
}

impl<I, O, R> FamilyExpression for Operator<I, O, Complex<R>>
where
    I: Eq,
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
    fn scaled_real(&self, coefficient: R) -> Self {
        self.scaled(Complex::new(coefficient, R::zero()))
    }
    fn adjoint_expression(&self) -> Self {
        self.adjoint()
    }
    fn hs_in(&self, rhs: &Self, context: &Self::Context) -> Result<Complex<R>, ExpressionError> {
        self.collect_terms_in(context)?
            .hs_inner_product_in(&rhs.collect_terms_in(context)?, context)
    }
}
