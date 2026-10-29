use eframe::egui::{self, Align, Layout, RichText};

use crate::actions::Action;
use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{spacing, text_size};
use crate::design::widgets::{Tone, status_pill};
use crate::text;

pub struct StatusBarModel {
    pub status_tone: Tone,
    pub status_text: &'static str,
    pub setup: String,
    pub version: String,
}

pub fn show(
    ui: &mut egui::Ui,
    palette: &Palette,
    model: &StatusBarModel,
    is_mac: bool,
    actions: &mut Vec<Action>,
) {
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = spacing::MD;
        status_pill(ui, palette, model.status_tone, model.status_text);
        let right_width = ui
            .painter()
            .layout_no_wrap(
                model.version.clone(),
                fonts::value(text_size::CAPTION),
                palette.text_muted,
            )
            .size()
            .x
            + 2.0 * spacing::XL
            + spacing::XXL * 3.0;
        let setup_width = (ui.available_width() - right_width).max(0.0);
        ui.allocate_ui_with_layout(
            egui::vec2(setup_width, ui.available_height()),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(&model.setup)
                            .font(fonts::ui(text_size::CAPTION))
                            .color(palette.text_muted),
                    )
                    .truncate(),
                );
            },
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(&model.version)
                    .font(fonts::value(text_size::CAPTION))
                    .color(palette.text_muted),
            );
            let palette_hint = text::with_shortcut(
                Action::OpenPalette.label(),
                Action::OpenPalette.shortcut_text(is_mac).as_deref(),
            );
            let commands = ui
                .add(
                    egui::Button::new(
                        RichText::new(text::with_shortcut(
                            text::BUTTON_PALETTE,
                            Action::OpenPalette.shortcut_text(is_mac).as_deref(),
                        ))
                        .font(fonts::ui_medium(text_size::CAPTION))
                        .color(palette.text_primary),
                    )
                    .frame(false)
                    .small(),
                )
                .on_hover_text(palette_hint);
            let _ = commands.clone().accessible_name(text::BUTTON_PALETTE);
            if commands.clicked() {
                actions.push(Action::OpenPalette);
            }
        });
    });
}
