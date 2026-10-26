use std::sync::Arc;

use realfft::{ComplexToReal, RealFftPlanner};

use crate::Complex64;
use crate::error::{DspError, check_length};

const MIN_FFT_LENGTH: usize = 4;

pub struct ImpulseResponseBuilder {
    fft_length: usize,
    inverse: Arc<dyn ComplexToReal<f64>>,
    spectrum: Vec<Complex64>,
    time: Vec<f64>,
    scratch: Vec<Complex64>,
}

impl ImpulseResponseBuilder {
    pub fn new(fft_length: usize) -> Result<Self, DspError> {
        if fft_length < MIN_FFT_LENGTH || !fft_length.is_multiple_of(2) {
            return Err(DspError::InvalidFftLength(fft_length));
        }
        let inverse = RealFftPlanner::<f64>::new().plan_fft_inverse(fft_length);
        let scratch = inverse.make_scratch_vec();
        Ok(Self {
            fft_length,
            spectrum: vec![Complex64::default(); fft_length / 2 + 1],
            time: vec![0.0; fft_length],
            inverse,
            scratch,
        })
    }

    pub fn fft_length(&self) -> usize {
        self.fft_length
    }

    pub fn zero_index(&self) -> usize {
        self.fft_length / 2
    }

    pub fn compute(
        &mut self,
        cross_spectrum: &[Complex64],
        reference_power: &[f64],
        min_power: f64,
        output: &mut [f32],
    ) -> Result<(), DspError> {
        let bin_count = self.spectrum.len();
        check_length("cross spectrum", bin_count, cross_spectrum.len())?;
        check_length("reference power", bin_count, reference_power.len())?;
        check_length("impulse output", self.fft_length, output.len())?;
        for ((slot, cross), power) in self
            .spectrum
            .iter_mut()
            .zip(cross_spectrum)
            .zip(reference_power)
        {
            let usable = power.is_finite()
                && *power > min_power
                && cross.re.is_finite()
                && cross.im.is_finite();
            *slot = if usable {
                cross / *power
            } else {
                Complex64::default()
            };
        }
        for edge in [0, bin_count - 1] {
            if let Some(slot) = self.spectrum.get_mut(edge) {
                slot.im = 0.0;
            }
        }
        self.inverse
            .process_with_scratch(&mut self.spectrum, &mut self.time, &mut self.scratch)
            .map_err(|error| DspError::Fft(error.to_string()))?;
        let scale = 1.0 / self.fft_length as f64;
        let zero_index = self.zero_index();
        let (after_zero, before_zero) = self.time.split_at(self.fft_length - zero_index);
        for (slot, sample) in output.iter_mut().zip(before_zero.iter().chain(after_zero)) {
            *slot = (sample * scale) as f32;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::TAU;

    use super::*;

    const FFT_LENGTH: usize = 1024;

    fn delayed_gain(delay_samples: f64, gain: f64) -> (Vec<Complex64>, Vec<f64>) {
        let bins = FFT_LENGTH / 2 + 1;
        let cross = (0..bins)
            .map(|bin| {
                Complex64::from_polar(gain, -TAU * bin as f64 * delay_samples / FFT_LENGTH as f64)
            })
            .collect();
        (cross, vec![1.0; bins])
    }

    fn peak(output: &[f32]) -> (usize, f32) {
        output
            .iter()
            .copied()
            .enumerate()
            .max_by(|left, right| left.1.abs().total_cmp(&right.1.abs()))
            .unwrap()
    }

    #[test]
    fn a_pure_delay_peaks_after_the_zero_index() {
        let mut builder = ImpulseResponseBuilder::new(FFT_LENGTH).unwrap();
        let (cross, power) = delayed_gain(37.0, 0.5);
        let mut output = vec![0.0_f32; FFT_LENGTH];
        builder.compute(&cross, &power, 0.0, &mut output).unwrap();
        let (index, value) = peak(&output);
        assert_eq!(index, builder.zero_index() + 37);
        assert!((value - 0.5).abs() < 1.0e-6, "{value}");
    }

    #[test]
    fn negative_delays_land_before_the_zero_index() {
        let mut builder = ImpulseResponseBuilder::new(FFT_LENGTH).unwrap();
        let (cross, power) = delayed_gain(-12.0, -1.0);
        let mut output = vec![0.0_f32; FFT_LENGTH];
        builder.compute(&cross, &power, 0.0, &mut output).unwrap();
        let (index, value) = peak(&output);
        assert_eq!(index, builder.zero_index() - 12);
        assert!((value + 1.0).abs() < 1.0e-6, "{value}");
    }

    #[test]
    fn silent_reference_bins_give_a_silent_response() {
        let mut builder = ImpulseResponseBuilder::new(FFT_LENGTH).unwrap();
        let (mut cross, mut power) = delayed_gain(5.0, 1.0);
        power.fill(0.0);
        cross[3] = Complex64::new(f64::NAN, 0.0);
        let mut output = vec![1.0_f32; FFT_LENGTH];
        builder
            .compute(&cross, &power, 1.0e-12, &mut output)
            .unwrap();
        assert!(output.iter().all(|sample| *sample == 0.0));
    }

    #[test]
    fn mismatched_lengths_are_rejected() {
        let mut builder = ImpulseResponseBuilder::new(FFT_LENGTH).unwrap();
        let (cross, power) = delayed_gain(0.0, 1.0);
        let mut short = vec![0.0_f32; 10];
        assert!(builder.compute(&cross, &power, 0.0, &mut short).is_err());
        assert!(ImpulseResponseBuilder::new(3).is_err());
    }
}
