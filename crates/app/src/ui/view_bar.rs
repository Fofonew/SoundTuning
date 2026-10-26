use eframe::egui::{self, Align, Layout, RichText};

use crate::actions::Action;
use crate::design::Palette;
use crate::design::fonts;
use crate::design::icons::{self, icon_button};
use crate::design::tokens::{spacing, text_size};
use crate::plot::{PlotLayout, ViewKind};
use crate::text;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewCommand {
    Main(ViewKind),
    Upper(ViewKind),
    ToggleStacked,
    ImpulseInDb(bool),
    ToggleFullscreen,
}

pub fn view_label(kind: ViewKind) -> &'static str {
    match kind {
        ViewKind::Magnitude => text::VIEW_MAGNITUDE,
        ViewKind::Phase => text::VIEW_PHASE,
        ViewKind::Impulse => text::VIEW_IMPULSE,
    }
}

fn view_action(kind: ViewKind) -> Action {
    match kind {
        ViewKind::Magnitude => Action::ViewMagnitude,
        ViewKind::Phase => Action::ViewPhase,
        ViewKind::Impulse => Action::ViewImpulse,
    }
}

fn hint(label: &str, action: Action, is_mac: bool) -> String {
    text::with_shortcut(label, action.shortcut_text(is_mac).as_deref())
}

fn caption(ui: &mut egui::Ui, palette: &Palette, label: &str) {
    ui.label(
        RichText::new(label)
            .font(fonts::ui_medium(text_size::CAPTION))
            .color(palette.text_muted),
    );
}

fn segmented(
    ui: &mut egui::Ui,
    selected: ViewKind,
    is_mac: bool,
    shortcuts: bool,
    on_pick: impl Fn(ViewKind) -> ViewCommand,
    commands: &mut Vec<ViewCommand>,
) {
    for kind in ViewKind::ALL {
        let response = ui.add(egui::Button::selectable(selected == kind, view_label(kind)));
        let tooltip = if kind == ViewKind::Impulse {
            text::VIEW_IMPULSE_HINT
        } else {
            view_label(kind)
        };
        let response = if shortcuts {
            response.on_hover_text(hint(tooltip, view_action(kind), is_mac))
        } else {
            response
                .on_hover_text(tooltip)
                .accessible_name(text::upper_view_label(view_label(kind)))
        };
        if response.clicked() && selected != kind {
            commands.push(on_pick(kind));
        }
    }
}

pub fn show(
    ui: &mut egui::Ui,
    palette: &Palette,
    layout: &PlotLayout,
    fullscreen: bool,
    is_mac: bool,
) -> Vec<ViewCommand> {
    let mut commands = Vec::new();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = spacing::XS;
        caption(ui, palette, text::VIEW_MAIN_LABEL);
        segmented(
            ui,
            layout.main,
            is_mac,
            true,
            ViewCommand::Main,
            &mut commands,
        );
        ui.add_space(spacing::MD);
        let stacked = icon_button(
            ui,
            palette,
            icons::STACK,
            text::VIEW_STACKED,
            &hint(
                text::ACTION_TOGGLE_STACKED_VIEWS,
                Action::ToggleStackedViews,
                is_mac,
            ),
            layout.upper.is_some(),
        );
        if stacked.clicked() {
            commands.push(ViewCommand::ToggleStacked);
        }
        if let Some(upper) = layout.upper {
            ui.add_space(spacing::SM);
            caption(ui, palette, text::VIEW_UPPER_LABEL);
            segmented(ui, upper, is_mac, false, ViewCommand::Upper, &mut commands);
        }
        if layout.shows(ViewKind::Impulse) {
            ui.add_space(spacing::MD);
            for (in_db, label) in [
                (false, text::IMPULSE_SCALE_LINEAR),
                (true, text::IMPULSE_SCALE_DB),
            ] {
                let response = ui
                    .add(egui::Button::selectable(
                        layout.impulse_in_db == in_db,
                        label,
                    ))
                    .on_hover_text(text::IMPULSE_SCALE_HINT);
                if response.clicked() && layout.impulse_in_db != in_db {
                    commands.push(ViewCommand::ImpulseInDb(in_db));
                }
            }
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let (label, glyph) = if fullscreen {
                (text::VIEW_EXIT_FULLSCREEN, icons::EXIT_FULLSCREEN)
            } else {
                (text::VIEW_FULLSCREEN, icons::FULLSCREEN)
            };
            let response = icon_button(
                ui,
                palette,
                glyph,
                label,
                &hint(label, Action::ToggleFullscreen, is_mac),
                fullscreen,
            );
            if response.clicked() {
                commands.push(ViewCommand::ToggleFullscreen);
            }
        });
    });
    commands
}
