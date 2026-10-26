use eframe::egui::{self, Align, Layout};
use engine::{FFT_LENGTH_CHOICES, TIME_CONSTANT_CHOICES_S};

use crate::actions::Action;
use crate::config::StoredAnalysis;
use crate::design::Palette;
use crate::design::icons::{self, icon_button};
use crate::design::tokens::spacing;
use crate::design::widgets::{Emphasis, action_button, action_button_width, chip, chip_width};
use crate::text;

pub(crate) const SMOOTHING_CHOICES: [u32; 6] = [0, 48, 24, 12, 6, 3];
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Density {
    Full,
    QuietShortcutsHidden,
    ShortcutsHidden,
    Compact,
}

impl Density {
    const ALL: [Self; 4] = [
        Self::Full,
        Self::QuietShortcutsHidden,
        Self::ShortcutsHidden,
        Self::Compact,
    ];

    fn shows_shortcut(self, emphasis: Emphasis) -> bool {
        match self {
            Self::Full => true,
            Self::QuietShortcutsHidden => emphasis != Emphasis::Quiet,
            Self::ShortcutsHidden | Self::Compact => false,
        }
    }

    fn shows_secondary_items(self) -> bool {
        self != Self::Compact
    }
}

struct BarButton {
    label: &'static str,
    action: Action,
    emphasis: Emphasis,
    enabled: bool,
    secondary: bool,
    icon: Option<&'static str>,
}

fn bar_buttons(model: &BottomBarModel) -> [BarButton; 6] {
    let button = |label, action, emphasis, enabled, secondary| BarButton {
        label,
        action,
        emphasis,
        enabled,
        secondary,
        icon: None,
    };
    let icon = |label, action, glyph| BarButton {
        label,
        action,
        emphasis: Emphasis::Quiet,
        enabled: true,
        secondary: false,
        icon: Some(glyph),
    };
    let (start_label, start_emphasis) = if model.running {
        (text::BUTTON_STOP, Emphasis::Secondary)
    } else {
        (text::BUTTON_START, Emphasis::Primary)
    };
    let freeze_label = if model.frozen {
        text::BUTTON_UNFREEZE
    } else {
        text::BUTTON_FREEZE
    };
    [
        button(
            start_label,
            Action::ToggleMeasurement,
            start_emphasis,
            true,
            false,
        ),
        button(
            freeze_label,
            Action::ToggleFreeze,
            Emphasis::Secondary,
            model.running,
            false,
        ),
        button(
            text::BUTTON_CAPTURE,
            Action::CaptureTrace,
            Emphasis::Secondary,
            model.running,
            false,
        ),
        icon(
            text::BUTTON_TOGGLE_PANEL,
            Action::ToggleSidePanel,
            icons::SIDEBAR,
        ),
        icon(
            text::BUTTON_AUDIO_SETTINGS,
            Action::OpenAudioSettings,
            icons::SLIDERS,
        ),
        icon(text::BUTTON_PALETTE, Action::OpenPalette, icons::COMMAND),
    ]
}

struct ChipTexts {
    fft: String,
    averaging: String,
    smoothing: String,
    coherence: String,
    generator: Option<String>,
}

impl ChipTexts {
    fn new(model: &BottomBarModel, analysis: &StoredAnalysis) -> Self {
        Self {
            fft: text::format_fft_length(analysis.fft_length),
            averaging: analysis
                .time_constant_s
                .map_or_else(|| text::AVERAGING_INFINITE.to_owned(), text::format_seconds),
            smoothing: text::format_smoothing(analysis.smoothing_fraction),
            coherence: text::format_decimal(analysis.coherence_threshold, 2),
            generator: model.generator_level_dbfs.map(text::format_dbfs),
        }
    }

    fn chips<'a>(&'a self, model: &'a BottomBarModel, density: Density) -> Vec<(&'a str, &'a str)> {
        let mut chips = vec![
            (text::CHIP_FFT, self.fft.as_str()),
            (text::CHIP_AVERAGING, self.averaging.as_str()),
            (text::CHIP_SMOOTHING, self.smoothing.as_str()),
            (text::CHIP_COHERENCE, self.coherence.as_str()),
        ];
        if density.shows_secondary_items() {
            chips.push((text::CHIP_REFERENCE, model.reference_label.as_str()));
        }
        if let Some(generator) = &self.generator {
            chips.push((text::CHIP_GENERATOR, generator.as_str()));
        }
        chips
    }
}

fn required_width(
    ui: &egui::Ui,
    model: &BottomBarModel,
    chips: &ChipTexts,
    buttons: &[BarButton],
    density: Density,
    is_mac: bool,
) -> f32 {
    let chip_widths: Vec<f32> = chips
        .chips(model, density)
        .iter()
        .map(|(caption, value)| chip_width(ui, caption, value))
        .collect();
    let button_widths: Vec<f32> = buttons
        .iter()
        .filter(|button| density.shows_secondary_items() || !button.secondary)
        .map(|button| {
            if button.icon.is_some() {
                return icons::icon_button_width();
            }
            let shortcut = density
                .shows_shortcut(button.emphasis)
                .then(|| button.action.shortcut_text(is_mac))
                .flatten();
            action_button_width(ui, button.label, shortcut.as_deref())
        })
        .collect();
    let items = chip_widths.len() + button_widths.len();
    chip_widths.iter().sum::<f32>()
        + button_widths.iter().sum::<f32>()
        + items.saturating_sub(1) as f32 * spacing::SM
        + spacing::LG
}

pub fn density_for(
    ui: &egui::Ui,
    model: &BottomBarModel,
    analysis: &StoredAnalysis,
    is_mac: bool,
) -> Density {
    let chips = ChipTexts::new(model, analysis);
    let buttons = bar_buttons(model);
    let available = ui.available_width();
    Density::ALL
        .into_iter()
        .find(|density| required_width(ui, model, &chips, &buttons, *density, is_mac) <= available)
        .unwrap_or(Density::Compact)
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
    let density = density_for(ui, model, analysis, is_mac);
    let texts = ChipTexts::new(model, analysis);
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = spacing::SM;

        let fft = chip(ui, palette, text::CHIP_FFT, &texts.fft, true);
        if let Some(forward) = step_direction(ui, &fft) {
            analysis.fft_length = cycle(&FFT_LENGTH_CHOICES, analysis.fft_length, forward);
            changes.analysis_changed = true;
        }

        let averaging = chip(ui, palette, text::CHIP_AVERAGING, &texts.averaging, true);
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

        let smoothing = chip(ui, palette, text::CHIP_SMOOTHING, &texts.smoothing, true);
        if let Some(forward) = step_direction(ui, &smoothing) {
            analysis.smoothing_fraction =
                cycle(&SMOOTHING_CHOICES, analysis.smoothing_fraction, forward);
            changes.analysis_changed = true;
        }

        let coherence = chip(ui, palette, text::CHIP_COHERENCE, &texts.coherence, true);
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

        if density.shows_secondary_items() {
            chip(
                ui,
                palette,
                text::CHIP_REFERENCE,
                &model.reference_label,
                false,
            );
        }

        if let (Some(level), Some(level_text)) = (model.generator_level_dbfs, &texts.generator) {
            let generator = chip(ui, palette, text::CHIP_GENERATOR, level_text, true);
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
            for button in bar_buttons(model) {
                if !density.shows_secondary_items() && button.secondary {
                    continue;
                }
                if let Some(glyph) = button.icon {
                    let tooltip = text::with_shortcut(
                        button.action.label(),
                        button.action.shortcut_text(is_mac).as_deref(),
                    );
                    if icon_button(ui, palette, glyph, button.label, &tooltip, false).clicked() {
                        actions.push(button.action);
                    }
                    continue;
                }
                let shortcut = density
                    .shows_shortcut(button.emphasis)
                    .then(|| button.action.shortcut_text(is_mac))
                    .flatten();
                if action_button(
                    ui,
                    palette,
                    button.label,
                    shortcut.as_deref(),
                    button.emphasis,
                    button.enabled,
                )
                .clicked()
                {
                    actions.push(button.action);
                }
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
