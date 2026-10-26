use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::time::Instant;

use audio_io::{
    AudioRoute, FilePlayback, FrameSource, GeneratorControls, LiveSession, MeterBank,
    MeterSnapshot, ReadReport, RecordingCursor, StereoFrame, StereoRecording, StreamFault,
    StreamHealth,
};
use dsp_core::DisplayGrid;
use engine::{EngineCommand, EngineHandle, MeasurementSnapshot};

use crate::config::AppConfig;
use crate::design::widgets::{METER_FLOOR_DBFS, MeterReading};
use crate::text;

const METER_RELEASE_DB_PER_SECOND: f32 = 24.0;
const SILENT_INPUT_ALERT_SECONDS: u64 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilePacing {
    RealTime,
    AsFastAsPossible,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceSpec {
    Live,
    File { path: PathBuf, pacing: FilePacing },
}

struct MeteredRecording {
    cursor: RecordingCursor,
    meters: Arc<MeterBank>,
}

impl FrameSource for MeteredRecording {
    fn sample_rate_hz(&self) -> u32 {
        self.cursor.sample_rate_hz()
    }

    fn read_frames(&mut self, destination: &mut Vec<StereoFrame>, max_frames: usize) -> ReadReport {
        let start = destination.len();
        let report = self.cursor.read_frames(destination, max_frames);
        if let Some(frames) = destination.get(start..) {
            self.meters
                .reference
                .record_samples(frames.iter().map(|frame| frame.reference));
            self.meters
                .measurement
                .record_samples(frames.iter().map(|frame| frame.measurement));
        }
        report
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Ballistics {
    peak_dbfs: f32,
    rms_dbfs: f32,
    clipped: bool,
}

impl Default for Ballistics {
    fn default() -> Self {
        Self {
            peak_dbfs: METER_FLOOR_DBFS,
            rms_dbfs: METER_FLOOR_DBFS,
            clipped: false,
        }
    }
}

impl Ballistics {
    fn update(&mut self, snapshot: MeterSnapshot, elapsed_s: f32) {
        let release = METER_RELEASE_DB_PER_SECOND * elapsed_s;
        self.peak_dbfs = snapshot
            .peak_dbfs
            .max(self.peak_dbfs - release)
            .max(METER_FLOOR_DBFS);
        self.rms_dbfs = snapshot.rms_dbfs.max(METER_FLOOR_DBFS);
        self.clipped = snapshot.clipped;
    }

    fn reading(self) -> MeterReading {
        MeterReading {
            peak_dbfs: self.peak_dbfs,
            rms_dbfs: self.rms_dbfs,
            clipped: self.clipped,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputAlert {
    DeviceDisconnected,
    PermissionDenied,
    SilentMeasurementInput,
    SilentReference,
}

enum Lifecycle {
    Idle,
    Starting(Receiver<Result<RunningMeasurement, String>>),
    Running(Box<RunningMeasurement>),
}

struct RunningMeasurement {
    engine: EngineHandle,
    _live_session: Option<LiveSession>,
    meters: Arc<MeterBank>,
    health: Option<Arc<StreamHealth>>,
    generator: Option<Arc<GeneratorControls>>,
    source_label: String,
    uses_separate_clocks: bool,
    route: AudioRoute,
    adjusted_sample_rate_hz: Option<u32>,
}

pub struct MeasurementController {
    grid: Arc<DisplayGrid>,
    source: SourceSpec,
    lifecycle: Lifecycle,
    display: MeasurementSnapshot,
    frozen: bool,
    last_error: Option<String>,
    reference_level: Ballistics,
    measurement_level: Ballistics,
    reference_silent_frames: u64,
    measurement_silent_frames: u64,
    last_meter_update: Instant,
}

impl MeasurementController {
    pub fn new(source: SourceSpec, config: &AppConfig) -> Self {
        let grid = Arc::new(DisplayGrid::default());
        let display = MeasurementSnapshot::empty(
            grid.len(),
            config.stream.sample_rate_hz,
            config.analysis.to_settings(),
        );
        Self {
            grid,
            source,
            lifecycle: Lifecycle::Idle,
            display,
            frozen: false,
            last_error: None,
            reference_level: Ballistics::default(),
            measurement_level: Ballistics::default(),
            reference_silent_frames: 0,
            measurement_silent_frames: 0,
            last_meter_update: Instant::now(),
        }
    }

    pub fn grid(&self) -> &Arc<DisplayGrid> {
        &self.grid
    }

    pub fn source(&self) -> &SourceSpec {
        &self.source
    }

    pub fn is_file_mode(&self) -> bool {
        matches!(self.source, SourceSpec::File { .. })
    }

    pub fn is_running(&self) -> bool {
        !matches!(self.lifecycle, Lifecycle::Idle)
    }

    pub fn is_starting(&self) -> bool {
        matches!(self.lifecycle, Lifecycle::Starting(_))
    }

    fn running(&self) -> Option<&RunningMeasurement> {
        match &self.lifecycle {
            Lifecycle::Running(running) => Some(running),
            Lifecycle::Idle | Lifecycle::Starting(_) => None,
        }
    }

    pub fn is_frozen(&self) -> bool {
        self.frozen
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    pub fn clear_error(&mut self) {
        self.last_error = None;
    }

    pub fn snapshot(&self) -> &MeasurementSnapshot {
        &self.display
    }

    pub fn source_label(&self) -> Option<&str> {
        self.running().map(|running| running.source_label.as_str())
    }

    pub fn uses_separate_clocks(&self) -> bool {
        self.running()
            .is_some_and(|running| running.uses_separate_clocks)
    }

    pub fn route(&self) -> Option<&AudioRoute> {
        self.running().map(|running| &running.route)
    }

    pub fn take_sample_rate_adjustment(&mut self) -> Option<u32> {
        match &mut self.lifecycle {
            Lifecycle::Running(running) => running.adjusted_sample_rate_hz.take(),
            Lifecycle::Idle | Lifecycle::Starting(_) => None,
        }
    }

    pub fn generator(&self) -> Option<&Arc<GeneratorControls>> {
        self.running()
            .and_then(|running| running.generator.as_ref())
    }

    fn reset_display(&mut self, config: &AppConfig) {
        self.display = MeasurementSnapshot::empty(
            self.grid.len(),
            config.stream.sample_rate_hz,
            config.analysis.to_settings(),
        );
        self.frozen = false;
        self.reference_level = Ballistics::default();
        self.measurement_level = Ballistics::default();
        self.reference_silent_frames = 0;
        self.measurement_silent_frames = 0;
    }

    pub fn start(&mut self, config: &AppConfig) {
        if self.is_running() {
            return;
        }
        self.reset_display(config);
        self.last_error = None;
        let (sender, receiver) = channel();
        let source = self.source.clone();
        let config = config.clone();
        let grid = Arc::clone(&self.grid);
        let spawned = std::thread::Builder::new()
            .name("measurement-start".into())
            .spawn(move || {
                let result = open_measurement(&source, &config, grid);
                if let Err(error) = &result {
                    tracing::error!(%error, "could not start the measurement");
                }
                if sender.send(result).is_err() {
                    tracing::debug!("measurement start abandoned");
                }
            });
        match spawned {
            Ok(_) => self.lifecycle = Lifecycle::Starting(receiver),
            Err(error) => self.last_error = Some(error.to_string()),
        }
    }

    pub fn stop(&mut self, config: &AppConfig) {
        let previous = std::mem::replace(&mut self.lifecycle, Lifecycle::Idle);
        if !matches!(previous, Lifecycle::Idle) {
            let releaser = std::thread::Builder::new()
                .name("measurement-stop".into())
                .spawn(move || match previous {
                    Lifecycle::Running(running) => drop(running),
                    Lifecycle::Starting(receiver) => {
                        if let Ok(Ok(running)) = receiver.recv() {
                            drop(running);
                        }
                    }
                    Lifecycle::Idle => {}
                });
            if let Err(error) = releaser {
                tracing::warn!(%error, "could not release the measurement in the background");
            }
        }
        self.reset_display(config);
    }

    pub fn toggle_freeze(&mut self) {
        self.frozen = !self.frozen;
    }

    pub fn send(&self, command: EngineCommand) {
        if let Some(running) = self.running() {
            running.engine.send(command);
        }
    }

    pub fn refresh(&mut self) -> bool {
        let now = Instant::now();
        let elapsed_s = now.duration_since(self.last_meter_update).as_secs_f32();
        self.last_meter_update = now;
        if let Lifecycle::Starting(receiver) = &self.lifecycle {
            match receiver.try_recv() {
                Ok(Ok(running)) => self.lifecycle = Lifecycle::Running(Box::new(running)),
                Ok(Err(error)) => {
                    self.lifecycle = Lifecycle::Idle;
                    self.last_error = Some(error);
                }
                Err(TryRecvError::Disconnected) => self.lifecycle = Lifecycle::Idle,
                Err(TryRecvError::Empty) => {}
            }
        }
        let Lifecycle::Running(running) = &mut self.lifecycle else {
            return false;
        };
        let reference = running.meters.reference.take_snapshot();
        let measurement = running.meters.measurement.take_snapshot();
        self.reference_silent_frames = reference.consecutive_silent_frames;
        self.measurement_silent_frames = measurement.consecutive_silent_frames;
        self.reference_level.update(reference, elapsed_s);
        self.measurement_level.update(measurement, elapsed_s);
        if self.frozen || !running.engine.has_new_snapshot() {
            return false;
        }
        self.display.clone_from(running.engine.latest_snapshot());
        true
    }

    pub fn meter_readings(&self) -> (MeterReading, MeterReading) {
        (
            self.reference_level.reading(),
            self.measurement_level.reading(),
        )
    }

    pub fn input_alert(&self) -> Option<InputAlert> {
        let running = self.running()?;
        if let Some(health) = &running.health {
            match health.fault() {
                StreamFault::DeviceDisconnected | StreamFault::StreamInvalidated => {
                    return Some(InputAlert::DeviceDisconnected);
                }
                StreamFault::PermissionDenied => return Some(InputAlert::PermissionDenied),
                StreamFault::None | StreamFault::DeviceBusy | StreamFault::Other => {}
            }
        }
        let silent_limit = u64::from(self.display.sample_rate_hz) * SILENT_INPUT_ALERT_SECONDS;
        if running.health.is_some() && self.measurement_silent_frames > silent_limit {
            return Some(InputAlert::SilentMeasurementInput);
        }
        if self.reference_silent_frames > silent_limit {
            return Some(InputAlert::SilentReference);
        }
        None
    }
}

fn open_measurement(
    source: &SourceSpec,
    config: &AppConfig,
    grid: Arc<DisplayGrid>,
) -> Result<RunningMeasurement, String> {
    match source {
        SourceSpec::Live => {
            let mut session =
                LiveSession::open(&config.stream).map_err(|error| text::audio_error(&error))?;
            let route = session.route().clone();
            let adjusted_sample_rate_hz = (route.sample_rate_hz != config.stream.sample_rate_hz)
                .then_some(route.sample_rate_hz);
            let engine_config = config.engine_config(&route);
            let frames = session
                .take_frame_source()
                .ok_or_else(|| text::ERROR_SOURCE_UNAVAILABLE.to_owned())?;
            let source_label =
                text::source_label(session.input_device_name(), session.sample_rate_hz());
            let meters = Arc::clone(session.meters());
            let health = Some(Arc::clone(session.health()));
            let generator = Some(Arc::clone(session.generator()));
            let uses_separate_clocks = session.uses_separate_clocks();
            let engine = EngineHandle::spawn(Box::new(frames), engine_config, grid)
                .map_err(|error| error.to_string())?;
            Ok(RunningMeasurement {
                engine,
                _live_session: Some(session),
                meters,
                health,
                generator,
                source_label,
                uses_separate_clocks,
                route,
                adjusted_sample_rate_hz,
            })
        }
        SourceSpec::File { path, pacing } => {
            let route = config.configured_route();
            let engine_config = config.engine_config(&route);
            let meters = Arc::new(MeterBank::default());
            let file_name = path.file_name().map_or_else(
                || path.display().to_string(),
                |name| name.to_string_lossy().into_owned(),
            );
            let (source, sample_rate_hz): (Box<dyn FrameSource>, u32) = match pacing {
                FilePacing::RealTime => {
                    let playback = FilePlayback::start(path, Arc::clone(&meters))
                        .map_err(|error| error.to_string())?;
                    let rate = playback.sample_rate_hz();
                    (Box::new(playback), rate)
                }
                FilePacing::AsFastAsPossible => {
                    let recording = Arc::new(
                        StereoRecording::read_wav(path).map_err(|error| error.to_string())?,
                    );
                    let rate = recording.sample_rate_hz;
                    (
                        Box::new(MeteredRecording {
                            cursor: RecordingCursor::new(recording, true),
                            meters: Arc::clone(&meters),
                        }),
                        rate,
                    )
                }
            };
            let engine = EngineHandle::spawn(source, engine_config, grid)
                .map_err(|error| error.to_string())?;
            Ok(RunningMeasurement {
                engine,
                _live_session: None,
                meters,
                health: None,
                generator: None,
                source_label: text::file_source_label(&file_name, sample_rate_hz),
                uses_separate_clocks: false,
                route,
                adjusted_sample_rate_hz: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meter_peak_releases_smoothly() {
        let mut ballistics = Ballistics::default();
        let loud = MeterSnapshot {
            peak_dbfs: -6.0,
            rms_dbfs: -12.0,
            clipped: false,
            consecutive_silent_frames: 0,
        };
        let quiet = MeterSnapshot {
            peak_dbfs: -40.0,
            ..loud
        };
        ballistics.update(loud, 0.016);
        ballistics.update(quiet, 0.5);
        assert!((ballistics.peak_dbfs - (-6.0 - 12.0)).abs() < 1.0e-3);
        assert_eq!(ballistics.rms_dbfs, -12.0);
    }

    #[test]
    fn idle_controller_has_no_alert_and_an_empty_snapshot() {
        let controller = MeasurementController::new(SourceSpec::Live, &AppConfig::default());
        assert!(!controller.is_running());
        assert_eq!(controller.input_alert(), None);
        assert_eq!(controller.snapshot().sequence, 0);
        assert_eq!(
            controller.snapshot().magnitude_db.len(),
            controller.grid().len()
        );
    }
}
