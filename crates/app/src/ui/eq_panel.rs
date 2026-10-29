use dsp_core::MAX_EQ_FILTERS;
use eframe::egui::{self, RichText};

use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{radius, spacing, stroke, text_size};
use crate::design::widgets::{
    Emphasis, action_button, combo_label, muted, section_title, spin_label,
};
use crate::eq_state::{
    AUTO_EQ_MAX_BOOST_LIMIT_DB, AUTO_EQ_MAX_CUT_LIMIT_DB, EqState, EqTarget, EqView, GAIN_LIMIT_DB,
    MAX_FREQUENCY_HZ, MAX_Q, MIN_FREQUENCY_HZ, MIN_Q, StoredFilter, StoredFilterKind, StoredSlope,
};
use crate::session::Session;
use crate::text;

const NEW_FILTER_FREQUENCY_HZ: f64 = 1000.0;
const KIND_COMBO_WIDTH: f32 = 120.0;
const SLOPE_COMBO_WIDTH: f32 = 150.0;
const SETTINGS_COLUMN_FRACTION: f32 = 0.38;
const SETTINGS_MIN_WIDTH: f32 = 240.0;

#[derive(Clone, Debug, PartialEq)]
pub enum EqCommand {
    Update(EqState),
    AddFilter,
    ExportText,
    ExportJson,
    AutoEq,
}

pub fn new_filter_at(frequency_hz: f64, gain_db: f64) -> StoredFilter {
    StoredFilter::new(
        StoredFilterKind::Bell,
        frequency_hz.clamp(MIN_FREQUENCY_HZ, MAX_FREQUENCY_HZ),
        gain_db.clamp(-GAIN_LIMIT_DB, GAIN_LIMIT_DB),
    )
}

pub fn default_new_filter() -> StoredFilter {
    new_filter_at(NEW_FILTER_FREQUENCY_HZ, 0.0)
}

pub fn show(ui: &mut egui::Ui, palette: &Palette, session: &Session) -> Vec<EqCommand> {
    let mut commands = Vec::new();
    let mut state = session.eq.clone();
    let left_width = (ui.available_width() * SETTINGS_COLUMN_FRACTION).max(SETTINGS_MIN_WIDTH);
    let height = ui.available_height();
    let mut remove = None;
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = spacing::LG;
        ui.allocate_ui_with_layout(
            egui::vec2(left_width, height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("eq-settings")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        settings_column(ui, palette, session, &mut state, &mut commands);
                    });
            },
        );
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                if state.filters().is_empty() {
                    section_title(ui, palette, text::EMPTY_EQ_TITLE);
                    muted(ui, palette, text::EMPTY_EQ_BODY);
                }
                egui::ScrollArea::vertical()
                    .id_salt("eq-filters")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if let Some(filters) = state.banks.get_mut(state.active_bank) {
                            for (index, filter) in filters.iter_mut().enumerate() {
                                if filter_editor(
                                    ui,
                                    palette,
                                    index,
                                    filter,
                                    state.selected_filter == Some(index),
                                ) {
                                    remove = Some(index);
                                }
                                ui.add_space(spacing::SM);
                            }
                        }
                    });
            },
        );
    });
    if let Some(index) = remove {
        state.remove_filter(index);
    }
    if state != session.eq {
        commands.insert(0, EqCommand::Update(state));
    }
    commands
}

fn settings_column(
    ui: &mut egui::Ui,
    palette: &Palette,
    session: &Session,
    state: &mut EqState,
    commands: &mut Vec<EqCommand>,
) {
    ui.horizontal_wrapped(|ui| {
        for bank in 0..2 {
            if ui
                .add(egui::Button::selectable(
                    state.active_bank == bank,
                    text::eq_bank_label(bank),
                ))
                .clicked()
                && state.active_bank != bank
            {
                state.switch_bank();
            }
        }
        ui.checkbox(&mut state.global_bypass, text::EQ_GLOBAL_BYPASS);
    });
    ui.horizontal_wrapped(|ui| {
        for (view, label) in [
            (EqView::Predicted, text::EQ_VIEW_PREDICTED),
            (EqView::EqOnly, text::EQ_VIEW_ONLY),
            (EqView::InvertedTrace, text::EQ_VIEW_INVERTED_SHORT),
        ] {
            if ui
                .add(egui::Button::selectable(state.view == view, label))
                .clicked()
            {
                state.view = view;
            }
        }
    });
    let target_name = match state.target {
        EqTarget::Live => text::EQ_TARGET_LIVE.to_owned(),
        EqTarget::Trace(id) => session.trace(id).map_or_else(
            || text::EQ_TARGET_LIVE.to_owned(),
            |trace| trace.name.clone(),
        ),
    };
    let combo = egui::ComboBox::from_id_salt("eq-target")
        .width(ui.available_width() - spacing::SM)
        .truncate()
        .selected_text(target_name)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut state.target, EqTarget::Live, text::EQ_TARGET_LIVE);
            for trace in &session.traces {
                ui.selectable_value(&mut state.target, EqTarget::Trace(trace.id), &trace.name);
            }
        });
    combo_label(&combo.response, text::EQ_TARGET_LABEL);
    let filter_count = state.filters().len();
    ui.horizontal_wrapped(|ui| {
        if action_button(
            ui,
            palette,
            text::ACTION_ADD_EQ_FILTER,
            crate::actions::Action::AddEqFilter
                .shortcut_text(cfg!(target_os = "macos"))
                .as_deref(),
            Emphasis::Secondary,
            filter_count < MAX_EQ_FILTERS,
        )
        .clicked()
        {
            commands.push(EqCommand::AddFilter);
        }
        if action_button(
            ui,
            palette,
            text::EQ_EXPORT_TEXT,
            None,
            Emphasis::Quiet,
            filter_count > 0,
        )
        .clicked()
        {
            commands.push(EqCommand::ExportText);
        }
        if action_button(
            ui,
            palette,
            text::EQ_EXPORT_JSON,
            None,
            Emphasis::Quiet,
            filter_count > 0,
        )
        .clicked()
        {
            commands.push(EqCommand::ExportJson);
        }
    });
    if auto_eq_section(ui, palette, state) {
        commands.push(EqCommand::AutoEq);
    }
}

fn auto_eq_section(ui: &mut egui::Ui, palette: &Palette, state: &mut EqState) -> bool {
    let mut run = false;
    egui::CollapsingHeader::new(
        RichText::new(text::AUTO_EQ_TITLE).font(fonts::ui_semibold(text_size::LABEL)),
    )
    .id_salt("auto-eq")
    .default_open(false)
    .show(ui, |ui| {
        let auto = &mut state.auto;
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(text::AUTO_EQ_RANGE).color(palette.text_muted));
            let high = auto.high_hz;
            let response = ui.add(
                egui::DragValue::new(&mut auto.low_hz)
                    .range(MIN_FREQUENCY_HZ..=high)
                    .speed(1.0)
                    .custom_formatter(|value, _| text::format_frequency(value))
                    .custom_parser(text::parse_number),
            );
            spin_label(&response, text::AUTO_EQ_RANGE_LOW);
            ui.label(RichText::new(text::RANGE_SEPARATOR).color(palette.text_muted));
            let low = auto.low_hz;
            let response = ui.add(
                egui::DragValue::new(&mut auto.high_hz)
                    .range(low..=MAX_FREQUENCY_HZ)
                    .speed(10.0)
                    .custom_formatter(|value, _| text::format_frequency(value))
                    .custom_parser(text::parse_number),
            );
            spin_label(&response, text::AUTO_EQ_RANGE_HIGH);
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(text::AUTO_EQ_MAX_BOOST).color(palette.text_muted));
            let response = ui.add(
                egui::DragValue::new(&mut auto.max_boost_db)
                    .range(0.0..=AUTO_EQ_MAX_BOOST_LIMIT_DB)
                    .speed(0.1)
                    .custom_formatter(|value, _| text::format_db(value))
                    .custom_parser(text::parse_number),
            );
            spin_label(&response, text::AUTO_EQ_MAX_BOOST);
            ui.label(RichText::new(text::AUTO_EQ_MAX_CUT).color(palette.text_muted));
            let response = ui.add(
                egui::DragValue::new(&mut auto.max_cut_db)
                    .range(0.0..=AUTO_EQ_MAX_CUT_LIMIT_DB)
                    .speed(0.1)
                    .custom_formatter(|value, _| text::format_db(-value))
                    .custom_parser(|input| text::parse_number(input).map(f64::abs)),
            );
            spin_label(&response, text::AUTO_EQ_MAX_CUT);
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(text::AUTO_EQ_MAX_FILTERS).color(palette.text_muted));
            let response = ui.add(
                egui::DragValue::new(&mut auto.max_filters)
                    .range(1..=MAX_EQ_FILTERS)
                    .speed(0.1),
            );
            spin_label(&response, text::AUTO_EQ_MAX_FILTERS);
        });
        muted(ui, palette, text::AUTO_EQ_HINT);
        ui.add_space(spacing::XS);
        run = action_button(
            ui,
            palette,
            text::AUTO_EQ_RUN,
            None,
            Emphasis::Primary,
            true,
        )
        .clicked();
    });
    run
}

fn filter_editor(
    ui: &mut egui::Ui,
    palette: &Palette,
    index: usize,
    filter: &mut StoredFilter,
    selected: bool,
) -> bool {
    let mut remove = false;
    egui::Frame::new()
        .fill(if selected {
            palette.accent.gamma_multiply(0.12)
        } else {
            palette.surface_raised
        })
        .stroke(egui::Stroke::new(
            stroke::BORDER,
            if selected {
                palette.accent
            } else {
                palette.border
            },
        ))
        .corner_radius(egui::CornerRadius::same(radius::MEDIUM))
        .inner_margin(egui::Margin::same(spacing::SM as i8))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(text::filter_number(index + 1))
                        .font(fonts::value_medium(text_size::LABEL))
                        .color(palette.text_muted),
                );
                let combo = egui::ComboBox::from_id_salt(("filter-kind", index))
                    .width(KIND_COMBO_WIDTH)
                    .truncate()
                    .selected_text(filter.kind.label())
                    .show_ui(ui, |ui| {
                        for kind in StoredFilterKind::MENU {
                            let current_slope = match filter.kind {
                                StoredFilterKind::LowCut(slope)
                                | StoredFilterKind::HighCut(slope) => Some(slope),
                                _ => None,
                            };
                            let kind = match (kind, current_slope) {
                                (StoredFilterKind::LowCut(_), Some(slope)) => {
                                    StoredFilterKind::LowCut(slope)
                                }
                                (StoredFilterKind::HighCut(_), Some(slope)) => {
                                    StoredFilterKind::HighCut(slope)
                                }
                                (other, _) => other,
                            };
                            if ui
                                .selectable_label(filter.kind.same_family(kind), kind.label())
                                .clicked()
                            {
                                *filter = StoredFilter {
                                    bypassed: filter.bypassed,
                                    ..StoredFilter::new(kind, filter.frequency_hz, filter.gain_db)
                                };
                            }
                        }
                    });
                combo_label(
                    &combo.response,
                    text::eq_control_label(text::EQ_KIND, index + 1),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    remove = ui
                        .add(egui::Button::new(text::TRACE_DELETE_GLYPH).frame(false))
                        .on_hover_text(text::EQ_REMOVE)
                        .clicked();
                });
            });
            ui.horizontal_wrapped(|ui| {
                let response = ui.add(
                    egui::DragValue::new(&mut filter.frequency_hz)
                        .range(MIN_FREQUENCY_HZ..=MAX_FREQUENCY_HZ)
                        .speed(1.0)
                        .custom_formatter(|value, _| text::format_frequency(value))
                        .custom_parser(text::parse_number),
                );
                spin_label(
                    &response,
                    text::eq_control_label(text::EQ_FREQUENCY, index + 1),
                );
                if filter.kind.to_dsp().uses_gain() {
                    let response = ui.add(
                        egui::DragValue::new(&mut filter.gain_db)
                            .range(-GAIN_LIMIT_DB..=GAIN_LIMIT_DB)
                            .speed(0.1)
                            .custom_formatter(|value, _| text::format_db(value))
                            .custom_parser(text::parse_number),
                    );
                    spin_label(&response, text::eq_control_label(text::EQ_GAIN, index + 1));
                }
            });
            if let StoredFilterKind::LowCut(slope) | StoredFilterKind::HighCut(slope) =
                &mut filter.kind
            {
                let combo = egui::ComboBox::from_id_salt(("filter-slope", index))
                    .width(SLOPE_COMBO_WIDTH)
                    .truncate()
                    .selected_text(slope.label())
                    .show_ui(ui, |ui| {
                        for option in StoredSlope::ALL {
                            ui.selectable_value(slope, option, option.label());
                        }
                    });
                combo_label(
                    &combo.response,
                    text::eq_control_label(text::EQ_SLOPE, index + 1),
                );
            }
            ui.horizontal_wrapped(|ui| {
                if filter.kind.to_dsp().uses_q() {
                    let response = ui.add(
                        egui::DragValue::new(&mut filter.q)
                            .range(MIN_Q..=MAX_Q)
                            .speed(0.01)
                            .custom_formatter(|value, _| text::format_q(value))
                            .custom_parser(text::parse_number),
                    );
                    spin_label(&response, text::eq_control_label(text::EQ_Q, index + 1));
                }
                if let Some(octaves) = filter.bandwidth_octaves() {
                    ui.label(
                        RichText::new(text::format_octaves(octaves))
                            .font(fonts::value(text_size::CAPTION))
                            .color(palette.text_muted),
                    );
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut filter.bypassed, text::EQ_BYPASS);
                });
            });
        });
    remove
}
