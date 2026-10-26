use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use audio_io::{
    FilePlayback, FrameSource, LiveFrameSource, LiveSession, MeterBank, MeterSnapshot, StereoFrame,
    StreamFault, StreamSettings,
};
use eframe::egui;

use crate::text;

const REPAINT_INTERVAL: Duration = Duration::from_millis(16);
const METER_FLOOR_DBFS: f32 = -60.0;
const METER_RELEASE_DB_PER_FRAME: f32 = 1.5;
const SILENT_INPUT_ALERT_SECONDS: u64 = 2;
const DRAIN_CHUNK_FRAMES: usize = 8192;

pub fn run_window(input_file: Option<PathBuf>) -> anyhow::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        text::APP_NAME,
        options,
        Box::new(move |_creation_context| Ok(Box::new(MeterWindow::new(input_file)))),
    )
    .map_err(|error| anyhow::anyhow!("window failed: {error}"))
}

enum ActiveSource {
    Stopped,
    Live {
        session: Box<LiveSession>,
        frames: LiveFrameSource,
    },
    File {
        playback: FilePlayback,
        meters: Arc<MeterBank>,
    },
}

#[derive(Clone, Copy, Debug, Default)]
struct DisplayedLevel {
    peak_dbfs: f32,
    clipped: bool,
}

impl DisplayedLevel {
    fn update(&mut self, snapshot: MeterSnapshot) {
        let released = self.peak_dbfs - METER_RELEASE_DB_PER_FRAME;
        self.peak_dbfs = snapshot.peak_dbfs.max(released).max(METER_FLOOR_DBFS);
        self.clipped = snapshot.clipped;
    }
}

pub struct MeterWindow {
    input_file: Option<PathBuf>,
    settings: StreamSettings,
    source: ActiveSource,
    reference_level: DisplayedLevel,
    measurement_level: DisplayedLevel,
    status_message: Option<String>,
    drained_frames: Vec<StereoFrame>,
}

impl MeterWindow {
    pub fn new(input_file: Option<PathBuf>) -> Self {
        let mut window = Self {
            input_file,
            settings: StreamSettings::default(),
            source: ActiveSource::Stopped,
            reference_level: DisplayedLevel {
                peak_dbfs: METER_FLOOR_DBFS,
                clipped: false,
            },
            measurement_level: DisplayedLevel {
                peak_dbfs: METER_FLOOR_DBFS,
                clipped: false,
            },
            status_message: None,
            drained_frames: Vec::with_capacity(DRAIN_CHUNK_FRAMES),
        };
        if window.input_file.is_some() {
            window.start();
        }
        window
    }

    fn is_running(&self) -> bool {
        !matches!(self.source, ActiveSource::Stopped)
    }

    fn start(&mut self) {
        self.status_message = None;
        let result = match &self.input_file {
            Some(path) => {
                let meters = Arc::new(MeterBank::default());
                FilePlayback::start(path, Arc::clone(&meters))
                    .map(|playback| ActiveSource::File { playback, meters })
            }
            None => LiveSession::open(&self.settings).and_then(|mut session| {
                session
                    .take_frame_source()
                    .map(|frames| ActiveSource::Live {
                        session: Box::new(session),
                        frames,
                    })
                    .ok_or(audio_io::AudioIoError::MissingGeneratorOutput)
            }),
        };
        match result {
            Ok(source) => self.source = source,
            Err(error) => {
                tracing::error!(%error, "could not start measurement source");
                self.status_message = Some(text::audio_error(&error.to_string()));
            }
        }
    }

    fn stop(&mut self) {
        self.source = ActiveSource::Stopped;
    }

    fn drain_frames(&mut self) {
        let frame_source: Option<&mut dyn FrameSource> = match &mut self.source {
            ActiveSource::Stopped => None,
            ActiveSource::Live { frames, .. } => Some(frames),
            ActiveSource::File { playback, .. } => Some(playback),
        };
        if let Some(frame_source) = frame_source {
            loop {
                self.drained_frames.clear();
                if frame_source
                    .read_frames(&mut self.drained_frames, DRAIN_CHUNK_FRAMES)
                    .frames_read
                    == 0
                {
                    break;
                }
            }
        }
    }

    fn meters(&self) -> Option<&Arc<MeterBank>> {
        match &self.source {
            ActiveSource::Stopped => None,
            ActiveSource::Live { session, .. } => Some(session.meters()),
            ActiveSource::File { meters, .. } => Some(meters),
        }
    }

    fn status_line(&self, measurement_snapshot: Option<MeterSnapshot>) -> String {
        if let Some(message) = &self.status_message {
            return message.clone();
        }
        match &self.source {
            ActiveSource::Stopped => text::STATUS_STOPPED.to_owned(),
            ActiveSource::File { playback, .. } => format!(
                "{} — {}",
                text::STATUS_RUNNING_FILE,
                text::source_label(
                    &playback.source_path().display().to_string(),
                    playback.sample_rate_hz()
                )
            ),
            ActiveSource::Live { session, .. } => match session.health().fault() {
                StreamFault::DeviceDisconnected | StreamFault::StreamInvalidated => {
                    text::STATUS_DEVICE_DISCONNECTED.to_owned()
                }
                StreamFault::PermissionDenied => text::STATUS_PERMISSION_DENIED.to_owned(),
                StreamFault::None | StreamFault::DeviceBusy | StreamFault::Other => {
                    let silent_limit =
                        u64::from(session.sample_rate_hz()) * SILENT_INPUT_ALERT_SECONDS;
                    if measurement_snapshot
                        .is_some_and(|snapshot| snapshot.consecutive_silent_frames > silent_limit)
                    {
                        text::STATUS_SILENT_INPUT.to_owned()
                    } else {
                        format!(
                            "{} — {}",
                            text::STATUS_RUNNING_LIVE,
                            text::source_label(
                                session.input_device_name(),
                                session.sample_rate_hz()
                            )
                        )
                    }
                }
            },
        }
    }
}

fn draw_meter(ui: &mut egui::Ui, label: &str, level: DisplayedLevel) {
    ui.horizontal(|ui| {
        ui.label(label);
        let desired_size = egui::vec2(ui.available_width() - 160.0, 20.0);
        let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
        let fraction = ((level.peak_dbfs - METER_FLOOR_DBFS) / -METER_FLOOR_DBFS).clamp(0.0, 1.0);
        let painter = ui.painter();
        painter.rect_filled(rect, 2.0, egui::Color32::from_gray(40));
        let mut filled = rect;
        filled.set_width(rect.width() * fraction);
        painter.rect_filled(filled, 2.0, egui::Color32::from_rgb(0, 158, 115));
        ui.monospace(text::format_dbfs(level.peak_dbfs));
        if level.clipped {
            ui.colored_label(egui::Color32::from_rgb(213, 94, 0), text::METER_CLIP);
        }
    });
}

impl eframe::App for MeterWindow {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.drain_frames();
        let snapshots = self.meters().map(|meters| {
            (
                meters.reference.take_snapshot(),
                meters.measurement.take_snapshot(),
            )
        });
        if let Some((reference, measurement)) = snapshots {
            self.reference_level.update(reference);
            self.measurement_level.update(measurement);
        }
        let status = self.status_line(snapshots.map(|(_, measurement)| measurement));

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading(text::APP_NAME);
            ui.label(status);
            ui.add_space(12.0);
            draw_meter(ui, text::METER_REFERENCE, self.reference_level);
            draw_meter(ui, text::METER_MEASUREMENT, self.measurement_level);
            ui.add_space(12.0);
            let button_label = if self.is_running() {
                text::BUTTON_STOP
            } else {
                text::BUTTON_START
            };
            if ui.button(button_label).clicked() {
                if self.is_running() {
                    self.stop();
                } else {
                    self.start();
                }
            }
        });
        ui.ctx().request_repaint_after(REPAINT_INTERVAL);
    }
}
