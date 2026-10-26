use audio_io::{DeviceInfo, ReferenceMode};
use eframe::egui::{
    self, Align2, CornerRadius, Frame, Margin, Rect, RichText, Role, Sense, Stroke, StrokeKind,
    WidgetInfo, vec2,
};

use crate::config::AppConfig;
use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{radius, size, spacing, stroke, text_size};
use crate::design::widgets::{
    Emphasis, MeterReading, action_button, combo_label, level_meter, muted,
};
use crate::device_choice;
use crate::text;

const WIZARD_WIDTH: f32 = 640.0;
const WIZARD_HEIGHT: f32 = 520.0;
const MIN_CARD_SIZE: f32 = 240.0;
const DEVICE_ROW_HEIGHT: f32 = 52.0;
const STEP_COUNT: usize = 3;
const OPTION_CARD_HEIGHT: f32 = 64.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WizardStep {
    #[default]
    Device,
    Channels,
    Test,
}

impl WizardStep {
    fn index(self) -> usize {
        match self {
            Self::Device => 1,
            Self::Channels => 2,
            Self::Test => 3,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Device => text::WIZARD_STEP_DEVICE,
            Self::Channels => text::WIZARD_STEP_CHANNELS,
            Self::Test => text::WIZARD_STEP_TEST,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WizardAction {
    None,
    RestartPreview,
    StopPreview,
    StartTest,
    RefreshDevices,
    Finish,
    Skip,
}

#[derive(Clone, Debug, Default)]
pub struct Wizard {
    pub step: WizardStep,
    pub devices: Vec<DeviceInfo>,
    refresh_requested: bool,
}

impl Wizard {
    pub fn new(devices: Vec<DeviceInfo>) -> Self {
        Self {
            step: WizardStep::Device,
            devices,
            refresh_requested: false,
        }
    }

    fn selected_device<'a>(&'a self, config: &AppConfig) -> Option<&'a DeviceInfo> {
        let selected = config.stream.input_device_id.as_deref()?;
        self.devices.iter().find(|device| device.id == selected)
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        palette: &Palette,
        config: &mut AppConfig,
        meters: Option<(MeterReading, MeterReading)>,
        error: Option<&str>,
    ) -> WizardAction {
        let mut action = WizardAction::None;
        let available = ui.available_rect_before_wrap();
        let card_rect = Rect::from_center_size(
            available.center(),
            vec2(
                WIZARD_WIDTH
                    .min(available.width() - 2.0 * spacing::LG)
                    .max(MIN_CARD_SIZE),
                WIZARD_HEIGHT
                    .min(available.height() - 2.0 * spacing::XL)
                    .max(MIN_CARD_SIZE),
            ),
        );
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(card_rect));
        Frame::new()
            .fill(palette.surface)
            .stroke(Stroke::new(stroke::BORDER, palette.border))
            .corner_radius(CornerRadius::same(radius::LARGE))
            .inner_margin(Margin::same(spacing::XL as i8))
            .show(&mut child, |ui| {
                ui.set_width(card_rect.width() - 2.0 * spacing::XL);
                ui.label(
                    RichText::new(text::wizard_step(self.step.index(), STEP_COUNT))
                        .font(fonts::ui_medium(text_size::CAPTION))
                        .color(palette.accent),
                );
                ui.label(
                    RichText::new(text::WIZARD_TITLE)
                        .font(fonts::ui_semibold(text_size::HEADING))
                        .color(palette.text_primary),
                );
                ui.add_space(spacing::SM);
                step_indicator(ui, palette, self.step);
                ui.add_space(spacing::LG);
                if let Some(error) = error {
                    ui.label(
                        RichText::new(format!("{} : {error}", text::WIZARD_PREVIEW_ERROR))
                            .font(fonts::ui_medium(text_size::BODY))
                            .color(palette.danger),
                    );
                    ui.add_space(spacing::SM);
                }
                action = match self.step {
                    WizardStep::Device => self.device_step(ui, palette, config),
                    WizardStep::Channels => self.channels_step(ui, palette, config, meters),
                    WizardStep::Test => self.test_step(ui, palette, meters),
                };
            });
        action
    }

    fn navigation(
        &mut self,
        ui: &mut egui::Ui,
        palette: &Palette,
        can_continue: bool,
        next_label: &str,
    ) -> (bool, bool, bool) {
        ui.add_space(spacing::LG);
        let mut back = false;
        let mut next = false;
        let mut skip = false;
        let mut refresh = false;
        ui.horizontal(|ui| {
            if self.step == WizardStep::Device {
                skip = action_button(
                    ui,
                    palette,
                    text::BUTTON_SKIP_WIZARD,
                    None,
                    Emphasis::Quiet,
                    true,
                )
                .clicked();
                refresh = action_button(
                    ui,
                    palette,
                    text::WIZARD_REFRESH,
                    None,
                    Emphasis::Secondary,
                    true,
                )
                .clicked();
            } else {
                back = action_button(
                    ui,
                    palette,
                    text::BUTTON_BACK,
                    None,
                    Emphasis::Secondary,
                    true,
                )
                .clicked();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                next = action_button(
                    ui,
                    palette,
                    next_label,
                    None,
                    Emphasis::Primary,
                    can_continue,
                )
                .clicked();
            });
        });
        self.refresh_requested = refresh;
        (back, next, skip)
    }

    fn device_step(
        &mut self,
        ui: &mut egui::Ui,
        palette: &Palette,
        config: &mut AppConfig,
    ) -> WizardAction {
        muted(ui, palette, text::WIZARD_DEVICE_HELP);
        ui.add_space(spacing::MD);
        let inputs: Vec<DeviceInfo> = self
            .devices
            .iter()
            .filter(|device| device.supports_input())
            .cloned()
            .collect();
        if inputs.is_empty() {
            muted(ui, palette, text::WIZARD_NO_DEVICE);
        }
        for device in &inputs {
            let selected = config.stream.input_device_id.as_deref() == Some(device.id.as_str());
            if device_row(ui, palette, device, selected).clicked() {
                config.stream.input_device_id = Some(device.id.clone());
                config.stream.output_device_id =
                    device.supports_output().then(|| device.id.clone());
                config.stream.measurement_channel_index = 0;
                if device.output_channel_count < 2 {
                    config.stream.generator.output_channel_indices = vec![0];
                }
                if let ReferenceMode::Loopback {
                    reference_channel_index,
                } = &mut config.stream.reference
                {
                    *reference_channel_index = (*reference_channel_index)
                        .min(device.input_channel_count.saturating_sub(1));
                }
            }
        }
        let can_continue = self.selected_device(config).is_some();
        let (_, next, skip) = self.navigation(ui, palette, can_continue, text::BUTTON_NEXT);
        if skip {
            return WizardAction::Skip;
        }
        if self.refresh_requested {
            return WizardAction::RefreshDevices;
        }
        if next {
            self.step = WizardStep::Channels;
            return WizardAction::RestartPreview;
        }
        WizardAction::None
    }

    fn channels_step(
        &mut self,
        ui: &mut egui::Ui,
        palette: &Palette,
        config: &mut AppConfig,
        meters: Option<(MeterReading, MeterReading)>,
    ) -> WizardAction {
        let input_count = self
            .selected_device(config)
            .map_or(1, |device| device.input_channel_count.max(1));
        let output_count = device_choice::resolved_output(&self.devices, &config.stream)
            .map_or(2, |device| device.output_channel_count);
        let mut changed = false;
        let is_internal = matches!(config.stream.reference, ReferenceMode::Internal);
        if option_card(
            ui,
            palette,
            text::WIZARD_REFERENCE_INTERNAL,
            text::WIZARD_REFERENCE_INTERNAL_HELP,
            is_internal,
        )
        .clicked()
            && !is_internal
        {
            config.stream.reference = ReferenceMode::Internal;
            config.stream.generator.enabled = true;
            changed = true;
        }
        ui.add_space(spacing::SM);
        if option_card(
            ui,
            palette,
            text::WIZARD_REFERENCE_LOOPBACK,
            text::WIZARD_REFERENCE_LOOPBACK_HELP,
            !is_internal,
        )
        .clicked()
            && is_internal
        {
            let reference_channel_index = usize::from(input_count > 1);
            config.stream.reference = ReferenceMode::Loopback {
                reference_channel_index,
            };
            config.stream.generator.enabled = false;
            changed = true;
        }
        ui.add_space(spacing::MD);
        egui::Grid::new("wizard-channels")
            .num_columns(2)
            .spacing([spacing::LG, spacing::SM])
            .show(ui, |ui| {
                ui.label(RichText::new(text::WIZARD_MEASUREMENT_CHANNEL).color(palette.text_muted));
                changed |= channel_combo(
                    ui,
                    ("measurement-channel", text::WIZARD_MEASUREMENT_CHANNEL),
                    &mut config.stream.measurement_channel_index,
                    input_count,
                );
                ui.end_row();
                match &mut config.stream.reference {
                    ReferenceMode::Loopback {
                        reference_channel_index,
                    } => {
                        ui.label(
                            RichText::new(text::WIZARD_LOOPBACK_CHANNEL).color(palette.text_muted),
                        );
                        changed |= channel_combo(
                            ui,
                            ("reference-channel", text::WIZARD_LOOPBACK_CHANNEL),
                            reference_channel_index,
                            input_count,
                        );
                    }
                    ReferenceMode::Internal => {
                        ui.label(
                            RichText::new(text::WIZARD_GENERATOR_OUTPUT).color(palette.text_muted),
                        );
                        changed |= output_selector(
                            ui,
                            &mut config.stream.generator.output_channel_indices,
                            output_count,
                        );
                    }
                }
                ui.end_row();
            });
        if let Some((reference, measurement)) = meters {
            ui.add_space(spacing::MD);
            ui.horizontal(|ui| {
                level_meter(ui, palette, text::METER_REFERENCE, reference);
                ui.add_space(spacing::LG);
                level_meter(ui, palette, text::METER_MEASUREMENT, measurement);
            });
        }
        let (back, next, _) = self.navigation(ui, palette, true, text::BUTTON_NEXT);
        if back {
            self.step = WizardStep::Device;
            return WizardAction::StopPreview;
        }
        if let ReferenceMode::Loopback {
            reference_channel_index,
        } = &mut config.stream.reference
            && *reference_channel_index == config.stream.measurement_channel_index
            && input_count > 1
        {
            *reference_channel_index = (config.stream.measurement_channel_index + 1) % input_count;
            changed = true;
        }
        if next {
            self.step = WizardStep::Test;
            return WizardAction::StartTest;
        }
        if changed {
            WizardAction::RestartPreview
        } else {
            WizardAction::None
        }
    }

    fn test_step(
        &mut self,
        ui: &mut egui::Ui,
        palette: &Palette,
        meters: Option<(MeterReading, MeterReading)>,
    ) -> WizardAction {
        muted(ui, palette, text::WIZARD_TEST_HELP);
        ui.add_space(spacing::MD);
        if let Some((reference, measurement)) = meters {
            ui.horizontal(|ui| {
                level_meter(ui, palette, text::METER_REFERENCE, reference);
                ui.add_space(spacing::LG);
                level_meter(ui, palette, text::METER_MEASUREMENT, measurement);
            });
        }
        let (back, next, _) = self.navigation(ui, palette, true, text::BUTTON_FINISH_WIZARD);
        if back {
            self.step = WizardStep::Channels;
        }
        if next {
            return WizardAction::Finish;
        }
        WizardAction::None
    }
}

fn step_indicator(ui: &mut egui::Ui, palette: &Palette, current: WizardStep) {
    ui.horizontal(|ui| {
        for step in [WizardStep::Device, WizardStep::Channels, WizardStep::Test] {
            let reached = step.index() <= current.index();
            let color = if reached {
                palette.accent
            } else {
                palette.text_muted
            };
            let (dot, _) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::hover());
            ui.painter().circle(
                dot.center(),
                9.0,
                if reached {
                    color.gamma_multiply(0.2)
                } else {
                    palette.surface_raised
                },
                Stroke::new(stroke::BORDER, color),
            );
            ui.painter().text(
                dot.center(),
                Align2::CENTER_CENTER,
                step.index().to_string(),
                fonts::value_medium(text_size::CAPTION),
                color,
            );
            ui.label(
                RichText::new(step.title())
                    .font(fonts::ui_medium(text_size::LABEL))
                    .color(if step == current {
                        palette.text_primary
                    } else {
                        palette.text_muted
                    }),
            );
            ui.add_space(spacing::MD);
        }
    });
}

fn selectable_frame(
    ui: &mut egui::Ui,
    palette: &Palette,
    height: f32,
    selected: bool,
    label: &str,
) -> (Rect, egui::Response) {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::click());
    response.widget_info(|| WidgetInfo::selected(Role::RadioButton, true, selected, label));
    let hovered = response.hovered();
    ui.painter().rect(
        rect,
        CornerRadius::same(radius::MEDIUM),
        if selected {
            palette.accent.gamma_multiply(0.14)
        } else if hovered {
            palette.surface_hover
        } else {
            palette.surface_raised
        },
        Stroke::new(
            if selected {
                stroke::FOCUS
            } else {
                stroke::BORDER
            },
            if selected {
                palette.accent
            } else {
                palette.border
            },
        ),
        StrokeKind::Inside,
    );
    (rect, response)
}

fn device_row(
    ui: &mut egui::Ui,
    palette: &Palette,
    device: &DeviceInfo,
    selected: bool,
) -> egui::Response {
    let name = text::device_name(device);
    let (rect, response) = selectable_frame(ui, palette, DEVICE_ROW_HEIGHT, selected, &name);
    ui.painter().text(
        rect.left_center() + vec2(spacing::LG, -9.0),
        Align2::LEFT_CENTER,
        name,
        fonts::ui_medium(text_size::BODY),
        palette.text_primary,
    );
    ui.painter().text(
        rect.left_center() + vec2(spacing::LG, 10.0),
        Align2::LEFT_CENTER,
        text::device_channels(device.input_channel_count, device.output_channel_count),
        fonts::value(text_size::CAPTION),
        palette.text_muted,
    );
    ui.add_space(spacing::XS);
    response
}

fn option_card(
    ui: &mut egui::Ui,
    palette: &Palette,
    title: &str,
    body: &str,
    selected: bool,
) -> egui::Response {
    let (rect, response) = selectable_frame(ui, palette, OPTION_CARD_HEIGHT, selected, title);
    ui.painter().text(
        rect.left_top() + vec2(spacing::LG, spacing::MD),
        Align2::LEFT_TOP,
        title,
        fonts::ui_semibold(text_size::BODY),
        palette.text_primary,
    );
    ui.painter().text(
        rect.left_top() + vec2(spacing::LG, spacing::MD + 20.0),
        Align2::LEFT_TOP,
        body,
        fonts::ui(text_size::LABEL),
        palette.text_muted,
    );
    response
}

pub(crate) fn channel_combo(
    ui: &mut egui::Ui,
    (id, label): (&str, &str),
    channel_index: &mut usize,
    channel_count: usize,
) -> bool {
    let before = *channel_index;
    let combo = egui::ComboBox::from_id_salt(id)
        .selected_text(text::format_input_channel(*channel_index))
        .height(size::BUTTON_HEIGHT * 8.0)
        .show_ui(ui, |ui| {
            for index in 0..channel_count {
                ui.selectable_value(channel_index, index, text::format_input_channel(index));
            }
        });
    combo_label(&combo.response, label);
    before != *channel_index
}

pub(crate) fn output_selector(
    ui: &mut egui::Ui,
    outputs: &mut Vec<usize>,
    output_count: usize,
) -> bool {
    let before = outputs.clone();
    ui.horizontal(|ui| {
        let options: [(&str, Vec<usize>); 3] = [
            (text::WIZARD_OUTPUT_LEFT, vec![0]),
            (text::WIZARD_OUTPUT_RIGHT, vec![1]),
            (text::WIZARD_OUTPUT_BOTH, vec![0, 1]),
        ];
        for (label, channels) in options {
            let available = channels
                .iter()
                .all(|channel| *channel < output_count.max(1));
            let selected = *outputs == channels;
            if ui
                .add_enabled(available, egui::Button::selectable(selected, label))
                .clicked()
            {
                *outputs = channels;
            }
        }
    });
    before != *outputs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_are_numbered_from_one() {
        assert_eq!(WizardStep::Device.index(), 1);
        assert_eq!(WizardStep::Test.index(), 3);
    }
}
