use dsp_core::MAX_EQ_FILTERS;
use eframe::egui::{self, RichText};

use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{radius, spacing, stroke, text_size};
use crate::design::widgets::{Emphasis, action_button, muted};
use crate::eq_state::{EqState, EqTarget, EqView, StoredFilter, StoredFilterKind, StoredSlope};
use crate::session::Session;
use crate::text;

const MIN_FREQUENCY_HZ: f64 = 20.0;
const MAX_FREQUENCY_HZ: f64 = 20_000.0;
const GAIN_LIMIT_DB: f64 = 24.0;
const MIN_Q: f64 = 0.1;
const MAX_Q: f64 = 20.0;
const NEW_FILTER_FREQUENCY_HZ: f64 = 1000.0;

#[derive(Clone, Debug, PartialEq)]
pub enum EqCommand {
    Update(EqState),
    AddFilter,
    ExportText,
    ExportJson,
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
    ui.add_space(spacing::MD);
    ui.horizontal(|ui| {
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
    ui.add_space(spacing::SM);
    ui.horizontal(|ui| {
        for (view, label) in [
            (EqView::Predicted, text::EQ_VIEW_PREDICTED),
            (EqView::EqOnly, text::EQ_VIEW_ONLY),
            (EqView::InvertedTrace, text::EQ_VIEW_INVERTED),
        ] {
            if ui
                .add(egui::Button::selectable(state.view == view, label))
                .clicked()
            {
                state.view = view;
            }
        }
    });
    ui.add_space(spacing::SM);
    let target_name = match state.target {
        EqTarget::Live => text::EQ_TARGET_LIVE.to_owned(),
        EqTarget::Trace(id) => session.trace(id).map_or_else(
            || text::EQ_TARGET_LIVE.to_owned(),
            |trace| trace.name.clone(),
        ),
    };
    egui::ComboBox::from_id_salt("eq-target")
        .width(ui.available_width())
        .selected_text(target_name)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut state.target, EqTarget::Live, text::EQ_TARGET_LIVE);
            for trace in &session.traces {
                ui.selectable_value(&mut state.target, EqTarget::Trace(trace.id), &trace.name);
            }
        });

    ui.add_space(spacing::MD);
    let filter_count = state.filters().len();
    if filter_count == 0 {
        muted(ui, palette, text::EMPTY_EQ_BODY);
    }
    let mut remove = None;
    egui::ScrollArea::vertical()
        .max_height(ui.available_height() - 140.0)
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
    if let Some(index) = remove {
        state.remove_filter(index);
    }

    ui.add_space(spacing::SM);
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
    ui.add_space(spacing::XS);
    ui.horizontal(|ui| {
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

    if state != session.eq {
        commands.insert(0, EqCommand::Update(state));
    }
    commands
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
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(text::filter_number(index + 1))
                        .font(fonts::value_medium(text_size::LABEL))
                        .color(palette.text_muted),
                );
                egui::ComboBox::from_id_salt(("filter-kind", index))
                    .selected_text(text::filter_kind_label(filter.kind))
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
                ui.checkbox(&mut filter.bypassed, text::EQ_BYPASS);
                remove = ui
                    .button(text::TRACE_DELETE_GLYPH)
                    .on_hover_text(text::EQ_REMOVE)
                    .clicked();
            });
            ui.horizontal(|ui| {
                ui.add(
                    egui::DragValue::new(&mut filter.frequency_hz)
                        .range(MIN_FREQUENCY_HZ..=MAX_FREQUENCY_HZ)
                        .speed(1.0)
                        .custom_formatter(|value, _| text::format_frequency(value)),
                );
                if filter.kind.to_dsp().uses_gain() {
                    ui.add(
                        egui::DragValue::new(&mut filter.gain_db)
                            .range(-GAIN_LIMIT_DB..=GAIN_LIMIT_DB)
                            .speed(0.1)
                            .custom_formatter(|value, _| text::format_db(value)),
                    );
                }
                if filter.kind.to_dsp().uses_q() {
                    ui.add(
                        egui::DragValue::new(&mut filter.q)
                            .range(MIN_Q..=MAX_Q)
                            .speed(0.01)
                            .custom_formatter(|value, _| text::format_q(value)),
                    );
                }
                if let StoredFilterKind::LowCut(slope) | StoredFilterKind::HighCut(slope) =
                    &mut filter.kind
                {
                    egui::ComboBox::from_id_salt(("filter-slope", index))
                        .selected_text(slope.label())
                        .show_ui(ui, |ui| {
                            for option in StoredSlope::ALL {
                                ui.selectable_value(slope, option, option.label());
                            }
                        });
                }
            });
            if let Some(octaves) = filter.bandwidth_octaves() {
                ui.label(
                    RichText::new(text::format_octaves(octaves))
                        .font(fonts::value(text_size::CAPTION))
                        .color(palette.text_muted),
                );
            }
        });
    remove
}
