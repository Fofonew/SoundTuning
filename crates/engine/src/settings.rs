use dsp_core::{Averaging, Smoothing};

pub const FFT_LENGTH_CHOICES: [usize; 4] = [8192, 16_384, 32_768, 65_536];
pub const DEFAULT_FFT_LENGTH: usize = 32_768;
pub const MAX_HOP_FRAMES: usize = 65_536;
pub const DEFAULT_HOP: usize = 2048;
pub const TIME_CONSTANT_CHOICES_S: [f64; 4] = [0.5, 1.0, 2.0, 4.0];
pub const DEFAULT_TIME_CONSTANT_S: f64 = 1.0;
pub const DEFAULT_COHERENCE_THRESHOLD: f64 = 0.5;
pub const DEFAULT_DELAY_INTERVAL_S: f64 = 0.5;
pub const DEFAULT_MIN_DELAY_CONFIDENCE_DB: f64 = 18.0;
pub const MAX_COMPENSATION_SAMPLES: usize = 65_536;
const MIN_HOPS_PER_WINDOW_DIVISOR: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnalysisSettings {
    pub fft_length: usize,
    pub hop: usize,
    pub averaging: Averaging,
    pub smoothing: Smoothing,
    pub coherence_threshold: f64,
}

impl Default for AnalysisSettings {
    fn default() -> Self {
        Self {
            fft_length: DEFAULT_FFT_LENGTH,
            hop: DEFAULT_HOP,
            averaging: Averaging::Exponential {
                time_constant_s: DEFAULT_TIME_CONSTANT_S,
            },
            smoothing: Smoothing::OctaveFraction(48),
            coherence_threshold: DEFAULT_COHERENCE_THRESHOLD,
        }
    }
}

impl AnalysisSettings {
    pub fn normalized(self) -> Self {
        let fft_length = if FFT_LENGTH_CHOICES.contains(&self.fft_length) {
            self.fft_length
        } else {
            DEFAULT_FFT_LENGTH
        };
        let hop = self
            .hop
            .clamp(fft_length / MIN_HOPS_PER_WINDOW_DIVISOR, fft_length);
        let coherence_threshold = if self.coherence_threshold.is_finite() {
            self.coherence_threshold.clamp(0.0, 1.0)
        } else {
            DEFAULT_COHERENCE_THRESHOLD
        };
        Self {
            fft_length,
            hop,
            coherence_threshold,
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DelayPolicy {
    pub interval_s: f64,
    pub min_confidence_db: f64,
    pub auto_apply_on_start: bool,
    pub tracking: bool,
}

impl Default for DelayPolicy {
    fn default() -> Self {
        Self {
            interval_s: DEFAULT_DELAY_INTERVAL_S,
            min_confidence_db: DEFAULT_MIN_DELAY_CONFIDENCE_DB,
            auto_apply_on_start: true,
            tracking: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_follow_the_specification() {
        let settings = AnalysisSettings::default();
        assert_eq!(settings.fft_length, 32_768);
        assert_eq!(settings.hop, 2048);
        assert_eq!(settings.coherence_threshold, 0.5);
    }

    #[test]
    fn invalid_values_are_normalized() {
        let settings = AnalysisSettings {
            fft_length: 1000,
            hop: 0,
            coherence_threshold: f64::NAN,
            ..AnalysisSettings::default()
        }
        .normalized();
        assert_eq!(settings.fft_length, DEFAULT_FFT_LENGTH);
        assert_eq!(settings.hop, DEFAULT_FFT_LENGTH / 32);
        assert_eq!(settings.coherence_threshold, DEFAULT_COHERENCE_THRESHOLD);
    }
}
