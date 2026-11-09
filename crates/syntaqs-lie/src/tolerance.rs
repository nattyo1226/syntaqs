//! Precision-preserving settings and effective scalar diagnostics.

use num_traits::{Float, FromPrimitive};

use crate::NumericalError;

/// An absolute-plus-relative bound, expressed in the measured quantity's units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tolerance<R> {
    /// Absolute allowance.
    pub absolute: R,
    /// Multiplier of the operation's explicitly supplied scale.
    pub relative: R,
}

/// A measurement and the exact effective bound used to classify it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToleranceCheck<R> {
    /// Nonnegative defect or residual.
    pub measured: R,
    /// Nonnegative reference scale, in the same units as the measurement.
    pub scale: R,
    /// Settings used for this check.
    pub tolerance: Tolerance<R>,
    /// Effective `absolute + relative * scale` bound.
    pub threshold: R,
}

impl<R: Float> ToleranceCheck<R> {
    /// Whether the measurement is within the inclusive effective bound.
    pub fn passed(&self) -> bool {
        self.measured <= self.threshold
    }
}

pub(crate) fn nonnegative<R: Float>(value: R, name: &'static str) -> Result<(), NumericalError> {
    if !value.is_finite() || value < R::zero() {
        return Err(NumericalError::InvalidSetting(name));
    }
    Ok(())
}

impl<R: Float> Tolerance<R> {
    /// Checks that both allowances are finite and nonnegative.
    pub fn validate(&self) -> Result<(), NumericalError> {
        nonnegative(self.absolute, "absolute tolerance")?;
        nonnegative(self.relative, "relative tolerance")
    }

    /// Computes a diagnostic without changing the measured data.
    ///
    /// An overflowing bound is an error, not an automatically passing check.
    pub fn check(&self, measured: R, scale: R) -> Result<ToleranceCheck<R>, NumericalError> {
        self.validate()?;
        nonnegative(measured, "measurement")?;
        nonnegative(scale, "scale")?;
        let threshold = self.absolute + self.relative * scale;
        if !threshold.is_finite() {
            return Err(NumericalError::NonFinite("effective tolerance"));
        }
        Ok(ToleranceCheck {
            measured,
            scale,
            tolerance: *self,
            threshold,
        })
    }
}

/// Independently configurable numerical decisions for Lie algebra analysis.
///
/// Defaults use `2e-5` for f32 and `1e-12` for f64. Spectral `None` settings
/// choose `max(default_floor, dimension * epsilon)`; `Some(r)` uses exactly r,
/// even zero. Changing a residual setting does not change spectral defaults.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumericalOptions<R> {
    /// Projection, closure, and reconstruction residuals, measured as norms.
    pub residual: Tolerance<R>,
    /// Symmetry/Hermiticity defects, measured as norms.
    pub hermiticity: Tolerance<R>,
    /// Imaginary parts of quantities required to be real.
    pub imaginary: Tolerance<R>,
    /// Negative squared-norm roundoff; scale is also in squared-norm units.
    /// Defaults to absolute zero and relative `64 * epsilon`.
    pub roundoff: Tolerance<R>,
    /// Optional exact relative override for singular-value rank/nullspace tests.
    pub rank_relative: Option<R>,
    /// Optional exact relative override for Gram eigenvalues (not their roots).
    pub gram_relative: Option<R>,
    /// Optional exact relative override for grouping symmetric eigenvalues.
    pub eigenvalue_relative: Option<R>,
}

macro_rules! defaults {
    ($real:ty, $floor:expr) => {
        impl Default for NumericalOptions<$real> {
            fn default() -> Self {
                let tolerance = Tolerance {
                    absolute: $floor,
                    relative: $floor,
                };
                Self {
                    residual: tolerance,
                    hermiticity: tolerance,
                    imaginary: tolerance,
                    roundoff: Tolerance {
                        absolute: 0.0,
                        relative: 64.0 * <$real>::EPSILON,
                    },
                    rank_relative: None,
                    gram_relative: None,
                    eigenvalue_relative: None,
                }
            }
        }
    };
}
defaults!(f32, 2e-5);
defaults!(f64, 1e-12);

impl<R: Float + FromPrimitive> NumericalOptions<R>
where
    Self: Default,
{
    /// Rejects invalid fields, including fields not used by a particular call.
    pub fn validate(&self) -> Result<(), NumericalError> {
        self.residual.validate()?;
        self.hermiticity.validate()?;
        self.imaginary.validate()?;
        self.roundoff.validate()?;
        for value in [
            self.rank_relative,
            self.gram_relative,
            self.eigenvalue_relative,
        ]
        .into_iter()
        .flatten()
        {
            nonnegative(value, "spectral relative tolerance")?;
        }
        Ok(())
    }

    fn relative(&self, explicit: Option<R>, dimension: usize) -> Result<R, NumericalError> {
        self.validate()?;
        if let Some(value) = explicit {
            return Ok(value);
        }
        let dimension =
            R::from_usize(dimension).ok_or(NumericalError::InvalidSetting("matrix dimension"))?;
        let result = Self::default()
            .residual
            .relative
            .max(dimension * R::epsilon());
        nonnegative(result, "effective spectral tolerance")?;
        Ok(result)
    }

    /// Effective relative singular-value cutoff; absolute cutoff is this times sigma_max.
    pub fn rank_cutoff(&self, rows: usize, columns: usize) -> Result<R, NumericalError> {
        self.relative(self.rank_relative, rows.max(columns))
    }

    /// Effective relative eigenvalue cutoff for the Gram matrix itself.
    ///
    /// Since Gram eigenvalues square the family's singular-value scale,
    /// this is not a cutoff on the original family's singular values.
    pub fn gram_cutoff(&self, dimension: usize) -> Result<R, NumericalError> {
        self.relative(self.gram_relative, dimension)
    }

    /// Effective relative eigenvalue gap; multiply by max(abs(eigenvalues)).
    pub fn eigenvalue_cutoff(&self, dimension: usize) -> Result<R, NumericalError> {
        self.relative(self.eigenvalue_relative, dimension)
    }
}
