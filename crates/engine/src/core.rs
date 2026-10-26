use std::f64::consts::TAU;
use std::sync::Arc;

use audio_io::StereoFrame;
use dsp_core::{
    BinTransfer, Complex64, CrossSpectrumAccumulator, DisplayGrid, GridProjector,
    ImpulseResponseBuilder, SpectrumAnalyzer, compute_transfer, min_valid_auto_spectrum_power,
    power_to_db,
};

use crate::delay_estimator::DelayEstimator;
use crate::error::EngineError;
use crate::history::SignalHistory;
use crate::settings::{AnalysisSettings, DelayPolicy, MAX_COMPENSATION_SAMPLES};
use crate::snapshot::{DelayMeasurement, DelayStatus, MeasurementSnapshot};

const MILLISECONDS_PER_SECOND: f64 = 1000.0;
const COMPENSATION_CHANGE_EPSILON_SAMPLES: f64 = 1.0e-9;
const TRACKING_MIN_CHANGE_SAMPLES: f64 = 1.0;
const CALIBRATION_TIMEOUT_S: f64 = 10.0;
const MAX_FFT_LENGTH: usize = 65_536;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EngineCommand {
    UpdateSettings(AnalysisSettings),
    UpdateDelayPolicy(DelayPolicy),
    ResetAverages,
    FindDelayNow,
    ApplyLatestDelay,
    SetCompensationMs(f64),
    AdjustCompensationMs(f64),
    StartLatencyCalibration,
    CancelLatencyCalibration,
    SetSystemLatencyMs(Option<f64>),
}

struct AnalysisBuffers {
    reference_analyzer: SpectrumAnalyzer,
    measurement_analyzer: SpectrumAnalyzer,
    reference_window: Vec<f64>,
    measurement_window: Vec<f64>,
    reference_spectrum: Vec<Complex64>,
    measurement_spectrum: Vec<Complex64>,
    accumulator: CrossSpectrumAccumulator,
    bins: Vec<BinTransfer>,
    rotated_cross_spectrum: Vec<Complex64>,
    projector: GridProjector,
    impulse: ImpulseResponseBuilder,
}

impl AnalysisBuffers {
    fn new(
        settings: AnalysisSettings,
        sample_rate_hz: u32,
        grid: &DisplayGrid,
    ) -> Result<Self, EngineError> {
        let fft_length = settings.fft_length;
        let reference_analyzer = SpectrumAnalyzer::new(fft_length)?;
        let measurement_analyzer = SpectrumAnalyzer::new(fft_length)?;
        let bin_count = reference_analyzer.bin_count();
        let sample_rate = f64::from(sample_rate_hz);
        Ok(Self {
            reference_analyzer,
            measurement_analyzer,
            reference_window: vec![0.0; fft_length],
            measurement_window: vec![0.0; fft_length],
            reference_spectrum: vec![Complex64::default(); bin_count],
            measurement_spectrum: vec![Complex64::default(); bin_count],
            accumulator: CrossSpectrumAccumulator::new(
                bin_count,
                settings.averaging,
                settings.hop,
                sample_rate,
            ),
            bins: vec![BinTransfer::default(); bin_count],
            rotated_cross_spectrum: vec![Complex64::default(); bin_count],
            projector: GridProjector::new(grid, fft_length, sample_rate, settings.smoothing),
            impulse: ImpulseResponseBuilder::new(fft_length)?,
        })
    }
}

pub struct MeasurementCore {
    sample_rate_hz: u32,
    settings: AnalysisSettings,
    policy: DelayPolicy,
    grid: Arc<DisplayGrid>,
    history: SignalHistory,
    analysis: AnalysisBuffers,
    grid_power: Vec<Option<f64>>,
    frames_since_hop: usize,
    frames_since_delay_job: usize,
    delay_requested: bool,
    delay_generation: u64,
    pending_job_generation: Option<u64>,
    sequence: u64,
    compensation_samples: f64,
    system_latency_samples: Option<f64>,
    calibrating_latency: bool,
    calibration_failed: bool,
    frames_since_calibration_start: usize,
    latest_delay: Option<DelayMeasurement>,
    latest_confident_delay: Option<DelayMeasurement>,
    initial_delay_applied: bool,
    discontinuities: u64,
    source_finished: bool,
    hop_error_reported: bool,
}

impl MeasurementCore {
    pub fn new(
        sample_rate_hz: u32,
        settings: AnalysisSettings,
        policy: DelayPolicy,
        grid: Arc<DisplayGrid>,
        delay_fft_length: usize,
    ) -> Result<Self, EngineError> {
        let settings = settings.normalized();
        let analysis = AnalysisBuffers::new(settings, sample_rate_hz, &grid)?;
        let history_capacity =
            MAX_FFT_LENGTH.max(delay_fft_length) + MAX_COMPENSATION_SAMPLES + MAX_FFT_LENGTH;
        Ok(Self {
            sample_rate_hz,
            settings,
            policy,
            history: SignalHistory::new(history_capacity),
            analysis,
            grid_power: vec![None; grid.len()],
            grid,
            frames_since_hop: 0,
            frames_since_delay_job: 0,
            delay_requested: true,
            delay_generation: 0,
            pending_job_generation: None,
            sequence: 0,
            compensation_samples: 0.0,
            system_latency_samples: None,
            calibrating_latency: false,
            calibration_failed: false,
            frames_since_calibration_start: 0,
            latest_delay: None,
            latest_confident_delay: None,
            initial_delay_applied: false,
            discontinuities: 0,
            source_finished: false,
            hop_error_reported: false,
        })
    }

    pub fn grid(&self) -> &Arc<DisplayGrid> {
        &self.grid
    }

    pub fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    pub fn settings(&self) -> AnalysisSettings {
        self.settings
    }

    pub fn update_count(&self) -> u64 {
        self.analysis.accumulator.update_count()
    }

    pub fn compensation_ms(&self) -> f64 {
        self.samples_to_ms(self.compensation_samples)
    }

    fn samples_to_ms(&self, samples: f64) -> f64 {
        samples / f64::from(self.sample_rate_hz) * MILLISECONDS_PER_SECOND
    }

    fn ms_to_samples(&self, milliseconds: f64) -> f64 {
        milliseconds / MILLISECONDS_PER_SECOND * f64::from(self.sample_rate_hz)
    }

    pub fn apply_settings(&mut self, settings: AnalysisSettings) -> Result<(), EngineError> {
        let settings = settings.normalized();
        if settings.fft_length != self.settings.fft_length {
            self.analysis = AnalysisBuffers::new(settings, self.sample_rate_hz, &self.grid)?;
        } else {
            self.analysis.accumulator.set_averaging(settings.averaging);
            self.analysis.accumulator.set_hop_samples(settings.hop);
            self.analysis.projector.set_smoothing(settings.smoothing);
        }
        self.settings = settings;
        Ok(())
    }

    pub fn handle_command(&mut self, command: EngineCommand) -> Result<(), EngineError> {
        match command {
            EngineCommand::UpdateSettings(settings) => self.apply_settings(settings)?,
            EngineCommand::UpdateDelayPolicy(policy) => self.policy = policy,
            EngineCommand::ResetAverages => self.analysis.accumulator.reset(),
            EngineCommand::FindDelayNow => self.delay_requested = true,
            EngineCommand::ApplyLatestDelay => {
                if let Some(confident) = self.latest_confident_delay {
                    self.apply_manual_compensation(confident.total_samples);
                }
            }
            EngineCommand::SetCompensationMs(milliseconds) => {
                self.apply_manual_compensation(self.ms_to_samples(milliseconds));
            }
            EngineCommand::AdjustCompensationMs(delta_ms) => {
                self.apply_manual_compensation(
                    self.compensation_samples + self.ms_to_samples(delta_ms),
                );
            }
            EngineCommand::StartLatencyCalibration => {
                self.calibrating_latency = true;
                self.calibration_failed = false;
                self.frames_since_calibration_start = 0;
                self.delay_generation += 1;
                self.delay_requested = true;
                self.note_discontinuity();
            }
            EngineCommand::CancelLatencyCalibration => self.calibrating_latency = false,
            EngineCommand::SetSystemLatencyMs(latency_ms) => {
                self.system_latency_samples = latency_ms
                    .filter(|value| value.is_finite())
                    .map(|value| self.ms_to_samples(value));
            }
        }
        Ok(())
    }

    fn apply_manual_compensation(&mut self, samples: f64) {
        self.initial_delay_applied = true;
        self.set_compensation_samples(samples, COMPENSATION_CHANGE_EPSILON_SAMPLES);
    }

    fn set_compensation_samples(&mut self, samples: f64, minimum_change_samples: f64) {
        if !samples.is_finite() {
            return;
        }
        let limit = MAX_COMPENSATION_SAMPLES as f64;
        let samples = samples.clamp(-limit, limit);
        if (samples - self.compensation_samples).abs() >= minimum_change_samples {
            self.compensation_samples = samples;
            self.analysis.accumulator.reset();
        }
    }

    pub fn note_discontinuity(&mut self) {
        self.history.mark_discontinuity();
        self.frames_since_hop = 0;
        self.discontinuities += 1;
    }

    pub fn set_source_finished(&mut self, finished: bool) {
        self.source_finished = finished;
    }

    pub fn push_frames(
        &mut self,
        frames: &[StereoFrame],
        delay_estimator: &mut DelayEstimator,
    ) -> usize {
        let mut hops = 0;
        for frame in frames {
            self.history.push(*frame);
            self.frames_since_hop += 1;
            self.frames_since_delay_job = self.frames_since_delay_job.saturating_add(1);
            self.frames_since_calibration_start =
                self.frames_since_calibration_start.saturating_add(1);
            if self.frames_since_hop >= self.settings.hop {
                self.frames_since_hop = 0;
                match self.run_hop() {
                    Ok(true) => {
                        hops += 1;
                        self.hop_error_reported = false;
                    }
                    Ok(false) => {}
                    Err(error) => {
                        if !self.hop_error_reported {
                            tracing::error!(%error, "measurement hop failed");
                            self.hop_error_reported = true;
                        }
                    }
                }
            }
        }
        self.expire_calibration();
        self.maybe_submit_delay_job(delay_estimator);
        hops
    }

    fn expire_calibration(&mut self) {
        let timeout_frames = (CALIBRATION_TIMEOUT_S * f64::from(self.sample_rate_hz)) as usize;
        if self.calibrating_latency && self.frames_since_calibration_start > timeout_frames {
            self.calibrating_latency = false;
            self.calibration_failed = true;
        }
    }

    fn compensation_ages(&self) -> (usize, usize) {
        let whole_samples = self.compensation_samples.floor() as i64;
        if whole_samples >= 0 {
            (whole_samples as usize, 0)
        } else {
            (0, whole_samples.unsigned_abs() as usize)
        }
    }

    fn run_hop(&mut self) -> Result<bool, EngineError> {
        let (reference_age, measurement_age) = self.compensation_ages();
        let analysis = &mut self.analysis;
        if !self.history.copy_latest(
            &mut analysis.reference_window,
            &mut analysis.measurement_window,
            reference_age,
            measurement_age,
        ) {
            return Ok(false);
        }
        analysis
            .reference_analyzer
            .analyze(&analysis.reference_window, &mut analysis.reference_spectrum)?;
        analysis.measurement_analyzer.analyze(
            &analysis.measurement_window,
            &mut analysis.measurement_spectrum,
        )?;
        analysis
            .accumulator
            .update(&analysis.reference_spectrum, &analysis.measurement_spectrum)?;
        Ok(true)
    }

    fn delay_interval_frames(&self) -> usize {
        let interval_s = if self.policy.interval_s.is_finite() {
            self.policy.interval_s.max(0.0)
        } else {
            0.0
        };
        (interval_s * f64::from(self.sample_rate_hz)) as usize
    }

    fn maybe_submit_delay_job(&mut self, delay_estimator: &mut DelayEstimator) {
        let is_due =
            self.delay_requested || self.frames_since_delay_job >= self.delay_interval_frames();
        if !is_due || self.history.contiguous_frames() < delay_estimator.fft_length() {
            return;
        }
        let Some(job) = delay_estimator.idle_job() else {
            return;
        };
        if self
            .history
            .copy_latest(&mut job.reference, &mut job.measurement, 0, 0)
        {
            delay_estimator.submit();
            self.pending_job_generation = Some(self.delay_generation);
            self.frames_since_delay_job = 0;
            self.delay_requested = false;
        }
    }

    pub fn poll_delay(&mut self, delay_estimator: &mut DelayEstimator) -> bool {
        let Some(outcome) = delay_estimator.poll() else {
            return false;
        };
        let job_generation = self.pending_job_generation.take();
        if job_generation != Some(self.delay_generation) {
            return false;
        }
        match outcome {
            Ok(Some(estimate)) => {
                let measurement = DelayMeasurement {
                    total_ms: self.samples_to_ms(estimate.delay_samples),
                    total_samples: estimate.delay_samples,
                    confidence_db: estimate.confidence_db,
                    inverted_polarity: estimate.inverted_polarity,
                    measured_at_sequence: self.sequence + 1,
                };
                self.latest_delay = Some(measurement);
                if estimate.confidence_db >= self.policy.min_confidence_db {
                    self.latest_confident_delay = Some(measurement);
                    self.apply_confident_estimate(estimate.delay_samples);
                }
                true
            }
            Ok(None) => true,
            Err(error) => {
                tracing::warn!(%error, "delay estimation failed");
                false
            }
        }
    }

    fn apply_confident_estimate(&mut self, delay_samples: f64) {
        if self.calibrating_latency {
            self.calibrating_latency = false;
            self.system_latency_samples = Some(delay_samples);
            self.initial_delay_applied = true;
            self.set_compensation_samples(delay_samples, COMPENSATION_CHANGE_EPSILON_SAMPLES);
        } else if self.policy.auto_apply_on_start && !self.initial_delay_applied {
            self.initial_delay_applied = true;
            self.set_compensation_samples(delay_samples, COMPENSATION_CHANGE_EPSILON_SAMPLES);
        } else if self.policy.tracking {
            self.set_compensation_samples(delay_samples, TRACKING_MIN_CHANGE_SAMPLES);
        }
    }

    pub fn delay_status(&self) -> DelayStatus {
        DelayStatus {
            latest: self.latest_delay,
            latest_confident: self.latest_confident_delay,
            applied_ms: self.compensation_ms(),
            system_latency_ms: self
                .system_latency_samples
                .map(|samples| self.samples_to_ms(samples)),
            calibrating_latency: self.calibrating_latency,
            calibration_failed: self.calibration_failed,
        }
    }

    pub fn is_warming_up(&self) -> bool {
        let (reference_age, measurement_age) = self.compensation_ages();
        self.analysis.accumulator.update_count() == 0
            || self.history.contiguous_frames()
                < self.settings.fft_length + reference_age.max(measurement_age)
    }

    pub fn fill_snapshot(&mut self, snapshot: &mut MeasurementSnapshot) -> Result<(), EngineError> {
        let analysis = &mut self.analysis;
        compute_transfer(&analysis.accumulator, &mut analysis.bins)?;
        let grid_len = self.grid.len();
        snapshot.magnitude_db.resize(grid_len, None);
        snapshot.coherence.resize(grid_len, None);
        snapshot.response.resize(grid_len, None);

        analysis
            .projector
            .project_power(&analysis.bins, &mut self.grid_power)?;
        for (magnitude, power) in snapshot.magnitude_db.iter_mut().zip(&self.grid_power) {
            *magnitude = power.and_then(power_to_db);
        }
        analysis
            .projector
            .project_coherence(&analysis.bins, &mut snapshot.coherence)?;

        let fractional_samples = self.compensation_samples - self.compensation_samples.floor();
        let phase_step = Complex64::from_polar(
            1.0,
            TAU * fractional_samples / self.settings.fft_length as f64,
        );
        let mut rotation = Complex64::new(1.0, 0.0);
        for (rotated, cross) in analysis
            .rotated_cross_spectrum
            .iter_mut()
            .zip(analysis.accumulator.gxy())
        {
            *rotated = cross * rotation;
            rotation *= phase_step;
        }
        let min_power = min_valid_auto_spectrum_power(analysis.accumulator.gxx());
        analysis.projector.project_complex_ratio(
            &analysis.rotated_cross_spectrum,
            analysis.accumulator.gxx(),
            min_power,
            &mut snapshot.response,
        )?;
        analysis.projector.capture(
            &analysis.bins,
            &analysis.rotated_cross_spectrum,
            analysis.accumulator.gxx(),
            min_power,
            &mut snapshot.spectrum,
        )?;
        snapshot.impulse.resize(analysis.impulse.fft_length(), 0.0);
        analysis.impulse.compute(
            &analysis.rotated_cross_spectrum,
            analysis.accumulator.gxx(),
            min_power,
            &mut snapshot.impulse,
        )?;
        snapshot.impulse_zero_index = analysis.impulse.zero_index();

        self.sequence += 1;
        snapshot.sequence = self.sequence;
        snapshot.sample_rate_hz = self.sample_rate_hz;
        snapshot.settings = self.settings;
        snapshot.averages = self.analysis.accumulator.update_count();
        snapshot.warming_up = self.is_warming_up();
        snapshot.discontinuities = self.discontinuities;
        snapshot.source_finished = self.source_finished;
        snapshot.delay = self.delay_status();
        Ok(())
    }
}
