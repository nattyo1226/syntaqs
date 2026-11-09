//! Ordered candidates, explicit diagnostics, and real projection coordinates.

use ndarray::{Array1, Array2, ArrayView1, OwnedRepr};
use ndarray_linalg::{Factorize, LUFactorized, Solve};
use num_complex::Complex;
use num_traits::{Float, One, Zero};

use crate::{
    FamilyExpression, LieError, NumericalError, NumericalOptions, ToleranceCheck,
    numerical::{finite, real_coordinate, squared_norm, symmetric_eigen},
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
        Self::new_in(&context, operators)
    }

    /// Checks every input against an explicit context without evaluating it.
    ///
    /// Conflicting bindings and missing factor indices fail even on expressions
    /// multiplied by zero. Rebind deliberately in core before passing an input
    /// if its old ambient space is to be replaced. Original handles are retained.
    pub fn new_in(
        context: &E::Context,
        operators: impl IntoIterator<Item = E>,
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

/// Effective real-linear independence diagnosis from Gram eigenvalues.
#[derive(Debug, Clone)]
pub struct IndependenceReport<R> {
    /// Eigenvalues of `Re(G)`, in ascending order.
    pub eigenvalues: Array1<R>,
    /// Number strictly above the effective cutoff.
    pub rank: usize,
    /// Number of input operators.
    pub dimension: usize,
    /// Effective relative Gram-eigenvalue cutoff.
    pub relative_cutoff: R,
    /// Absolute cutoff, `relative_cutoff * max(eigenvalues)`.
    pub threshold: R,
}

impl<R> IndependenceReport<R> {
    /// Whether every family direction is independent under the Gram cutoff.
    pub fn is_independent(&self) -> bool {
        self.rank == self.dimension
    }
}

/// Per-element Hermiticity and real-linear independence, without a closure claim.
pub struct FamilyValidation<'a, E: FamilyExpression> {
    family: &'a OperatorFamily<E>,
    hermiticity: Vec<ToleranceCheck<E::Real>>,
    independence: IndependenceReport<E::Real>,
    options: NumericalOptions<E::Real>,
}

impl<'a, E: FamilyExpression> FamilyValidation<'a, E> {
    /// The exact originating family.
    pub fn family(&self) -> &'a OperatorFamily<E> {
        self.family
    }
    /// Checks of `||B_i-B_i†||_HS` against scale `||B_i||_HS`, in input order.
    pub fn hermiticity(&self) -> &[ToleranceCheck<E::Real>] {
        &self.hermiticity
    }
    /// Real-linear independence, also meaningful for non-Hermitian candidates.
    pub fn independence(&self) -> &IndependenceReport<E::Real> {
        &self.independence
    }
    /// Settings used by this report.
    pub fn options(&self) -> &NumericalOptions<E::Real> {
        &self.options
    }
    /// Whether both Hermiticity and independence checks pass; closure is untested.
    pub fn passed(&self) -> bool {
        self.hermiticity.iter().all(ToleranceCheck::passed) && self.independence.is_independent()
    }
}

/// Full complex normalized-HS Gram matrix, retaining the originating family.
pub struct GramMatrix<'a, E: FamilyExpression> {
    family: &'a OperatorFamily<E>,
    values: Array2<Complex<E::Real>>,
    norms: Array1<E::Real>,
    options: NumericalOptions<E::Real>,
}

impl<'a, E: FamilyExpression> GramMatrix<'a, E> {
    /// The exact originating family.
    pub fn family(&self) -> &'a OperatorFamily<E> {
        self.family
    }
    /// `G[i,j] = Tr(B_i† B_j)/D`, including complex entries of invalid candidates.
    pub fn values(&self) -> &Array2<Complex<E::Real>> {
        &self.values
    }
    /// Numerical settings used when evaluating this matrix and its norms.
    pub fn options(&self) -> &NumericalOptions<E::Real> {
        &self.options
    }
}

/// An explicitly reusable real Gram LU factorization with fixed settings.
pub struct GramFactorization<'a, E: FamilyExpression> {
    family: &'a OperatorFamily<E>,
    factor: LUFactorized<OwnedRepr<E::Real>>,
    norms: Array1<E::Real>,
    independence: IndependenceReport<E::Real>,
    options: NumericalOptions<E::Real>,
}

/// Real projection coordinates tied to a particular ordered family.
pub struct FamilyCoordinates<'a, E: FamilyExpression> {
    family: &'a OperatorFamily<E>,
    values: Array1<E::Real>,
}

struct TargetCoordinates<'a, E: FamilyExpression> {
    coordinates: FamilyCoordinates<'a, E>,
    norm: E::Real,
}

impl<'a, E: FamilyExpression> FamilyCoordinates<'a, E> {
    /// The exact originating family, including its context and order.
    pub fn family(&self) -> &'a OperatorFamily<E> {
        self.family
    }
    /// Real coefficients in family order, including for an out-of-span target.
    pub fn values(&self) -> &Array1<E::Real> {
        &self.values
    }
    /// Reconstructs the projection represented by these coordinates.
    pub fn reconstruct(&self) -> Result<E, LieError<E::Real>> {
        self.family.reconstruct(self.values.view())
    }
}

/// A projection, its expression residual, and the effective span-membership check.
pub struct Projection<'a, E: FamilyExpression> {
    coordinates: FamilyCoordinates<'a, E>,
    target: E,
    operator: E,
    residual: E,
    check: ToleranceCheck<E::Real>,
}

impl<'a, E: FamilyExpression> Projection<'a, E> {
    /// Real projection coordinates retaining the originating family.
    pub fn coordinates(&self) -> &FamilyCoordinates<'a, E> {
        &self.coordinates
    }
    /// Shared original target expression.
    pub fn target(&self) -> &E {
        &self.target
    }
    /// Reconstructed projection in the family context.
    pub fn operator(&self) -> &E {
        &self.operator
    }
    /// Shared expression `target - projection` in the family context.
    pub fn residual_operator(&self) -> &E {
        &self.residual
    }
    /// Scalar normalized HS norm of the residual.
    pub fn residual_norm(&self) -> E::Real {
        self.check.measured
    }
    /// Residual check with the target's normalized HS norm as scale.
    pub fn span_check(&self) -> &ToleranceCheck<E::Real> {
        &self.check
    }
    /// Whether the target is in the span within the recorded residual bound.
    pub fn in_span(&self) -> bool {
        self.check.passed()
    }
}

fn norm<E: FamilyExpression>(
    operator: &E,
    context: &E::Context,
    scale_squared: E::Real,
    options: &NumericalOptions<E::Real>,
) -> Result<E::Real, LieError<E::Real>>
where
    NumericalOptions<E::Real>: Default,
{
    let value = operator.hs_in(operator, context)?;
    finite([value.re, value.im, scale_squared])?;
    let scale = scale_squared.max(value.re.hypot(value.im));
    Ok(squared_norm(value, scale, options)?.sqrt())
}

fn hermiticity<E: FamilyExpression>(
    operator: &E,
    context: &E::Context,
    operator_norm: E::Real,
    options: &NumericalOptions<E::Real>,
) -> Result<ToleranceCheck<E::Real>, LieError<E::Real>>
where
    NumericalOptions<E::Real>: Default,
{
    let difference = E::sum_in(
        vec![
            operator.clone(),
            operator.adjoint_expression().scaled_real(-E::Real::one()),
        ],
        context,
    )?;
    let sum_norm = operator_norm + operator_norm;
    let defect = norm(&difference, context, sum_norm * sum_norm, options)?;
    Ok(options.hermiticity.check(defect, operator_norm)?)
}

impl<E: FamilyExpression> OperatorFamily<E>
where
    NumericalOptions<E::Real>: Default,
{
    /// Evaluates the fixed normalized-HS Gram matrix without demanding validity.
    pub fn gram(
        &self,
        options: &NumericalOptions<E::Real>,
    ) -> Result<GramMatrix<'_, E>, LieError<E::Real>> {
        options.validate()?;
        let n = self.len();
        let mut values = Array2::zeros((n, n));
        for i in 0..n {
            for j in 0..n {
                let value = self.operators[i].hs_in(&self.operators[j], &self.context)?;
                finite([value.re, value.im])?;
                values[(i, j)] = value;
            }
        }
        let norms = (0..n)
            .map(|i| {
                let value = values[(i, i)];
                squared_norm(value, value.re.hypot(value.im), options).map(Float::sqrt)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(GramMatrix {
            family: self,
            values,
            norms: Array1::from_vec(norms),
            options: *options,
        })
    }

    /// Checks candidate Hermiticity and independence; use `closure` separately.
    /// Dependent and non-Hermitian candidates return failed diagnostics, not errors.
    pub fn validate(
        &self,
        options: &NumericalOptions<E::Real>,
    ) -> Result<FamilyValidation<'_, E>, LieError<E::Real>> {
        self.gram(options)?.validate()
    }

    /// Obtains real projection coordinates, also for an out-of-span Hermitian target.
    /// Both the family and target must be Hermitian; the family must be independent.
    pub fn coefficients(
        &self,
        target: &E,
        options: &NumericalOptions<E::Real>,
    ) -> Result<FamilyCoordinates<'_, E>, LieError<E::Real>> {
        self.gram(options)?.factorize()?.coefficients(target)
    }

    /// Projects a Hermitian target and records residual and span diagnostics.
    pub fn project(
        &self,
        target: &E,
        options: &NumericalOptions<E::Real>,
    ) -> Result<Projection<'_, E>, LieError<E::Real>> {
        self.gram(options)?.factorize()?.project(target)
    }

    /// Returns the normalized HS norm of `target - project(target)`.
    pub fn residual(
        &self,
        target: &E,
        options: &NumericalOptions<E::Real>,
    ) -> Result<E::Real, LieError<E::Real>> {
        Ok(self.project(target, options)?.residual_norm())
    }

    /// Tests the residual against `options.residual`, scaled by the target HS norm.
    pub fn in_span(
        &self,
        target: &E,
        options: &NumericalOptions<E::Real>,
    ) -> Result<bool, LieError<E::Real>> {
        Ok(self.project(target, options)?.in_span())
    }
}

impl<'a, E: FamilyExpression> GramMatrix<'a, E>
where
    NumericalOptions<E::Real>: Default,
{
    fn real_matrix(&self) -> Array2<E::Real> {
        self.values.mapv(|value| value.re)
    }

    /// Diagnoses real-linear independence from `Re(G)` without discarding inputs.
    /// For Hermitian operators `G` itself is real within the imaginary tolerance.
    pub fn independence(&self) -> Result<IndependenceReport<E::Real>, LieError<E::Real>> {
        let spectrum = symmetric_eigen(&self.real_matrix(), &self.options)?;
        let relative_cutoff = self.options.gram_cutoff(self.family.len())?;
        let maximum = spectrum
            .values
            .iter()
            .copied()
            .fold(E::Real::zero(), Float::max);
        let threshold = relative_cutoff * maximum;
        finite([threshold])?;
        if spectrum.values.iter().any(|&value| value < -threshold) {
            return Err(NumericalError::IndefiniteGram.into());
        }
        Ok(IndependenceReport {
            rank: spectrum
                .values
                .iter()
                .filter(|&&value| value > threshold)
                .count(),
            dimension: self.family.len(),
            eigenvalues: spectrum.values,
            relative_cutoff,
            threshold,
        })
    }

    fn hermiticity(&self) -> Result<Vec<ToleranceCheck<E::Real>>, LieError<E::Real>> {
        self.family
            .iter()
            .zip(&self.norms)
            .map(|(operator, &scale)| {
                hermiticity(operator, &self.family.context, scale, &self.options)
            })
            .collect()
    }

    /// Reuses this matrix for Hermiticity and independence diagnostics.
    pub fn validate(&self) -> Result<FamilyValidation<'a, E>, LieError<E::Real>> {
        Ok(FamilyValidation {
            family: self.family,
            hermiticity: self.hermiticity()?,
            independence: self.independence()?,
            options: self.options,
        })
    }

    /// Checks Hermiticity, real Gram entries, and independence, then factors `G`.
    /// No pseudoinverse or element removal is used for a singular family.
    pub fn factorize(&self) -> Result<GramFactorization<'a, E>, LieError<E::Real>> {
        for (index, check) in self.hermiticity()?.into_iter().enumerate() {
            if !check.passed() {
                return Err(LieError::NonHermitianElement { index, check });
            }
        }
        for ((i, j), &value) in self.values.indexed_iter() {
            real_coordinate(value, self.norms[i] * self.norms[j], &self.options)?;
        }
        let independence = self.independence()?;
        if !independence.is_independent() {
            return Err(NumericalError::SingularBasis {
                rank: independence.rank,
                dimension: independence.dimension,
            }
            .into());
        }
        let factor = self
            .real_matrix()
            .factorize()
            .map_err(NumericalError::from)?;
        Ok(GramFactorization {
            family: self.family,
            factor,
            norms: self.norms.clone(),
            independence,
            options: self.options,
        })
    }
}

impl<'a, E: FamilyExpression> GramFactorization<'a, E>
where
    NumericalOptions<E::Real>: Default,
{
    /// The exact originating family.
    pub fn family(&self) -> &'a OperatorFamily<E> {
        self.family
    }
    /// Fixed settings used for this factorization and subsequent targets.
    pub fn options(&self) -> &NumericalOptions<E::Real> {
        &self.options
    }
    /// Effective Gram-eigenvalue diagnostics used to admit the factorization.
    pub fn independence(&self) -> &IndependenceReport<E::Real> {
        &self.independence
    }

    fn solve_target(&self, target: &E) -> Result<TargetCoordinates<'a, E>, LieError<E::Real>> {
        target.in_context(&self.family.context)?;
        let target_norm = norm(target, &self.family.context, E::Real::zero(), &self.options)?;
        let check = hermiticity(target, &self.family.context, target_norm, &self.options)?;
        if !check.passed() {
            return Err(LieError::NonHermitianTarget { check });
        }
        let rhs = self
            .family
            .iter()
            .zip(&self.norms)
            .map(|(operator, &operator_norm)| {
                let value = operator.hs_in(target, &self.family.context)?;
                Ok(real_coordinate(
                    value,
                    operator_norm * target_norm,
                    &self.options,
                )?)
            })
            .collect::<Result<Vec<_>, LieError<E::Real>>>()?;
        let values = self
            .factor
            .solve(&Array1::from_vec(rhs))
            .map_err(NumericalError::from)?;
        finite(values.iter().copied())?;
        Ok(TargetCoordinates {
            coordinates: FamilyCoordinates {
                family: self.family,
                values,
            },
            norm: target_norm,
        })
    }

    /// Solves `G c = b` for a new target without refactoring the family Gram matrix.
    pub fn coefficients(&self, target: &E) -> Result<FamilyCoordinates<'a, E>, LieError<E::Real>> {
        Ok(self.solve_target(target)?.coordinates)
    }

    /// Projects a new target using the retained factorization and settings.
    pub fn project(&self, target: &E) -> Result<Projection<'a, E>, LieError<E::Real>> {
        let TargetCoordinates {
            coordinates,
            norm: target_norm,
        } = self.solve_target(target)?;
        let operator = coordinates.reconstruct()?;
        let projection_norm = norm(
            &operator,
            &self.family.context,
            E::Real::zero(),
            &self.options,
        )?;
        let residual = E::sum_in(
            vec![target.clone(), operator.scaled_real(-E::Real::one())],
            &self.family.context,
        )?;
        let sum_norm = target_norm + projection_norm;
        let residual_norm = norm(
            &residual,
            &self.family.context,
            sum_norm * sum_norm,
            &self.options,
        )?;
        let check = self.options.residual.check(residual_norm, target_norm)?;
        Ok(Projection {
            coordinates,
            target: target.clone(),
            operator,
            residual,
            check,
        })
    }
}
