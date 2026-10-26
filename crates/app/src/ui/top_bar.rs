use eframe::egui::{self, Align, Layout, RichText, vec2};

use crate::actions::Action;
use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{size, spacing, stroke, text_size};
use crate::design::widgets::{
    Emphasis, MeterReading, Tone, action_button, action_button_width, confidence_segments,
    level_meter, status_pill,
};
use crate::text;

const DELAY_BLOCK_WIDTH: f32 = 232.0;
const FULL_LEFT_MIN_WIDTH: f32 = 320.0;
const MIN_SOURCE_WIDTH: f32 = 160.0;
const FINE_GROUP_WIDTH: f32 = 2.0 * size::MIN_CLICK_HEIGHT + TRACKING_WIDTH + 2.0 * spacing::XS;
const TRACKING_WIDTH: f32 = 76.0;
const METER_GROUP_WIDTH: f32 = 2.0 * size::METER_WIDTH + spacing::LG;
const METER_BLOCK_HEIGHT: f32 = 30.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DelayReadout {
    pub milliseconds: Option<f64>,
    pub distance_m: Option<f64>,
    pub samples: Option<f64>,
    pub confidence_db: Option<f64>,
    pub approximate: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TopBarModel {
    pub status_tone: Tone,
    pub status_text: &'static str,
    pub source_label: Option<String>,
    pub reference_meter: MeterReading,
    pub measurement_meter: MeterReading,
    pub delay: DelayReadout,
    pub can_apply_delay: bool,
    pub running: bool,
    pub tracking: bool,
}

pub fn show(
    ui: &mut egui::Ui,
    palette: &Palette,
    model: &TopBarModel,
    is_mac: bool,
    actions: &mut Vec<Action>,
) {
    let total_width = ui.available_width();
    let full_right_width = right_group_width(ui, is_mac, false);
    let compact = total_width < full_right_width + FULL_LEFT_MIN_WIDTH;
    let right_width = if compact {
        right_group_width(ui, is_mac, true)
    } else {
        full_right_width
    };
    let group_gap = if compact { spacing::MD } else { spacing::XL };
    let left_width = (total_width - right_width).max(if compact { 0.0 } else { MIN_SOURCE_WIDTH });
    let shortcut = |action: Action| {
        if compact {
            None
        } else {
            action.shortcut_text(is_mac)
        }
    };
    let height = ui.available_height();
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            vec2(left_width, height),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.x = spacing::MD;
                status_pill(ui, palette, model.status_tone, model.status_text);
                if !compact {
                    ui.label(
                        RichText::new(text::APP_NAME)
                            .font(fonts::ui_semibold(text_size::TITLE))
                            .color(palette.text_primary),
                    );
                }
                if let Some(source) = &model.source_label {
                    ui.add(
                        egui::Label::new(
                            RichText::new(source)
                                .font(fonts::ui(text_size::LABEL))
                                .color(palette.text_muted),
                        )
                        .truncate(),
                    );
                }
            },
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = spacing::SM;
            if action_button(
                ui,
                palette,
                text::BUTTON_APPLY_DELAY,
                shortcut(Action::ApplyDelay).as_deref(),
                Emphasis::Secondary,
                model.can_apply_delay,
            )
            .clicked()
            {
                actions.push(Action::ApplyDelay);
            }
            if action_button(
                ui,
                palette,
                text::BUTTON_FIND_DELAY,
                shortcut(Action::FindDelay).as_deref(),
                Emphasis::Secondary,
                model.running,
            )
            .clicked()
            {
                actions.push(Action::FindDelay);
            }
            ui.add_space(spacing::MD);
            fine_controls(ui, palette, model, is_mac, actions);
            ui.add_space(spacing::MD);
            ui.allocate_ui_with_layout(
                vec2(DELAY_BLOCK_WIDTH, height),
                Layout::top_down(Align::Max),
                |ui| delay_block(ui, palette, &model.delay),
            );
            ui.add_space(group_gap);
            ui.allocate_ui_with_layout(
                vec2(METER_GROUP_WIDTH, METER_BLOCK_HEIGHT),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing.x = spacing::LG;
                    level_meter(ui, palette, text::METER_REFERENCE, model.reference_meter);
                    level_meter(
                        ui,
                        palette,
                        text::METER_MEASUREMENT,
                        model.measurement_meter,
                    );
                },
            );
        });
    });
}

fn right_group_width(ui: &egui::Ui, is_mac: bool, compact: bool) -> f32 {
    let shortcut = |action: Action| {
        if compact {
            None
        } else {
            action.shortcut_text(is_mac)
        }
    };
    let buttons = action_button_width(
        ui,
        text::BUTTON_APPLY_DELAY,
        shortcut(Action::ApplyDelay).as_deref(),
    ) + action_button_width(
        ui,
        text::BUTTON_FIND_DELAY,
        shortcut(Action::FindDelay).as_deref(),
    ) + spacing::SM;
    let gaps = if compact {
        4.0 * spacing::MD
    } else {
        4.0 * spacing::XL
    };
    buttons + FINE_GROUP_WIDTH + DELAY_BLOCK_WIDTH + METER_GROUP_WIDTH + gaps
}

fn delay_block(ui: &mut egui::Ui, palette: &Palette, delay: &DelayReadout) {
    ui.spacing_mut().item_spacing.y = 0.0;
    let main = delay
        .milliseconds
        .map_or_else(|| text::DELAY_NONE.to_owned(), text::format_milliseconds);
    ui.label(
        RichText::new(main)
            .font(fonts::value_semibold(text_size::DISPLAY))
            .color(palette.text_primary),
    );
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = spacing::SM;
        confidence_segments(ui, palette, delay.confidence_db, text::DELAY_CONFIDENCE);
        let details = match (delay.distance_m, delay.samples) {
            (Some(distance), Some(samples)) => {
                text::delay_details(distance, samples, delay.approximate)
            }
            _ => text::DELAY_TITLE.to_owned(),
        };
        ui.label(
            RichText::new(details)
                .font(fonts::value(text_size::CAPTION))
                .color(palette.text_muted),
        );
    });
}

fn fine_controls(
    ui: &mut egui::Ui,
    palette: &Palette,
    model: &TopBarModel,
    is_mac: bool,
    actions: &mut Vec<Action>,
) {
    ui.spacing_mut().item_spacing.x = spacing::XS;
    let tracking = ui
        .add_sized(
            vec2(TRACKING_WIDTH, size::MIN_CLICK_HEIGHT),
            egui::Button::new(
                RichText::new(text::tracking_label(model.tracking))
                    .font(fonts::ui_medium(text_size::LABEL))
                    .color(if model.tracking {
                        palette.accent
                    } else {
                        palette.text_muted
                    }),
            )
            .selected(model.tracking)
            .stroke(egui::Stroke::new(
                stroke::BORDER,
                if model.tracking {
                    palette.accent
                } else {
                    palette.border
                },
            )),
        )
        .on_hover_text(hint(Action::ToggleDelayTracking, is_mac));
    tracking.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::Role::CheckBox,
            true,
            model.tracking,
            text::ACTION_TOGGLE_DELAY_TRACKING,
        )
    });
    if tracking.clicked() {
        actions.push(Action::ToggleDelayTracking);
    }
    for (action, glyph) in [
        (Action::NudgeDelayUp, text::BUTTON_NUDGE_UP),
        (Action::NudgeDelayDown, text::BUTTON_NUDGE_DOWN),
    ] {
        let button = ui
            .add_enabled(
                model.running,
                egui::Button::new(
                    RichText::new(glyph)
                        .font(fonts::ui_semibold(text_size::HEADING))
                        .color(palette.text_primary),
                )
                .min_size(vec2(size::MIN_CLICK_HEIGHT, size::MIN_CLICK_HEIGHT)),
            )
            .on_hover_text(hint(action, is_mac));
        button.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Button, model.running, action.label())
        });
        if button.clicked() {
            actions.push(action);
        }
    }
}

fn hint(action: Action, is_mac: bool) -> String {
    match action.shortcut_text(is_mac) {
        Some(shortcut) => format!("{} ({shortcut})", action.label()),
        None => action.label().to_owned(),
    }
}
