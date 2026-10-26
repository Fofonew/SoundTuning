use eframe::egui::{self, Align, Layout};
use engine::{FFT_LENGTH_CHOICES, TIME_CONSTANT_CHOICES_S};

use crate::actions::Action;
use crate::config::StoredAnalysis;
use crate::design::Palette;
use crate::design::tokens::spacing;
use crate::design::widgets::{Emphasis, action_button, chip};
use crate::text;

const SMOOTHING_CHOICES: [u32; 6] = [0, 48, 24, 12, 6, 3];
const COHERENCE_STEP: f64 = 0.05;
const GENERATOR_STEP_DB: f32 = 1.0;
const GENERATOR_MIN_DBFS: f32 = -60.0;
const GENERATOR_MAX_DBFS: f32 = -6.0;
const SCROLL_STEP_PIXELS: f32 = 24.0;

#[derive(Clone, Debug, PartialEq)]
pub struct BottomBarModel {
    pub running: bool,
    pub frozen: bool,
    pub reference_label: String,
    pub generator_level_dbfs: Option<f32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BottomBarChanges {
    pub analysis_changed: bool,
    pub generator_level_dbfs: Option<f32>,
}

fn cycle<T: PartialEq + Copy>(choices: &[T], current: T, forward: bool) -> T {
    let index = choices
        .iter()
        .position(|choice| *choice == current)
        .unwrap_or(0);
    let next = if forward {
        (index + 1) % choices.len()
    } else {
        (index + choices.len() - 1) % choices.len()
    };
    choices.get(next).copied().unwrap_or(current)
}

fn scroll_direction(ui: &egui::Ui, response: &egui::Response) -> Option<bool> {
    if !response.hovered() {
        return None;
    }
    let delta = ui.input_mut(|input| std::mem::take(&mut input.smooth_scroll_delta.y));
    ui.ctx().data_mut(|data| {
        let total = data.get_temp_mut_or_default::<f32>(response.id);
        *total += delta;
        let step = if *total >= SCROLL_STEP_PIXELS {
            Some(true)
        } else if *total <= -SCROLL_STEP_PIXELS {
            Some(false)
        } else {
            None
        };
        if step.is_some() {
            *total = 0.0;
        }
        step
    })
}

fn keyboard_direction(ui: &egui::Ui, response: &egui::Response) -> Option<bool> {
    if !response.has_focus() {
        return None;
    }
    ui.input_mut(|input| {
        if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)
            || input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight)
        {
            Some(true)
        } else if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)
            || input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft)
        {
            Some(false)
        } else {
            None
        }
    })
}

fn step_direction(ui: &egui::Ui, response: &egui::Response) -> Option<bool> {
    if let Some(direction) = keyboard_direction(ui, response) {
        return Some(direction);
    }
    if response.clicked() {
        Some(true)
    } else if response.secondary_clicked() {
        Some(false)
    } else {
        scroll_direction(ui, response)
    }
}

pub fn show(
    ui: &mut egui::Ui,
    palette: &Palette,
    model: &BottomBarModel,
    analysis: &mut StoredAnalysis,
    is_mac: bool,
    actions: &mut Vec<Action>,
) -> BottomBarChanges {
    let mut changes = BottomBarChanges::default();
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = spacing::SM;

        let fft = chip(
            ui,
            palette,
            text::CHIP_FFT,
            &text::format_fft_length(analysis.fft_length),
            true,
        );
        if let Some(forward) = step_direction(ui, &fft) {
            analysis.fft_length = cycle(&FFT_LENGTH_CHOICES, analysis.fft_length, forward);
            changes.analysis_changed = true;
        }

        let averaging_value = analysis
            .time_constant_s
            .map_or_else(|| text::AVERAGING_INFINITE.to_owned(), text::format_seconds);
        let averaging = chip(ui, palette, text::CHIP_AVERAGING, &averaging_value, true);
        if let Some(forward) = step_direction(ui, &averaging) {
            let choices: Vec<Option<f64>> = TIME_CONSTANT_CHOICES_S
                .iter()
                .copied()
                .map(Some)
                .chain(std::iter::once(None))
                .collect();
            analysis.time_constant_s = cycle(&choices, analysis.time_constant_s, forward);
            changes.analysis_changed = true;
        }

        let smoothing = chip(
            ui,
            palette,
            text::CHIP_SMOOTHING,
            &text::format_smoothing(analysis.smoothing_fraction),
            true,
        );
        if let Some(forward) = step_direction(ui, &smoothing) {
            analysis.smoothing_fraction =
                cycle(&SMOOTHING_CHOICES, analysis.smoothing_fraction, forward);
            changes.analysis_changed = true;
        }

        let coherence = chip(
            ui,
            palette,
            text::CHIP_COHERENCE,
            &text::format_decimal(analysis.coherence_threshold, 2),
            true,
        );
        if let Some(forward) = step_direction(ui, &coherence) {
            let delta = if forward {
                COHERENCE_STEP
            } else {
                -COHERENCE_STEP
            };
            analysis.coherence_threshold =
                ((analysis.coherence_threshold + delta) * 20.0).round() / 20.0;
            analysis.coherence_threshold = analysis.coherence_threshold.clamp(0.0, 0.95);
            changes.analysis_changed = true;
        }

        chip(
            ui,
            palette,
            text::CHIP_REFERENCE,
            &model.reference_label,
            false,
        );

        if let Some(level) = model.generator_level_dbfs {
            let generator = chip(
                ui,
                palette,
                text::CHIP_GENERATOR,
                &text::format_dbfs(level),
                true,
            );
            if let Some(forward) =
                keyboard_direction(ui, &generator).or_else(|| scroll_direction(ui, &generator))
            {
                let delta = if forward {
                    GENERATOR_STEP_DB
                } else {
                    -GENERATOR_STEP_DB
                };
                changes.generator_level_dbfs =
                    Some((level + delta).clamp(GENERATOR_MIN_DBFS, GENERATOR_MAX_DBFS));
            }
            if generator.clicked() {
                actions.push(Action::ToggleGenerator);
            }
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = spacing::SM;
            let (label, emphasis) = if model.running {
                (text::BUTTON_STOP, Emphasis::Secondary)
            } else {
                (text::BUTTON_START, Emphasis::Primary)
            };
            if action_button(
                ui,
                palette,
                label,
                Action::ToggleMeasurement.shortcut_text(is_mac).as_deref(),
                emphasis,
                true,
            )
            .clicked()
            {
                actions.push(Action::ToggleMeasurement);
            }
            let freeze_label = if model.frozen {
                text::BUTTON_UNFREEZE
            } else {
                text::BUTTON_FREEZE
            };
            if action_button(
                ui,
                palette,
                freeze_label,
                Action::ToggleFreeze.shortcut_text(is_mac).as_deref(),
                Emphasis::Secondary,
                model.running,
            )
            .clicked()
            {
                actions.push(Action::ToggleFreeze);
            }
            if action_button(
                ui,
                palette,
                text::BUTTON_CAPTURE,
                Action::CaptureTrace.shortcut_text(is_mac).as_deref(),
                Emphasis::Secondary,
                model.running,
            )
            .clicked()
            {
                actions.push(Action::CaptureTrace);
            }
            if action_button(
                ui,
                palette,
                text::BUTTON_TOGGLE_PANEL,
                Action::ToggleSidePanel.shortcut_text(is_mac).as_deref(),
                Emphasis::Quiet,
                true,
            )
            .clicked()
            {
                actions.push(Action::ToggleSidePanel);
            }
            if action_button(
                ui,
                palette,
                text::BUTTON_PALETTE,
                Action::OpenPalette.shortcut_text(is_mac).as_deref(),
                Emphasis::Quiet,
                true,
            )
            .clicked()
            {
                actions.push(Action::OpenPalette);
            }
        });
    });
    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycling_wraps_in_both_directions() {
        assert_eq!(cycle(&[1, 2, 3], 3, true), 1);
        assert_eq!(cycle(&[1, 2, 3], 1, false), 3);
        assert_eq!(cycle(&[1, 2, 3], 9, true), 2);
    }
}
