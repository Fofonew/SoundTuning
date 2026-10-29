use eframe::egui::{
    self, Align, CornerRadius, Layout, RichText, Sense, Stroke, StrokeKind, pos2, vec2,
};

use crate::alignment_state::{
    AlignmentChoice, AlignmentRange, AlignmentState, BAND_LIMIT_HIGH_HZ, BAND_LIMIT_LOW_HZ,
    MAX_BANDWIDTH_OCTAVES, MIN_BANDWIDTH_OCTAVES, SEARCH_RANGE_MS,
};
use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{radius, size, spacing, stroke, text_size};
use crate::design::widgets::{combo_label, field_caption, muted, spin_label, toggle_button};
use crate::session::Session;
use crate::text;

const DELAY_STEP_MS: f64 = 0.01;
const ROW_CAPTION_WIDTH: f32 = 76.0;
const CROSSOVER_DRAG_SPEED: f64 = 1.0;
const BANDWIDTH_DRAG_SPEED: f64 = 0.05;
const POWER_BAR_HEIGHT: f32 = 4.0;
const RANGE_BUTTON_WIDTH: f32 = 56.0;
const POLARITY_BOX: f32 = 14.0;
const CHOICE_MIN_WIDTH: f32 = 56.0;
const LEFT_COLUMN_FRACTION: f32 = 0.44;
pub const CONTROLS_HEIGHT: f32 = 132.0;

#[derive(Clone, Debug, PartialEq)]
pub enum AlignmentCommand {
    Update(AlignmentState),
}

pub fn choice_color(palette: &Palette, choice: AlignmentChoice) -> egui::Color32 {
    let index = AlignmentChoice::ALL
        .iter()
        .position(|candidate| *candidate == choice)
        .unwrap_or(0);
    palette.alignment_sums[index % palette.alignment_sums.len()]
}

pub fn show(ui: &mut egui::Ui, palette: &Palette, session: &Session) -> Vec<AlignmentCommand> {
    let mut commands = Vec::new();
    let mut state = session.alignment.clone();
    if session.traces.len() < 2 {
        ui.add_space(spacing::SM);
        field_caption(ui, palette, text::EMPTY_ALIGNMENT_TITLE);
        muted(ui, palette, text::EMPTY_ALIGNMENT_BODY);
        return commands;
    }
    let width = ui.available_width();
    let left_width = (width * LEFT_COLUMN_FRACTION).max(0.0);
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = spacing::LG;
        ui.allocate_ui_with_layout(
            vec2(left_width, ui.available_height()),
            Layout::top_down(Align::Min),
            |ui| {
                ui.spacing_mut().item_spacing.y = spacing::XS;
                range_controls(ui, palette, &mut state);
                trace_pickers(ui, palette, session, &mut state);
                if state.is_ready() {
                    applied_setting(ui, palette, session, &state);
                }
            },
        );
        ui.allocate_ui_with_layout(
            vec2(ui.available_width(), ui.available_height()),
            Layout::top_down(Align::Min),
            |ui| {
                ui.spacing_mut().item_spacing.y = spacing::XS;
                if state.is_ready() {
                    solutions(ui, palette, &mut state);
                }
            },
        );
    });
    if state != session.alignment {
        commands.push(AlignmentCommand::Update(state));
    }
    commands
}

fn caption_cell(ui: &mut egui::Ui, palette: &Palette, caption: &str) {
    crate::design::widgets::caption_cell(ui, palette, caption, ROW_CAPTION_WIDTH);
}

fn trace_pickers(
    ui: &mut egui::Ui,
    palette: &Palette,
    session: &Session,
    state: &mut AlignmentState,
) {
    for (label, picker, slot, id_salt) in [
        (
            text::ALIGNMENT_DELAYED,
            text::ALIGNMENT_DELAY_PICKER,
            &mut state.delayed_trace,
            "alignment-delayed",
        ),
        (
            text::ALIGNMENT_REFERENCE,
            text::ALIGNMENT_REFERENCE_PICKER,
            &mut state.reference_trace,
            "alignment-reference",
        ),
    ] {
        ui.horizontal(|ui| {
            caption_cell(ui, palette, picker);
            let selected_name = slot
                .and_then(|id| session.trace(id))
                .map_or(text::ALIGNMENT_CHOOSE_TRACE, |trace| trace.name.as_str());
            let combo = egui::ComboBox::from_id_salt(id_salt)
                .width(ui.available_width() - spacing::SM)
                .truncate()
                .selected_text(selected_name)
                .show_ui(ui, |ui| {
                    for trace in &session.traces {
                        ui.selectable_value(slot, Some(trace.id), &trace.name);
                    }
                });
            combo_label(&combo.response, label);
        });
    }
}

fn range_controls(ui: &mut egui::Ui, palette: &Palette, state: &mut AlignmentState) {
    ui.horizontal(|ui| {
        caption_cell(ui, palette, text::ALIGNMENT_RANGE);
        ui.spacing_mut().item_spacing.x = size::SEGMENT_GAP;
        for range in AlignmentRange::ALL {
            let response = toggle_button(
                ui,
                palette,
                text::alignment_range(range),
                state.range == range,
                Some(RANGE_BUTTON_WIDTH),
            );
            let _ = response
                .clone()
                .accessible_name(text::alignment_range_label(range));
            if response.clicked() {
                state.range = range;
            }
        }
    });
    if state.range != AlignmentRange::Custom {
        return;
    }
    ui.horizontal(|ui| {
        ui.add_space(ROW_CAPTION_WIDTH + ui.spacing().item_spacing.x);
        {
            let response = ui.add(
                egui::DragValue::new(&mut state.crossover_hz)
                    .range(BAND_LIMIT_LOW_HZ..=BAND_LIMIT_HIGH_HZ)
                    .speed(CROSSOVER_DRAG_SPEED)
                    .custom_formatter(|value, _| text::format_frequency(value))
                    .custom_parser(text::parse_number),
            );
            spin_label(&response, text::ALIGNMENT_CROSSOVER_LABEL);
            let response = ui.add(
                egui::DragValue::new(&mut state.bandwidth_octaves)
                    .range(MIN_BANDWIDTH_OCTAVES..=MAX_BANDWIDTH_OCTAVES)
                    .speed(BANDWIDTH_DRAG_SPEED)
                    .custom_formatter(|value, _| text::format_octaves(value))
                    .custom_parser(text::parse_number),
            );
            spin_label(&response, text::ALIGNMENT_BANDWIDTH_LABEL);
        }
        let (low_hz, high_hz) = state.band_hz();
        field_caption(ui, palette, &text::band_label(low_hz, high_hz))
            .on_hover_text(text::ALIGNMENT_BAND_HINT);
    });
}

fn solutions(ui: &mut egui::Ui, palette: &Palette, state: &mut AlignmentState) {
    let found = state.solutions;
    let columns = AlignmentChoice::ALL.len() as f32;
    let cell_width = ((ui.available_width() - ROW_CAPTION_WIDTH - columns * size::SEGMENT_GAP)
        / columns)
        .max(CHOICE_MIN_WIDTH);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = size::SEGMENT_GAP;
        ui.add_space(ROW_CAPTION_WIDTH);
        for choice in AlignmentChoice::ALL {
            let color = choice_color(palette, choice);
            let selected = state.choice == choice;
            let (rect, response) =
                ui.allocate_exact_size(vec2(cell_width, size::MIN_CLICK_HEIGHT), Sense::click());
            response.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::Role::Button,
                    true,
                    selected,
                    text::alignment_choice(choice),
                )
            });
            ui.painter().rect(
                rect,
                CornerRadius::same(radius::SMALL),
                if selected {
                    color
                } else {
                    color.gamma_multiply(0.16)
                },
                Stroke::new(
                    stroke::BORDER,
                    if response.hovered() || selected {
                        color
                    } else {
                        color.gamma_multiply(0.5)
                    },
                ),
                StrokeKind::Inside,
            );
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text::alignment_choice(choice),
                fonts::ui_semibold(text_size::LABEL),
                if selected {
                    palette.background
                } else {
                    palette.text_primary
                },
            );
            let response = response.on_hover_text(text::alignment_choice_hint(
                choice,
                state.setting(choice),
                selected && state.shows_sum(),
            ));
            if response.clicked() {
                state.select(choice);
            }
        }
    });
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = size::SEGMENT_GAP;
        caption_cell(ui, palette, text::ALIGNMENT_DELAY_MS);
        for choice in AlignmentChoice::ALL {
            if choice == AlignmentChoice::Custom {
                let before = state.custom_delay_ms;
                let response = ui.add_sized(
                    vec2(cell_width, size::MIN_CLICK_HEIGHT),
                    egui::DragValue::new(&mut state.custom_delay_ms)
                        .speed(DELAY_STEP_MS)
                        .range(-SEARCH_RANGE_MS..=SEARCH_RANGE_MS)
                        .fixed_decimals(2)
                        .custom_formatter(|value, _| text::format_decimal(value, 2))
                        .custom_parser(text::parse_number),
                );
                spin_label(&response, text::ALIGNMENT_CUSTOM_DELAY_LABEL);
                if before != state.custom_delay_ms {
                    state.choice = AlignmentChoice::Custom;
                    state.sum_hidden = false;
                }
                continue;
            }
            let value = state.setting(choice).map_or_else(
                || text::DELAY_NONE.to_owned(),
                |(delay_ms, _)| text::format_decimal(delay_ms, 2),
            );
            value_cell(ui, palette, cell_width, &value);
        }
    });
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = size::SEGMENT_GAP;
        caption_cell(ui, palette, text::ALIGNMENT_INV_POL);
        for choice in AlignmentChoice::ALL {
            let (rect, response) = ui.allocate_exact_size(
                vec2(cell_width, size::MIN_CLICK_HEIGHT),
                if choice == AlignmentChoice::Custom {
                    Sense::click()
                } else {
                    Sense::hover()
                },
            );
            let inverted = match choice {
                AlignmentChoice::Custom => Some(state.custom_inverted),
                other => state.setting(other).map(|(_, inverted)| inverted),
            };
            if choice == AlignmentChoice::Custom {
                response.widget_info(|| {
                    egui::WidgetInfo::selected(
                        egui::Role::CheckBox,
                        true,
                        state.custom_inverted,
                        text::ALIGNMENT_CUSTOM_POLARITY_LABEL,
                    )
                });
                if response.clicked() {
                    state.custom_inverted = !state.custom_inverted;
                    state.choice = AlignmentChoice::Custom;
                    state.sum_hidden = false;
                }
            }
            polarity_box(
                ui,
                palette,
                rect,
                inverted,
                choice == AlignmentChoice::Custom && response.hovered(),
            );
        }
    });
    if let Some(found) = found {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = size::SEGMENT_GAP;
            ui.add_space(ROW_CAPTION_WIDTH);
            for choice in AlignmentChoice::ALL {
                let (rect, response) =
                    ui.allocate_exact_size(vec2(cell_width, POWER_BAR_HEIGHT), Sense::hover());
                if let Some(fraction) = found.relative_power(choice) {
                    let color = choice_color(palette, choice);
                    ui.painter().rect_filled(
                        rect,
                        CornerRadius::same(1),
                        color.gamma_multiply(0.2),
                    );
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(
                            rect.min,
                            vec2(rect.width() * fraction as f32, rect.height()),
                        ),
                        CornerRadius::same(1),
                        color,
                    );
                    response.on_hover_text(text::ALIGNMENT_POWER_HINT);
                }
            }
        });
    } else {
        muted(ui, palette, text::ALIGNMENT_NO_SOLUTION);
    }
}

fn polarity_box(
    ui: &egui::Ui,
    palette: &Palette,
    cell: egui::Rect,
    inverted: Option<bool>,
    hovered: bool,
) {
    let Some(inverted) = inverted else {
        ui.painter().text(
            cell.center(),
            egui::Align2::CENTER_CENTER,
            text::DELAY_NONE,
            fonts::value(text_size::CAPTION),
            palette.text_muted,
        );
        return;
    };
    let rect = egui::Rect::from_center_size(cell.center(), vec2(POLARITY_BOX, POLARITY_BOX));
    ui.painter().rect(
        rect,
        CornerRadius::same(2),
        if inverted {
            palette.accent
        } else {
            egui::Color32::TRANSPARENT
        },
        Stroke::new(
            stroke::BORDER,
            if hovered || inverted {
                palette.accent
            } else {
                palette.text_muted
            },
        ),
        StrokeKind::Inside,
    );
    if inverted {
        let check = Stroke::new(stroke::FOCUS, palette.background);
        let left = pos2(rect.left() + 3.0, rect.center().y);
        let bottom = pos2(rect.center().x - 1.0, rect.bottom() - 3.5);
        let right = pos2(rect.right() - 3.0, rect.top() + 3.5);
        ui.painter().line_segment([left, bottom], check);
        ui.painter().line_segment([bottom, right], check);
    }
}

fn value_cell(ui: &mut egui::Ui, palette: &Palette, width: f32, value: &str) {
    ui.add_sized(
        vec2(width, size::MIN_CLICK_HEIGHT),
        egui::Label::new(
            RichText::new(value)
                .font(fonts::value_medium(text_size::VALUE))
                .color(palette.text_primary),
        ),
    );
}

fn applied_setting(
    ui: &mut egui::Ui,
    palette: &Palette,
    session: &Session,
    state: &AlignmentState,
) {
    let name = |id: Option<u64>| {
        id.and_then(|id| session.trace(id))
            .map_or_else(String::new, |trace| trace.name.clone())
    };
    let instruction = state.selected_setting().map_or_else(
        || text::DELAY_NONE.to_owned(),
        |(delay_ms, inverted)| {
            text::alignment_instruction(
                delay_ms,
                inverted,
                &name(state.reference_trace),
                &name(state.delayed_trace),
            )
        },
    );
    ui.horizontal(|ui| {
        caption_cell(ui, palette, text::ALIGNMENT_APPLIED_SHORT);
        ui.add(
            egui::Label::new(
                RichText::new(instruction)
                    .font(fonts::value_semibold(text_size::VALUE))
                    .color(choice_color(palette, state.choice)),
            )
            .truncate(),
        )
        .on_hover_text(text::ALIGNMENT_APPLIED);
    });
}
