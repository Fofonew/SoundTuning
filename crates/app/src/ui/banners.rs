use eframe::egui::{self, CornerRadius, Frame, Margin, Stroke};

use crate::actions::Action;
use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{radius, spacing, stroke, text_size};
use crate::design::widgets::{Emphasis, Tone, action_button};
use crate::measurement::InputAlert;
use crate::text;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BannerInputs {
    pub running: bool,
    pub start_error: Option<String>,
    pub alert: Option<InputAlert>,
    pub live_internal_reference: bool,
    pub latency_calibrated: bool,
    pub separate_clocks: bool,
    pub low_coherence: bool,
    pub clipping: bool,
    pub calibrating: bool,
    pub calibration_failed: bool,
    pub file_finished: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Banner {
    pub tone: Tone,
    pub text: String,
    pub action: Option<(&'static str, Action)>,
}

pub fn collect_banners(inputs: &BannerInputs) -> Vec<Banner> {
    let mut banners = Vec::new();
    let banner = |tone, text: &str, action| Banner {
        tone,
        text: text.to_owned(),
        action,
    };
    if let Some(error) = &inputs.start_error {
        banners.push(Banner {
            tone: Tone::Danger,
            text: text::start_failed(error),
            action: Some((text::BUTTON_RETRY, Action::RestartMeasurement)),
        });
    }
    match inputs.alert {
        Some(InputAlert::DeviceDisconnected) => banners.push(banner(
            Tone::Danger,
            text::BANNER_DISCONNECTED,
            Some((text::BUTTON_RESUME, Action::RestartMeasurement)),
        )),
        Some(InputAlert::PermissionDenied) => banners.push(banner(
            Tone::Danger,
            text::BANNER_PERMISSION,
            Some((text::BUTTON_RETRY, Action::RestartMeasurement)),
        )),
        Some(InputAlert::SilentMeasurementInput) => {
            banners.push(banner(Tone::Danger, text::BANNER_SILENT_INPUT, None));
        }
        Some(InputAlert::SilentReference) => {
            banners.push(banner(Tone::Warning, text::BANNER_SILENT_REFERENCE, None));
        }
        None => {}
    }
    if inputs.running && inputs.clipping {
        banners.push(banner(Tone::Danger, text::BANNER_CLIPPING, None));
    }
    if inputs.running && inputs.calibrating {
        banners.push(banner(Tone::Accent, text::BANNER_CALIBRATION_RUNNING, None));
    } else if inputs.running && inputs.calibration_failed {
        banners.push(banner(
            Tone::Warning,
            text::BANNER_CALIBRATION_FAILED,
            Some((text::BUTTON_RETRY, Action::CalibrateLatency)),
        ));
    } else if inputs.running && inputs.live_internal_reference && !inputs.latency_calibrated {
        banners.push(banner(
            Tone::Warning,
            text::BANNER_LATENCY_NOT_CALIBRATED,
            Some((text::BUTTON_CALIBRATE, Action::CalibrateLatency)),
        ));
    }
    if inputs.running && inputs.separate_clocks {
        banners.push(banner(Tone::Warning, text::BANNER_SEPARATE_CLOCKS, None));
    }
    if inputs.running && inputs.low_coherence && inputs.alert.is_none() {
        banners.push(banner(Tone::Warning, text::BANNER_LOW_COHERENCE, None));
    }
    if inputs.running && inputs.file_finished {
        banners.push(banner(Tone::Neutral, text::BANNER_FILE_FINISHED, None));
    }
    banners
}

pub fn show_banners(
    ui: &mut egui::Ui,
    palette: &Palette,
    banners: &[Banner],
    actions: &mut Vec<Action>,
) {
    for banner in banners {
        let color = banner.tone.color(palette);
        Frame::new()
            .fill(color.gamma_multiply(0.12))
            .stroke(Stroke::new(stroke::BORDER, color.gamma_multiply(0.7)))
            .corner_radius(CornerRadius::same(radius::MEDIUM))
            .inner_margin(Margin::symmetric(spacing::MD as i8, spacing::XS as i8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let (dot, _) =
                        ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                    ui.painter().circle_filled(dot.center(), 4.0, color);
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(&banner.text)
                                .font(fonts::ui_medium(text_size::BODY))
                                .color(palette.text_primary),
                        )
                        .wrap(),
                    );
                    if let Some((label, action)) = banner.action {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if action_button(ui, palette, label, None, Emphasis::Secondary, true)
                                .clicked()
                            {
                                actions.push(action);
                            }
                        });
                    }
                });
            });
        ui.add_space(spacing::XS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_shown_when_idle_and_healthy() {
        assert!(collect_banners(&BannerInputs::default()).is_empty());
    }

    #[test]
    fn uncalibrated_internal_reference_offers_calibration() {
        let banners = collect_banners(&BannerInputs {
            running: true,
            live_internal_reference: true,
            ..BannerInputs::default()
        });
        assert_eq!(banners.len(), 1);
        assert_eq!(
            banners[0].action.map(|(_, action)| action),
            Some(Action::CalibrateLatency)
        );
    }

    #[test]
    fn disconnection_is_a_danger_with_resume() {
        let banners = collect_banners(&BannerInputs {
            running: true,
            alert: Some(InputAlert::DeviceDisconnected),
            low_coherence: true,
            ..BannerInputs::default()
        });
        assert_eq!(banners.len(), 1);
        assert_eq!(banners[0].tone, Tone::Danger);
        assert_eq!(
            banners[0].action.map(|(label, _)| label),
            Some(text::BUTTON_RESUME)
        );
    }
}
