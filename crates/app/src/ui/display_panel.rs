use eframe::egui::{self, RichText};

use crate::config::{SMOOTHING_CHOICES, StoredAnalysis};
use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{size, spacing, text_size};
use crate::design::widgets::{combo_label, field_caption, spin_label, toggle_button};
use crate::plot::{ExternalModule, PlotLayout, ViewKind};
use crate::text;
use crate::ui::graph_bar::ViewCommand;

const MASK_STEP: f64 = 0.05;
const MASK_MAX: f64 = 0.95;
const MASK_DRAG_SPEED: f64 = 0.01;

#[derive(Debug, Default, PartialEq)]
pub struct DisplayChanges {
    pub commands: Vec<ViewCommand>,
    pub analysis_changed: bool,
}

fn accessible(label: &str, upper: bool) -> String {
    if upper {
        text::upper_view_label(label)
    } else {
        label.to_owned()
    }
}

pub fn show(
    ui: &mut egui::Ui,
    palette: &Palette,
    kind: ViewKind,
    upper: bool,
    layout: &PlotLayout,
    analysis: &mut StoredAnalysis,
) -> DisplayChanges {
    let mut changes = DisplayChanges::default();
    ui.spacing_mut().item_spacing.y = spacing::SM;
    ui.label(
        RichText::new(text::DISPLAY_TITLE)
            .font(fonts::ui_semibold(text_size::LABEL))
            .color(palette.text_primary),
    );
    let width = ui.available_width();
    match kind {
        ViewKind::Magnitude => {
            let mut show_coherence = layout.show_coherence;
            let checkbox = ui.checkbox(&mut show_coherence, text::SHOW_COHERENCE);
            let _ = checkbox
                .clone()
                .accessible_name(accessible(text::SHOW_COHERENCE, upper));
            if checkbox.changed() {
                changes
                    .commands
                    .push(ViewCommand::ShowCoherence(show_coherence));
            }
            field_caption(ui, palette, text::COHERENCE_MASK)
                .on_hover_text(text::COHERENCE_MASK_HINT);
            let mut threshold = analysis.coherence_threshold;
            let response = ui.add_sized(
                egui::vec2(width, size::MIN_CLICK_HEIGHT),
                egui::DragValue::new(&mut threshold)
                    .range(0.0..=MASK_MAX)
                    .speed(MASK_DRAG_SPEED)
                    .custom_formatter(|value, _| text::format_decimal(value, 2))
                    .custom_parser(text::parse_number),
            );
            spin_label(&response, accessible(text::COHERENCE_MASK, upper));
            if response.changed() {
                analysis.coherence_threshold =
                    ((threshold / MASK_STEP).round() * MASK_STEP).clamp(0.0, MASK_MAX);
                changes.analysis_changed = true;
            }
            smoothing(ui, palette, upper, width, analysis, &mut changes);
        }
        ViewKind::Phase => smoothing(ui, palette, upper, width, analysis, &mut changes),
        ViewKind::Impulse => {
            field_caption(ui, palette, text::UNIT);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = size::SEGMENT_GAP;
                let half = (width - size::SEGMENT_GAP) / 2.0;
                for (in_db, label) in [
                    (false, text::IMPULSE_SCALE_LINEAR),
                    (true, text::IMPULSE_SCALE_DB),
                ] {
                    let response = toggle_button(
                        ui,
                        palette,
                        label,
                        layout.impulse_in_db == in_db,
                        Some(half),
                    )
                    .on_hover_text(text::IMPULSE_SCALE_HINT);
                    if response.clicked() && layout.impulse_in_db != in_db {
                        changes.commands.push(ViewCommand::ImpulseInDb(in_db));
                    }
                }
            });
        }
        ViewKind::VirtualEq => {
            for (module, label) in [
                (ExternalModule::DelayAnalysis, text::MODULE_DELAY_ANALYSIS),
                (ExternalModule::VirtualEq, text::MODULE_VIRTUAL_EQ),
            ] {
                let response =
                    toggle_button(ui, palette, label, layout.module == module, Some(width));
                let _ = response
                    .clone()
                    .accessible_name(text::display_option_label(label));
                if response.clicked() && layout.module != module {
                    changes.commands.push(ViewCommand::Module(module));
                }
            }
        }
    }
    changes
}

fn smoothing(
    ui: &mut egui::Ui,
    palette: &Palette,
    upper: bool,
    width: f32,
    analysis: &mut StoredAnalysis,
    changes: &mut DisplayChanges,
) {
    field_caption(ui, palette, text::SMOOTHING);
    let mut fraction = analysis.smoothing_fraction;
    let combo = egui::ComboBox::from_id_salt(("display-smoothing", upper))
        .width(width - spacing::SM)
        .selected_text(text::format_smoothing(fraction))
        .show_ui(ui, |ui| {
            for choice in SMOOTHING_CHOICES {
                ui.selectable_value(&mut fraction, choice, text::format_smoothing(choice));
            }
        });
    combo_label(&combo.response, accessible(text::SMOOTHING, upper));
    if fraction != analysis.smoothing_fraction {
        analysis.smoothing_fraction = fraction;
        changes.analysis_changed = true;
    }
}
