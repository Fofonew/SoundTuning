use audio_io::{ReferenceMode, StreamSettings};
use eframe::egui::{
    self, Align, Align2, CornerRadius, Layout, Rect, RichText, Sense, Stroke, TextStyle, pos2, vec2,
};
use engine::{FFT_LENGTH_CHOICES, TIME_CONSTANT_CHOICES_S};

use crate::actions::Action;
use crate::config::StoredAnalysis;
use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{radius, size, spacing, stroke, text_size};
use crate::design::widgets::{
    Emphasis, MeterReading, caption_cell, confidence_segments, field_caption, section_header,
    sized_action_button, spin_label, tab_strip, toggle_button,
};
use crate::session::Session;
use crate::text;
use crate::ui::traces_panel::{self, OverlayListState, TraceCommand};

const CHANNEL_BUTTON_WIDTH: f32 = 28.0;
const MAX_CHANNEL_BUTTONS: usize = 8;
const ROW_CAPTION_WIDTH: f32 = 84.0;
const CHANNEL_CAPTION_WIDTH: f32 = 112.0;
const FREEZE_BUTTON_WIDTH: f32 = 84.0;
const NUDGE_BUTTON_WIDTH: f32 = 28.0;
const DELAY_DRAG_SPEED_MS: f64 = 0.01;
const DELAY_LIMIT_MS: f64 = 1000.0;
const DELAY_TEXT_STYLE: &str = "delay-offset";
const DELAY_VALUE_HEIGHT: f32 = 44.0;
const GENERATOR_DRAG_SPEED_DB: f64 = 0.5;
const GENERATOR_MIN_DBFS: f64 = -60.0;
const GENERATOR_MAX_DBFS: f64 = 0.0;
const GENERATOR_SLIDER_STEP_DB: f64 = 0.5;
const GENERATOR_VALUE_WIDTH: f32 = 64.0;
const INPUT_METER_CAPTION_WIDTH: f32 = 64.0;
const INPUT_METER_HEIGHT: f32 = 10.0;
const INPUT_METER_ROW_HEIGHT: f32 = 18.0;
const INPUT_METER_VALUE_WIDTH: f32 = 32.0;
const INPUT_METER_CLIP_WIDTH: f32 = 36.0;
const INPUT_METER_FLOOR_DBFS: f32 = -60.0;
const INPUT_METER_MIN_BAR_WIDTH: f32 = 24.0;
const INPUT_METER_WARNING_DBFS: f32 = -6.0;
const INPUT_METER_TICKS_DBFS: [f32; 4] = [-48.0, -24.0, -12.0, -6.0];
const LIST_MIN_HEIGHT: f32 = 168.0;
const METER_BAR_WIDTH: f32 = 14.0;
const METER_TICKS_DBFS: [f32; 5] = [0.0, -18.0, -36.0, -54.0, -72.0];
const METER_RANGE_DBFS: f32 = 72.0;
const METER_WARNING_DBFS: f32 = -6.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ControlTab {
    #[default]
    Measure,
    Tools,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListTab {
    #[default]
    Overlays,
    Levels,
}

#[derive(Debug, Default)]
pub struct ControlPanelState {
    pub tab: ControlTab,
    pub list_tab: ListTab,
    pub overlays: OverlayListState,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DelayReadout {
    pub milliseconds: Option<f64>,
    pub distance_m: Option<f64>,
    pub samples: Option<f64>,
    pub confidence_db: Option<f64>,
}

pub struct ControlModel<'a> {
    pub running: bool,
    pub frozen: bool,
    pub file_mode: bool,
    pub delay: DelayReadout,
    pub tracking: bool,
    pub stream: &'a StreamSettings,
    pub input_channels: usize,
    pub output_channels: usize,
    pub sample_rate_hz: u32,
    pub reference_meter: MeterReading,
    pub measurement_meter: MeterReading,
    pub session: &'a Session,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ControlCommand {
    SetDelayMs(f64),
    ApplyStream(StreamSettings),
    SetGeneratorLevel(f32),
    ResetAverages,
    Trace(TraceCommand),
}

#[derive(Debug, Default, PartialEq)]
pub struct ControlChanges {
    pub commands: Vec<ControlCommand>,
    pub analysis_changed: bool,
}

pub fn tabs(ui: &mut egui::Ui, palette: &Palette, state: &mut ControlPanelState) {
    tab_strip(
        ui,
        palette,
        "control-tabs",
        &[
            (ControlTab::Measure, text::TAB_MEASURE),
            (ControlTab::Tools, text::TAB_TOOLS),
        ],
        &mut state.tab,
    );
}

pub fn show_measure(
    ui: &mut egui::Ui,
    palette: &Palette,
    model: &ControlModel<'_>,
    analysis: &mut StoredAnalysis,
    state: &mut ControlPanelState,
    is_mac: bool,
    actions: &mut Vec<Action>,
) -> ControlChanges {
    let mut changes = ControlChanges::default();
    let top_height = (ui.available_height() - LIST_MIN_HEIGHT).max(0.0);
    egui::ScrollArea::vertical()
        .id_salt("measure-controls")
        .max_height(top_height)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = spacing::XS;
            ui.add_space(spacing::SM);
            input_section(ui, palette, model, is_mac, actions, &mut changes);
            ui.add_space(spacing::SM);
            analysis_row(ui, palette, model, is_mac, actions);
            ui.add_space(spacing::SM);
            output_section(ui, palette, model, is_mac, actions, &mut changes);
            ui.add_space(spacing::SM);
            parameters_section(ui, palette, model, analysis, &mut changes);
            ui.add_space(spacing::SM);
            let width = ui.available_width();
            if sized_action_button(
                ui,
                palette,
                text::BUTTON_CAPTURE_MEASUREMENT,
                Action::CaptureTrace.shortcut_text(is_mac).as_deref(),
                Emphasis::Secondary,
                model.running,
                width,
            )
            .clicked()
            {
                actions.push(Action::CaptureTrace);
            }
        });
    ui.add_space(spacing::SM);
    tab_strip(
        ui,
        palette,
        "list-tabs",
        &[
            (ListTab::Overlays, text::TAB_OVERLAYS),
            (ListTab::Levels, text::TAB_LEVELS),
        ],
        &mut state.list_tab,
    );
    ui.add_space(spacing::SM);
    match state.list_tab {
        ListTab::Overlays => {
            for command in traces_panel::show(ui, palette, model.session, &mut state.overlays) {
                changes.commands.push(ControlCommand::Trace(command));
            }
        }
        ListTab::Levels => levels(ui, palette, model),
    }
    changes
}

fn caption_row(
    ui: &mut egui::Ui,
    palette: &Palette,
    caption: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    caption_row_with_width(ui, palette, caption, ROW_CAPTION_WIDTH, add_contents);
}

fn caption_row_with_width(
    ui: &mut egui::Ui,
    palette: &Palette,
    caption: &str,
    width: f32,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = size::SEGMENT_GAP;
        caption_cell(ui, palette, caption, width);
        add_contents(ui);
    });
}

fn channel_buttons(
    ui: &mut egui::Ui,
    palette: &Palette,
    count: usize,
    is_selected: impl Fn(usize) -> bool,
    label: impl Fn(usize) -> String,
    enabled: bool,
) -> Option<usize> {
    let mut clicked = None;
    let fits = count as f32 * (CHANNEL_BUTTON_WIDTH + size::SEGMENT_GAP) <= ui.available_width();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(size::SEGMENT_GAP, size::SEGMENT_GAP);
        if count > MAX_CHANNEL_BUTTONS || !fits {
            let selected = (0..count).find(|index| is_selected(*index)).unwrap_or(0);
            let mut choice = selected;
            ui.add_enabled_ui(enabled, |ui| {
                egui::ComboBox::from_id_salt(label(0))
                    .selected_text(text::channel_number(selected))
                    .show_ui(ui, |ui| {
                        for index in 0..count {
                            ui.selectable_value(&mut choice, index, text::channel_number(index));
                        }
                    })
            });
            if choice != selected {
                clicked = Some(choice);
            }
            return;
        }
        for index in 0..count {
            let response = ui
                .add_enabled_ui(enabled, |ui| {
                    toggle_button(
                        ui,
                        palette,
                        &text::channel_number(index),
                        is_selected(index),
                        Some(CHANNEL_BUTTON_WIDTH),
                    )
                })
                .inner;
            let _ = response.clone().accessible_name(label(index));
            if response.clicked() {
                clicked = Some(index);
            }
        }
    });
    clicked
}

fn input_section(
    ui: &mut egui::Ui,
    palette: &Palette,
    model: &ControlModel<'_>,
    is_mac: bool,
    actions: &mut Vec<Action>,
    changes: &mut ControlChanges,
) {
    section_header(ui, palette, text::SECTION_INPUT);
    let editable = !model.file_mode;
    let measurement_channel = model.stream.measurement_channel_index;
    let mut signal_channel = None;
    caption_row_with_width(
        ui,
        palette,
        text::SIGNAL_CHANNEL,
        CHANNEL_CAPTION_WIDTH,
        |ui| {
            signal_channel = channel_buttons(
                ui,
                palette,
                model.input_channels,
                |index| index == measurement_channel,
                text::signal_channel_label,
                editable,
            );
        },
    );
    if let Some(index) = signal_channel
        && index != measurement_channel
    {
        let mut stream = model.stream.clone();
        stream.measurement_channel_index = index;
        changes.commands.push(ControlCommand::ApplyStream(stream));
    }

    ui.add_space(spacing::XS);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = spacing::SM;
        field_caption(ui, palette, text::DELAY_OFFSET).on_hover_text(text::DELAY_OFFSET_HINT);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            confidence_segments(
                ui,
                palette,
                model.delay.confidence_db,
                text::DELAY_CONFIDENCE,
            );
        });
    });
    delay_value(ui, palette, model, changes);
    if let (Some(distance), Some(samples)) = (model.delay.distance_m, model.delay.samples) {
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            ui.label(
                RichText::new(text::delay_details(distance, samples))
                    .font(fonts::value(text_size::CAPTION))
                    .color(palette.text_muted),
            );
        });
    }
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = size::SEGMENT_GAP;
        for (action, glyph) in [
            (Action::NudgeDelayDown, text::BUTTON_NUDGE_DOWN),
            (Action::NudgeDelayUp, text::BUTTON_NUDGE_UP),
        ] {
            let response = ui
                .add_enabled_ui(model.running, |ui| {
                    toggle_button(ui, palette, glyph, false, Some(NUDGE_BUTTON_WIDTH))
                })
                .inner
                .on_hover_text(hint(action, is_mac));
            let _ = response.clone().accessible_name(action.label());
            if response.clicked() && model.running {
                actions.push(action);
            }
        }
        ui.add_space(spacing::SM);
        let remaining = ui.available_width() - size::SEGMENT_GAP;
        let half = remaining / 2.0;
        let peak = sized_action_button(
            ui,
            palette,
            text::BUTTON_FIND_DELAY,
            Action::FindDelay.shortcut_text(is_mac).as_deref(),
            Emphasis::Secondary,
            model.running,
            half,
        )
        .on_hover_text(Action::FindDelay.label());
        if peak.clicked() {
            actions.push(Action::FindDelay);
        }
        let auto = toggle_button(ui, palette, text::BUTTON_AUTO, model.tracking, Some(half))
            .on_hover_text(hint(Action::ToggleDelayTracking, is_mac));
        auto.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::Role::CheckBox,
                true,
                model.tracking,
                text::ACTION_TOGGLE_DELAY_TRACKING,
            )
        });
        if auto.clicked() {
            actions.push(Action::ToggleDelayTracking);
        }
    });

    ui.add_space(spacing::XS);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(size::SEGMENT_GAP, size::SEGMENT_GAP);
        caption_cell(ui, palette, text::REFERENCE_CHANNEL, CHANNEL_CAPTION_WIDTH);
        let internal = matches!(model.stream.reference, ReferenceMode::Internal);
        let out = ui
            .add_enabled_ui(editable, |ui| {
                toggle_button(ui, palette, text::REFERENCE_CHANNEL_OUT, internal, None)
            })
            .inner
            .on_hover_text(if editable {
                text::REFERENCE_CHANNEL_OUT_HINT
            } else {
                text::CHANNEL_FILE_MODE
            });
        if out.clicked() && !internal && editable {
            let mut stream = model.stream.clone();
            stream.reference = ReferenceMode::Internal;
            changes.commands.push(ControlCommand::ApplyStream(stream));
        }
        let loopback = match model.stream.reference {
            ReferenceMode::Loopback {
                reference_channel_index,
            } => Some(reference_channel_index),
            ReferenceMode::Internal => None,
        };
        if let Some(index) = channel_buttons(
            ui,
            palette,
            model.input_channels,
            |index| loopback == Some(index),
            text::reference_channel_label,
            editable,
        ) && loopback != Some(index)
        {
            let mut stream = model.stream.clone();
            stream.reference = ReferenceMode::Loopback {
                reference_channel_index: index,
            };
            changes.commands.push(ControlCommand::ApplyStream(stream));
        }
    });
    ui.add_space(spacing::XS);
    input_meters(ui, palette, model);
}

fn input_meters(ui: &mut egui::Ui, palette: &Palette, model: &ControlModel<'_>) {
    input_meter_row(ui, palette, text::LEVEL_SIGNAL, model.measurement_meter);
    input_meter_row(ui, palette, text::LEVEL_REFERENCE, model.reference_meter);
}

fn input_meter_row(ui: &mut egui::Ui, palette: &Palette, label: &str, reading: MeterReading) {
    let width = ui.available_width();
    let (rect, response) =
        ui.allocate_exact_size(vec2(width, INPUT_METER_ROW_HEIGHT), Sense::hover());
    let value_text = text::format_meter_dbfs(reading.peak_dbfs);
    let accessible = if reading.clipped {
        format!("{label} {value_text} {}", text::LEVEL_CLIP)
    } else {
        format!("{label} {value_text}")
    };
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Meter, true, &accessible));
    if !ui.is_rect_visible(rect) {
        return;
    }
    let painter = ui.painter_at(rect);
    let center_y = rect.center().y;
    painter.text(
        pos2(rect.left(), center_y),
        Align2::LEFT_CENTER,
        label,
        fonts::ui_medium(text_size::CAPTION),
        palette.text_muted,
    );
    let clip = Rect::from_center_size(
        pos2(rect.right() - INPUT_METER_CLIP_WIDTH / 2.0, center_y),
        vec2(INPUT_METER_CLIP_WIDTH, INPUT_METER_ROW_HEIGHT - spacing::XS),
    );
    painter.rect_filled(
        clip,
        CornerRadius::same(radius::SMALL),
        if reading.clipped {
            palette.danger
        } else {
            palette.surface_hover
        },
    );
    painter.text(
        clip.center(),
        Align2::CENTER_CENTER,
        text::LEVEL_CLIP,
        fonts::ui_semibold(text_size::CAPTION),
        if reading.clipped {
            palette.surface
        } else {
            palette.text_muted
        },
    );
    painter.text(
        pos2(clip.left() - spacing::XS, center_y),
        Align2::RIGHT_CENTER,
        &value_text,
        fonts::value(text_size::CAPTION),
        if reading.clipped {
            palette.danger
        } else {
            palette.text_primary
        },
    );
    let bar = Rect::from_min_max(
        pos2(
            rect.left() + INPUT_METER_CAPTION_WIDTH,
            center_y - INPUT_METER_HEIGHT / 2.0,
        ),
        pos2(
            clip.left() - spacing::XS - INPUT_METER_VALUE_WIDTH,
            center_y + INPUT_METER_HEIGHT / 2.0,
        ),
    );
    if bar.width() < INPUT_METER_MIN_BAR_WIDTH {
        return;
    }
    painter.rect_filled(
        bar,
        CornerRadius::same(radius::SMALL),
        palette.surface_hover,
    );
    let x_for = |dbfs: f32| {
        let fraction = ((dbfs - INPUT_METER_FLOOR_DBFS) / -INPUT_METER_FLOOR_DBFS).clamp(0.0, 1.0);
        bar.left() + fraction * bar.width()
    };
    let fill = if reading.clipped {
        palette.danger
    } else if reading.peak_dbfs > INPUT_METER_WARNING_DBFS {
        palette.warning
    } else {
        palette.success
    };
    painter.rect_filled(
        Rect::from_min_max(bar.left_top(), pos2(x_for(reading.rms_dbfs), bar.bottom())),
        CornerRadius::same(radius::SMALL),
        fill,
    );
    let peak_x = x_for(reading.peak_dbfs);
    painter.line_segment(
        [pos2(peak_x, bar.top()), pos2(peak_x, bar.bottom())],
        Stroke::new(stroke::TRACE_LIVE, fill),
    );
    for tick in INPUT_METER_TICKS_DBFS {
        let x = x_for(tick);
        painter.line_segment(
            [pos2(x, bar.top()), pos2(x, bar.bottom())],
            Stroke::new(stroke::BORDER, palette.surface),
        );
    }
}

fn delay_value(
    ui: &mut egui::Ui,
    palette: &Palette,
    model: &ControlModel<'_>,
    changes: &mut ControlChanges,
) {
    let style_name = TextStyle::Name(DELAY_TEXT_STYLE.into());
    ui.scope(|ui| {
        let style = ui.style_mut();
        style.text_styles.insert(
            style_name.clone(),
            fonts::value_semibold(text_size::DISPLAY),
        );
        style.drag_value_text_style = style_name;
        style.visuals.widgets.inactive.weak_bg_fill = palette.surface_raised;
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(stroke::BORDER, palette.border);
        let mut value = model.delay.milliseconds.unwrap_or(0.0);
        let width = ui.available_width();
        let response = ui
            .add_enabled_ui(model.running, |ui| {
                ui.add_sized(
                    vec2(width, DELAY_VALUE_HEIGHT),
                    egui::DragValue::new(&mut value)
                        .speed(DELAY_DRAG_SPEED_MS)
                        .range(-DELAY_LIMIT_MS..=DELAY_LIMIT_MS)
                        .fixed_decimals(2)
                        .custom_formatter(|value, _| text::format_decimal(value, 2))
                        .custom_parser(text::parse_number),
                )
            })
            .inner
            .on_hover_text(text::DELAY_OFFSET_HINT);
        spin_label(&response, text::DELAY_OFFSET);
        if response.changed() && model.running {
            changes.commands.push(ControlCommand::SetDelayMs(value));
        }
    });
}

fn analysis_row(
    ui: &mut egui::Ui,
    palette: &Palette,
    model: &ControlModel<'_>,
    is_mac: bool,
    actions: &mut Vec<Action>,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = spacing::XS;
        let width = ui.available_width() - FREEZE_BUTTON_WIDTH - spacing::XS;
        let (label, emphasis) = if model.running {
            (text::BUTTON_STOP_ANALYSIS, Emphasis::Active)
        } else {
            (text::BUTTON_START_ANALYSIS, Emphasis::Primary)
        };
        if sized_action_button(
            ui,
            palette,
            label,
            Action::ToggleMeasurement.shortcut_text(is_mac).as_deref(),
            emphasis,
            true,
            width,
        )
        .clicked()
        {
            actions.push(Action::ToggleMeasurement);
        }
        let freeze = if model.frozen {
            text::BUTTON_UNFREEZE
        } else {
            text::BUTTON_FREEZE
        };
        if sized_action_button(
            ui,
            palette,
            freeze,
            None,
            Emphasis::Secondary,
            model.running,
            FREEZE_BUTTON_WIDTH,
        )
        .on_hover_text(hint(Action::ToggleFreeze, is_mac))
        .clicked()
        {
            actions.push(Action::ToggleFreeze);
        }
    });
}

fn output_section(
    ui: &mut egui::Ui,
    palette: &Palette,
    model: &ControlModel<'_>,
    is_mac: bool,
    actions: &mut Vec<Action>,
    changes: &mut ControlChanges,
) {
    section_header(ui, palette, text::SECTION_OUTPUT);
    let editable = !model.file_mode;
    let width = ui.available_width();
    let play = ui
        .add_enabled_ui(editable, |ui| {
            toggle_button(
                ui,
                palette,
                text::BUTTON_PLAY_SIGNAL,
                model.stream.generator.enabled && editable,
                Some(width),
            )
        })
        .inner
        .on_hover_text(hint(Action::ToggleGenerator, is_mac));
    if play.clicked() && editable {
        actions.push(Action::ToggleGenerator);
    }
    caption_row(ui, palette, text::OUTPUT_PINK_NOISE, |ui| {
        let mut level = f64::from(model.stream.generator.level_dbfs);
        let gaps = spacing::XS + 2.0 * ui.spacing().item_spacing.x;
        ui.spacing_mut().slider_width =
            (ui.available_width() - GENERATOR_VALUE_WIDTH - gaps).max(0.0);
        let slider = ui
            .add_enabled(
                editable,
                egui::Slider::new(&mut level, GENERATOR_MIN_DBFS..=GENERATOR_MAX_DBFS)
                    .step_by(GENERATOR_SLIDER_STEP_DB)
                    .trailing_fill(true)
                    .show_value(false),
            )
            .on_hover_text(text::OUTPUT_LEVEL_HINT);
        spin_label(&slider, text::OUTPUT_LEVEL_LABEL);
        ui.add_space(spacing::XS);
        let value = ui
            .add_enabled_ui(editable, |ui| {
                ui.add_sized(
                    vec2(GENERATOR_VALUE_WIDTH, size::BUTTON_HEIGHT),
                    egui::DragValue::new(&mut level)
                        .speed(GENERATOR_DRAG_SPEED_DB)
                        .range(GENERATOR_MIN_DBFS..=GENERATOR_MAX_DBFS)
                        .custom_formatter(|value, _| text::format_level_db(value))
                        .custom_parser(text::parse_number),
                )
            })
            .inner;
        spin_label(&value, text::OUTPUT_LEVEL_VALUE_LABEL);
        if slider.changed() || value.changed() {
            changes
                .commands
                .push(ControlCommand::SetGeneratorLevel(level as f32));
        }
    });
    if model.output_channels > 0 {
        caption_row(ui, palette, text::PLAY_CHANNEL, |ui| {
            let selected = &model.stream.generator.output_channel_indices;
            if let Some(index) = channel_buttons(
                ui,
                palette,
                model.output_channels,
                |index| selected.contains(&index),
                text::play_channel_label,
                editable,
            ) {
                let mut stream = model.stream.clone();
                let channels = &mut stream.generator.output_channel_indices;
                if let Some(position) = channels.iter().position(|channel| *channel == index) {
                    if channels.len() > 1 {
                        channels.remove(position);
                    }
                } else {
                    channels.push(index);
                    channels.sort_unstable();
                }
                if stream != *model.stream {
                    changes.commands.push(ControlCommand::ApplyStream(stream));
                }
            }
        });
    }
}

fn parameters_section(
    ui: &mut egui::Ui,
    palette: &Palette,
    model: &ControlModel<'_>,
    analysis: &mut StoredAnalysis,
    changes: &mut ControlChanges,
) {
    section_header(ui, palette, text::SECTION_PARAMETERS);
    caption_row(ui, palette, text::FFT_SIZE, |ui| {
        let mut length = analysis.fft_length;
        let combo = egui::ComboBox::from_id_salt("fft-size")
            .width(ui.available_width() - spacing::XL)
            .selected_text(text::fft_size_choice(length, model.sample_rate_hz))
            .show_ui(ui, |ui| {
                for choice in FFT_LENGTH_CHOICES {
                    ui.selectable_value(
                        &mut length,
                        choice,
                        text::fft_size_choice(choice, model.sample_rate_hz),
                    );
                }
            });
        crate::design::widgets::combo_label(&combo.response, text::FFT_SIZE);
        if length != analysis.fft_length {
            analysis.fft_length = length;
            changes.analysis_changed = true;
        }
    });
    caption_row(ui, palette, text::AVERAGES, |ui| {
        let reset_width =
            crate::design::widgets::toggle_button_width(ui, text::BUTTON_RESET_AVERAGES);
        let combo_width = ui.available_width() - reset_width - spacing::XL - size::SEGMENT_GAP;
        let mut time_constant = analysis.time_constant_s;
        let combo = egui::ComboBox::from_id_salt("averages")
            .width(combo_width)
            .selected_text(text::averages_choice(time_constant))
            .show_ui(ui, |ui| {
                for choice in TIME_CONSTANT_CHOICES_S
                    .iter()
                    .copied()
                    .map(Some)
                    .chain(std::iter::once(None))
                {
                    ui.selectable_value(&mut time_constant, choice, text::averages_choice(choice));
                }
            });
        crate::design::widgets::combo_label(&combo.response, text::AVERAGES);
        if time_constant != analysis.time_constant_s {
            analysis.time_constant_s = time_constant;
            changes.analysis_changed = true;
        }
        let reset = ui
            .add_enabled_ui(model.running, |ui| {
                toggle_button(ui, palette, text::BUTTON_RESET_AVERAGES, false, None)
            })
            .inner
            .on_hover_text(text::RESET_AVERAGES_HINT);
        if reset.clicked() && model.running {
            changes.commands.push(ControlCommand::ResetAverages);
        }
    });
}

fn levels(ui: &mut egui::Ui, palette: &Palette, model: &ControlModel<'_>) {
    let height = ui.available_height().max(LIST_MIN_HEIGHT - spacing::XXL);
    ui.columns(2, |columns| {
        vertical_meter(
            &mut columns[0],
            palette,
            text::LEVEL_SIGNAL,
            model.measurement_meter,
            height,
        );
        vertical_meter(
            &mut columns[1],
            palette,
            text::LEVEL_REFERENCE,
            model.reference_meter,
            height,
        );
    });
}

fn vertical_meter(
    ui: &mut egui::Ui,
    palette: &Palette,
    label: &str,
    reading: MeterReading,
    height: f32,
) {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let accessible = format!(
        "{label} {} {}",
        text::LEVEL_PEAK,
        text::format_meter_dbfs(reading.peak_dbfs)
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Meter, true, &accessible));
    let painter = ui.painter_at(rect);
    let caption = fonts::ui_medium(text_size::CAPTION);
    let value = fonts::value_semibold(text_size::VALUE);
    painter.text(
        rect.left_top(),
        Align2::LEFT_TOP,
        text::LEVEL_UNIT,
        caption.clone(),
        palette.text_muted,
    );
    let mut y = rect.top() + text_size::CAPTION + spacing::SM;
    for (name, level) in [
        (text::LEVEL_PEAK, reading.peak_dbfs),
        (text::LEVEL_RMS, reading.rms_dbfs),
    ] {
        painter.text(
            pos2(rect.left(), y),
            Align2::LEFT_TOP,
            name,
            caption.clone(),
            palette.text_muted,
        );
        painter.text(
            pos2(rect.left(), y + text_size::CAPTION + size::SEGMENT_GAP),
            Align2::LEFT_TOP,
            text::format_meter_dbfs(level),
            value.clone(),
            if reading.clipped && name == text::LEVEL_PEAK {
                palette.danger
            } else {
                palette.text_primary
            },
        );
        y += text_size::CAPTION + text_size::VALUE + spacing::SM;
    }
    painter.text(
        pos2(rect.left(), rect.bottom()),
        Align2::LEFT_BOTTOM,
        label,
        fonts::ui_semibold(text_size::LABEL),
        palette.text_primary,
    );
    let bar = Rect::from_min_max(
        pos2(
            rect.right() - METER_BAR_WIDTH - spacing::SM,
            rect.top() + spacing::XS,
        ),
        pos2(
            rect.right() - spacing::SM,
            rect.bottom() - text_size::LABEL - spacing::SM,
        ),
    );
    painter.rect_filled(
        bar,
        CornerRadius::same(radius::SMALL),
        palette.surface_hover,
    );
    let level_y = |dbfs: f32| {
        let fraction = ((dbfs + METER_RANGE_DBFS) / METER_RANGE_DBFS).clamp(0.0, 1.0);
        bar.bottom() - fraction * bar.height()
    };
    let fill = if reading.clipped {
        palette.danger
    } else if reading.peak_dbfs > METER_WARNING_DBFS {
        palette.warning
    } else {
        palette.success
    };
    painter.rect_filled(
        Rect::from_min_max(
            pos2(bar.left(), level_y(reading.rms_dbfs)),
            bar.right_bottom(),
        ),
        CornerRadius::same(radius::SMALL),
        fill,
    );
    painter.line_segment(
        [
            pos2(bar.left(), level_y(reading.peak_dbfs)),
            pos2(bar.right(), level_y(reading.peak_dbfs)),
        ],
        Stroke::new(stroke::FOCUS, fill),
    );
    for tick in METER_TICKS_DBFS {
        let y = level_y(tick);
        painter.text(
            pos2(bar.left() - spacing::XS, y),
            Align2::RIGHT_CENTER,
            text::format_decimal(f64::from(tick), 0),
            fonts::value(text_size::CAPTION),
            palette.text_muted,
        );
    }
}

fn hint(action: Action, is_mac: bool) -> String {
    text::with_shortcut(action.label(), action.shortcut_text(is_mac).as_deref())
}
