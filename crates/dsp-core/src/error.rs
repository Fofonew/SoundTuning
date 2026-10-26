use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum DspError {
    #[error("{what}: expected length {expected}, got {actual}")]
    LengthMismatch {
        what: &'static str,
        expected: usize,
        actual: usize,
    },
    #[error("FFT length {0} must be even and at least 4")]
    InvalidFftLength(usize),
    #[error("FFT failed: {0}")]
    Fft(String),
}

pub(crate) fn check_length(
    what: &'static str,
    expected: usize,
    actual: usize,
) -> Result<(), DspError> {
    if expected == actual {
        Ok(())
    } else {
        Err(DspError::LengthMismatch {
            what,
            expected,
            actual,
        })
    }
}
