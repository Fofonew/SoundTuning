use crate::Complex64;
use crate::error::{DspError, check_length};

const DENORMAL_FLUSH_THRESHOLD: f64 = 1.0e-280;

fn flush_denormal(value: f64) -> f64 {
    if value.abs() < DENORMAL_FLUSH_THRESHOLD {
        0.0
    } else {
        value
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Averaging {
    Exponential { time_constant_s: f64 },
    Infinite,
}

pub fn exponential_smoothing_coefficient(
    hop_samples: usize,
    time_constant_s: f64,
    sample_rate_hz: f64,
) -> f64 {
    let time_constant_samples = time_constant_s * sample_rate_hz;
    if !(time_constant_samples.is_finite() && time_constant_samples > 0.0) {
        return 1.0;
    }
    1.0 - (-(hop_samples as f64) / time_constant_samples).exp()
}

#[derive(Clone, Debug)]
pub struct CrossSpectrumAccumulator {
    gxx: Vec<f64>,
    gyy: Vec<f64>,
    gxy: Vec<Complex64>,
    averaging: Averaging,
    hop_samples: usize,
    sample_rate_hz: f64,
    update_count: u64,
}

impl CrossSpectrumAccumulator {
    pub fn new(
        bin_count: usize,
        averaging: Averaging,
        hop_samples: usize,
        sample_rate_hz: f64,
    ) -> Self {
        Self {
            gxx: vec![0.0; bin_count],
            gyy: vec![0.0; bin_count],
            gxy: vec![Complex64::default(); bin_count],
            averaging,
            hop_samples: hop_samples.max(1),
            sample_rate_hz,
            update_count: 0,
        }
    }

    pub fn set_hop_samples(&mut self, hop_samples: usize) {
        self.hop_samples = hop_samples.max(1);
    }

    pub fn reset(&mut self) {
        self.gxx.fill(0.0);
        self.gyy.fill(0.0);
        self.gxy.fill(Complex64::default());
        self.update_count = 0;
    }

    pub fn set_averaging(&mut self, averaging: Averaging) {
        let changes_kind = matches!(
            (self.averaging, averaging),
            (Averaging::Infinite, Averaging::Exponential { .. })
                | (Averaging::Exponential { .. }, Averaging::Infinite)
        );
        self.averaging = averaging;
        if changes_kind {
            self.reset();
        }
    }

    pub fn averaging(&self) -> Averaging {
        self.averaging
    }

    pub fn update_count(&self) -> u64 {
        self.update_count
    }

    pub fn bin_count(&self) -> usize {
        self.gxx.len()
    }

    #[cfg(test)]
    fn current_weight_for_tests(&self) -> f64 {
        exponential_smoothing_coefficient(self.hop_samples, 1.0, self.sample_rate_hz)
    }

    fn current_weight(&self) -> f64 {
        match self.averaging {
            Averaging::Infinite => 1.0 / self.update_count as f64,
            Averaging::Exponential { time_constant_s } => {
                let coefficient = exponential_smoothing_coefficient(
                    self.hop_samples,
                    time_constant_s,
                    self.sample_rate_hz,
                );
                if self.update_count == 1 {
                    1.0
                } else {
                    coefficient
                }
            }
        }
    }

    pub fn update(&mut self, x: &[Complex64], y: &[Complex64]) -> Result<(), DspError> {
        check_length("reference spectrum", self.bin_count(), x.len())?;
        check_length("measurement spectrum", self.bin_count(), y.len())?;
        self.update_count += 1;
        let weight = self.current_weight();
        let keep = 1.0 - weight;
        for (((gxx, gyy), gxy), (x_bin, y_bin)) in self
            .gxx
            .iter_mut()
            .zip(self.gyy.iter_mut())
            .zip(self.gxy.iter_mut())
            .zip(x.iter().zip(y.iter()))
        {
            let bins_are_finite = x_bin.re.is_finite()
                && x_bin.im.is_finite()
                && y_bin.re.is_finite()
                && y_bin.im.is_finite();
            if !bins_are_finite {
                continue;
            }
            *gxx = flush_denormal(keep * *gxx + weight * x_bin.norm_sqr());
            *gyy = flush_denormal(keep * *gyy + weight * y_bin.norm_sqr());
            let cross = *gxy * keep + x_bin.conj() * y_bin * weight;
            *gxy = Complex64::new(flush_denormal(cross.re), flush_denormal(cross.im));
        }
        Ok(())
    }

    pub fn gxx(&self) -> &[f64] {
        &self.gxx
    }

    pub fn gyy(&self) -> &[f64] {
        &self.gyy
    }

    pub fn gxy(&self) -> &[Complex64] {
        &self.gxy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coefficient_matches_definition() {
        let alpha = exponential_smoothing_coefficient(2048, 1.0, 48_000.0);
        assert!((alpha - (1.0 - (-2048.0_f64 / 48_000.0).exp())).abs() < 1.0e-15);
    }

    #[test]
    fn invalid_time_constant_means_no_memory() {
        assert_eq!(exponential_smoothing_coefficient(2048, 0.0, 48_000.0), 1.0);
        assert_eq!(
            exponential_smoothing_coefficient(2048, f64::NAN, 48_000.0),
            1.0
        );
    }

    #[test]
    fn infinite_averaging_is_arithmetic_mean() {
        let mut accumulator = CrossSpectrumAccumulator::new(1, Averaging::Infinite, 2048, 48_000.0);
        for value in [1.0, 2.0, 3.0, 6.0] {
            let x = [Complex64::new(value, 0.0)];
            accumulator.update(&x, &x).unwrap();
        }
        assert!((accumulator.gxx()[0] - (1.0 + 4.0 + 9.0 + 36.0) / 4.0).abs() < 1.0e-12);
    }

    #[test]
    fn first_exponential_update_takes_the_input_directly() {
        let mut accumulator = CrossSpectrumAccumulator::new(
            1,
            Averaging::Exponential {
                time_constant_s: 4.0,
            },
            2048,
            48_000.0,
        );
        let x = [Complex64::new(2.0, 0.0)];
        let y = [Complex64::new(0.0, 3.0)];
        accumulator.update(&x, &y).unwrap();
        assert_eq!(accumulator.gxx()[0], 4.0);
        assert_eq!(accumulator.gyy()[0], 9.0);
        assert_eq!(accumulator.gxy()[0], Complex64::new(0.0, 6.0));
    }

    #[test]
    fn long_silence_decays_to_exact_zero() {
        let mut accumulator = CrossSpectrumAccumulator::new(
            1,
            Averaging::Exponential {
                time_constant_s: 0.5,
            },
            2048,
            48_000.0,
        );
        let tone = [Complex64::new(1.0, 1.0)];
        let silence = [Complex64::default()];
        accumulator.update(&tone, &tone).unwrap();
        for _ in 0..20_000 {
            accumulator.update(&silence, &silence).unwrap();
        }
        assert_eq!(accumulator.gxx()[0], 0.0);
        assert_eq!(accumulator.gxy()[0], Complex64::default());
    }

    #[test]
    fn non_finite_bins_are_ignored() {
        let mut accumulator = CrossSpectrumAccumulator::new(2, Averaging::Infinite, 2048, 48_000.0);
        let good = [Complex64::new(1.0, 0.0); 2];
        let corrupted = [Complex64::new(f64::NAN, 0.0), Complex64::new(1.0, 0.0)];
        accumulator.update(&good, &good).unwrap();
        accumulator.update(&corrupted, &good).unwrap();
        accumulator.update(&good, &good).unwrap();
        assert!(accumulator.gxx().iter().all(|value| value.is_finite()));
        assert!(accumulator.gxy().iter().all(|value| value.re.is_finite()));
        assert!((accumulator.gxx()[0] - 1.0).abs() < 1.0e-12);
    }

    #[test]
    fn changing_time_constant_keeps_the_measurement() {
        let mut accumulator = CrossSpectrumAccumulator::new(
            1,
            Averaging::Exponential {
                time_constant_s: 1.0,
            },
            2048,
            48_000.0,
        );
        let x = [Complex64::new(2.0, 0.0)];
        accumulator.update(&x, &x).unwrap();
        accumulator.set_averaging(Averaging::Exponential {
            time_constant_s: 4.0,
        });
        assert_eq!(accumulator.update_count(), 1);
        accumulator.set_averaging(Averaging::Infinite);
        assert_eq!(accumulator.update_count(), 0);
    }

    #[test]
    fn zero_hop_is_treated_as_one_sample() {
        let accumulator = CrossSpectrumAccumulator::new(
            1,
            Averaging::Exponential {
                time_constant_s: 1.0,
            },
            0,
            48_000.0,
        );
        assert!(accumulator.current_weight_for_tests() > 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut accumulator = CrossSpectrumAccumulator::new(2, Averaging::Infinite, 2048, 48_000.0);
        let x = [Complex64::new(1.0, 1.0); 2];
        accumulator.update(&x, &x).unwrap();
        accumulator.reset();
        assert_eq!(accumulator.update_count(), 0);
        assert!(accumulator.gxx().iter().all(|value| *value == 0.0));
    }

    #[test]
    fn mismatched_lengths_are_rejected() {
        let mut accumulator = CrossSpectrumAccumulator::new(2, Averaging::Infinite, 2048, 48_000.0);
        let x = [Complex64::default(); 3];
        assert!(accumulator.update(&x, &x).is_err());
    }
}
