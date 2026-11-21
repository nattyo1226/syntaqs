//! Ordered candidates, explicit diagnostics, and real projection coordinates.

use ndarray::{Array1, Array2, ArrayView1, OwnedRepr};
use ndarray_linalg::{Factorize, LUFactorized, Solve};
use num_complex::Complex;
use num_traits::{Float, One, Zero};

use crate::{
    FamilyExpression, LieError, NumericalError, NumericalOptions, ToleranceCheck,
    numerical::kernels::{finite, real_coordinate, squared_norm, symmetric_eigen},
};

/// A nonempty ordered family of shared core operator expressions.
///
/// Construction checks context only. Hermiticity and independence are explicit
/// numerical checks; a family does not certify closure or carry a valid flag.
#[derive(Debug)]
pub struct OperatorFamily<E: FamilyExpression> {
    operators: Vec<E>,
    context: E::Context,
}

impl<E: FamilyExpression> Clone for OperatorFamily<E> {
    fn clone(&self) -> Self {
        Self {
            operators: self.operators.clone(),
            context: self.context.clone(),
        }
    }
}

impl<E: FamilyExpression> OperatorFamily<E> {
    /// Keeps input order and infers the context without evaluating operators.
    ///
    /// Local expressions use `()`. For placed expressions at least one input
    /// must be bound; its space is used, and every existing binding must agree.
    /// Use [`Self::new_in`] for entirely unbound placed expressions.
    pub fn new(operators: impl IntoIterator<Item = E>) -> Result<Self, LieError<E::Real>> {
        let operators: Vec<_> = operators.into_iter().collect();
        if operators.is_empty() {
            return Err(LieError::EmptyFamily);
        }
        let context = E::inferred_context(&operators)?;
        Self::new_in(operators, &context)
    }

    /// Checks every input against an explicit context without evaluating it.
    ///
    /// Conflicting bindings and missing factor indices fail even on expressions
    /// multiplied by zero. Rebind deliberately in core before passing an input
    /// if its old ambient space is to be replaced. Original handles are retained.
    pub fn new_in(
        operators: impl IntoIterator<Item = E>,
        context: &E::Context,
    ) -> Result<Self, LieError<E::Real>> {
        let operators: Vec<_> = operators.into_iter().collect();
        if operators.is_empty() {
            return Err(LieError::EmptyFamily);
        }
        for operator in &operators {
            operator.in_context(context)?;
        }
        Ok(Self {
            operators,
            context: context.clone(),
        })
    }

    /// Ordered input expressions, sharing their original nodes.
    pub fn operators(&self) -> &[E] {
        &self.operators
    }

    /// Common local or ambient context.
    pub fn context(&self) -> &E::Context {
        &self.context
    }

    /// Number of input operators; always positive.
    pub fn len(&self) -> usize {
        self.operators.len()
    }

    /// Always false: an empty family cannot be constructed.
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Iterates over the inputs in their retained order.
    pub fn iter(&self) -> std::slice::Iter<'_, E> {
        self.operators.iter()
    }

    /// Reconstructs a real linear combination in this family's context.
    ///
    /// This operation needs neither independence nor Hermiticity. Coefficients
    /// must have exactly the family length and be finite; none are pruned.
    pub fn reconstruct(
        &self,
        coefficients: ArrayView1<'_, E::Real>,
    ) -> Result<E, LieError<E::Real>> {
        if coefficients.len() != self.len() {
            return Err(NumericalError::Shape("one coefficient per family element").into());
        }
        finite(coefficients.iter().copied())?;
        Ok(E::sum_in(
            self.iter()
                .zip(coefficients)
                .map(|(operator, &value)| operator.scaled_real(value))
                .collect(),
            &self.context,
        )?)
    }
}

/// Gram matrices and projection results.
pub mod projection;
/// Family transformations and orthonormality reports.
pub mod transform;
/// Family validation reports.
pub mod validation;

pub(crate) use projection::{FamilyCoordinates, GramFactorization, Projection};
pub(crate) use validation::{FamilyValidation, IndependenceReport};
