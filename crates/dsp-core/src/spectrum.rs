use std::sync::Arc;

use realfft::{RealFftPlanner, RealToComplex};

use crate::Complex64;
use crate::error::{DspError, check_length};
use crate::window::hann_window;

const MIN_FFT_LENGTH: usize = 4;

pub struct SpectrumAnalyzer {
    fft_length: usize,
    window: Vec<f64>,
    forward: Arc<dyn RealToComplex<f64>>,
    windowed: Vec<f64>,
    scratch: Vec<Complex64>,
}

impl SpectrumAnalyzer {
    pub fn new(fft_length: usize) -> Result<Self, DspError> {
        if fft_length < MIN_FFT_LENGTH || !fft_length.is_multiple_of(2) {
            return Err(DspError::InvalidFftLength(fft_length));
        }
        let forward = RealFftPlanner::<f64>::new().plan_fft_forward(fft_length);
        let scratch = forward.make_scratch_vec();
        Ok(Self {
            fft_length,
            window: hann_window(fft_length),
            forward,
            windowed: vec![0.0; fft_length],
            scratch,
        })
    }

    pub fn fft_length(&self) -> usize {
        self.fft_length
    }

    pub fn bin_count(&self) -> usize {
        self.fft_length / 2 + 1
    }

    pub fn analyze(&mut self, frame: &[f64], spectrum: &mut [Complex64]) -> Result<(), DspError> {
        check_length("analysis frame", self.fft_length, frame.len())?;
        check_length("spectrum", self.bin_count(), spectrum.len())?;
        for ((windowed, sample), weight) in self
            .windowed
            .iter_mut()
            .zip(frame.iter())
            .zip(self.window.iter())
        {
            *windowed = sample * weight;
        }
        self.forward
            .process_with_scratch(&mut self.windowed, spectrum, &mut self.scratch)
            .map_err(|error| DspError::Fft(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::TAU;

    use super::*;

    #[test]
    fn sine_on_bin_center_peaks_at_that_bin() {
        let fft_length = 1024;
        let bin = 37;
        let frame: Vec<f64> = (0..fft_length)
            .map(|index| (TAU * bin as f64 * index as f64 / fft_length as f64).sin())
            .collect();
        let mut analyzer = SpectrumAnalyzer::new(fft_length).unwrap();
        let mut spectrum = vec![Complex64::default(); analyzer.bin_count()];
        analyzer.analyze(&frame, &mut spectrum).unwrap();
        let peak_bin = spectrum
            .iter()
            .enumerate()
            .max_by(|left, right| left.1.norm().total_cmp(&right.1.norm()))
            .map(|(index, _)| index)
            .unwrap();
        assert_eq!(peak_bin, bin);
    }

    #[test]
    fn odd_or_tiny_lengths_are_rejected() {
        assert!(SpectrumAnalyzer::new(1023).is_err());
        assert!(SpectrumAnalyzer::new(2).is_err());
    }

    #[test]
    fn wrong_frame_length_is_an_error() {
        let mut analyzer = SpectrumAnalyzer::new(16).unwrap();
        let mut spectrum = vec![Complex64::default(); 9];
        assert!(matches!(
            analyzer.analyze(&[0.0; 8], &mut spectrum),
            Err(DspError::LengthMismatch { .. })
        ));
    }
}
