//! Ordinary Hermitian Lie brackets and representations in retained family coordinates.

use ndarray::{Array2, Array3};
use num_traits::Zero;

use crate::{
    FamilyCoordinates, FamilyExpression, GramFactorization, LieError, NumericalError,
    NumericalOptions, OperatorFamily, Projection, ToleranceCheck, numerical::finite,
};

/// Constructs `-i(AB-BA)`, including on graded spaces (an ordinary bracket).
///
/// This constructs an expression without numerical validation. Core resolves
/// physical grading signs and rejects incompatible placed contexts.
pub fn lie_bracket<E: FamilyExpression>(left: &E, right: &E) -> Result<E, LieError<E::Real>> {
    Ok(left.lie_bracket_expression(right)?)
}

/// Span diagnosis for one ordered pair of family elements.
pub struct BracketClosureCheck<R> {
    /// Left input index.
    pub left: usize,
    /// Right input index.
    pub right: usize,
    /// Residual against the bracket's normalized HS norm scale.
    pub check: ToleranceCheck<R>,
}

/// Pairwise closure diagnostics for an independent Hermitian family.
///
/// Failed checks are reports, not errors. Projected coefficients of a nonclosed
/// family are deliberately not exposed as structure constants.
pub struct ClosureReport<'a, E: FamilyExpression> {
    family: &'a OperatorFamily<E>,
    values: Array3<E::Real>,
    checks: Vec<BracketClosureCheck<E::Real>>,
    options: NumericalOptions<E::Real>,
}

impl<'a, E: FamilyExpression> ClosureReport<'a, E> {
    /// Exact originating family, context, precision, and element order.
    pub fn family(&self) -> &'a OperatorFamily<E> {
        self.family
    }
    /// All ordered pairs, with left index outermost; includes diagonal pairs.
    pub fn checks(&self) -> &[BracketClosureCheck<E::Real>] {
        &self.checks
    }
    /// Effective settings used for projection and span checks.
    pub fn options(&self) -> &NumericalOptions<E::Real> {
        &self.options
    }
    /// Whether every bracket lies in the span under the recorded tolerance.
    pub fn passed(&self) -> bool {
        self.checks.iter().all(|pair| pair.check.passed())
    }
    /// Requires closure before retaining these coefficients as Lie structure constants.
    pub fn structure_constants(self) -> Result<StructureConstants<'a, E>, LieError<E::Real>> {
        for pair in &self.checks {
            if !pair.check.passed() {
                return Err(LieError::NonClosedPair {
                    left: pair.left,
                    right: pair.right,
                    check: pair.check,
                });
            }
        }
        Ok(StructureConstants { closure: self })
    }
}

/// Real coefficients `-i[B_i,B_j] = sum_k f[k,i,j] B_k`, output index first.
pub struct StructureConstants<'a, E: FamilyExpression> {
    closure: ClosureReport<'a, E>,
}

impl<'a, E: FamilyExpression> StructureConstants<'a, E> {
    /// Exact originating family and context.
    pub fn family(&self) -> &'a OperatorFamily<E> {
        self.closure.family
    }
    /// Three axes `(output, left, right)` in the originating family's order.
    pub fn values(&self) -> &Array3<E::Real> {
        &self.closure.values
    }
    /// Closure diagnostics admitting this representation.
    pub fn closure(&self) -> &ClosureReport<'a, E> {
        &self.closure
    }
    /// Settings retained with the representation.
    pub fn options(&self) -> &NumericalOptions<E::Real> {
        &self.closure.options
    }
    /// Retains `ad_i[k,j]=f[k,i,j]`; columns describe action on each `B_j`.
    pub fn adjoint(&self) -> AdjointRepresentation<'a, E> {
        let n = self.family().len();
        AdjointRepresentation {
            family: self.family(),
            matrices: (0..n)
                .map(|i| Array2::from_shape_fn((n, n), |(k, j)| self.values()[(k, i, j)]))
                .collect(),
            options: *self.options(),
        }
    }
}

/// Adjoint action on real Lie algebra coordinates, in original family order.
pub struct AdjointRepresentation<'a, E: FamilyExpression> {
    family: &'a OperatorFamily<E>,
    matrices: Vec<Array2<E::Real>>,
    options: NumericalOptions<E::Real>,
}

impl<'a, E: FamilyExpression> AdjointRepresentation<'a, E> {
    /// Exact originating family and context.
    pub fn family(&self) -> &'a OperatorFamily<E> {
        self.family
    }
    /// One matrix per family element, with `(output, input)` axes.
    pub fn matrices(&self) -> &[Array2<E::Real>] {
        &self.matrices
    }
    /// Settings used to admit the underlying structure constants.
    pub fn options(&self) -> &NumericalOptions<E::Real> {
        &self.options
    }
    /// Classical `K[i,j]=Tr(ad_i ad_j)`, without conjugation or normalization.
    pub fn killing_form(&self) -> Result<KillingForm<'a, E>, LieError<E::Real>> {
        let n = self.family.len();
        let values = Array2::from_shape_fn((n, n), |(i, j)| {
            let mut trace = E::Real::zero();
            for k in 0..n {
                for l in 0..n {
                    trace += self.matrices[i][(k, l)] * self.matrices[j][(l, k)];
                }
            }
            trace
        });
        finite(values.iter().copied())?;
        Ok(KillingForm {
            family: self.family,
            values,
            options: self.options,
        })
    }
}

/// Classical Killing form in a particular ordered operator family.
pub struct KillingForm<'a, E: FamilyExpression> {
    family: &'a OperatorFamily<E>,
    values: Array2<E::Real>,
    options: NumericalOptions<E::Real>,
}

impl<'a, E: FamilyExpression> KillingForm<'a, E> {
    /// Exact originating family and context.
    pub fn family(&self) -> &'a OperatorFamily<E> {
        self.family
    }
    /// `K[i,j]=Tr(ad_i ad_j)` in family order.
    pub fn values(&self) -> &Array2<E::Real> {
        &self.values
    }
    /// Settings used to admit the underlying structure constants.
    pub fn options(&self) -> &NumericalOptions<E::Real> {
        &self.options
    }
}

impl<E: FamilyExpression> OperatorFamily<E> {
    /// Constructs the bracket of two indexed family elements in its common context.
    pub fn bracket(&self, left: usize, right: usize) -> Result<E, LieError<E::Real>> {
        let a = self
            .operators()
            .get(left)
            .ok_or(NumericalError::Shape("left bracket index"))?;
        let b = self
            .operators()
            .get(right)
            .ok_or(NumericalError::Shape("right bracket index"))?;
        lie_bracket(
            &a.in_context(self.context())?,
            &b.in_context(self.context())?,
        )
    }
}

impl<E: FamilyExpression> OperatorFamily<E>
where
    NumericalOptions<E::Real>: Default,
{
    /// Projects a bracket; an out-of-span result remains explicitly a projection.
    pub fn bracket_projection(
        &self,
        left: usize,
        right: usize,
        options: &NumericalOptions<E::Real>,
    ) -> Result<Projection<'_, E>, LieError<E::Real>> {
        self.gram(options)?
            .factorize()?
            .bracket_projection(left, right)
    }
    /// Real projection coefficients of a bracket, including for nonclosed families.
    pub fn bracket_coefficients(
        &self,
        left: usize,
        right: usize,
        options: &NumericalOptions<E::Real>,
    ) -> Result<FamilyCoordinates<'_, E>, LieError<E::Real>> {
        self.gram(options)?
            .factorize()?
            .bracket_coefficients(left, right)
    }
    /// Checks every ordered bracket; requires Hermiticity and independence.
    pub fn closure(
        &self,
        options: &NumericalOptions<E::Real>,
    ) -> Result<ClosureReport<'_, E>, LieError<E::Real>> {
        self.gram(options)?.factorize()?.closure()
    }
    /// Requires Hermiticity, independence, and closure before retaining coefficients.
    pub fn structure_constants(
        &self,
        options: &NumericalOptions<E::Real>,
    ) -> Result<StructureConstants<'_, E>, LieError<E::Real>> {
        self.closure(options)?.structure_constants()
    }
}

impl<'a, E: FamilyExpression> GramFactorization<'a, E>
where
    NumericalOptions<E::Real>: Default,
{
    /// Reuses the Gram factorization to project one ordinary Hermitian bracket.
    pub fn bracket_projection(
        &self,
        left: usize,
        right: usize,
    ) -> Result<Projection<'a, E>, LieError<E::Real>> {
        self.project(&self.family().bracket(left, right)?)
    }
    /// Reuses the Gram factorization to obtain real bracket projection coefficients.
    pub fn bracket_coefficients(
        &self,
        left: usize,
        right: usize,
    ) -> Result<FamilyCoordinates<'a, E>, LieError<E::Real>> {
        self.coefficients(&self.family().bracket(left, right)?)
    }
    /// Reuses this factorization for all ordered pairwise closure checks.
    pub fn closure(&self) -> Result<ClosureReport<'a, E>, LieError<E::Real>> {
        let n = self.family().len();
        let mut values = Array3::zeros((n, n, n));
        let mut checks = Vec::new();
        for i in 0..n {
            for j in 0..n {
                let projection = self.bracket_projection(i, j)?;
                for (k, &value) in projection.coordinates().values().iter().enumerate() {
                    values[(k, i, j)] = value;
                }
                checks.push(BracketClosureCheck {
                    left: i,
                    right: j,
                    check: *projection.span_check(),
                });
            }
        }
        Ok(ClosureReport {
            family: self.family(),
            values,
            checks,
            options: *self.options(),
        })
    }
    /// Checks closure using this factorization and retains structure constants.
    pub fn structure_constants(&self) -> Result<StructureConstants<'a, E>, LieError<E::Real>> {
        self.closure()?.structure_constants()
    }
}
