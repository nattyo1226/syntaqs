//! Ordinary traces and Hilbert–Schmidt contractions.

use crate::{
    Adjoint, Conjugate, Divide, ExpressionError, HilbertSpace, Identity, LocalOperator, Operator,
    Product, Trace,
    coefficient::{add, multiply, zero},
    expression::traversal::{LinearTerms, OrdinaryOpTerms, operator_view},
    space::FactorPositions,
};
use core::ops::Mul;
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

fn one<C: One>() -> C {
    C::one()
}

impl<I: Ord, O, C> Operator<I, O, C> {
    /// Computes the ordinary full trace using the existing binding.
    /// An exact-zero local trace stops that tensor term, not the enclosing sum.
    pub fn trace(&self) -> Result<C, ExpressionError>
    where
        C: Clone + Zero + One + Conjugate<Output = C>,
        O: Identity + crate::HomogeneousParity + crate::ParityOperator,
        O: Product<C, Term = O> + Adjoint<C, Term = O> + Trace<C>,
    {
        self.trace_in(self.bound_space()?)
    }

    /// Computes full trace on an explicit space, overriding an old binding.
    pub fn trace_in(&self, ambient: &HilbertSpace<I>) -> Result<C, ExpressionError>
    where
        C: Clone + Zero + One + Conjugate<Output = C>,
        O: Identity + crate::HomogeneousParity + crate::ParityOperator,
        O: Product<C, Term = O> + Adjoint<C, Term = O> + Trace<C>,
    {
        self.validate(&FactorPositions::new(ambient))?;
        self.trace_on(ambient)
    }

    fn trace_on(&self, ambient: &HilbertSpace<I>) -> Result<C, ExpressionError>
    where
        C: Clone + Zero + One + Conjugate<Output = C>,
        O: Identity + crate::HomogeneousParity + crate::ParityOperator,
        O: Product<C, Term = O> + Adjoint<C, Term = O> + Trace<C>,
    {
        self.trace_on_with(ambient, |local| local.trace())
    }

    // Both contractions use ordinary local traces after grading has been
    // resolved. Only the HS caller supplies division; ordinary trace keeps its
    // original coefficient bounds.
    fn trace_on_with(
        &self,
        ambient: &HilbertSpace<I>,
        local_trace: impl Fn(&LocalOperator<O, C>) -> C,
    ) -> Result<C, ExpressionError>
    where
        C: Clone + Zero + One + Conjugate<Output = C>,
        O: Identity + crate::HomogeneousParity + crate::ParityOperator,
        O: Product<C, Term = O> + Adjoint<C, Term = O> + Trace<C>,
    {
        let expression = self.expression.group_local();
        let mut result = zero();
        for (coefficient, factors) in OrdinaryOpTerms::new(&expression, ambient) {
            if coefficient.as_ref().is_some_and(Zero::is_zero) {
                continue;
            }
            let mut value = one();
            for index in ambient.indices() {
                let local_trace_value = match factors
                    .iter()
                    .find(|(factor, _)| factor.factor_index() == Some(index))
                {
                    Some((_, local)) => local_trace(local),
                    None => local_trace(&LocalOperator::<O, C>::identity()),
                };
                if local_trace_value.is_zero() {
                    value = local_trace_value;
                    break;
                }
                value = multiply(value, local_trace_value);
                if value.is_zero() {
                    break;
                }
            }
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
        Ok(result)
    }

    fn normalized_trace_on(&self, ambient: &HilbertSpace<I>) -> Result<C, ExpressionError>
    where
        C: Clone + Zero + One + Conjugate<Output = C> + Divide<Output = C>,
        O: Identity + crate::HomogeneousParity + crate::ParityOperator,
        O: Product<C, Term = O> + Adjoint<C, Term = O> + Trace<C>,
    {
        let dimension = O::identity().trace();
        self.trace_on_with(ambient, |local| {
            local.trace_with(|value| value.divide(dimension.clone()))
        })
    }

    /// Computes the normalized Hilbert–Schmidt inner product `Tr(self† rhs)/D`.
    ///
    /// Each ordinary local trace is divided by `Tr(I)` before contraction, so
    /// no integer total dimension or large unnormalized trace is constructed.
    /// Local dimensions must be nonzero and support mathematical division.
    /// Exact-zero local contractions skip later factors and redundant scaling;
    /// zero primitive traces need no normalization division.
    pub fn hs_inner_product(&self, rhs: &Self) -> Result<C, ExpressionError>
    where
        C: Clone + Zero + One + Conjugate<Output = C> + Divide<Output = C>,
        O: Identity + crate::HomogeneousParity + crate::ParityOperator,
        O: Product<C, Term = O> + Adjoint<C, Term = O> + Trace<C>,
    {
        let product = (self.adjoint() * rhs)?;
        let ambient = product.bound_space()?;
        product.validate(&FactorPositions::new(ambient))?;
        product.normalized_trace_on(ambient)
    }

    /// Computes `Tr(self† rhs)/D` on an explicit ambient space.
    ///
    /// Uses local trace normalization, with the same coefficient requirements
    /// as [`Self::hs_inner_product`]. The explicit space overrides old bindings.
    pub fn hs_inner_product_in(
        &self,
        rhs: &Self,
        ambient: &HilbertSpace<I>,
    ) -> Result<C, ExpressionError>
    where
        C: Clone + Zero + One + Conjugate<Output = C> + Divide<Output = C>,
        O: Identity + crate::HomogeneousParity + crate::ParityOperator,
        O: Product<C, Term = O> + Adjoint<C, Term = O> + Trace<C>,
    {
        (self.bind(ambient)?.adjoint() * &rhs.bind(ambient)?)?.normalized_trace_on(ambient)
    }
}
