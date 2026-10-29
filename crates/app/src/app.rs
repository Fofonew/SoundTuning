use std::time::{Duration, Instant};

use audio_io::{AudioRoute, DeviceInfo, ReferenceMode, list_devices};
use dsp_core::distance_m_for_delay_ms;
use eframe::egui::{self, CornerRadius, Frame, Margin, Panel, Stroke};
use engine::EngineCommand;

use crate::actions::{Action, triggered_by_keyboard};
use crate::config::{AppConfig, ConfigStore};
use crate::design::tokens::{STORED_TRACE_COLOR_COUNT, size, spacing, stroke};
use crate::design::widgets::Tone;
use crate::design::{Theme, fonts, theme};
use crate::device_choice;
use crate::eq_state::{EqState, EqTarget, EqView, inverted_db, predicted_db};
use crate::measurement::{MeasurementController, SourceSpec};
use crate::plot::{
    ExternalModule, PlotLayout, PlotView, RangePreset, TraceStyle, ViewKind, upper_pane_height,
};
use crate::session::{
    AlignmentSumKey, AverageError, CaptureError, LoadIssue, Session, SessionStore, SessionWriter,
    UndoHistory,
};
use crate::text;
use crate::ui::alignment_panel::{self, AlignmentCommand};
use crate::ui::audio_panel::{self, AudioCommand, AudioSettingsPanel};
use crate::ui::banners::{BannerInputs, collect_banners, show_banners};
use crate::ui::control_panel::{
    self, ControlCommand, ControlModel, ControlPanelState, ControlTab, DelayReadout, LatencyStatus,
};
use crate::ui::display_panel;
use crate::ui::eq_panel::{self, EqCommand};
use crate::ui::graph_bar::{self, LegendItem, ViewCommand};
use crate::ui::impulse_panel::{ImpulseCurve, ImpulseInputs, ImpulsePanel};
use crate::ui::mouse_bar::{self, MouseBarModel};
use crate::ui::notifications::Notifications;
use crate::ui::palette::CommandPalette;
use crate::ui::pane_splitter::SplitArea;
use crate::ui::plot_panel::{
    BandSelection, OverlayTrace, PlotEvents, PlotInputs, PlotPanel, default_phase_view,
};
use crate::ui::status_bar::{self, StatusBarModel};
use crate::ui::traces_panel::TraceCommand;
use crate::ui::wizard::{Wizard, WizardAction};
use crate::update::{CheckTrigger, Release, UpdateChecker, UpdateSource, UpdateStatus, Version};

const CONFIG_SAVE_DELAY: Duration = Duration::from_millis(800);
const SESSION_SAVE_DELAY: Duration = Duration::from_secs(2);
const EXPORT_FILE_PREFIX: &str = "soundtuning";
const IDLE_REPAINT_INTERVAL: Duration = Duration::from_millis(250);
const LOW_COHERENCE_BAND_HZ: (f64, f64) = (100.0, 10_000.0);
const MILLISECONDS_PER_SECOND: f64 = 1000.0;
const DELAY_NUDGE_MS: f64 = 0.01;
const DELAY_DRIFT_NOTIFY_MS: f64 = 0.5;

type CachedSum = (Option<AlignmentSumKey>, Option<Vec<Option<f64>>>);
type ModuleSums = Vec<(crate::alignment_state::AlignmentChoice, Vec<Option<f64>>)>;
type CachedModuleSums = (
    (
        Option<AlignmentSumKey>,
        crate::alignment_state::AlignmentState,
    ),
    ModuleSums,
);
const MODULE_LEVEL_MARGIN_DB: f64 = 3.0;
const MODULE_MIN_SPAN_DB: f64 = 12.0;
const MODULE_PLOT_MIN_HEIGHT: f32 = 90.0;
const FALLBACK_CHANNEL_COUNT: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MergeKey {
    Offset(u64),
    Alignment,
    Eq,
}

type CachedEqCurve = (EqState, u64, Vec<Option<f64>>);
const FALLBACK_SAMPLE_RATE_HZ: u32 = 48_000;
const WINDOW_FIT_FRAMES: u32 = 30;
const MONITOR_FILL_LIMIT: f32 = 0.95;

pub fn window_exceeds_monitor(inner_size: egui::Vec2, monitor_size: egui::Vec2) -> bool {
    inner_size.x > monitor_size.x * MONITOR_FILL_LIMIT
        || inner_size.y > monitor_size.y * MONITOR_FILL_LIMIT
}

pub fn central_area_fraction(window_width: f32, window_height: f32, side_panel_open: bool) -> f32 {
    let side = if side_panel_open {
        size::CONTROL_PANEL_WIDTH
    } else {
        0.0
    };
    let central_width = (window_width - side).max(0.0);
    let central_height = (window_height - size::STATUS_BAR_HEIGHT).max(0.0);
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
    pub updates: UpdateSource,
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
            updates: UpdateSource::GitHub,
        }
    }
}

pub struct SoundTuningApp {
    config: AppConfig,
    store: ConfigStore,
    config_dirty_since: Option<Instant>,
    measurement: MeasurementController,
    plot: PlotPanel,
    phase_plot: PlotPanel,
    delay_plot: PlotPanel,
    delay_plot_default: PlotView,
    module_sums: Option<CachedModuleSums>,
    delay_undo: Option<f64>,
    impulse_plot: ImpulsePanel,
    phase_cache: Vec<(std::sync::Arc<crate::session::Trace>, Vec<Option<f64>>)>,
    plot_fullscreen: bool,
    viewport_fullscreen: bool,
    os_reported_fullscreen: bool,
    window_fit_frames_left: u32,
    command_palette: CommandPalette,
    notifications: Notifications,
    wizard: Option<Wizard>,
    control: ControlPanelState,
    applied_theme: Option<Theme>,
    is_mac: bool,
    awaiting_delay_result: bool,
    last_notified_delay_sequence: u64,
    device_catalog: DeviceCatalog,
    known_devices: Vec<DeviceInfo>,
    low_coherence_cache: (u64, bool),
    session: Session,
    session_writer: SessionWriter,
    save_failure_notified: bool,
    history: UndoHistory,
    session_dirty_since: Option<Instant>,
    last_merge: Option<MergeKey>,
    drift_notified: bool,
    alignment_sum: Option<CachedSum>,
    eq_curve: Option<CachedEqCurve>,
    last_plot_rect: Option<egui::Rect>,
    audio_settings: Option<AudioSettingsPanel>,
    export_directory: std::path::PathBuf,
    updates: UpdateChecker,
    available_update: Option<Release>,
    url_to_open: Option<String>,
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
            phase_plot: PlotPanel::phase(),
            delay_plot: PlotPanel::module(),
            delay_plot_default: PlotView::default(),
            module_sums: None,
            delay_undo: None,
            impulse_plot: ImpulsePanel::default(),
            phase_cache: Vec::new(),
            plot_fullscreen: false,
            viewport_fullscreen: false,
            os_reported_fullscreen: false,
            window_fit_frames_left: WINDOW_FIT_FRAMES,
            command_palette: CommandPalette::default(),
            notifications: Notifications::default(),
            wizard,
            control: ControlPanelState::default(),
            applied_theme: None,
            is_mac: cfg!(target_os = "macos"),
            awaiting_delay_result: false,
            last_notified_delay_sequence: 0,
            device_catalog: options.devices.clone(),
            known_devices: Vec::new(),
            low_coherence_cache: (0, false),
            session: loaded.session,
            session_writer: SessionWriter::spawn(options.session_store.clone()),
            save_failure_notified: false,
            history: UndoHistory::default(),
            session_dirty_since: None,
            last_merge: None,
            drift_notified: false,
            alignment_sum: None,
            eq_curve: None,
            last_plot_rect: None,
            audio_settings: None,
            export_directory: options.export_directory.clone(),
            updates: UpdateChecker::new(options.updates.clone(), Version::current()),
            available_update: None,
            url_to_open: None,
        };
        app.updates.start(CheckTrigger::Startup);
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

    pub fn available_update(&self) -> Option<&Release> {
        self.available_update.as_ref()
    }

    pub fn notification_texts(&self) -> Vec<String> {
        self.notifications.texts().map(str::to_owned).collect()
    }

    pub fn plot_view(&self) -> PlotView {
        self.plot.view
    }

    pub fn module(&self) -> Option<ExternalModule> {
        self.config
            .plot_layout
            .shows(ViewKind::VirtualEq)
            .then_some(self.config.plot_layout.module)
    }

    pub fn show_module(&mut self, module: ExternalModule) {
        self.config.plot_layout.show_module(module);
        self.mark_config_dirty();
    }

    pub fn control_tab(&self) -> ControlTab {
        self.control.tab
    }

    pub fn set_control_tab(&mut self, tab: ControlTab) {
        self.control.tab = tab;
    }

    pub fn is_audio_settings_open(&self) -> bool {
        self.audio_settings.is_some()
    }

    pub fn audio_settings_mut(&mut self) -> Option<&mut AudioSettingsPanel> {
        self.audio_settings.as_mut()
    }

    pub fn apply_test_audio(&mut self, command: AudioCommand) {
        self.apply_audio_command(command);
    }

    fn apply_audio_command(&mut self, command: AudioCommand) {
        match command {
            AudioCommand::Apply(stream) => {
                self.config.stream = stream;
                self.mark_config_dirty();
                if self.measurement.is_running() {
                    self.stop_measurement();
                    self.start_measurement();
                }
                self.notifications
                    .push(text::NOTIFY_AUDIO_APPLIED, Tone::Success, None);
            }
            AudioCommand::RefreshDevices => {
                let devices = self.available_devices();
                if let Some(panel) = self.audio_settings.as_mut() {
                    panel.replace_devices(devices);
                }
            }
            AudioCommand::Close => {
                self.audio_settings = None;
                self.control.tab = ControlTab::Measure;
            }
        }
    }

    pub fn last_plot_rect(&self) -> Option<egui::Rect> {
        self.last_plot_rect
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn apply_test_offset(&mut self, trace_id: u64, offset_db: f64) {
        self.apply_trace_command(TraceCommand::SetOffset(trace_id, offset_db));
        self.last_merge = None;
    }

    pub fn apply_test_trace_command(&mut self, command: TraceCommand) {
        self.apply_trace_command(command);
    }

    pub fn apply_test_alignment(&mut self, command: AlignmentCommand) {
        self.apply_alignment_command(command);
        self.last_merge = None;
    }

    fn edit_session_merged(&mut self, key: MergeKey, edit: impl FnOnce(&mut Session)) {
        if self.last_merge == Some(key) {
            edit(&mut self.session);
            self.session_dirty_since.get_or_insert_with(Instant::now);
        } else if self.record_session_edit(edit) {
            self.last_merge = Some(key);
        }
    }

    fn end_merge_when_idle(&mut self, context: &egui::Context) {
        let pointer_down = context.input(|input| {
            input.pointer.any_down() || input.smooth_scroll_delta.y.abs() > f32::EPSILON
        });
        let focused = context.memory(|memory| memory.focused()).is_some();
        if !pointer_down && !focused {
            self.last_merge = None;
        }
    }

    fn apply_band_selection(&mut self, selection: BandSelection) {
        let frequencies = std::sync::Arc::clone(self.measurement.grid());
        self.edit_session_merged(MergeKey::Alignment, |session| {
            session
                .alignment
                .set_custom_band(selection.low_hz, selection.high_hz);
            if selection.finished {
                session.refresh_alignment_solutions(frequencies.frequencies_hz());
            }
        });
        if selection.finished {
            self.last_merge = None;
        }
    }

    pub fn apply_test_eq(&mut self, command: EqCommand) {
        self.apply_eq_command(command);
        self.last_merge = None;
    }

    fn apply_eq_command(&mut self, command: EqCommand) {
        match command {
            EqCommand::Update(state) => {
                self.edit_session_merged(MergeKey::Eq, |session| session.eq = state);
            }
            EqCommand::AddFilter => self.add_eq_filter(eq_panel::default_new_filter()),
            EqCommand::AutoEq => self.run_auto_eq(),
            EqCommand::ExportText => {
                let contents = self.session.eq.export_text();
                self.export_file("txt", &contents);
            }
            EqCommand::ExportJson => match self.session.eq.export_json() {
                Ok(contents) => self.export_file("json", &contents),
                Err(error) => {
                    tracing::warn!(%error, "EQ export failed");
                    self.notifications
                        .push(text::NOTIFY_EXPORT_FAILED, Tone::Danger, None);
                }
            },
        }
    }

    fn run_auto_eq(&mut self) {
        let Some(target_db) = self.eq_target_db() else {
            self.notifications
                .push(text::NOTIFY_AUTO_EQ_NO_TARGET, Tone::Warning, None);
            return;
        };
        let settings = self.session.eq.auto.to_dsp();
        let sample_rate_hz = f64::from(self.eq_sample_rate_hz());
        let result = dsp_core::auto_eq(
            &target_db,
            self.measurement.grid().frequencies_hz(),
            sample_rate_hz,
            &settings,
        );
        match result {
            Ok(Some(result)) => {
                self.edit_session(|session| session.eq.replace_with_auto(&result.filters));
                self.last_merge = None;
                self.notifications.push(
                    text::auto_eq_done(result.filters.len(), result.rms_error_db),
                    Tone::Success,
                    Some(Action::Undo),
                );
            }
            Ok(None) => {
                self.notifications
                    .push(text::NOTIFY_AUTO_EQ_NOT_ENOUGH, Tone::Warning, None)
            }
            Err(error) => {
                tracing::warn!(%error, "auto EQ failed");
                self.notifications
                    .push(text::NOTIFY_AUTO_EQ_NOT_ENOUGH, Tone::Warning, None);
            }
        }
    }

    fn add_eq_filter(&mut self, filter: crate::eq_state::StoredFilter) {
        let mut added = None;
        self.edit_session(|session| added = session.eq.add_filter(filter));
        self.last_merge = None;
        if added.is_none() {
            self.notifications
                .push(text::NOTIFY_EQ_FULL, Tone::Warning, None);
        }
    }

    fn apply_eq_plot_events(&mut self, events: &PlotEvents) {
        if let Some(drag) = events.eq_drag {
            if drag.started {
                self.last_merge = None;
            }
            self.edit_session_merged(MergeKey::Eq, |session| {
                session
                    .eq
                    .move_filter(drag.index, drag.frequency_hz, drag.gain_db);
            });
            if drag.finished {
                self.last_merge = None;
            }
        } else if let Some(selection) = events.eq_selected {
            self.session.eq.selected_filter = selection;
        }
        if let Some((index, factor)) = events.eq_q_factor {
            self.edit_session_merged(MergeKey::Eq, |session| session.eq.scale_q(index, factor));
        }
        if let Some((frequency_hz, gain_db)) = events.double_click_hz_db {
            self.add_eq_filter(eq_panel::new_filter_at(frequency_hz, gain_db));
        }
    }

    fn eq_sample_rate_hz(&self) -> u32 {
        let rate = match self.session.eq.target {
            EqTarget::Trace(id) => self
                .session
                .trace(id)
                .map(|trace| trace.metadata.sample_rate_hz),
            EqTarget::Live => Some(self.measurement.snapshot().sample_rate_hz),
        };
        rate.filter(|rate| *rate > 0)
            .unwrap_or(FALLBACK_SAMPLE_RATE_HZ)
    }

    fn eq_target_db(&self) -> Option<Vec<Option<f64>>> {
        match self.session.eq.target {
            EqTarget::Trace(id) => self.session.trace(id).map(|trace| trace.displayed_db()),
            EqTarget::Live => {
                let snapshot = self.measurement.snapshot();
                (snapshot.averages > 0).then(|| snapshot.magnitude_db.clone())
            }
        }
    }

    fn eq_overlays(
        &mut self,
        palette: &crate::design::Palette,
    ) -> Vec<(&'static str, Vec<Option<f64>>, TraceStyle)> {
        let sample_rate_hz = u64::from(self.eq_sample_rate_hz());
        let stale = self
            .eq_curve
            .as_ref()
            .is_none_or(|(state, rate, _)| *rate != sample_rate_hz || *state != self.session.eq);
        if stale {
            let curve = self.session.eq.response_db(
                self.measurement.grid().frequencies_hz(),
                sample_rate_hz as f64,
            );
            self.eq_curve = Some((self.session.eq.clone(), sample_rate_hz, curve));
        }
        let Some((_, _, curve)) = self.eq_curve.as_ref() else {
            return Vec::new();
        };
        let strong = |color| TraceStyle {
            color,
            width: stroke::TRACE_LIVE,
            halo: None,
        };
        let light = TraceStyle {
            color: palette.accent,
            width: stroke::TRACE_STORED,
            halo: None,
        };
        let target = self.eq_target_db();
        match (self.session.eq.view, target) {
            (EqView::Predicted, Some(target)) => vec![
                (text::EQ_CURVE_LABEL, curve.clone(), light),
                (
                    text::EQ_PREDICTED_LABEL,
                    predicted_db(&target, curve),
                    strong(palette.predicted),
                ),
            ],
            (EqView::InvertedTrace, Some(target)) => vec![
                (
                    text::EQ_INVERTED_LABEL,
                    inverted_db(&target, self.measurement.grid().frequencies_hz()),
                    strong(palette.text_muted),
                ),
                (
                    text::EQ_CURVE_LABEL,
                    curve.clone(),
                    strong(palette.predicted),
                ),
            ],
            _ => vec![(
                text::EQ_CURVE_LABEL,
                curve.clone(),
                strong(palette.predicted),
            )],
        }
    }

    fn refresh_alignment_sum(&mut self) {
        let key = self.session.alignment_sum_key();
        if self.alignment_sum.as_ref().map(|(cached, _)| *cached) == Some(key) {
            return;
        }
        let sum = self
            .session
            .predicted_alignment_sum_db(self.measurement.grid().frequencies_hz());
        self.alignment_sum = Some((key, sum));
    }

    fn apply_alignment_command(&mut self, command: AlignmentCommand) {
        let frequencies = std::sync::Arc::clone(self.measurement.grid());
        match command {
            AlignmentCommand::Update(mut state) => {
                let current = &self.session.alignment;
                if (state.reference_trace, state.delayed_trace)
                    != (current.reference_trace, current.delayed_trace)
                {
                    state.sum_hidden = false;
                }
                let needs_solutions = state.reference_trace != current.reference_trace
                    || state.delayed_trace != current.delayed_trace
                    || state.band_hz() != current.band_hz();
                let continuous = state.reference_trace == current.reference_trace
                    && state.delayed_trace == current.delayed_trace
                    && state.range == current.range
                    && state.choice == current.choice
                    && state.custom_inverted == current.custom_inverted;
                let edit = |session: &mut Session| {
                    session.alignment = state;
                    if needs_solutions {
                        session.refresh_alignment_solutions(frequencies.frequencies_hz());
                    }
                };
                if continuous {
                    self.edit_session_merged(MergeKey::Alignment, edit);
                } else {
                    self.edit_session(edit);
                    self.last_merge = None;
                }
            }
        }
    }

    fn edit_session(&mut self, edit: impl FnOnce(&mut Session)) {
        self.record_session_edit(edit);
    }

    fn record_session_edit(&mut self, edit: impl FnOnce(&mut Session)) -> bool {
        let before = self.session.clone();
        edit(&mut self.session);
        let changed = self.session != before;
        if changed {
            self.delay_undo = None;
            self.history.record(before);
            self.session_dirty_since.get_or_insert_with(Instant::now);
        }
        changed
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
                self.edit_session_merged(MergeKey::Offset(id), |session| {
                    if let Some(trace) = session.trace_mut(id) {
                        trace.offset_db = offset_db;
                    }
                });
            }
            TraceCommand::SetSmoothing(id, smoothing_fraction) => self.edit_session(|session| {
                if let Some(trace) = session.trace_mut(id) {
                    trace.set_smoothing(smoothing_fraction);
                }
            }),
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
            TraceCommand::RemoveSelected => {
                let selected: Vec<u64> = self
                    .session
                    .traces
                    .iter()
                    .filter(|trace| trace.selected)
                    .map(|trace| trace.id)
                    .collect();
                if !selected.is_empty() {
                    self.edit_session(|session| {
                        for id in &selected {
                            session.remove(*id);
                        }
                    });
                    self.notifications.push(
                        text::overlays_removed(selected.len()),
                        Tone::Neutral,
                        Some(Action::Undo),
                    );
                }
            }
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
            self.last_merge = None;
        }
    }

    fn export_file(&mut self, extension: &str, contents: &str) {
        let stem = format!(
            "{EXPORT_FILE_PREFIX}-{}",
            chrono::Local::now().format("%Y%m%d-%H%M%S")
        );
        match write_export(&self.export_directory, &stem, extension, contents) {
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

    fn export_csv(&mut self) {
        let csv = self
            .session
            .to_csv(self.measurement.grid().frequencies_hz());
        self.export_file("csv", &csv);
    }

    fn adopt_adjusted_sample_rate(&mut self) {
        let Some(sample_rate_hz) = self.measurement.take_sample_rate_adjustment() else {
            return;
        };
        self.notifications.push(
            text::sample_rate_adjusted(sample_rate_hz, self.config.stream.sample_rate_hz),
            Tone::Warning,
            None,
        );
    }

    fn current_route(&self) -> AudioRoute {
        self.measurement
            .route()
            .cloned()
            .unwrap_or_else(|| self.config.configured_route())
    }

    fn available_devices(&mut self) -> Vec<DeviceInfo> {
        let mut devices = self.device_catalog.devices();
        if self.measurement.is_running() {
            let selected = [
                self.config.stream.input_device_id.as_deref(),
                self.config.stream.output_device_id.as_deref(),
            ];
            for id in selected.into_iter().flatten() {
                if device_choice::find(&devices, id).is_none()
                    && let Some(known) = device_choice::find(&self.known_devices, id)
                {
                    devices.push(known.clone());
                }
            }
        }
        self.known_devices.clone_from(&devices);
        devices
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
        self.delay_undo = None;
        self.awaiting_delay_result = false;
        self.last_notified_delay_sequence = 0;
        self.measurement.start(&self.config);
    }

    fn stop_measurement(&mut self) {
        self.delay_undo = None;
        self.awaiting_delay_result = false;
        self.last_notified_delay_sequence = 0;
        self.measurement.stop(&self.config);
    }

    fn apply_view_command(&mut self, command: ViewCommand) {
        let layout = &mut self.config.plot_layout;
        match command {
            ViewCommand::Main(kind) => layout.set_main(kind),
            ViewCommand::Upper(kind) => layout.set_upper(kind),
            ViewCommand::ToggleStacked => layout.toggle_stacked(),
            ViewCommand::ImpulseInDb(in_db) => layout.impulse_in_db = in_db,
            ViewCommand::Module(module) => layout.module = module,
            ViewCommand::MouseMode(mode) => layout.mouse_mode = mode,
            ViewCommand::ShowCoherence(show) => layout.show_coherence = show,
            ViewCommand::FullView(kind) => {
                match kind {
                    ViewKind::Impulse => self.impulse_plot.view = Default::default(),
                    ViewKind::VirtualEq => self.delay_plot.view = self.delay_plot_default,
                    ViewKind::Magnitude | ViewKind::Phase => self.handle_action(Action::ResetView),
                }
                return;
            }
            ViewCommand::ToggleFullscreen => {
                self.plot_fullscreen = !self.plot_fullscreen;
                return;
            }
        }
        self.mark_config_dirty();
    }

    fn apply_latest_delay(&mut self) -> bool {
        let delay = self.measurement.snapshot().delay;
        let Some(latest) = delay.latest_confident else {
            return false;
        };
        if self.delay_undo.is_none() && !self.config.delay_tracking {
            self.delay_undo = Some(delay.applied_ms);
        }
        self.measurement.send(EngineCommand::ApplyLatestDelay);
        self.notifications.push(
            text::peak_applied(delay.acoustic_ms(latest.total_ms)),
            Tone::Success,
            Some(Action::Undo),
        );
        true
    }

    fn apply_control_command(&mut self, command: ControlCommand) {
        match command {
            ControlCommand::SetDelayMs(acoustic_ms) => {
                let latency_ms = self
                    .measurement
                    .snapshot()
                    .delay
                    .system_latency_ms
                    .unwrap_or(0.0);
                self.delay_undo = None;
                self.measurement
                    .send(EngineCommand::SetCompensationMs(acoustic_ms + latency_ms));
            }
            ControlCommand::ApplyStream(stream) => {
                self.config.stream = stream;
                self.mark_config_dirty();
                if self.measurement.is_running() {
                    self.stop_measurement();
                    self.start_measurement();
                }
            }
            ControlCommand::SetGeneratorLevel(level) => {
                self.config.stream.generator.level_dbfs = level;
                if let Some(generator) = self.measurement.generator() {
                    generator.set_level_dbfs(level);
                }
                self.mark_config_dirty();
            }
            ControlCommand::ResetAverages => {
                self.measurement.send(EngineCommand::ResetAverages);
            }
            ControlCommand::Trace(command) => self.apply_trace_command(command),
        }
    }

    fn channel_counts(&self) -> (usize, usize) {
        let stream = &self.config.stream;
        let used_inputs = match stream.reference {
            ReferenceMode::Internal => stream.measurement_channel_index,
            ReferenceMode::Loopback {
                reference_channel_index,
            } => stream
                .measurement_channel_index
                .max(reference_channel_index),
        } + 1;
        let used_outputs = stream
            .generator
            .output_channel_indices
            .iter()
            .max()
            .map_or(0, |index| index + 1);
        if self.measurement.is_file_mode() {
            return (FALLBACK_CHANNEL_COUNT.max(used_inputs), 0);
        }
        let inputs = device_choice::resolved_input(&self.known_devices, stream).map_or_else(
            || FALLBACK_CHANNEL_COUNT.max(used_inputs),
            |device| device.input_channel_count,
        );
        let outputs = device_choice::resolved_output(&self.known_devices, stream).map_or_else(
            || FALLBACK_CHANNEL_COUNT.max(used_outputs),
            |device| device.output_channel_count,
        );
        (inputs, outputs)
    }

    fn latency_status(&self) -> LatencyStatus {
        if !self.config.uses_internal_reference() || self.measurement.is_file_mode() {
            return LatencyStatus::NotNeeded;
        }
        match self
            .config
            .calibrated_latency_ms(&self.current_route())
            .or(self.measurement.snapshot().delay.system_latency_ms)
        {
            Some(latency_ms) => LatencyStatus::Calibrated(latency_ms),
            None => LatencyStatus::NotCalibrated,
        }
    }

    fn setup_line(&self) -> String {
        let source = self
            .measurement
            .source_label()
            .map(str::to_owned)
            .or_else(|| {
                device_choice::resolved_input(&self.known_devices, &self.config.stream).map(
                    |device| {
                        text::source_label(
                            &text::device_name(device),
                            self.config.stream.sample_rate_hz,
                        )
                    },
                )
            });
        let reference = match self.config.stream.reference {
            ReferenceMode::Loopback {
                reference_channel_index,
            } => Some(text::format_input_channel(reference_channel_index)),
            ReferenceMode::Internal => None,
        };
        let outputs = text::output_channels(&self.config.stream.generator.output_channel_indices);
        let generator = (!self.measurement.is_file_mode()
            && (self.is_live_internal() || self.config.stream.generator.enabled))
            .then_some((self.config.stream.generator.level_dbfs, outputs.as_str()));
        text::setup_line(
            self.measurement.is_file_mode(),
            source.as_deref(),
            reference.as_deref(),
            generator,
        )
    }

    fn refresh_module_sums(&mut self) {
        let key = (
            self.session.alignment_sum_key().map(|mut key| {
                key.clear_setting();
                key
            }),
            self.session.alignment.clone(),
        );
        if self
            .module_sums
            .as_ref()
            .is_some_and(|(cached, _)| *cached == key)
        {
            return;
        }
        let frequencies = std::sync::Arc::clone(self.measurement.grid());
        let sums = self
            .session
            .predicted_alignment_sums_db(frequencies.frequencies_hz());
        let (low_hz, high_hz) = self.session.alignment.band_hz();
        let levels = frequencies
            .frequencies_hz()
            .iter()
            .enumerate()
            .filter(|(_, frequency_hz)| (low_hz..=high_hz).contains(*frequency_hz))
            .flat_map(|(index, _)| {
                sums.iter()
                    .filter_map(move |(_, values)| values.get(index).copied().flatten())
            })
            .fold(None, |range: Option<(f64, f64)>, value| {
                Some(range.map_or((value, value), |(low, high)| {
                    (low.min(value), high.max(value))
                }))
            });
        let mut view = PlotView::default();
        view.show_frequency_range(low_hz, high_hz);
        if let Some((low_db, high_db)) = levels {
            let center = (low_db + high_db) / 2.0;
            let half =
                ((high_db - low_db) / 2.0 + MODULE_LEVEL_MARGIN_DB).max(MODULE_MIN_SPAN_DB / 2.0);
            view.show_level_range(center - half, center + half);
        }
        self.delay_plot_default = view;
        self.delay_plot.view = view;
        self.module_sums = Some((key, sums));
    }

    pub fn is_plot_fullscreen(&self) -> bool {
        self.plot_fullscreen
    }

    pub fn plot_layout(&self) -> PlotLayout {
        self.config.plot_layout
    }

    pub fn set_plot_layout(&mut self, layout: PlotLayout) {
        self.config.plot_layout = layout;
        self.mark_config_dirty();
    }

    fn refresh_phase_cache(&mut self) {
        let traces = &self.session.traces;
        self.phase_cache
            .retain(|(cached, _)| traces.iter().any(|trace| trace.id == cached.id));
        for trace in traces.iter().filter(|trace| trace.visible) {
            match self
                .phase_cache
                .iter_mut()
                .find(|(cached, _)| cached.id == trace.id)
            {
                Some((cached, _)) if std::sync::Arc::ptr_eq(cached, trace) => {}
                Some(entry) => *entry = (std::sync::Arc::clone(trace), trace.phase_deg()),
                None => self
                    .phase_cache
                    .push((std::sync::Arc::clone(trace), trace.phase_deg())),
            }
        }
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
                if self.measurement.is_running() {
                    self.measurement.send(EngineCommand::FindDelayNow);
                    self.awaiting_delay_result = true;
                }
            }
            Action::ApplyDelay => {
                if self.measurement.is_running() {
                    self.apply_latest_delay();
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
            Action::ResetView => {
                self.plot.view = PlotView::default();
                self.phase_plot.view = default_phase_view();
            }
            Action::ViewMagnitude => {
                self.apply_view_command(ViewCommand::Main(ViewKind::Magnitude))
            }
            Action::ViewPhase => self.apply_view_command(ViewCommand::Main(ViewKind::Phase)),
            Action::ViewImpulse => self.apply_view_command(ViewCommand::Main(ViewKind::Impulse)),
            Action::ViewVirtualEq => {
                let module = self.config.plot_layout.module;
                self.show_module(module);
            }
            Action::ToggleStackedViews => self.apply_view_command(ViewCommand::ToggleStacked),
            Action::ToggleFullscreen => self.apply_view_command(ViewCommand::ToggleFullscreen),
            Action::CheckForUpdates => {
                if !self.updates.is_checking() {
                    self.notifications
                        .push(text::NOTIFY_UPDATE_CHECKING, Tone::Neutral, None);
                }
                self.updates.start(CheckTrigger::Manual);
            }
            Action::OpenUpdatePage => {
                if let Some(release) = self.available_update.take() {
                    self.url_to_open = Some(release.page_url);
                }
            }
            Action::DismissUpdate => self.available_update = None,
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
            Action::Undo if self.delay_undo.is_some() && self.measurement.is_running() => {
                if let Some(previous_ms) = self.delay_undo.take() {
                    self.measurement
                        .send(EngineCommand::SetCompensationMs(previous_ms));
                    let acoustic = self.measurement.snapshot().delay.acoustic_ms(previous_ms);
                    self.notifications
                        .push(text::delay_restored(acoustic), Tone::Neutral, None);
                }
            }
            Action::Undo => {
                let view = self.session.eq.view;
                let text = if self.history.undo(&mut self.session) {
                    self.session.eq.keep_view_after_history(view);
                    self.session_dirty_since.get_or_insert_with(Instant::now);
                    text::NOTIFY_UNDONE
                } else {
                    text::NOTIFY_NOTHING_TO_UNDO
                };
                self.last_merge = None;
                self.notifications.push(text, Tone::Neutral, None);
            }
            Action::Redo => {
                let view = self.session.eq.view;
                if self.history.redo(&mut self.session) {
                    self.session.eq.keep_view_after_history(view);
                    self.session_dirty_since.get_or_insert_with(Instant::now);
                    self.notifications
                        .push(text::NOTIFY_REDONE, Tone::Neutral, None);
                }
                self.last_merge = None;
            }
            Action::ExportSession => self.export_csv(),
            Action::AddEqFilter => {
                self.show_module(ExternalModule::VirtualEq);
                self.add_eq_filter(eq_panel::default_new_filter());
            }
            Action::OpenAudioSettings => {
                if self.control.tab == ControlTab::Tools && self.config.side_panel_open {
                    self.control.tab = ControlTab::Measure;
                    self.audio_settings = None;
                } else {
                    self.config.side_panel_open = true;
                    self.control.tab = ControlTab::Tools;
                }
                self.mark_config_dirty();
            }
            Action::ToggleDelayTracking => {
                self.delay_undo = None;
                self.config.delay_tracking = !self.config.delay_tracking;
                self.measurement
                    .send(EngineCommand::UpdateDelayPolicy(self.config.delay_policy()));
                self.mark_config_dirty();
                self.notifications.push(
                    if self.config.delay_tracking {
                        text::NOTIFY_TRACKING_ON
                    } else {
                        text::NOTIFY_TRACKING_OFF
                    },
                    Tone::Neutral,
                    None,
                );
            }
            Action::NudgeDelayDown | Action::NudgeDelayUp => {
                if self.measurement.is_running() {
                    self.delay_undo = None;
                    let step = if action == Action::NudgeDelayUp {
                        DELAY_NUDGE_MS
                    } else {
                        -DELAY_NUDGE_MS
                    };
                    self.measurement
                        .send(EngineCommand::AdjustCompensationMs(step));
                }
            }
        }
    }

    fn process_engine_events(&mut self) {
        let delay = self.measurement.snapshot().delay;
        let from_current_engine = self.measurement.snapshot().sequence > 0;
        if let Some(latency_ms) = delay.system_latency_ms.filter(|_| from_current_engine) {
            let key = self.config.latency_key(&self.current_route());
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
            self.apply_latest_delay();
        }
    }

    fn notify_delay_drift(&mut self) {
        let delay = self.measurement.snapshot().delay;
        let running = self.measurement.is_running() && self.measurement.snapshot().sequence > 0;
        let drift = delay
            .drift_ms()
            .filter(|drift| drift.abs() > DELAY_DRIFT_NOTIFY_MS);
        match (drift, delay.latest_confident) {
            (Some(drift_ms), Some(latest))
                if running
                    && !self.config.delay_tracking
                    && !self.awaiting_delay_result
                    && !delay.calibrating_latency =>
            {
                if !self.drift_notified {
                    self.drift_notified = true;
                    self.notifications.push(
                        text::delay_drift(delay.acoustic_ms(latest.total_ms), drift_ms),
                        Tone::Warning,
                        Some(Action::ApplyDelay),
                    );
                }
            }
            (None, _) => self.drift_notified = false,
            _ => {}
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
            approximate: self.is_live_internal()
                && self
                    .config
                    .calibrated_latency_ms(&self.current_route())
                    .is_none(),
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

    fn fit_window_to_monitor(&mut self, context: &egui::Context) {
        if self.window_fit_frames_left == 0 {
            return;
        }
        self.window_fit_frames_left -= 1;
        let (inner_size, monitor_size) = context.input(|input| {
            let viewport = input.viewport();
            (
                viewport.inner_rect.map(|rect| rect.size()),
                viewport.monitor_size,
            )
        });
        if let (Some(inner_size), Some(monitor_size)) = (inner_size, monitor_size) {
            self.window_fit_frames_left = 0;
            if window_exceeds_monitor(inner_size, monitor_size) {
                context.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
            }
        }
    }

    fn banner_inputs(&self) -> BannerInputs {
        let snapshot = self.measurement.snapshot();
        let (reference, measurement) = self.measurement.meter_readings();
        BannerInputs {
            running: self.measurement.is_running(),
            start_error: self.measurement.last_error().map(str::to_owned),
            alert: self.measurement.input_alert(),
            live_internal_reference: self.is_live_internal(),
            latency_calibrated: self
                .config
                .calibrated_latency_ms(&self.current_route())
                .is_some()
                || snapshot.delay.system_latency_ms.is_some(),
            separate_clocks: self.measurement.uses_separate_clocks(),
            low_coherence: self.low_coherence_cache.1 && self.measurement.is_running(),
            clipping: reference.clipped || measurement.clipped,
            calibrating: snapshot.delay.calibrating_latency,
            calibration_failed: snapshot.delay.calibration_failed,
            file_finished: snapshot.source_finished,
            available_update: self
                .available_update
                .as_ref()
                .map(|release| release.version),
        }
    }

    fn poll_updates(&mut self) {
        let Some((trigger, status)) = self.updates.poll() else {
            return;
        };
        match (trigger, status) {
            (_, UpdateStatus::Available(release)) => self.available_update = Some(release),
            (CheckTrigger::Manual, UpdateStatus::UpToDate) => {
                self.notifications
                    .push(text::up_to_date(Version::current()), Tone::Success, None)
            }
            (CheckTrigger::Manual, UpdateStatus::Unreachable) => {
                self.notifications
                    .push(text::NOTIFY_UPDATE_UNREACHABLE, Tone::Warning, None);
            }
            (CheckTrigger::Startup, UpdateStatus::UpToDate | UpdateStatus::Unreachable) => {}
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
                let devices = self.available_devices();
                if let Some(wizard) = self.wizard.as_mut() {
                    wizard.devices = devices;
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
        self.fit_window_to_monitor(&context);
        self.apply_theme(&context);
        let palette = self.config.theme.palette();
        self.measurement.refresh();
        if self.config.delay_tracking {
            self.delay_undo = None;
        }
        self.adopt_adjusted_sample_rate();
        self.update_low_coherence();
        self.process_engine_events();
        self.notify_delay_drift();
        self.poll_updates();

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

        let right_inset = if self.wizard.is_none() {
            size::DISPLAY_PANEL_WIDTH + spacing::SM
        } else {
            0.0
        };
        self.notifications
            .show(&context, palette, self.is_mac, right_inset, &mut actions);
        self.command_palette
            .show(&context, palette, self.is_mac, &mut actions);
        for action in actions {
            self.handle_action(action);
        }
        if let Some(url) = self.url_to_open.take() {
            context.open_url(egui::OpenUrl::new_tab(url));
        }
        let (os_fullscreen, escape) = context.input(|input| {
            (
                input.viewport().fullscreen,
                input.key_pressed(egui::Key::Escape),
            )
        });
        if self.plot_fullscreen
            && self.viewport_fullscreen
            && (escape && !context.egui_wants_keyboard_input() && !self.command_palette.is_open()
                || self.os_reported_fullscreen && os_fullscreen == Some(false))
        {
            self.plot_fullscreen = false;
        }
        self.os_reported_fullscreen = os_fullscreen == Some(true);
        if self.viewport_fullscreen != self.plot_fullscreen {
            self.viewport_fullscreen = self.plot_fullscreen;
            context.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.plot_fullscreen));
        }
        self.end_merge_when_idle(&context);
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
        let show_chrome = !self.plot_fullscreen;
        if show_chrome {
            let (status_tone, status_text) = self.status();
            let status_model = StatusBarModel {
                status_tone,
                status_text,
                setup: self.setup_line(),
                version: text::version_label(Version::current()),
            };
            Panel::bottom("status-bar")
                .exact_size(size::STATUS_BAR_HEIGHT)
                .frame(
                    Frame::new()
                        .fill(palette.surface)
                        .inner_margin(Margin::symmetric(spacing::MD as i8, 0))
                        .stroke(Stroke::new(stroke::BORDER, palette.border)),
                )
                .show(ui, |ui| {
                    status_bar::show(ui, palette, &status_model, self.is_mac, actions)
                });
        }

        let panel_open = self.config.side_panel_open && show_chrome;
        if panel_open && self.control.tab == ControlTab::Tools && self.audio_settings.is_none() {
            let devices = self.available_devices();
            self.audio_settings = Some(AudioSettingsPanel::new(devices, &self.config.stream));
        }
        if self.control.tab != ControlTab::Tools {
            self.audio_settings = None;
        }
        let mut control_commands = Vec::new();
        let mut audio_commands = Vec::new();
        let mut analysis = self.config.analysis.clone();
        let mut analysis_changed = false;
        if panel_open {
            let (reference_meter, measurement_meter) = self.measurement.meter_readings();
            let (input_channels, output_channels) = self.channel_counts();
            let delay = self.delay_readout();
            let latency = self.latency_status();
            let stream = self.config.stream.clone();
            let file_mode = self.measurement.is_file_mode();
            let running = self.measurement.is_running();
            let frozen = self.measurement.is_frozen();
            let sample_rate_hz = self.measurement.snapshot().sample_rate_hz;
            let theme = self.config.theme;
            Panel::left("control-panel")
                .exact_size(size::CONTROL_PANEL_WIDTH)
                .frame(
                    Frame::new()
                        .fill(palette.surface)
                        .inner_margin(Margin::symmetric(spacing::MD as i8, spacing::SM as i8))
                        .stroke(Stroke::new(stroke::BORDER, palette.border)),
                )
                .show(ui, |ui| {
                    control_panel::tabs(ui, palette, &mut self.control);
                    match self.control.tab {
                        ControlTab::Measure => {
                            let model = ControlModel {
                                running,
                                frozen,
                                file_mode,
                                delay,
                                tracking: self.config.delay_tracking,
                                stream: &stream,
                                input_channels,
                                output_channels,
                                latency,
                                sample_rate_hz,
                                reference_meter,
                                measurement_meter,
                                session: &self.session,
                            };
                            let changes = control_panel::show_measure(
                                ui,
                                palette,
                                &model,
                                &mut analysis,
                                &mut self.control,
                                self.is_mac,
                                actions,
                            );
                            control_commands = changes.commands;
                            analysis_changed |= changes.analysis_changed;
                        }
                        ControlTab::Tools => {
                            ui.add_space(spacing::SM);
                            tools_general(ui, palette, theme, self.is_mac, actions);
                            ui.add_space(spacing::MD);
                            if let Some(panel) = self.audio_settings.as_mut() {
                                audio_commands = audio_panel::show(
                                    ui,
                                    palette,
                                    panel,
                                    &self.config.stream,
                                    file_mode,
                                );
                            }
                        }
                    }
                });
        }
        for command in control_commands {
            self.apply_control_command(command);
        }
        for command in audio_commands {
            self.apply_audio_command(command);
        }

        let layout = self.config.plot_layout;
        let module_shown = layout.shows(ViewKind::VirtualEq);
        let eq_mode = module_shown && layout.module == ExternalModule::VirtualEq;
        let delay_module = module_shown && layout.module == ExternalModule::DelayAnalysis;
        let eq_curves = if eq_mode {
            self.eq_overlays(palette)
        } else {
            Vec::new()
        };
        let eq_handles = if eq_mode {
            self.session.eq.handles()
        } else {
            Vec::new()
        };
        let alignment_mode = delay_module && self.session.alignment.is_ready();
        let alignment_pair = if alignment_mode {
            [
                self.session.alignment.reference_trace,
                self.session.alignment.delayed_trace,
            ]
        } else {
            [None, None]
        };
        if alignment_mode && self.session.alignment.shows_sum() {
            self.refresh_alignment_sum();
        }
        if alignment_mode {
            self.refresh_module_sums();
        }
        let alignment_sum: Option<(String, Vec<Option<f64>>, TraceStyle)> = (alignment_mode
            && self.session.alignment.shows_sum())
        .then(|| {
            let choice = self.session.alignment.choice;
            let (_, sum) = self.alignment_sum.as_ref()?;
            Some((
                text::alignment_sum_label(choice),
                sum.clone()?,
                TraceStyle {
                    color: alignment_panel::choice_color(palette, choice),
                    width: stroke::TRACE_LIVE,
                    halo: None,
                },
            ))
        })
        .flatten();
        let module_curves: Vec<(String, Vec<Option<f64>>, TraceStyle)> = if alignment_mode {
            self.module_sums
                .as_ref()
                .map(|(_, sums)| {
                    sums.iter()
                        .map(|(choice, values)| {
                            let selected = *choice == self.session.alignment.choice;
                            (
                                text::alignment_choice(*choice).to_owned(),
                                values.clone(),
                                TraceStyle {
                                    color: alignment_panel::choice_color(palette, *choice),
                                    width: if selected {
                                        stroke::TRACE_LIVE
                                    } else {
                                        stroke::TRACE_STORED
                                    },
                                    halo: None,
                                },
                            )
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        let banners = collect_banners(&self.banner_inputs());
        if layout.shows(ViewKind::Phase) {
            self.refresh_phase_cache();
        }
        let mut band_selection = None;
        let mut plot_events = PlotEvents::default();
        let mut view_commands = Vec::new();
        let mut alignment_commands = Vec::new();
        let mut eq_commands = Vec::new();
        let mut module_double_click = false;
        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(palette.background)
                    .inner_margin(Margin::same(spacing::SM as i8))
                    .corner_radius(CornerRadius::ZERO),
            )
            .show(ui, |ui| {
                let grid = std::sync::Arc::clone(self.measurement.grid());
                let trace_style = |trace: &crate::session::Trace| TraceStyle {
                    color: palette.stored_traces[trace.color_index % STORED_TRACE_COLOR_COUNT],
                    width: stroke::TRACE_STORED,
                    halo: None,
                };
                let in_alignment_pair = |trace: &std::sync::Arc<crate::session::Trace>| {
                    alignment_pair.contains(&Some(trace.id))
                };
                let overlays: Vec<OverlayTrace<'_>> =
                    self.session
                        .traces
                        .iter()
                        .filter(|trace| trace.visible && !in_alignment_pair(trace))
                        .map(|trace| stored_overlay(trace, trace_style(trace)))
                        .chain(alignment_sum.as_ref().map(|(label, values_db, style)| {
                            OverlayTrace {
                                label,
                                values_db,
                                offset_db: 0.0,
                                style: *style,
                            }
                        }))
                        .chain(alignment_pair.iter().filter_map(|id| {
                            let trace = self.session.trace((*id)?)?;
                            Some(stored_overlay(trace, trace_style(trace)))
                        }))
                        .chain(
                            eq_curves
                                .iter()
                                .map(|(label, values_db, style)| OverlayTrace {
                                    label,
                                    values_db,
                                    offset_db: 0.0,
                                    style: *style,
                                }),
                        )
                        .collect();
                let module_overlays: Vec<OverlayTrace<'_>> = module_curves
                    .iter()
                    .map(|(label, values_db, style)| OverlayTrace {
                        label,
                        values_db,
                        offset_db: 0.0,
                        style: *style,
                    })
                    .collect();
                let phase_overlays: Vec<OverlayTrace<'_>> = self
                    .session
                    .traces
                    .iter()
                    .filter(|trace| trace.visible)
                    .filter_map(|trace| {
                        let (_, phases) = self
                            .phase_cache
                            .iter()
                            .find(|(cached, _)| std::sync::Arc::ptr_eq(cached, trace))?;
                        Some(OverlayTrace {
                            label: &trace.name,
                            values_db: phases,
                            offset_db: 0.0,
                            style: trace_style(trace),
                        })
                    })
                    .collect();
                let snapshot = self.measurement.snapshot();
                let show_live = snapshot.averages > 0;
                let impulse_overlays: Vec<ImpulseCurve<'_>> = self
                    .session
                    .traces
                    .iter()
                    .filter(|trace| trace.visible && layout.shows(ViewKind::Impulse))
                    .filter_map(|trace| {
                        let impulse = trace.impulse.as_ref()?;
                        Some(ImpulseCurve {
                            label: &trace.name,
                            samples: &impulse.samples,
                            start_ms: impulse.start_ms,
                            sample_rate_hz: impulse.sample_rate_hz,
                            style: trace_style(trace),
                        })
                    })
                    .collect();
                let live_legend = LegendItem {
                    label: text::LIVE_TRACE_LABEL,
                    color: palette.trace_live,
                };
                let legend_for = |kind: ViewKind| -> Vec<LegendItem<'_>> {
                    let items = match kind {
                        ViewKind::Magnitude => &overlays,
                        ViewKind::Phase => &phase_overlays,
                        ViewKind::VirtualEq => return Vec::new(),
                        ViewKind::Impulse => {
                            return show_live
                                .then_some(live_legend)
                                .into_iter()
                                .chain(impulse_overlays.iter().map(|curve| LegendItem {
                                    label: curve.label,
                                    color: curve.style.color,
                                }))
                                .collect();
                        }
                    };
                    let live = (show_live && kind != ViewKind::VirtualEq).then_some(live_legend);
                    live.into_iter()
                        .chain(items.iter().map(|overlay| LegendItem {
                            label: overlay.label,
                            color: overlay.style.color,
                        }))
                        .collect()
                };

                let area = ui.available_rect_before_wrap();
                let stacked = layout.upper.is_some();
                let bar_height = size::MOUSE_BAR_HEIGHT;
                let available = area.height() - bar_height;
                let upper_height = upper_pane_height(available, self.config.upper_pane_fraction);
                let (upper_rect, bar_rect, lower_rect) = if stacked {
                    (
                        Some(egui::Rect::from_min_size(
                            area.min,
                            egui::vec2(area.width(), upper_height),
                        )),
                        egui::Rect::from_min_size(
                            egui::pos2(area.left(), area.top() + upper_height),
                            egui::vec2(area.width(), bar_height),
                        ),
                        egui::Rect::from_min_max(
                            egui::pos2(area.left(), area.top() + upper_height + bar_height),
                            area.max,
                        ),
                    )
                } else {
                    (
                        None,
                        egui::Rect::from_min_size(area.min, egui::vec2(area.width(), bar_height)),
                        egui::Rect::from_min_max(
                            egui::pos2(area.left(), area.top() + bar_height),
                            area.max,
                        ),
                    )
                };
                let panes = upper_rect
                    .zip(layout.upper)
                    .map(|(rect, kind)| (kind, true, rect))
                    .into_iter()
                    .chain(std::iter::once((layout.main, false, lower_rect)));
                let mut banner_anchor = None;
                let mut readout = None;
                for (kind, upper, rect) in panes {
                    let display_rect = egui::Rect::from_min_max(
                        egui::pos2(rect.right() - size::DISPLAY_PANEL_WIDTH, rect.top()),
                        rect.max,
                    );
                    let graph_rect = egui::Rect::from_min_max(
                        rect.min,
                        egui::pos2(display_rect.left() - spacing::SM, rect.bottom()),
                    );
                    ui.painter().rect(
                        display_rect,
                        CornerRadius::same(crate::design::tokens::radius::SMALL),
                        palette.surface,
                        Stroke::new(stroke::BORDER, palette.border),
                        egui::StrokeKind::Inside,
                    );
                    let mut display_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(display_rect.shrink(spacing::SM))
                            .layout(egui::Layout::top_down(egui::Align::Min)),
                    );
                    let display = display_panel::show(
                        &mut display_ui,
                        palette,
                        kind,
                        upper,
                        &layout,
                        &mut analysis,
                    );
                    view_commands.extend(display.commands);
                    analysis_changed |= display.analysis_changed;

                    let legend = legend_for(kind);
                    let mut pane = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(graph_rect)
                            .layout(egui::Layout::top_down(egui::Align::Min)),
                    );
                    view_commands.extend(graph_bar::show(
                        &mut pane,
                        palette,
                        kind,
                        upper,
                        &legend,
                        self.is_mac,
                    ));
                    pane.add_space(spacing::XS);
                    let is_main = !upper;
                    let pane_plot_rect = match kind {
                        ViewKind::Magnitude => {
                            let inputs = PlotInputs {
                                frequencies_hz: grid.frequencies_hz(),
                                snapshot,
                                show_live,
                                running: self.measurement.is_running(),
                                overlays: &overlays,
                                band_hz: alignment_mode.then(|| self.session.alignment.band_hz()),
                                eq_handles: &eq_handles,
                                show_empty_state: true,
                                show_coherence: layout.show_coherence,
                                mouse_mode: layout.mouse_mode,
                            };
                            let events =
                                self.plot
                                    .show(&mut pane, palette, &inputs, self.is_mac, actions);
                            if events.double_click_hz_db.is_some() && !eq_mode {
                                actions.push(Action::ResetView);
                            }
                            band_selection = events.band_selection;
                            readout = readout.or_else(|| events.hover_readout.clone());
                            self.last_plot_rect = events.plot_rect;
                            let plot_rect = events.plot_rect;
                            plot_events = events;
                            plot_rect
                        }
                        ViewKind::Phase => {
                            self.phase_plot.view.low_hz = self.plot.view.low_hz;
                            self.phase_plot.view.high_hz = self.plot.view.high_hz;
                            let before = (self.plot.view.low_hz, self.plot.view.high_hz);
                            let inputs = PlotInputs {
                                frequencies_hz: grid.frequencies_hz(),
                                snapshot,
                                show_live,
                                running: self.measurement.is_running(),
                                overlays: &phase_overlays,
                                band_hz: None,
                                eq_handles: &[],
                                show_empty_state: is_main,
                                show_coherence: false,
                                mouse_mode: layout.mouse_mode,
                            };
                            let events = self.phase_plot.show(
                                &mut pane,
                                palette,
                                &inputs,
                                self.is_mac,
                                actions,
                            );
                            self.plot.view.low_hz = self.phase_plot.view.low_hz;
                            self.plot.view.high_hz = self.phase_plot.view.high_hz;
                            if before != (self.plot.view.low_hz, self.plot.view.high_hz) {
                                pane.ctx().request_repaint();
                            }
                            if events.double_click_hz_db.is_some() && !eq_mode {
                                actions.push(Action::ResetView);
                            }
                            readout = readout.or(events.hover_readout);
                            events.plot_rect
                        }
                        ViewKind::Impulse => {
                            let inputs = ImpulseInputs {
                                live: show_live
                                    .then(|| ImpulseCurve {
                                        label: text::LIVE_TRACE_LABEL,
                                        samples: &snapshot.impulse,
                                        start_ms: snapshot.impulse_time_ms(0),
                                        sample_rate_hz: snapshot.sample_rate_hz,
                                        style: TraceStyle {
                                            color: palette.trace_live,
                                            width: stroke::TRACE_LIVE,
                                            halo: None,
                                        },
                                    })
                                    .filter(|curve| !curve.samples.is_empty()),
                                overlays: &impulse_overlays,
                                in_db: layout.impulse_in_db,
                                speed_of_sound_m_per_s: dsp_core::speed_of_sound_m_per_s(
                                    self.config.air_temperature_celsius,
                                ),
                                warming_up: self.measurement.is_running() && snapshot.warming_up,
                                mouse_mode: layout.mouse_mode,
                            };
                            let events = self.impulse_plot.show(&mut pane, palette, &inputs);
                            readout = readout.or(events.hover_readout);
                            events.plot_rect
                        }
                        ViewKind::VirtualEq => {
                            if layout.module == ExternalModule::DelayAnalysis {
                                let controls_height = alignment_panel::CONTROLS_HEIGHT;
                                let plot_height = pane.available_height() - controls_height;
                                if self.session.alignment.is_ready()
                                    && plot_height >= MODULE_PLOT_MIN_HEIGHT
                                {
                                    let plot_rect = egui::Rect::from_min_size(
                                        pane.cursor().min,
                                        egui::vec2(pane.available_width(), plot_height),
                                    );
                                    let mut plot_ui = pane.new_child(
                                        egui::UiBuilder::new()
                                            .max_rect(plot_rect)
                                            .layout(egui::Layout::top_down(egui::Align::Min)),
                                    );
                                    let inputs = PlotInputs {
                                        frequencies_hz: grid.frequencies_hz(),
                                        snapshot,
                                        show_live: false,
                                        running: false,
                                        overlays: &module_overlays,
                                        band_hz: None,
                                        eq_handles: &[],
                                        show_empty_state: false,
                                        show_coherence: false,
                                        mouse_mode: layout.mouse_mode,
                                    };
                                    let events = self.delay_plot.show(
                                        &mut plot_ui,
                                        palette,
                                        &inputs,
                                        self.is_mac,
                                        actions,
                                    );
                                    module_double_click = events.double_click_hz_db.is_some();
                                    readout = readout.or(events.hover_readout);
                                    pane.advance_cursor_after_rect(plot_rect);
                                }
                                alignment_commands =
                                    alignment_panel::show(&mut pane, palette, &self.session);
                            } else {
                                eq_commands = eq_panel::show(&mut pane, palette, &self.session);
                            }
                            None
                        }
                    };
                    if banner_anchor.is_none() {
                        banner_anchor = pane_plot_rect;
                    }
                    if !layout.shows(ViewKind::Magnitude) && is_main {
                        self.last_plot_rect = pane_plot_rect;
                    }
                }
                let split = stacked.then_some(SplitArea {
                    top: area.top(),
                    available_height: available,
                });
                let bar = mouse_bar::show(
                    ui,
                    palette,
                    bar_rect,
                    &MouseBarModel {
                        readout: readout.as_deref(),
                        mode: layout.mouse_mode,
                        stacked,
                        fullscreen: self.plot_fullscreen,
                    },
                    split,
                    self.is_mac,
                );
                view_commands.extend(bar.commands);
                if let Some(fraction) = bar.upper_fraction
                    && (fraction - self.config.upper_pane_fraction).abs() > f32::EPSILON
                {
                    self.config.upper_pane_fraction = fraction;
                    self.config_dirty_since.get_or_insert_with(Instant::now);
                }
                ui.allocate_rect(area, egui::Sense::hover());
                if let (false, Some(plot_rect)) = (banners.is_empty(), banner_anchor) {
                    egui::Area::new(egui::Id::unique("state-banners"))
                        .order(egui::Order::Middle)
                        .fixed_pos(plot_rect.left_top() + egui::vec2(spacing::SM, spacing::SM))
                        .show(ui.ctx(), |ui| {
                            ui.set_width(plot_rect.width() - 2.0 * spacing::SM);
                            show_banners(ui, palette, &banners, actions);
                        });
                }
            });
        if analysis_changed && analysis != self.config.analysis {
            self.config.analysis = analysis;
            self.measurement.send(EngineCommand::UpdateSettings(
                self.config.analysis.to_settings(),
            ));
            self.mark_config_dirty();
        }
        for command in view_commands {
            self.apply_view_command(command);
        }
        for command in alignment_commands {
            self.apply_alignment_command(command);
        }
        for command in eq_commands {
            self.apply_eq_command(command);
        }
        if module_double_click {
            self.delay_plot.view = self.delay_plot_default;
        }
        if let Some(selection) = band_selection {
            self.apply_band_selection(selection);
        }
        if eq_mode {
            self.apply_eq_plot_events(&plot_events);
        }
    }
}

fn tools_general(
    ui: &mut egui::Ui,
    palette: &crate::design::Palette,
    theme: Theme,
    is_mac: bool,
    actions: &mut Vec<Action>,
) {
    use crate::design::widgets::{field_caption, section_header, toggle_button};
    section_header(ui, palette, text::TOOLS_GENERAL);
    field_caption(ui, palette, text::TOOLS_COLOR_SCHEME);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = size::SEGMENT_GAP;
        let half = (ui.available_width() - size::SEGMENT_GAP) / 2.0;
        for (candidate, label) in [
            (Theme::Night, text::TOOLS_NIGHT),
            (Theme::Daylight, text::TOOLS_DAYLIGHT),
        ] {
            let response = toggle_button(ui, palette, label, theme == candidate, Some(half))
                .on_hover_text(text::with_shortcut(
                    Action::ToggleTheme.label(),
                    Action::ToggleTheme.shortcut_text(is_mac).as_deref(),
                ));
            if response.clicked() && theme != candidate {
                actions.push(Action::ToggleTheme);
            }
        }
    });
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = size::SEGMENT_GAP;
        let half = (ui.available_width() - size::SEGMENT_GAP) / 2.0;
        if toggle_button(ui, palette, text::TOOLS_CHECK_UPDATES, false, Some(half)).clicked() {
            actions.push(Action::CheckForUpdates);
        }
        if toggle_button(ui, palette, text::BUTTON_PALETTE, false, Some(half))
            .on_hover_text(text::with_shortcut(
                Action::OpenPalette.label(),
                Action::OpenPalette.shortcut_text(is_mac).as_deref(),
            ))
            .clicked()
        {
            actions.push(Action::OpenPalette);
        }
    });
}

fn stored_overlay(trace: &crate::session::Trace, style: TraceStyle) -> OverlayTrace<'_> {
    OverlayTrace {
        label: &trace.name,
        values_db: &trace.magnitude_db,
        offset_db: trace.offset_db,
        style,
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
