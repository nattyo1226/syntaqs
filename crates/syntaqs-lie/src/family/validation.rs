//! Validation reports for an operator family.

use super::*;

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
    pub(super) family: &'a OperatorFamily<E>,
    pub(super) hermiticity: Vec<ToleranceCheck<E::Real>>,
    pub(super) independence: IndependenceReport<E::Real>,
    pub(super) options: NumericalOptions<E::Real>,
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
