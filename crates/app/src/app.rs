use std::time::{Duration, Instant};

use audio_io::{DeviceInfo, ReferenceMode, list_devices};
use dsp_core::distance_m_for_delay_ms;
use eframe::egui::{self, CornerRadius, Frame, Margin, Panel, Stroke};
use engine::EngineCommand;

use crate::actions::{Action, triggered_by_keyboard};
use crate::config::{AppConfig, ConfigStore};
use crate::design::tokens::{STORED_TRACE_COLOR_COUNT, size, spacing, stroke};
use crate::design::widgets::Tone;
use crate::design::{Theme, fonts, theme};
use crate::measurement::{MeasurementController, SourceSpec};
use crate::plot::{PlotView, RangePreset, TraceStyle};
use crate::session::{
    AverageError, CaptureError, LoadIssue, Session, SessionStore, SessionWriter, UndoHistory,
};
use crate::text;
use crate::ui::banners::{BannerInputs, collect_banners, show_banners};
use crate::ui::bottom_bar::{self, BottomBarModel};
use crate::ui::notifications::Notifications;
use crate::ui::palette::CommandPalette;
use crate::ui::plot_panel::{OverlayTrace, PlotInputs, PlotPanel};
use crate::ui::side_panel::{self, SideTab};
use crate::ui::top_bar::{self, DelayReadout, TopBarModel};
use crate::ui::traces_panel::{self, RenameState, TraceCommand};
use crate::ui::wizard::{Wizard, WizardAction};

const CONFIG_SAVE_DELAY: Duration = Duration::from_millis(800);
const SESSION_SAVE_DELAY: Duration = Duration::from_secs(2);
const EXPORT_FILE_PREFIX: &str = "soundtuning";
const IDLE_REPAINT_INTERVAL: Duration = Duration::from_millis(250);
const LOW_COHERENCE_BAND_HZ: (f64, f64) = (100.0, 10_000.0);
const MILLISECONDS_PER_SECOND: f64 = 1000.0;

pub fn top_bar_height() -> f32 {
    size::TOP_BAR_HEIGHT + 2.0 * spacing::SM
}

pub fn central_area_fraction(window_width: f32, window_height: f32, side_panel_open: bool) -> f32 {
    let side = if side_panel_open {
        size::SIDE_PANEL_WIDTH
    } else {
        0.0
    };
    let central_width = (window_width - side).max(0.0);
    let central_height = (window_height - top_bar_height() - size::BOTTOM_BAR_HEIGHT).max(0.0);
    central_width * central_height / (window_width * window_height).max(1.0)
}

#[derive(Clone, Debug, PartialEq)]
pub enum DeviceCatalog {
    System,
    Fixed(Vec<DeviceInfo>),
}

impl DeviceCatalog {
    fn devices(&self) -> Vec<DeviceInfo> {
        match self {
            Self::System => list_devices().unwrap_or_else(|error| {
                tracing::warn!(%error, "could not list audio devices");
                Vec::new()
            }),
            Self::Fixed(devices) => devices.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppOptions {
    pub source: SourceSpec,
    pub store: ConfigStore,
    pub session_store: SessionStore,
    pub export_directory: std::path::PathBuf,
    pub devices: DeviceCatalog,
    pub initial_config: Option<AppConfig>,
    pub start_measurement: bool,
}

impl AppOptions {
    pub fn live() -> Self {
        Self {
            source: SourceSpec::Live,
            store: ConfigStore::default_location(),
            session_store: SessionStore::default_location(),
            export_directory: default_export_directory(),
            devices: DeviceCatalog::System,
            initial_config: None,
            start_measurement: false,
        }
    }
}

pub struct SoundTuningApp {
    config: AppConfig,
    store: ConfigStore,
    config_dirty_since: Option<Instant>,
    measurement: MeasurementController,
    plot: PlotPanel,
    command_palette: CommandPalette,
    notifications: Notifications,
    wizard: Option<Wizard>,
    side_tab: SideTab,
    applied_theme: Option<Theme>,
    is_mac: bool,
    awaiting_delay_result: bool,
    last_notified_delay_sequence: u64,
    device_catalog: DeviceCatalog,
    low_coherence_cache: (u64, bool),
    session: Session,
    session_writer: SessionWriter,
    save_failure_notified: bool,
    history: UndoHistory,
    session_dirty_since: Option<Instant>,
    rename: RenameState,
    last_offset_edit: Option<u64>,
    export_directory: std::path::PathBuf,
}

impl SoundTuningApp {
    pub fn new(context: &egui::Context, options: AppOptions) -> Self {
        fonts::install(context);
        let config = options
            .initial_config
            .clone()
            .unwrap_or_else(|| options.store.load());
        let is_file_mode = matches!(options.source, SourceSpec::File { .. });
        let wizard = (!config.wizard_completed && !is_file_mode)
            .then(|| Wizard::new(options.devices.devices()));
        let measurement = MeasurementController::new(options.source.clone(), &config);
        let loaded = options.session_store.load();
        let mut app = Self {
            config,
            store: options.store,
            config_dirty_since: None,
            measurement,
            plot: PlotPanel::new(PlotView::default()),
            command_palette: CommandPalette::default(),
            notifications: Notifications::default(),
            wizard,
            side_tab: SideTab::default(),
            applied_theme: None,
            is_mac: cfg!(target_os = "macos"),
            awaiting_delay_result: false,
            last_notified_delay_sequence: 0,
            device_catalog: options.devices.clone(),
            low_coherence_cache: (0, false),
            session: loaded.session,
            session_writer: SessionWriter::spawn(options.session_store.clone()),
            save_failure_notified: false,
            history: UndoHistory::default(),
            session_dirty_since: None,
            rename: RenameState::default(),
            last_offset_edit: None,
            export_directory: options.export_directory.clone(),
        };
        app.apply_theme(context);
        if let Some(LoadIssue::Corrupt {
            restored_backup, ..
        }) = loaded.issue
        {
            app.notifications.push(
                if restored_backup {
                    text::NOTIFY_SESSION_CORRUPT_RESTORED
                } else {
                    text::NOTIFY_SESSION_CORRUPT_EMPTY
                },
                Tone::Warning,
                None,
            );
        }
        if options.start_measurement && app.wizard.is_none() {
            app.start_measurement();
        }
        app
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn measurement(&self) -> &MeasurementController {
        &self.measurement
    }

    pub fn measurement_mut(&mut self) -> &mut MeasurementController {
        &mut self.measurement
    }

    pub fn is_palette_open(&self) -> bool {
        self.command_palette.is_open()
    }

    pub fn is_wizard_visible(&self) -> bool {
        self.wizard.is_some()
    }

    pub fn notification_texts(&self) -> Vec<String> {
        self.notifications.texts().map(str::to_owned).collect()
    }

    pub fn plot_view(&self) -> PlotView {
        self.plot.view
    }

    pub fn side_tab(&self) -> SideTab {
        self.side_tab
    }

    pub fn set_side_tab(&mut self, tab: SideTab) {
        self.side_tab = tab;
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn apply_test_offset(&mut self, trace_id: u64, offset_db: f64) {
        self.apply_trace_command(TraceCommand::SetOffset(trace_id, offset_db));
        self.last_offset_edit = None;
    }

    fn edit_session(&mut self, edit: impl FnOnce(&mut Session)) {
        let before = self.session.clone();
        edit(&mut self.session);
        if self.session != before {
            self.history.record(before);
            self.session_dirty_since.get_or_insert_with(Instant::now);
        }
    }

    fn save_session_if_due(&mut self) {
        for result in self.session_writer.poll() {
            self.handle_save_result(result);
        }
        if self
            .session_dirty_since
            .is_some_and(|since| since.elapsed() >= SESSION_SAVE_DELAY)
        {
            self.session_dirty_since = None;
            if let Some(result) = self.session_writer.request(self.session.clone()) {
                self.handle_save_result(result);
            }
        }
    }

    fn handle_save_result(&mut self, result: Result<(), String>) {
        match result {
            Ok(()) => self.save_failure_notified = false,
            Err(error) => {
                tracing::warn!(%error, "could not save the session");
                self.session_dirty_since.get_or_insert_with(Instant::now);
                if !self.save_failure_notified {
                    self.save_failure_notified = true;
                    self.notifications
                        .push(text::NOTIFY_SESSION_SAVE_FAILED, Tone::Danger, None);
                }
            }
        }
    }

    pub fn is_session_save_pending(&self) -> bool {
        self.session_dirty_since.is_some()
    }

    fn capture_trace(&mut self) {
        let snapshot = self.measurement.snapshot().clone();
        if !self.measurement.is_running() || snapshot.averages == 0 {
            self.notifications
                .push(text::NOTIFY_NOTHING_TO_CAPTURE, Tone::Warning, None);
            return;
        }
        let now = chrono::Local::now();
        let clock = now.format("%H:%M").to_string();
        let smoothing = self.config.analysis.smoothing_fraction;
        let mut captured = Err(CaptureError::NoAverages);
        self.edit_session(|session| {
            captured = session.capture(&snapshot, smoothing, now.timestamp(), &clock);
        });
        match captured {
            Ok(id) => {
                if let Some(name) = self.session.trace(id).map(|trace| trace.name.clone()) {
                    self.notifications.push(
                        text::trace_captured(&name),
                        Tone::Success,
                        Some(Action::Undo),
                    );
                }
            }
            Err(CaptureError::NoAverages) => {
                self.notifications
                    .push(text::NOTIFY_NOTHING_TO_CAPTURE, Tone::Warning, None);
            }
            Err(CaptureError::TooManyTraces) => {
                self.notifications
                    .push(text::NOTIFY_TOO_MANY_TRACES, Tone::Warning, None);
            }
        }
    }

    fn apply_trace_command(&mut self, command: TraceCommand) {
        let is_offset_edit = matches!(command, TraceCommand::SetOffset(..));
        match command {
            TraceCommand::ToggleVisible(id) => self.edit_session(|session| {
                if let Some(trace) = session.trace_mut(id) {
                    trace.visible = !trace.visible;
                }
            }),
            TraceCommand::CycleColor(id) => self.edit_session(|session| {
                if let Some(trace) = session.trace_mut(id) {
                    trace.color_index = (trace.color_index + 1) % STORED_TRACE_COLOR_COUNT;
                }
            }),
            TraceCommand::SetOffset(id, offset_db) => {
                if self.last_offset_edit == Some(id) {
                    if let Some(trace) = self.session.trace_mut(id) {
                        trace.offset_db = offset_db;
                    }
                    self.session_dirty_since.get_or_insert_with(Instant::now);
                } else {
                    self.edit_session(|session| {
                        if let Some(trace) = session.trace_mut(id) {
                            trace.offset_db = offset_db;
                        }
                    });
                    self.last_offset_edit = Some(id);
                }
            }
            TraceCommand::ToggleSelected(id) => {
                if let Some(trace) = self.session.trace_mut(id) {
                    trace.selected = !trace.selected;
                    self.session_dirty_since.get_or_insert_with(Instant::now);
                }
            }
            TraceCommand::Rename(id, name) => self.edit_session(|session| {
                if let Some(trace) = session.trace_mut(id) {
                    trace.name = name;
                }
            }),
            TraceCommand::Delete(id) => {
                self.edit_session(|session| session.remove(id));
                self.notifications.push(
                    text::NOTIFY_TRACE_DELETED,
                    Tone::Neutral,
                    Some(Action::Undo),
                );
            }
            TraceCommand::AverageSelected => {
                let now = chrono::Local::now();
                let clock = now.format("%H:%M").to_string();
                let mut created = Err(AverageError::NeedTwoTraces);
                self.edit_session(|session| {
                    created = session.average_selected(now.timestamp(), &clock);
                });
                let message = match created {
                    Ok(_) => None,
                    Err(AverageError::NeedTwoTraces) => Some(text::NOTIFY_AVERAGE_NEEDS_TWO),
                    Err(AverageError::TooManyTraces) => Some(text::NOTIFY_TOO_MANY_TRACES),
                    Err(AverageError::IncompatibleTraces) => {
                        Some(text::NOTIFY_AVERAGE_INCOMPATIBLE)
                    }
                };
                if let Some(message) = message {
                    self.notifications.push(message, Tone::Warning, None);
                }
            }
            TraceCommand::ExportCsv => self.export_csv(),
        }
        if !is_offset_edit {
            self.last_offset_edit = None;
        }
    }

    fn export_csv(&mut self) {
        let csv = self
            .session
            .to_csv(self.measurement.grid().frequencies_hz());
        let stem = format!(
            "{EXPORT_FILE_PREFIX}-{}",
            chrono::Local::now().format("%Y%m%d-%H%M%S")
        );
        match write_export(&self.export_directory, &stem, "csv", &csv) {
            Ok(path) => self
                .notifications
                .push(text::exported_to(&path), Tone::Success, None),
            Err(error) => {
                tracing::warn!(%error, "export failed");
                self.notifications
                    .push(text::NOTIFY_EXPORT_FAILED, Tone::Danger, None);
            }
        }
    }

    fn mark_config_dirty(&mut self) {
        self.config_dirty_since.get_or_insert_with(Instant::now);
    }

    fn save_config_if_due(&mut self) {
        if self
            .config_dirty_since
            .is_some_and(|since| since.elapsed() >= CONFIG_SAVE_DELAY)
        {
            self.config_dirty_since = None;
            if let Err(error) = self.store.save(&self.config) {
                tracing::warn!(%error, "could not save the configuration");
            }
        }
    }

    fn apply_theme(&mut self, context: &egui::Context) {
        if self.applied_theme != Some(self.config.theme) {
            theme::apply(context, self.config.theme);
            self.applied_theme = Some(self.config.theme);
        }
    }

    fn start_measurement(&mut self) {
        self.awaiting_delay_result = false;
        self.last_notified_delay_sequence = 0;
        self.measurement.start(&self.config);
    }

    fn stop_measurement(&mut self) {
        self.awaiting_delay_result = false;
        self.last_notified_delay_sequence = 0;
        self.measurement.stop(&self.config);
    }

    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::ToggleMeasurement => {
                if self.measurement.is_running() {
                    self.stop_measurement();
                } else {
                    self.start_measurement();
                }
            }
            Action::RestartMeasurement => {
                self.stop_measurement();
                self.start_measurement();
            }
            Action::FindDelay => {
                self.measurement.send(EngineCommand::FindDelayNow);
                self.awaiting_delay_result = true;
            }
            Action::ApplyDelay => {
                if !self.measurement.is_running() {
                    return;
                }
                self.measurement.send(EngineCommand::ApplyLatestDelay);
                if let Some(latest) = self.measurement.snapshot().delay.latest_confident {
                    let acoustic = self
                        .measurement
                        .snapshot()
                        .delay
                        .acoustic_ms(latest.total_ms);
                    self.notifications.push(
                        format!(
                            "{} : {}",
                            text::NOTIFY_DELAY_APPLIED,
                            text::format_milliseconds(acoustic)
                        ),
                        Tone::Success,
                        None,
                    );
                }
            }
            Action::ToggleFreeze => {
                if self.measurement.is_running() {
                    self.measurement.toggle_freeze();
                }
            }
            Action::RangeFull => self.plot.view = self.plot.view.with_preset(RangePreset::Full),
            Action::RangeSub => self.plot.view = self.plot.view.with_preset(RangePreset::Sub),
            Action::RangeMid => self.plot.view = self.plot.view.with_preset(RangePreset::Mid),
            Action::RangeHigh => self.plot.view = self.plot.view.with_preset(RangePreset::High),
            Action::ResetView => self.plot.view = PlotView::default(),
            Action::ToggleTheme => {
                self.config.theme = self.config.theme.toggled();
                self.mark_config_dirty();
                self.notifications.push(
                    match self.config.theme {
                        Theme::Night => text::NOTIFY_THEME_NIGHT,
                        Theme::Daylight => text::NOTIFY_THEME_DAY,
                    },
                    Tone::Neutral,
                    None,
                );
            }
            Action::OpenPalette => self.command_palette.toggle(),
            Action::ToggleSidePanel => {
                self.config.side_panel_open = !self.config.side_panel_open;
                self.mark_config_dirty();
            }
            Action::ToggleGenerator => {
                self.config.stream.generator.enabled = !self.config.stream.generator.enabled;
                if let Some(generator) = self.measurement.generator() {
                    generator.set_enabled(self.config.stream.generator.enabled);
                }
                self.mark_config_dirty();
            }
            Action::CalibrateLatency => {
                self.measurement
                    .send(EngineCommand::StartLatencyCalibration);
            }
            Action::CaptureTrace => self.capture_trace(),
            Action::Undo => {
                let text = if self.history.undo(&mut self.session) {
                    self.session_dirty_since.get_or_insert_with(Instant::now);
                    text::NOTIFY_UNDONE
                } else {
                    text::NOTIFY_NOTHING_TO_UNDO
                };
                self.last_offset_edit = None;
                self.notifications.push(text, Tone::Neutral, None);
            }
            Action::Redo => {
                if self.history.redo(&mut self.session) {
                    self.session_dirty_since.get_or_insert_with(Instant::now);
                    self.notifications
                        .push(text::NOTIFY_REDONE, Tone::Neutral, None);
                }
                self.last_offset_edit = None;
            }
            Action::ExportSession => self.export_csv(),
            Action::AddEqFilter => {}
        }
    }

    fn process_engine_events(&mut self) {
        let delay = self.measurement.snapshot().delay;
        let from_current_engine = self.measurement.snapshot().sequence > 0;
        if let Some(latency_ms) = delay.system_latency_ms.filter(|_| from_current_engine) {
            let key = self.config.latency_key();
            let stored = self.config.system_latency_ms.get(&key).copied();
            if self.config.uses_internal_reference()
                && stored.is_none_or(|stored| (stored - latency_ms).abs() > f64::EPSILON)
            {
                self.config.system_latency_ms.insert(key, latency_ms);
                self.mark_config_dirty();
                self.notifications
                    .push(text::latency_calibrated(latency_ms), Tone::Success, None);
            }
        }
        if let Some(confident) = delay.latest_confident
            && confident.measured_at_sequence != self.last_notified_delay_sequence
            && self.awaiting_delay_result
        {
            self.last_notified_delay_sequence = confident.measured_at_sequence;
            self.awaiting_delay_result = false;
            self.notifications.push(
                text::delay_found(delay.acoustic_ms(confident.total_ms)),
                Tone::Accent,
                Some(Action::ApplyDelay),
            );
        }
    }

    fn delay_readout(&self) -> DelayReadout {
        let snapshot = self.measurement.snapshot();
        let delay = snapshot.delay;
        let has_delay = delay.latest_confident.is_some() || delay.applied_ms.abs() > f64::EPSILON;
        if !has_delay {
            return DelayReadout::default();
        }
        let acoustic_ms = delay.applied_acoustic_ms();
        DelayReadout {
            milliseconds: Some(acoustic_ms),
            distance_m: Some(distance_m_for_delay_ms(
                acoustic_ms,
                self.config.air_temperature_celsius,
            )),
            samples: Some(
                acoustic_ms / MILLISECONDS_PER_SECOND * f64::from(snapshot.sample_rate_hz),
            ),
            confidence_db: delay.latest.map(|latest| latest.confidence_db),
            approximate: self.is_live_internal() && self.config.calibrated_latency_ms().is_none(),
        }
    }

    fn is_live_internal(&self) -> bool {
        !self.measurement.is_file_mode() && self.config.uses_internal_reference()
    }

    fn status(&self) -> (Tone, &'static str) {
        let snapshot = self.measurement.snapshot();
        if self.measurement.last_error().is_some() {
            (Tone::Danger, text::STATUS_ERROR)
        } else if !self.measurement.is_running() {
            (Tone::Neutral, text::STATUS_STOPPED)
        } else if self.measurement.is_frozen() {
            (Tone::Warning, text::STATUS_FROZEN)
        } else if snapshot.source_finished {
            (Tone::Neutral, text::STATUS_FILE_FINISHED)
        } else if snapshot.warming_up {
            (Tone::Accent, text::STATUS_WARMING_UP)
        } else {
            (Tone::Success, text::STATUS_RUNNING)
        }
    }

    fn update_low_coherence(&mut self) {
        let sequence = self.measurement.snapshot().sequence;
        if self.low_coherence_cache.0 != sequence {
            self.low_coherence_cache = (sequence, self.compute_low_coherence());
        }
    }

    fn compute_low_coherence(&self) -> bool {
        let snapshot = self.measurement.snapshot();
        if snapshot.warming_up || !self.measurement.is_running() {
            return false;
        }
        let grid = self.measurement.grid();
        let mut values: Vec<f64> = grid
            .frequencies_hz()
            .iter()
            .zip(&snapshot.coherence)
            .filter(|(frequency_hz, _)| {
                (LOW_COHERENCE_BAND_HZ.0..=LOW_COHERENCE_BAND_HZ.1).contains(*frequency_hz)
            })
            .filter_map(|(_, value)| *value)
            .collect();
        if values.is_empty() {
            return false;
        }
        values.sort_by(f64::total_cmp);
        let median = values[values.len() / 2];
        median < snapshot.settings.coherence_threshold
    }

    fn banner_inputs(&self) -> BannerInputs {
        let snapshot = self.measurement.snapshot();
        let (reference, measurement) = self.measurement.meter_readings();
        BannerInputs {
            running: self.measurement.is_running(),
            start_error: self.measurement.last_error().map(str::to_owned),
            alert: self.measurement.input_alert(),
            live_internal_reference: self.is_live_internal(),
            latency_calibrated: self.config.calibrated_latency_ms().is_some()
                || snapshot.delay.system_latency_ms.is_some(),
            separate_clocks: self.measurement.uses_separate_clocks(),
            low_coherence: self.low_coherence_cache.1 && self.measurement.is_running(),
            clipping: reference.clipped || measurement.clipped,
            calibrating: snapshot.delay.calibrating_latency,
            calibration_failed: snapshot.delay.calibration_failed,
            file_finished: snapshot.source_finished,
        }
    }

    fn reference_label(&self) -> String {
        match self.config.stream.reference {
            ReferenceMode::Internal => text::REFERENCE_INTERNAL.to_owned(),
            ReferenceMode::Loopback {
                reference_channel_index,
            } => text::format_input_channel(reference_channel_index),
        }
    }

    fn show_wizard(&mut self, ui: &mut egui::Ui) {
        let palette = self.config.theme.palette();
        let meters = self
            .measurement
            .is_running()
            .then(|| self.measurement.meter_readings());
        let error = self.measurement.last_error().map(str::to_owned);
        let Some(wizard) = self.wizard.as_mut() else {
            return;
        };
        let mut config = self.config.clone();
        let action = egui::CentralPanel::default()
            .frame(Frame::new().fill(palette.background))
            .show(ui, |ui| {
                wizard.show(ui, palette, &mut config, meters, error.as_deref())
            })
            .inner;
        if config != self.config {
            self.config = config;
            self.mark_config_dirty();
        }
        match action {
            WizardAction::None => {}
            WizardAction::RestartPreview => {
                let mut preview = self.config.clone();
                preview.stream.generator.enabled = false;
                self.measurement.stop(&self.config);
                self.measurement.start(&preview);
            }
            WizardAction::StartTest => {
                self.config.stream.generator.enabled = true;
                self.mark_config_dirty();
                self.stop_measurement();
                self.start_measurement();
            }
            WizardAction::RefreshDevices => {
                if let Some(wizard) = self.wizard.as_mut() {
                    wizard.devices = self.device_catalog.devices();
                }
            }
            WizardAction::StopPreview => self.stop_measurement(),
            WizardAction::Finish | WizardAction::Skip => {
                self.wizard = None;
                self.config.wizard_completed = true;
                self.mark_config_dirty();
                if action == WizardAction::Finish {
                    self.stop_measurement();
                    self.start_measurement();
                } else {
                    self.stop_measurement();
                }
            }
        }
    }
}

impl eframe::App for SoundTuningApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let context = ui.ctx().clone();
        self.apply_theme(&context);
        let palette = self.config.theme.palette();
        self.measurement.refresh();
        self.update_low_coherence();
        self.process_engine_events();

        let mut actions = if self.wizard.is_none() && !self.command_palette.is_open() {
            triggered_by_keyboard(&context)
        } else {
            Vec::new()
        };
        if self.command_palette.is_open()
            && context.input_mut(|input| {
                Action::OpenPalette
                    .shortcut()
                    .is_some_and(|shortcut| input.consume_shortcut(&shortcut))
            })
        {
            self.command_palette.close();
        }

        if self.wizard.is_some() {
            self.show_wizard(ui);
        } else {
            self.show_main(ui, palette, &mut actions);
        }

        self.notifications
            .show(&context, palette, self.is_mac, &mut actions);
        self.command_palette
            .show(&context, palette, self.is_mac, &mut actions);
        for action in actions {
            self.handle_action(action);
        }
        self.save_config_if_due();
        self.save_session_if_due();
        if self.measurement.is_running() {
            context.request_repaint();
        } else {
            context.request_repaint_after(IDLE_REPAINT_INTERVAL);
        }
    }

    fn on_exit(&mut self) {
        if let Err(error) = self.store.save(&self.config) {
            tracing::warn!(%error, "could not save the configuration on exit");
        }
        if let Err(error) = self.session_writer.finish(&self.session) {
            tracing::warn!(%error, "could not save the session on exit");
        }
    }
}

impl SoundTuningApp {
    fn show_main(
        &mut self,
        ui: &mut egui::Ui,
        palette: &'static crate::design::Palette,
        actions: &mut Vec<Action>,
    ) {
        let bar_frame = |fill| {
            Frame::new()
                .fill(fill)
                .inner_margin(Margin::symmetric(spacing::LG as i8, spacing::SM as i8))
                .stroke(Stroke::new(stroke::BORDER, palette.border))
        };
        let (reference_meter, measurement_meter) = self.measurement.meter_readings();
        let (status_tone, status_text) = self.status();
        let top_model = TopBarModel {
            status_tone,
            status_text,
            source_label: self.measurement.source_label().map(str::to_owned),
            reference_meter,
            measurement_meter,
            delay: self.delay_readout(),
            can_apply_delay: self.measurement.snapshot().delay.latest_confident.is_some(),
            running: self.measurement.is_running(),
        };
        Panel::top("top-bar")
            .exact_size(top_bar_height())
            .frame(bar_frame(palette.surface))
            .show(ui, |ui| {
                top_bar::show(ui, palette, &top_model, self.is_mac, actions)
            });

        let bottom_model = BottomBarModel {
            running: self.measurement.is_running(),
            frozen: self.measurement.is_frozen(),
            reference_label: self.reference_label(),
            generator_level_dbfs: (self.is_live_internal() || self.config.stream.generator.enabled)
                .then_some(self.config.stream.generator.level_dbfs)
                .filter(|_| !self.measurement.is_file_mode()),
        };
        let mut analysis = self.config.analysis.clone();
        let changes = Panel::bottom("bottom-bar")
            .exact_size(size::BOTTOM_BAR_HEIGHT)
            .frame(bar_frame(palette.surface))
            .show(ui, |ui| {
                bottom_bar::show(
                    ui,
                    palette,
                    &bottom_model,
                    &mut analysis,
                    self.is_mac,
                    actions,
                )
            })
            .inner;
        if changes.analysis_changed {
            self.config.analysis = analysis;
            self.measurement.send(EngineCommand::UpdateSettings(
                self.config.analysis.to_settings(),
            ));
            self.mark_config_dirty();
        }
        if let Some(level) = changes.generator_level_dbfs {
            self.config.stream.generator.level_dbfs = level;
            if let Some(generator) = self.measurement.generator() {
                generator.set_level_dbfs(level);
            }
            self.mark_config_dirty();
        }

        let mut trace_commands = Vec::new();
        if self.config.side_panel_open {
            Panel::right("side-panel")
                .exact_size(size::SIDE_PANEL_WIDTH)
                .frame(
                    Frame::new()
                        .fill(palette.surface)
                        .inner_margin(Margin::same(spacing::LG as i8))
                        .stroke(Stroke::new(stroke::BORDER, palette.border)),
                )
                .show(ui, |ui| {
                    side_panel::tab_bar(ui, palette, &mut self.side_tab);
                    match self.side_tab {
                        SideTab::Traces if !self.session.traces.is_empty() => {
                            trace_commands =
                                traces_panel::show(ui, palette, &self.session, &mut self.rename);
                        }
                        SideTab::Traces => side_panel::empty_state(
                            ui,
                            palette,
                            text::EMPTY_TRACES_TITLE,
                            text::EMPTY_TRACES_BODY,
                        ),
                        SideTab::Alignment => side_panel::empty_state(
                            ui,
                            palette,
                            text::EMPTY_ALIGNMENT_TITLE,
                            text::EMPTY_ALIGNMENT_BODY,
                        ),
                        SideTab::Eq => side_panel::empty_state(
                            ui,
                            palette,
                            text::EMPTY_EQ_TITLE,
                            text::EMPTY_EQ_BODY,
                        ),
                    }
                });
        }

        for command in trace_commands {
            self.apply_trace_command(command);
        }

        let banners = collect_banners(&self.banner_inputs());
        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(palette.background)
                    .inner_margin(Margin::same(spacing::LG as i8))
                    .corner_radius(CornerRadius::ZERO),
            )
            .show(ui, |ui| {
                let grid = std::sync::Arc::clone(self.measurement.grid());
                let overlays: Vec<OverlayTrace<'_>> = self
                    .session
                    .traces
                    .iter()
                    .filter(|trace| trace.visible)
                    .map(|trace| OverlayTrace {
                        label: &trace.name,
                        values_db: &trace.magnitude_db,
                        offset_db: trace.offset_db,
                        style: TraceStyle {
                            color: palette.stored_traces
                                [trace.color_index % STORED_TRACE_COLOR_COUNT],
                            width: stroke::TRACE_STORED,
                            halo: None,
                        },
                    })
                    .collect();
                let inputs = PlotInputs {
                    frequencies_hz: grid.frequencies_hz(),
                    snapshot: self.measurement.snapshot(),
                    show_live: self.measurement.snapshot().averages > 0,
                    running: self.measurement.is_running(),
                    overlays: &overlays,
                };
                let events = self.plot.show(ui, palette, &inputs, self.is_mac, actions);
                if events.double_click_hz_db.is_some() {
                    actions.push(Action::ResetView);
                }
                if let (false, Some(plot_rect)) = (banners.is_empty(), events.plot_rect) {
                    egui::Area::new(egui::Id::unique("state-banners"))
                        .order(egui::Order::Middle)
                        .fixed_pos(plot_rect.left_top() + egui::vec2(spacing::SM, spacing::SM))
                        .show(ui.ctx(), |ui| {
                            ui.set_width(plot_rect.width() - 2.0 * spacing::SM);
                            show_banners(ui, palette, &banners, actions);
                        });
                }
            });
    }
}

pub fn default_export_directory() -> std::path::PathBuf {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    home.join("Documents").join(text::EXPORT_DIRECTORY_NAME)
}

pub fn write_export(
    directory: &std::path::Path,
    stem: &str,
    extension: &str,
    contents: &str,
) -> anyhow::Result<String> {
    use std::io::Write;
    std::fs::create_dir_all(directory)?;
    let mut suffix = 1_u32;
    loop {
        let file_name = if suffix == 1 {
            format!("{stem}.{extension}")
        } else {
            format!("{stem}-{suffix}.{extension}")
        };
        let path = directory.join(file_name);
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                file.write_all(contents.as_bytes())?;
                file.sync_all()?;
                return Ok(path.display().to_string());
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => suffix += 1,
            Err(error) => return Err(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn central_zone_covers_at_least_seventy_percent_with_the_panel_open() {
        for (width, height) in [(1440.0, 900.0), (1920.0, 1080.0)] {
            let fraction = central_area_fraction(width, height, true);
            assert!(fraction >= 0.70, "{width}x{height}: {fraction}");
        }
        assert!(central_area_fraction(1440.0, 900.0, false) > 0.8);
    }
}
