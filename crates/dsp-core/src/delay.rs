use std::f64::consts::TAU;
use std::sync::Arc;

use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};

use crate::Complex64;
use crate::error::{DspError, check_length};

pub const DEFAULT_DELAY_FFT_LENGTH: usize = 131_072;
const NOISE_EXCLUSION_FRACTION: usize = 100;
const MIN_PEAK_MAGNITUDE: f64 = 1.0e-30;
const MIN_PARABOLA_CURVATURE: f64 = 1.0e-300;
const MILLISECONDS_PER_SECOND: f64 = 1000.0;
const MIN_DELAY_FFT_LENGTH: usize = 16;
const MAX_CONFIDENCE_DB: f64 = 200.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DelayEstimate {
    pub delay_samples: f64,
    pub confidence_db: f64,
    pub inverted_polarity: bool,
}

impl DelayEstimate {
    pub fn delay_ms(&self, sample_rate_hz: f64) -> f64 {
        self.delay_samples / sample_rate_hz * MILLISECONDS_PER_SECOND
    }
}

pub struct DelayFinder {
    fft_length: usize,
    forward: Arc<dyn RealToComplex<f64>>,
    inverse: Arc<dyn ComplexToReal<f64>>,
    reference_buffer: Vec<f64>,
    measurement_buffer: Vec<f64>,
    reference_spectrum: Vec<Complex64>,
    measurement_spectrum: Vec<Complex64>,
    cross_spectrum: Vec<Complex64>,
    correlation: Vec<f64>,
    forward_scratch: Vec<Complex64>,
    inverse_scratch: Vec<Complex64>,
}

impl DelayFinder {
    pub fn new(fft_length: usize) -> Result<Self, DspError> {
        if fft_length < MIN_DELAY_FFT_LENGTH || !fft_length.is_multiple_of(2) {
            return Err(DspError::InvalidFftLength(fft_length));
        }
        let mut planner = RealFftPlanner::<f64>::new();
        let forward = planner.plan_fft_forward(fft_length);
        let inverse = planner.plan_fft_inverse(fft_length);
        let bin_count = fft_length / 2 + 1;
        Ok(Self {
            fft_length,
            forward_scratch: forward.make_scratch_vec(),
            inverse_scratch: inverse.make_scratch_vec(),
            forward,
            inverse,
            reference_buffer: vec![0.0; fft_length],
            measurement_buffer: vec![0.0; fft_length],
            reference_spectrum: vec![Complex64::default(); bin_count],
            measurement_spectrum: vec![Complex64::default(); bin_count],
            cross_spectrum: vec![Complex64::default(); bin_count],
            correlation: vec![0.0; fft_length],
        })
    }

    pub fn fft_length(&self) -> usize {
        self.fft_length
    }

    pub fn cross_spectrum_of(
        &mut self,
        reference: &[f64],
        measurement: &[f64],
    ) -> Result<&[Complex64], DspError> {
        check_length("delay reference", self.fft_length, reference.len())?;
        check_length("delay measurement", self.fft_length, measurement.len())?;
        self.reference_buffer.copy_from_slice(reference);
        self.measurement_buffer.copy_from_slice(measurement);
        let fft_error = |error: realfft::FftError| DspError::Fft(error.to_string());
        self.forward
            .process_with_scratch(
                &mut self.reference_buffer,
                &mut self.reference_spectrum,
                &mut self.forward_scratch,
            )
            .map_err(fft_error)?;
        self.forward
            .process_with_scratch(
                &mut self.measurement_buffer,
                &mut self.measurement_spectrum,
                &mut self.forward_scratch,
            )
            .map_err(fft_error)?;
        for ((cross, x_bin), y_bin) in self
            .cross_spectrum
            .iter_mut()
            .zip(&self.reference_spectrum)
            .zip(&self.measurement_spectrum)
        {
            *cross = x_bin.conj() * y_bin;
        }
        Ok(&self.cross_spectrum)
    }

    pub fn estimate(
        &mut self,
        reference: &[f64],
        measurement: &[f64],
    ) -> Result<Option<DelayEstimate>, DspError> {
        self.cross_spectrum_of(reference, measurement)?;
        let mut cross = std::mem::take(&mut self.cross_spectrum);
        let result = self.estimate_from_cross_spectrum(&mut cross);
        self.cross_spectrum = cross;
        result
    }

    pub fn estimate_from_cross_spectrum(
        &mut self,
        cross_spectrum: &mut [Complex64],
    ) -> Result<Option<DelayEstimate>, DspError> {
        check_length(
            "cross spectrum",
            self.fft_length / 2 + 1,
            cross_spectrum.len(),
        )?;
        if let Some(first) = cross_spectrum.first_mut() {
            first.im = 0.0;
        }
        if let Some(last) = cross_spectrum.last_mut() {
            last.im = 0.0;
        }
        self.inverse
            .process_with_scratch(
                cross_spectrum,
                &mut self.correlation,
                &mut self.inverse_scratch,
            )
            .map_err(|error| DspError::Fft(error.to_string()))?;
        Ok(locate_correlation_peak(&self.correlation))
    }
}

fn locate_correlation_peak(correlation: &[f64]) -> Option<DelayEstimate> {
    let length = correlation.len();
    let (peak_index, peak_value) = correlation
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, value)| value.is_finite())
        .max_by(|left, right| left.1.abs().total_cmp(&right.1.abs()))?;
    let peak_magnitude = peak_value.abs();
    if peak_magnitude < MIN_PEAK_MAGNITUDE {
        return None;
    }

    let magnitude_at = |offset: isize| -> f64 {
        let index = (peak_index as isize + offset).rem_euclid(length as isize) as usize;
        correlation.get(index).map_or(0.0, |value| value.abs())
    };
    let before = magnitude_at(-1);
    let after = magnitude_at(1);
    let curvature = before - 2.0 * peak_magnitude + after;
    let fractional_offset = if curvature.abs() > MIN_PARABOLA_CURVATURE {
        (0.5 * (before - after) / curvature).clamp(-0.5, 0.5)
    } else {
        0.0
    };

    let signed_index = if peak_index > length / 2 {
        peak_index as f64 - length as f64
    } else {
        peak_index as f64
    };

    let exclusion = (length / NOISE_EXCLUSION_FRACTION).max(1);
    let mut noise_energy = 0.0;
    let mut noise_count = 0_usize;
    for (index, value) in correlation.iter().enumerate() {
        let distance = index
            .abs_diff(peak_index)
            .min(length - index.abs_diff(peak_index));
        if distance > exclusion && value.is_finite() {
            noise_energy += value * value;
            noise_count += 1;
        }
    }
    let noise_rms = if noise_count > 0 {
        (noise_energy / noise_count as f64).sqrt()
    } else {
        0.0
    };
    let confidence_db = if noise_rms > MIN_PEAK_MAGNITUDE {
        20.0 * (peak_magnitude / noise_rms).log10()
    } else {
        MAX_CONFIDENCE_DB
    };

    Some(DelayEstimate {
        delay_samples: signed_index + fractional_offset,
        confidence_db,
        inverted_polarity: peak_value < 0.0,
    })
}

pub fn rotate_phase_for_delay(
    response: &mut [Option<Complex64>],
    frequencies_hz: &[f64],
    delay_s: f64,
) -> Result<(), DspError> {
    check_length("frequencies", response.len(), frequencies_hz.len())?;
    for (value, frequency_hz) in response.iter_mut().zip(frequencies_hz) {
        if let Some(value) = value {
            *value *= Complex64::from_polar(1.0, TAU * frequency_hz * delay_s);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pseudo_random_signal(length: usize, seed: u64) -> Vec<f64> {
        let mut state = seed.max(1);
        (0..length)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                (state >> 11) as f64 / (1_u64 << 53) as f64 - 0.5
            })
            .collect()
    }

    #[test]
    fn integer_delay_is_found_exactly() {
        let length = 4096;
        let reference = pseudo_random_signal(length, 3);
        let delay = 37;
        let measurement: Vec<f64> = (0..length)
            .map(|index| reference[(index + length - delay) % length])
            .collect();
        let mut finder = DelayFinder::new(length).unwrap();
        let estimate = finder.estimate(&reference, &measurement).unwrap().unwrap();
        assert!((estimate.delay_samples - delay as f64).abs() < 0.01);
        assert!(!estimate.inverted_polarity);
        assert!(estimate.confidence_db > 20.0);
    }

    #[test]
    fn negative_delay_and_inverted_polarity_are_reported() {
        let length = 4096;
        let reference = pseudo_random_signal(length, 9);
        let lead = 20;
        let measurement: Vec<f64> = (0..length)
            .map(|index| -reference[(index + lead) % length])
            .collect();
        let mut finder = DelayFinder::new(length).unwrap();
        let estimate = finder.estimate(&reference, &measurement).unwrap().unwrap();
        assert!((estimate.delay_samples + lead as f64).abs() < 0.01);
        assert!(estimate.inverted_polarity);
    }

    #[test]
    fn silence_has_no_delay() {
        let mut finder = DelayFinder::new(1024).unwrap();
        let silence = vec![0.0; 1024];
        assert_eq!(finder.estimate(&silence, &silence).unwrap(), None);
    }

    #[test]
    fn delay_converts_to_milliseconds() {
        let estimate = DelayEstimate {
            delay_samples: 480.0,
            confidence_db: 30.0,
            inverted_polarity: false,
        };
        assert!((estimate.delay_ms(48_000.0) - 10.0).abs() < 1.0e-12);
    }

    #[test]
    fn phase_rotation_compensates_pure_delay() {
        let frequencies = [100.0, 1000.0, 5000.0];
        let delay_s = 0.00123;
        let mut response: Vec<Option<Complex64>> = frequencies
            .iter()
            .map(|frequency| Some(Complex64::from_polar(1.0, -TAU * frequency * delay_s)))
            .collect();
        rotate_phase_for_delay(&mut response, &frequencies, delay_s).unwrap();
        for value in response {
            assert!((value.unwrap() - Complex64::new(1.0, 0.0)).norm() < 1.0e-9);
        }
    }
}
