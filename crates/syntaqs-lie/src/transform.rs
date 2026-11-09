//! Real changes of family coordinates and normalized-HS orthogonalization.

use ndarray::{Array2, ArrayView2};
use num_traits::{Float, One, Zero};

use crate::{
    FamilyExpression, FamilyValidation, LieError, NumericalError, NumericalOptions, OperatorFamily,
    ToleranceCheck,
    numerical::{finite, gram_spectrum},
};

/// A new family and its retained coordinates in an immutable originating family.
pub struct FamilyTransform<'a, E: FamilyExpression> {
    source: &'a OperatorFamily<E>,
    family: OperatorFamily<E>,
    matrix: Array2<E::Real>,
    options: Option<NumericalOptions<E::Real>>,
}

impl<'a, E: FamilyExpression> FamilyTransform<'a, E> {
    /// Original family and context defining the rows of the transformation.
    pub fn source(&self) -> &'a OperatorFamily<E> {
        self.source
    }
    /// New family, retaining the original common context.
    pub fn family(&self) -> &OperatorFamily<E> {
        &self.family
    }
    /// Columns define `B'_j=sum_i B_i U[i,j]`; shape is `(old_len,new_len)`.
    pub fn matrix(&self) -> &Array2<E::Real> {
        &self.matrix
    }
    /// Settings used for orthogonalization; absent for explicit transforms/selections.
    pub fn options(&self) -> Option<&NumericalOptions<E::Real>> {
        self.options.as_ref()
    }
}

/// Hermiticity, independence, and the normalized-HS Gram defect from identity.
pub struct OrthonormalityReport<'a, E: FamilyExpression> {
    validation: FamilyValidation<'a, E>,
    check: ToleranceCheck<E::Real>,
}

impl<'a, E: FamilyExpression> OrthonormalityReport<'a, E> {
    /// Exact originating family.
    pub fn family(&self) -> &'a OperatorFamily<E> {
        self.validation.family()
    }
    /// Hermiticity and real-independence diagnostics using the same Gram matrix.
    pub fn validation(&self) -> &FamilyValidation<'a, E> {
        &self.validation
    }
    /// Frobenius norm of complex `G-I`, checked at scale `sqrt(family.len())`.
    pub fn check(&self) -> &ToleranceCheck<E::Real> {
        &self.check
    }
    /// Settings used by all diagnostics.
    pub fn options(&self) -> &NumericalOptions<E::Real> {
        self.validation.options()
    }
    /// Whether validation and the Gram identity check both pass.
    pub fn passed(&self) -> bool {
        self.validation.passed() && self.check.passed()
    }
}

impl<E: FamilyExpression> OperatorFamily<E> {
    /// Constructs a real transformed family, with new operators in matrix columns.
    ///
    /// Rectangular and singular transformations are allowed: this is candidate
    /// construction, not validation. The number of columns must be positive.
    pub fn transform(
        &self,
        matrix: ArrayView2<'_, E::Real>,
    ) -> Result<FamilyTransform<'_, E>, LieError<E::Real>> {
        if matrix.nrows() != self.len() {
            return Err(
                NumericalError::Shape("transformation rows must match family length").into(),
            );
        }
        if matrix.ncols() == 0 {
            return Err(LieError::EmptyFamily);
        }
        finite(matrix.iter().copied())?;
        let operators = matrix
            .columns()
            .into_iter()
            .map(|column| self.reconstruct(column))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(FamilyTransform {
            source: self,
            family: Self::new_in(self.context(), operators)?,
            matrix: matrix.to_owned(),
            options: None,
        })
    }

    /// Selects elements in the requested order and retains their selection matrix.
    /// Repeated indices are allowed; an empty selection is an error.
    pub fn select(&self, indices: &[usize]) -> Result<FamilyTransform<'_, E>, LieError<E::Real>> {
        let mut matrix = Array2::zeros((self.len(), indices.len()));
        for (j, &i) in indices.iter().enumerate() {
            if i >= self.len() {
                return Err(NumericalError::Shape("selection index outside family").into());
            }
            matrix[(i, j)] = E::Real::one();
        }
        self.transform(matrix.view())
    }
}

impl<E: FamilyExpression> OperatorFamily<E>
where
    NumericalOptions<E::Real>: Default,
{
    /// Diagnoses Hermitian normalized-HS orthonormality without requiring closure.
    pub fn orthonormality(
        &self,
        options: &NumericalOptions<E::Real>,
    ) -> Result<OrthonormalityReport<'_, E>, LieError<E::Real>> {
        let gram = self.gram(options)?;
        let mut defect = E::Real::zero();
        let mut scale = E::Real::zero();
        for ((i, j), value) in gram.values().indexed_iter() {
            let diagonal = if i == j {
                E::Real::one()
            } else {
                E::Real::zero()
            };
            defect = defect.hypot(value.re - diagonal).hypot(value.im);
            scale = scale.hypot(diagonal);
        }
        Ok(OrthonormalityReport {
            validation: gram.validate()?,
            check: options.residual.check(defect, scale)?,
        })
    }

    fn orthogonal_transform(
        &self,
        options: &NumericalOptions<E::Real>,
        normalize: bool,
    ) -> Result<FamilyTransform<'_, E>, LieError<E::Real>> {
        let gram = self.gram(options)?;
        // Require independent Hermitian inputs, including imaginary Gram checks.
        gram.factorize()?;
        let (spectrum, _) = gram_spectrum(&gram.values().mapv(|z| z.re), options)?;
        let mut matrix = spectrum.vectors;
        if normalize {
            for (j, mut column) in matrix.columns_mut().into_iter().enumerate() {
                let divisor = spectrum.values[j].sqrt();
                column.mapv_inplace(|x| x / divisor);
            }
        }
        let mut result = self.transform(matrix.view())?;
        result.options = Some(*options);
        Ok(result)
    }

    /// Produces an HS-orthogonal family via Gram eigenvectors, retaining `U`.
    /// Requires independent Hermitian inputs. Vectors are not normalized.
    pub fn orthogonalize(
        &self,
        options: &NumericalOptions<E::Real>,
    ) -> Result<FamilyTransform<'_, E>, LieError<E::Real>> {
        self.orthogonal_transform(options, false)
    }

    /// Produces an HS-orthonormal family using columns `v_j/sqrt(lambda_j)`.
    /// Requires independent Hermitian inputs; no pseudoinverse or pruning is used.
    pub fn orthonormalize(
        &self,
        options: &NumericalOptions<E::Real>,
    ) -> Result<FamilyTransform<'_, E>, LieError<E::Real>> {
        self.orthogonal_transform(options, true)
    }
}
