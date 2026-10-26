use audio_io::{DeviceInfo, ReferenceMode, StreamSettings, negotiate_sample_rate};
use eframe::egui::{self, Align, Layout, RichText};

use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{spacing, text_size};
use crate::design::widgets::{Emphasis, action_button, combo_label, muted, section_title};
use crate::device_choice;
use crate::text;
use crate::ui::wizard::{channel_combo, output_selector};

const BUFFER_CHOICES: [Option<u32>; 6] =
    [None, Some(64), Some(128), Some(256), Some(512), Some(1024)];
const STEREO_CHANNELS: usize = 2;

#[derive(Clone, Debug, PartialEq)]
pub enum AudioCommand {
    Apply(StreamSettings),
    RefreshDevices,
    Close,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AudioSettingsPanel {
    pub devices: Vec<DeviceInfo>,
    pub draft: StreamSettings,
}

impl AudioSettingsPanel {
    pub fn new(devices: Vec<DeviceInfo>, current: &StreamSettings) -> Self {
        let mut panel = Self {
            devices,
            draft: current.clone(),
        };
        panel.normalize();
        panel
    }

    pub fn replace_devices(&mut self, devices: Vec<DeviceInfo>) {
        self.devices = devices;
        self.normalize();
    }

    fn input_device(&self) -> Option<&DeviceInfo> {
        device_choice::resolved_input(&self.devices, &self.draft)
    }

    fn output_device(&self) -> Option<&DeviceInfo> {
        device_choice::resolved_output(&self.devices, &self.draft)
    }

    fn input_channel_count(&self) -> usize {
        self.input_device()
            .map_or(1, |device| device.input_channel_count.max(1))
    }

    fn output_channel_count(&self) -> usize {
        self.output_device()
            .map_or(0, |device| device.output_channel_count)
    }

    fn sample_rates(&self) -> Vec<u32> {
        device_choice::shared_sample_rates(&self.devices, &self.draft)
    }

    pub fn select_input(&mut self, id: &str) {
        self.draft.input_device_id = Some(id.to_owned());
        let input = device_choice::find(&self.devices, id);
        let output_driver = self
            .draft
            .output_device_id
            .as_deref()
            .and_then(|output| device_choice::find(&self.devices, output))
            .map(|device| device.driver.as_str());
        if let Some(input) = input
            && output_driver.is_some_and(|driver| driver != input.driver)
        {
            self.draft.output_device_id = None;
        }
        if self.output_channel_count() == 0
            && let Some(device) = input.filter(|device| device.supports_output())
        {
            self.draft.output_device_id = Some(device.id.clone());
        }
        self.normalize();
    }

    pub fn select_output(&mut self, id: Option<&str>) {
        self.draft.output_device_id = id.map(str::to_owned);
        self.normalize();
    }

    fn normalize(&mut self) {
        let inputs = self.input_channel_count();
        self.draft.measurement_channel_index = self.draft.measurement_channel_index.min(inputs - 1);
        if let ReferenceMode::Loopback {
            reference_channel_index,
        } = &mut self.draft.reference
        {
            *reference_channel_index = (*reference_channel_index).min(inputs - 1);
        }
        let rates = self.sample_rates();
        if !rates.is_empty() && !rates.contains(&self.draft.sample_rate_hz) {
            self.draft.sample_rate_hz =
                negotiate_sample_rate(self.draft.sample_rate_hz, &rates, None)
                    .or_else(|| rates.first().copied())
                    .unwrap_or(self.draft.sample_rate_hz);
        }
        let outputs = self.output_channel_count();
        if outputs < STEREO_CHANNELS
            || self
                .draft
                .generator
                .output_channel_indices
                .iter()
                .any(|channel| *channel >= outputs)
        {
            self.draft.generator.output_channel_indices = vec![0];
        }
    }
}

fn field_label(ui: &mut egui::Ui, palette: &Palette, label: &str) {
    ui.add_space(spacing::SM);
    ui.label(
        RichText::new(label)
            .font(fonts::ui_medium(text_size::LABEL))
            .color(palette.text_muted),
    );
}

pub fn show(
    ui: &mut egui::Ui,
    palette: &Palette,
    panel: &mut AudioSettingsPanel,
    current: &StreamSettings,
    file_mode: bool,
) -> Vec<AudioCommand> {
    let mut commands = Vec::new();
    ui.horizontal(|ui| {
        section_title(ui, palette, text::AUDIO_SETTINGS_TITLE);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let close = ui
                .add(egui::Button::new(text::TRACE_DELETE_GLYPH).frame(false))
                .on_hover_text(text::AUDIO_SETTINGS_CLOSE);
            let close = close.accessible_name(text::AUDIO_SETTINGS_CLOSE);
            if close.clicked() {
                commands.push(AudioCommand::Close);
            }
        });
    });
    if file_mode {
        muted(ui, palette, text::AUDIO_FILE_MODE_NOTE);
    }
    let width = ui.available_width();
    egui::ScrollArea::vertical()
        .auto_shrink([false, true])
        .max_height((ui.available_height() - 3.0 * spacing::XXL).max(0.0))
        .show(ui, |ui| {
            ui.set_max_width(width);
            if panel.devices.is_empty() {
                muted(ui, palette, text::WIZARD_NO_DEVICE);
            }

            field_label(ui, palette, text::AUDIO_INPUT_DEVICE);
            let input_name = match panel.draft.input_device_id.as_deref() {
                Some(id) => device_choice::find(&panel.devices, id)
                    .map_or_else(|| id.to_owned(), text::device_name),
                None => text::AUDIO_SYSTEM_INPUT.to_owned(),
            };
            let mut chosen_input = None;
            let combo = egui::ComboBox::from_id_salt("audio-input")
                .width(width - spacing::SM)
                .truncate()
                .selected_text(input_name)
                .show_ui(ui, |ui| {
                    for device in panel
                        .devices
                        .iter()
                        .filter(|device| device.supports_input())
                    {
                        let selected =
                            panel.draft.input_device_id.as_deref() == Some(device.id.as_str());
                        if ui
                            .selectable_label(selected, text::device_name(device))
                            .clicked()
                        {
                            chosen_input = Some(device.id.clone());
                        }
                    }
                });
            combo_label(&combo.response, text::AUDIO_INPUT_DEVICE);
            if let Some(id) = chosen_input {
                panel.select_input(&id);
            }

            field_label(ui, palette, text::AUDIO_OUTPUT_DEVICE);
            let auto_output = text::auto_output_label(device_choice::resolved_output(
                &panel.devices,
                &StreamSettings {
                    output_device_id: None,
                    ..panel.draft.clone()
                },
            ));
            let output_name = match panel.draft.output_device_id.as_deref() {
                Some(id) => device_choice::find(&panel.devices, id)
                    .map_or_else(|| id.to_owned(), text::device_name),
                None => auto_output.clone(),
            };
            let mut chosen_output = None;
            let combo = egui::ComboBox::from_id_salt("audio-output")
                .width(width - spacing::SM)
                .truncate()
                .selected_text(output_name)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(panel.draft.output_device_id.is_none(), auto_output)
                        .clicked()
                    {
                        chosen_output = Some(None);
                    }
                    for device in panel
                        .devices
                        .iter()
                        .filter(|device| device.supports_output())
                    {
                        let selected =
                            panel.draft.output_device_id.as_deref() == Some(device.id.as_str());
                        if ui
                            .selectable_label(selected, text::device_name(device))
                            .clicked()
                        {
                            chosen_output = Some(Some(device.id.clone()));
                        }
                    }
                });
            combo_label(&combo.response, text::AUDIO_OUTPUT_DEVICE);
            if let Some(id) = chosen_output {
                panel.select_output(id.as_deref());
            }

            field_label(ui, palette, text::AUDIO_SAMPLE_RATE);
            let mut rates = panel.sample_rates();
            if rates.is_empty() {
                muted(ui, palette, text::AUDIO_NO_COMMON_RATE);
                rates.push(panel.draft.sample_rate_hz);
            }
            let combo = egui::ComboBox::from_id_salt("audio-sample-rate")
                .width(width - spacing::SM)
                .selected_text(text::format_sample_rate(panel.draft.sample_rate_hz))
                .show_ui(ui, |ui| {
                    for rate in rates {
                        ui.selectable_value(
                            &mut panel.draft.sample_rate_hz,
                            rate,
                            text::format_sample_rate(rate),
                        );
                    }
                });
            combo_label(&combo.response, text::AUDIO_SAMPLE_RATE);

            field_label(ui, palette, text::AUDIO_BUFFER);
            let combo = egui::ComboBox::from_id_salt("audio-buffer")
                .width(width - spacing::SM)
                .selected_text(text::format_buffer(panel.draft.buffer_frames))
                .show_ui(ui, |ui| {
                    for choice in BUFFER_CHOICES {
                        ui.selectable_value(
                            &mut panel.draft.buffer_frames,
                            choice,
                            text::format_buffer(choice),
                        );
                    }
                });
            combo_label(&combo.response, text::AUDIO_BUFFER);

            field_label(ui, palette, text::AUDIO_REFERENCE_MODE);
            let inputs = panel.input_channel_count();
            ui.horizontal_wrapped(|ui| {
                let internal = matches!(panel.draft.reference, ReferenceMode::Internal);
                if ui
                    .add(egui::Button::selectable(
                        internal,
                        text::AUDIO_REFERENCE_INTERNAL,
                    ))
                    .clicked()
                {
                    panel.draft.reference = ReferenceMode::Internal;
                    panel.draft.generator.enabled = true;
                }
                if ui
                    .add(egui::Button::selectable(
                        !internal,
                        text::AUDIO_REFERENCE_LOOPBACK,
                    ))
                    .clicked()
                    && internal
                {
                    panel.draft.reference = ReferenceMode::Loopback {
                        reference_channel_index: usize::from(inputs > 1),
                    };
                    panel.draft.generator.enabled = false;
                }
            });

            field_label(ui, palette, text::WIZARD_MEASUREMENT_CHANNEL);
            channel_combo(
                ui,
                (
                    "audio-measurement-channel",
                    text::WIZARD_MEASUREMENT_CHANNEL,
                ),
                &mut panel.draft.measurement_channel_index,
                inputs,
            );
            let outputs = panel.output_channel_count();
            match &mut panel.draft.reference {
                ReferenceMode::Loopback {
                    reference_channel_index,
                } => {
                    field_label(ui, palette, text::WIZARD_LOOPBACK_CHANNEL);
                    channel_combo(
                        ui,
                        ("audio-reference-channel", text::WIZARD_LOOPBACK_CHANNEL),
                        reference_channel_index,
                        inputs,
                    );
                }
                ReferenceMode::Internal => {
                    field_label(ui, palette, text::WIZARD_GENERATOR_OUTPUT);
                    if outputs == 0 {
                        muted(ui, palette, text::AUDIO_NO_OUTPUT);
                    } else {
                        output_selector(
                            ui,
                            &mut panel.draft.generator.output_channel_indices,
                            outputs,
                        );
                    }
                }
            }
            panel.normalize();
        });

    ui.add_space(spacing::MD);
    let changed = panel.draft != *current;
    ui.horizontal_wrapped(|ui| {
        if action_button(
            ui,
            palette,
            text::AUDIO_APPLY,
            None,
            Emphasis::Primary,
            changed,
        )
        .clicked()
        {
            commands.push(AudioCommand::Apply(panel.draft.clone()));
        }
        if action_button(
            ui,
            palette,
            text::WIZARD_REFRESH,
            None,
            Emphasis::Quiet,
            true,
        )
        .clicked()
        {
            commands.push(AudioCommand::RefreshDevices);
        }
    });
    muted(ui, palette, text::AUDIO_APPLY_HINT);
    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device(id: &str, inputs: usize, outputs: usize, rates: &[u32]) -> DeviceInfo {
        DeviceInfo {
            id: id.into(),
            name: id.into(),
            driver: "CoreAudio".into(),
            is_default_driver: true,
            input_channel_count: inputs,
            output_channel_count: outputs,
            input_sample_rates_hz: if inputs > 0 {
                rates.to_vec()
            } else {
                Vec::new()
            },
            output_sample_rates_hz: if outputs > 0 {
                rates.to_vec()
            } else {
                Vec::new()
            },
            is_default_input: false,
            is_default_output: false,
        }
    }

    #[test]
    fn choosing_a_smaller_device_clamps_channels_and_rate() {
        let devices = vec![
            device("uax", 2, 2, &[44_100, 48_000, 96_000]),
            device("mic", 1, 0, &[44_100]),
        ];
        let mut current = StreamSettings {
            input_device_id: Some("uax".into()),
            output_device_id: Some("uax".into()),
            measurement_channel_index: 1,
            sample_rate_hz: 96_000,
            ..StreamSettings::default()
        };
        current.generator.output_channel_indices = vec![1];
        let mut panel = AudioSettingsPanel::new(devices, &current);
        assert_eq!(panel.draft, current);
        panel.select_input("mic");
        assert_eq!(panel.draft.measurement_channel_index, 0);
        assert_eq!(panel.draft.sample_rate_hz, 44_100);
        assert_eq!(panel.draft.output_device_id.as_deref(), Some("uax"));
        panel.select_output(None);
        assert_eq!(panel.draft.generator.output_channel_indices, vec![0]);
    }

    #[test]
    fn a_duplex_input_is_also_the_automatic_output() {
        let devices = vec![device("uax", 2, 2, &[48_000])];
        let mut panel = AudioSettingsPanel::new(devices, &StreamSettings::default());
        panel.select_input("uax");
        assert_eq!(
            panel.output_device().map(|device| device.id.as_str()),
            Some("uax")
        );
        assert_eq!(panel.draft.sample_rate_hz, 48_000);
    }

    #[test]
    fn a_laptop_microphone_uses_the_system_output_at_a_shared_rate() {
        let mut microphone = device("mic", 1, 0, &[44_100, 48_000]);
        microphone.is_default_input = true;
        let mut speakers = device("speakers", 0, 2, &[44_100]);
        speakers.is_default_output = true;
        let mut panel =
            AudioSettingsPanel::new(vec![microphone, speakers], &StreamSettings::default());
        panel.select_input("mic");
        assert_eq!(panel.draft.output_device_id, None);
        assert_eq!(panel.output_channel_count(), 2);
        assert_eq!(panel.sample_rates(), vec![44_100]);
        assert_eq!(panel.draft.sample_rate_hz, 44_100);
    }

    #[test]
    fn switching_to_an_asio_input_drops_an_output_from_another_driver() {
        let mut asio = device("asio:UAX", 2, 2, &[48_000]);
        asio.driver = "ASIO".into();
        asio.is_default_driver = false;
        let devices = vec![device("wasapi:speakers", 0, 2, &[48_000]), asio];
        let current = StreamSettings {
            output_device_id: Some("wasapi:speakers".into()),
            ..StreamSettings::default()
        };
        let mut panel = AudioSettingsPanel::new(devices, &current);
        panel.select_input("asio:UAX");
        assert_eq!(panel.draft.output_device_id, None);
        assert_eq!(
            panel.output_device().map(|device| device.id.as_str()),
            Some("asio:UAX")
        );
    }
}
