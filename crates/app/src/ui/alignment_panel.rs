use eframe::egui::{self, CornerRadius, RichText, vec2};

use crate::alignment_state::{
    AlignmentChoice, AlignmentRange, AlignmentState, BAND_LIMIT_HIGH_HZ, BAND_LIMIT_LOW_HZ,
    MAX_BANDWIDTH_OCTAVES, MIN_BANDWIDTH_OCTAVES, SEARCH_RANGE_MS,
};
use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{radius, size, spacing, text_size};
use crate::design::widgets::{combo_label, muted, section_title, spin_label};
use crate::session::Session;
use crate::text;

const DELAY_STEP_MS: f64 = 0.01;
const TRACE_LABEL_WIDTH: f32 = 72.0;
const CROSSOVER_DRAG_SPEED: f64 = 1.0;
const BANDWIDTH_DRAG_SPEED: f64 = 0.05;
const POWER_BAR_HEIGHT: f32 = 6.0;
const COMPACT_BUTTON_PADDING: f32 = 2.0;

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
    ui.add_space(spacing::MD);
    if session.traces.len() < 2 {
        section_title(ui, palette, text::EMPTY_ALIGNMENT_TITLE);
        muted(ui, palette, text::EMPTY_ALIGNMENT_BODY);
        return commands;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            section_title(ui, palette, text::ALIGNMENT_TITLE);
            ui.add_space(spacing::SM);
            trace_pickers(ui, palette, session, &mut state);
            ui.add_space(spacing::MD);
            range_controls(ui, palette, &mut state);
            if state.is_ready() {
                ui.add_space(spacing::LG);
                solutions(ui, palette, &mut state);
                ui.add_space(spacing::LG);
                applied_setting(ui, palette, session, &state);
            }
        });

    if state != session.alignment {
        commands.push(AlignmentCommand::Update(state));
    }
    commands
}

fn trace_pickers(
    ui: &mut egui::Ui,
    palette: &Palette,
    session: &Session,
    state: &mut AlignmentState,
) {
    for (label, slot, id_salt) in [
        (
            text::ALIGNMENT_REFERENCE,
            &mut state.reference_trace,
            "alignment-reference",
        ),
        (
            text::ALIGNMENT_DELAYED,
            &mut state.delayed_trace,
            "alignment-delayed",
        ),
    ] {
        ui.horizontal(|ui| {
            ui.add_sized(
                vec2(TRACE_LABEL_WIDTH, size::MIN_CLICK_HEIGHT),
                egui::Label::new(RichText::new(label).color(palette.text_muted)),
            );
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
        ui.add_space(spacing::XS);
    }
}

fn range_controls(ui: &mut egui::Ui, palette: &Palette, state: &mut AlignmentState) {
    ui.label(RichText::new(text::ALIGNMENT_RANGE).color(palette.text_muted));
    ui.horizontal(|ui| {
        for range in AlignmentRange::ALL {
            if ui
                .add(egui::Button::selectable(
                    state.range == range,
                    text::alignment_range(range),
                ))
                .accessible_name(text::alignment_range_label(range))
                .clicked()
            {
                state.range = range;
            }
        }
    });
    if state.range == AlignmentRange::Custom {
        ui.horizontal(|ui| {
            ui.label(RichText::new(text::ALIGNMENT_CROSSOVER).color(palette.text_muted));
            let response = ui.add(
                egui::DragValue::new(&mut state.crossover_hz)
                    .range(BAND_LIMIT_LOW_HZ..=BAND_LIMIT_HIGH_HZ)
                    .speed(CROSSOVER_DRAG_SPEED)
                    .custom_formatter(|value, _| text::format_frequency(value))
                    .custom_parser(text::parse_number),
            );
            spin_label(&response, text::ALIGNMENT_CROSSOVER_LABEL);
            ui.label(RichText::new(text::ALIGNMENT_BANDWIDTH).color(palette.text_muted));
            let response = ui.add(
                egui::DragValue::new(&mut state.bandwidth_octaves)
                    .range(MIN_BANDWIDTH_OCTAVES..=MAX_BANDWIDTH_OCTAVES)
                    .speed(BANDWIDTH_DRAG_SPEED)
                    .custom_formatter(|value, _| text::format_octaves(value))
                    .custom_parser(text::parse_number),
            );
            spin_label(&response, text::ALIGNMENT_BANDWIDTH_LABEL);
        });
    }
    let (low_hz, high_hz) = state.band_hz();
    muted(
        ui,
        palette,
        &format!(
            "{} {} {}",
            text::format_frequency(low_hz),
            text::RANGE_SEPARATOR,
            text::format_frequency(high_hz)
        ),
    );
    muted(ui, palette, text::ALIGNMENT_BAND_HINT);
}

fn solutions(ui: &mut egui::Ui, palette: &Palette, state: &mut AlignmentState) {
    let found = state.solutions;
    ui.spacing_mut().item_spacing.x = spacing::XS;
    ui.columns(AlignmentChoice::ALL.len(), |columns| {
        for (column, choice) in columns.iter_mut().zip(AlignmentChoice::ALL) {
            column.spacing_mut().item_spacing.y = spacing::XS;
            column.spacing_mut().button_padding.x = COMPACT_BUTTON_PADDING;
            let color = choice_color(palette, choice);
            let selected = state.choice == choice;
            let header = column.add_sized(
                vec2(column.available_width(), size::MIN_CLICK_HEIGHT),
                egui::Button::selectable(
                    selected,
                    RichText::new(text::alignment_choice(choice))
                        .font(fonts::ui_semibold(text_size::LABEL))
                        .color(color),
                ),
            );
            let header = header.on_hover_text(text::alignment_choice_summary(
                choice,
                state.setting(choice),
            ));
            if header.clicked() {
                state.choice = choice;
            }
            if choice == AlignmentChoice::Custom {
                let before = (state.custom_delay_ms, state.custom_inverted);
                let response = column.add_sized(
                    vec2(column.available_width(), size::MIN_CLICK_HEIGHT),
                    egui::DragValue::new(&mut state.custom_delay_ms)
                        .speed(DELAY_STEP_MS)
                        .range(-SEARCH_RANGE_MS..=SEARCH_RANGE_MS)
                        .fixed_decimals(2)
                        .custom_formatter(|value, _| text::format_decimal(value, 2))
                        .custom_parser(text::parse_number),
                );
                spin_label(&response, text::ALIGNMENT_CUSTOM_DELAY_LABEL);
                let polarity = column
                    .add_sized(
                        vec2(column.available_width(), size::MIN_CLICK_HEIGHT),
                        egui::Button::selectable(
                            state.custom_inverted,
                            polarity_text(state.custom_inverted),
                        ),
                    )
                    .accessible_name(text::ALIGNMENT_CUSTOM_POLARITY_LABEL);
                if polarity.clicked() {
                    state.custom_inverted = !state.custom_inverted;
                }
                if before != (state.custom_delay_ms, state.custom_inverted) {
                    state.choice = AlignmentChoice::Custom;
                }
                continue;
            }
            match state.setting(choice) {
                Some((delay_ms, inverted)) => {
                    value_cell(column, palette, &text::format_decimal(delay_ms, 2));
                    value_cell(column, palette, &polarity_text(inverted));
                }
                None => {
                    value_cell(column, palette, text::DELAY_NONE);
                    value_cell(column, palette, text::DELAY_NONE);
                }
            }
            if let Some(fraction) = found.and_then(|found| found.relative_power(choice)) {
                power_bar(column, color, fraction).on_hover_text(text::ALIGNMENT_POWER_HINT);
            }
        }
    });
    ui.label(
        RichText::new(text::ALIGNMENT_UNITS)
            .font(fonts::caption())
            .color(palette.text_muted),
    );
    if found.is_none() {
        muted(ui, palette, text::ALIGNMENT_NO_SOLUTION);
    }
}

fn polarity_text(inverted: bool) -> String {
    format!(
        "{} {}",
        text::ALIGNMENT_POLARITY_ROW,
        if inverted {
            text::ALIGNMENT_POLARITY_INVERTED
        } else {
            text::ALIGNMENT_POLARITY_NORMAL
        }
    )
}

fn value_cell(ui: &mut egui::Ui, palette: &Palette, value: &str) {
    ui.add_sized(
        vec2(ui.available_width(), size::MIN_CLICK_HEIGHT),
        egui::Label::new(
            RichText::new(value)
                .font(fonts::value_medium(text_size::VALUE))
                .color(palette.text_primary),
        ),
    );
}

fn power_bar(ui: &mut egui::Ui, color: egui::Color32, fraction: f64) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), POWER_BAR_HEIGHT),
        egui::Sense::hover(),
    );
    let filled = egui::Rect::from_min_size(
        rect.min,
        vec2(rect.width() * fraction as f32, rect.height()),
    );
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(radius::SMALL),
        color.gamma_multiply(0.2),
    );
    ui.painter()
        .rect_filled(filled, CornerRadius::same(radius::SMALL), color);
    response
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
    ui.label(
        RichText::new(text::ALIGNMENT_APPLIED)
            .font(fonts::ui_medium(text_size::LABEL))
            .color(palette.text_muted),
    );
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
    ui.add(
        egui::Label::new(
            RichText::new(instruction)
                .font(fonts::value_semibold(text_size::VALUE_LARGE))
                .color(choice_color(palette, state.choice)),
        )
        .wrap(),
    );
}
