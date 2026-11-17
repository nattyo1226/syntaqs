use crate::NormalizationError;
use num_traits::Zero;

pub(crate) fn nonzero<C: Zero>(value: &C) -> Result<(), NormalizationError> {
    if value.is_zero() {
        Err(NormalizationError::ZeroNorm)
    } else {
        Ok(())
    }
}
