use eframe::egui::{self, RichText, vec2};

use crate::alignment_state::{
    AlignmentState, BAND_LIMIT_HIGH_HZ, BAND_LIMIT_LOW_HZ, StoredCriterion,
};
use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{size, spacing, text_size};
use crate::design::widgets::{Emphasis, action_button, muted, section_title};
use crate::session::Session;
use crate::text;

const DELAY_STEP_MS: f64 = 0.01;
const TRACE_LABEL_WIDTH: f32 = 56.0;
const DELAY_LIMIT_MS: f64 = 50.0;

#[derive(Clone, Debug, PartialEq)]
pub enum AlignmentCommand {
    Update(AlignmentState),
    UseProposal,
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

    for (label, slot, id_salt) in [
        (text::ALIGNMENT_TRACE_A, &mut state.trace_a, "alignment-a"),
        (text::ALIGNMENT_TRACE_B, &mut state.trace_b, "alignment-b"),
    ] {
        ui.horizontal(|ui| {
            ui.add_sized(
                vec2(TRACE_LABEL_WIDTH, size::MIN_CLICK_HEIGHT),
                egui::Label::new(RichText::new(label).color(palette.text_muted)),
            );
            let selected_name = slot
                .and_then(|id| session.trace(id))
                .map_or(text::ALIGNMENT_CHOOSE_TRACE, |trace| trace.name.as_str());
            egui::ComboBox::from_id_salt(id_salt)
                .width(ui.available_width() - spacing::SM)
                .truncate()
                .selected_text(selected_name)
                .show_ui(ui, |ui| {
                    for trace in &session.traces {
                        ui.selectable_value(slot, Some(trace.id), &trace.name);
                    }
                });
        });
        ui.add_space(spacing::XS);
    }

    ui.add_space(spacing::MD);
    ui.horizontal(|ui| {
        for (criterion, label) in [
            (
                StoredCriterion::MaximumLevel,
                text::ALIGNMENT_CRITERION_LEVEL,
            ),
            (StoredCriterion::Flattest, text::ALIGNMENT_CRITERION_FLAT),
        ] {
            if ui
                .add(egui::Button::selectable(
                    state.criterion == criterion,
                    label,
                ))
                .clicked()
            {
                state.criterion = criterion;
            }
        }
    });

    ui.add_space(spacing::SM);
    ui.horizontal(|ui| {
        ui.label(RichText::new(text::ALIGNMENT_BAND).color(palette.text_muted));
        ui.add(
            egui::DragValue::new(&mut state.band_low_hz)
                .range(BAND_LIMIT_LOW_HZ..=state.band_high_hz)
                .speed(1.0)
                .custom_formatter(|value, _| text::format_frequency(value)),
        );
        ui.label(RichText::new(text::RANGE_SEPARATOR).color(palette.text_muted));
        ui.add(
            egui::DragValue::new(&mut state.band_high_hz)
                .range(state.band_low_hz..=BAND_LIMIT_HIGH_HZ)
                .speed(1.0)
                .custom_formatter(|value, _| text::format_frequency(value)),
        );
    });
    muted(ui, palette, text::ALIGNMENT_BAND_HINT);

    if state.is_ready() {
        ui.add_space(spacing::MD);
        if let (Some(delay_ms), Some(inverted)) = (state.proposal_delay_ms, state.proposal_inverted)
        {
            ui.label(
                RichText::new(text::ALIGNMENT_PROPOSAL)
                    .font(fonts::ui_medium(text_size::LABEL))
                    .color(palette.text_muted),
            );
            ui.label(
                RichText::new(text::alignment_instruction(delay_ms, inverted))
                    .font(fonts::value_semibold(text_size::TITLE))
                    .color(palette.accent),
            );
            if action_button(
                ui,
                palette,
                text::ALIGNMENT_USE_PROPOSAL,
                None,
                Emphasis::Secondary,
                true,
            )
            .clicked()
            {
                commands.push(AlignmentCommand::UseProposal);
            }
        } else {
            muted(ui, palette, text::ALIGNMENT_NO_PROPOSAL);
        }

        ui.add_space(spacing::LG);
        ui.label(
            RichText::new(text::ALIGNMENT_APPLIED)
                .font(fonts::ui_medium(text_size::LABEL))
                .color(palette.text_muted),
        );
        ui.label(
            RichText::new(text::alignment_instruction(state.delay_ms, state.inverted))
                .font(fonts::value_semibold(text_size::VALUE_LARGE))
                .color(palette.text_primary),
        );
        ui.add_space(spacing::SM);
        ui.horizontal(|ui| {
            ui.label(RichText::new(text::ALIGNMENT_DELTA).color(palette.text_muted));
            ui.add(
                egui::DragValue::new(&mut state.delay_ms)
                    .speed(DELAY_STEP_MS)
                    .range(-DELAY_LIMIT_MS..=DELAY_LIMIT_MS)
                    .fixed_decimals(2)
                    .custom_formatter(|value, _| text::format_signed_milliseconds(value)),
            );
        });
        ui.checkbox(&mut state.inverted, text::ALIGNMENT_INVERT_B);
        muted(ui, palette, text::ALIGNMENT_SIGN_HINT);
    }

    if state != session.alignment {
        commands.push(AlignmentCommand::Update(state));
    }
    commands
}
