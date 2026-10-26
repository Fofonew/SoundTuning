use crate::Complex64;
use crate::averaging::CrossSpectrumAccumulator;
use crate::error::{DspError, check_length};

const MIN_RELATIVE_AUTO_SPECTRUM_POWER: f64 = 1.0e-12;
const MIN_ABSOLUTE_AUTO_SPECTRUM_POWER: f64 = 1.0e-30;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BinTransfer {
    pub response: Option<Complex64>,
    pub coherence: Option<f64>,
}

pub fn compute_bin_transfer(gxx: f64, gyy: f64, gxy: Complex64, min_power: f64) -> BinTransfer {
    let reference_is_valid = gxx.is_finite() && gxx > min_power;
    let measurement_is_valid = gyy.is_finite() && gyy > min_power;
    if !reference_is_valid || !gxy.re.is_finite() || !gxy.im.is_finite() {
        return BinTransfer::default();
    }
    let response = gxy / gxx;
    let coherence = measurement_is_valid
        .then(|| (gxy.norm_sqr() / (gxx * gyy)).clamp(0.0, 1.0))
        .filter(|value| value.is_finite());
    BinTransfer {
        response: (response.re.is_finite() && response.im.is_finite()).then_some(response),
        coherence,
    }
}

pub fn min_valid_auto_spectrum_power(gxx: &[f64]) -> f64 {
    let max_power = gxx
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .fold(0.0_f64, f64::max);
    (max_power * MIN_RELATIVE_AUTO_SPECTRUM_POWER).max(MIN_ABSOLUTE_AUTO_SPECTRUM_POWER)
}

pub fn compute_transfer(
    accumulator: &CrossSpectrumAccumulator,
    output: &mut [BinTransfer],
) -> Result<(), DspError> {
    check_length("transfer output", accumulator.bin_count(), output.len())?;
    let min_power = min_valid_auto_spectrum_power(accumulator.gxx());
    for (((bin, gxx), gyy), gxy) in output
        .iter_mut()
        .zip(accumulator.gxx())
        .zip(accumulator.gyy())
        .zip(accumulator.gxy())
    {
        *bin = compute_bin_transfer(*gxx, *gyy, *gxy, min_power);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_signals_have_unity_response_and_coherence() {
        let bin = compute_bin_transfer(4.0, 4.0, Complex64::new(4.0, 0.0), 1.0e-30);
        assert_eq!(bin.response, Some(Complex64::new(1.0, 0.0)));
        assert_eq!(bin.coherence, Some(1.0));
    }

    #[test]
    fn silent_reference_bin_is_invalid() {
        let bin = compute_bin_transfer(0.0, 1.0, Complex64::new(0.0, 0.0), 1.0e-30);
        assert_eq!(bin, BinTransfer::default());
    }

    #[test]
    fn silent_measurement_keeps_response_but_not_coherence() {
        let bin = compute_bin_transfer(1.0, 0.0, Complex64::new(0.0, 0.0), 1.0e-30);
        assert_eq!(bin.response, Some(Complex64::new(0.0, 0.0)));
        assert_eq!(bin.coherence, None);
    }

    #[test]
    fn non_finite_inputs_are_invalid() {
        let bin = compute_bin_transfer(f64::INFINITY, 1.0, Complex64::new(1.0, 0.0), 1.0e-30);
        assert_eq!(bin, BinTransfer::default());
        let bin = compute_bin_transfer(1.0, 1.0, Complex64::new(f64::NAN, 0.0), 1.0e-30);
        assert_eq!(bin, BinTransfer::default());
    }

    #[test]
    fn coherence_never_exceeds_one_from_rounding() {
        let bin = compute_bin_transfer(1.0, 1.0, Complex64::new(1.000_000_1, 0.0), 1.0e-30);
        assert_eq!(bin.coherence, Some(1.0));
    }

    #[test]
    fn relative_threshold_scales_with_signal_level() {
        let threshold = min_valid_auto_spectrum_power(&[1.0e6, 0.0, 3.0]);
        assert!((threshold - 1.0e-6).abs() < 1.0e-18);
        assert_eq!(
            min_valid_auto_spectrum_power(&[0.0; 4]),
            MIN_ABSOLUTE_AUTO_SPECTRUM_POWER
        );
    }
}
