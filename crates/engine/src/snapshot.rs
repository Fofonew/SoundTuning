use dsp_core::{CapturedSpectrum, Complex64};

use crate::settings::AnalysisSettings;

const MILLISECONDS_PER_SECOND: f64 = 1000.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DelayMeasurement {
    pub total_ms: f64,
    pub total_samples: f64,
    pub confidence_db: f64,
    pub inverted_polarity: bool,
    pub measured_at_sequence: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DelayStatus {
    pub latest: Option<DelayMeasurement>,
    pub latest_confident: Option<DelayMeasurement>,
    pub applied_ms: f64,
    pub system_latency_ms: Option<f64>,
    pub calibrating_latency: bool,
    pub calibration_failed: bool,
}

impl DelayStatus {
    pub fn drift_ms(&self) -> Option<f64> {
        self.latest_confident
            .map(|measurement| measurement.total_ms - self.applied_ms)
    }

    pub fn acoustic_ms(&self, total_ms: f64) -> f64 {
        total_ms - self.system_latency_ms.unwrap_or(0.0)
    }

    pub fn applied_acoustic_ms(&self) -> f64 {
        self.acoustic_ms(self.applied_ms)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MeasurementSnapshot {
    pub sequence: u64,
    pub sample_rate_hz: u32,
    pub settings: AnalysisSettings,
    pub averages: u64,
    pub warming_up: bool,
    pub discontinuities: u64,
    pub source_finished: bool,
    pub magnitude_db: Vec<Option<f64>>,
    pub coherence: Vec<Option<f64>>,
    pub response: Vec<Option<Complex64>>,
    pub spectrum: CapturedSpectrum,
    pub impulse: Vec<f32>,
    pub impulse_zero_index: usize,
    pub delay: DelayStatus,
}

impl MeasurementSnapshot {
    pub fn empty(grid_len: usize, sample_rate_hz: u32, settings: AnalysisSettings) -> Self {
        Self {
            sequence: 0,
            sample_rate_hz,
            settings,
            averages: 0,
            warming_up: true,
            discontinuities: 0,
            source_finished: false,
            magnitude_db: vec![None; grid_len],
            coherence: vec![None; grid_len],
            response: vec![None; grid_len],
            spectrum: CapturedSpectrum::default(),
            impulse: Vec::new(),
            impulse_zero_index: 0,
            delay: DelayStatus::default(),
        }
    }

    pub fn impulse_time_ms(&self, index: usize) -> f64 {
        (index as f64 - self.impulse_zero_index as f64) * MILLISECONDS_PER_SECOND
            / f64::from(self.sample_rate_hz)
    }

    pub fn is_coherent(&self, index: usize) -> bool {
        self.coherence
            .get(index)
            .copied()
            .flatten()
            .is_some_and(|value| value >= self.settings.coherence_threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acoustic_delay_subtracts_system_latency() {
        let status = DelayStatus {
            latest: None,
            latest_confident: None,
            applied_ms: 10.5,
            system_latency_ms: Some(7.3),
            calibrating_latency: false,
            calibration_failed: false,
        };
        assert!((status.applied_acoustic_ms() - 3.2).abs() < 1.0e-12);
    }

    #[test]
    fn drift_compares_latest_confident_estimate_with_applied_delay() {
        let measurement = DelayMeasurement {
            total_ms: 12.5,
            total_samples: 600.0,
            confidence_db: 30.0,
            inverted_polarity: false,
            measured_at_sequence: 1,
        };
        let status = DelayStatus {
            latest_confident: Some(measurement),
            applied_ms: 10.0,
            ..DelayStatus::default()
        };
        assert!((status.drift_ms().unwrap() - 2.5).abs() < 1.0e-12);
        assert_eq!(DelayStatus::default().drift_ms(), None);
    }

    #[test]
    fn coherence_mask_uses_threshold() {
        let mut snapshot = MeasurementSnapshot::empty(3, 48_000, AnalysisSettings::default());
        snapshot.coherence = vec![Some(0.49), Some(0.5), None];
        assert!(!snapshot.is_coherent(0));
        assert!(snapshot.is_coherent(1));
        assert!(!snapshot.is_coherent(2));
        assert!(!snapshot.is_coherent(9));
    }
}
