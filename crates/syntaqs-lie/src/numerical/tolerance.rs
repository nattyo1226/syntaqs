//! Precision-preserving tolerances and diagnostics.

use num_traits::Float;

use super::NumericalError;

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

pub(super) fn nonnegative<R: Float>(value: R, name: &'static str) -> Result<(), NumericalError> {
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
