use eframe::egui::{
    self, Align, Color32, CornerRadius, Layout, Rect, Role, Sense, Stroke, StrokeKind, WidgetInfo,
    pos2, vec2,
};

use crate::actions::Action;
use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{radius, size, spacing, stroke, text_size};
use crate::design::widgets::toggle_button;
use crate::plot::{ExternalModule, MouseMode, ViewKind};
use crate::text;

const VIEW_BUTTON_WIDTH: f32 = 44.0;
const MODULE_BUTTON_WIDTH: f32 = 84.0;
const GROUP_GAP: f32 = 16.0;
const LEGEND_HEIGHT: f32 = 20.0;
const LEGEND_SWATCH: f32 = 10.0;
const ARROW_REACH: f32 = 7.0;
const ARROW_HEAD: f32 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewCommand {
    Main(ViewKind),
    Upper(ViewKind),
    ToggleStacked,
    ImpulseInDb(bool),
    ToggleFullscreen,
    FullView(ViewKind),
    Module(ExternalModule),
    MouseMode(MouseMode),
    ShowCoherence(bool),
}

#[derive(Clone, Copy, Debug)]
pub struct LegendItem<'a> {
    pub label: &'a str,
    pub color: Color32,
}

pub fn view_label(kind: ViewKind) -> &'static str {
    match kind {
        ViewKind::Magnitude => text::VIEW_MAGNITUDE,
        ViewKind::Phase => text::VIEW_PHASE,
        ViewKind::Impulse => text::VIEW_IMPULSE,
        ViewKind::VirtualEq => text::VIEW_VIRTUAL_EQ,
    }
}

fn view_action(kind: ViewKind) -> Action {
    match kind {
        ViewKind::Magnitude => Action::ViewMagnitude,
        ViewKind::Phase => Action::ViewPhase,
        ViewKind::Impulse => Action::ViewImpulse,
        ViewKind::VirtualEq => Action::ViewVirtualEq,
    }
}

fn view_hint(kind: ViewKind) -> &'static str {
    match kind {
        ViewKind::Magnitude => text::ACTION_VIEW_MAGNITUDE,
        ViewKind::Phase => text::ACTION_VIEW_PHASE,
        ViewKind::Impulse => text::ACTION_VIEW_IMPULSE,
        ViewKind::VirtualEq => text::ACTION_VIEW_VIRTUAL_EQ,
    }
}

pub fn height(has_legend: bool) -> f32 {
    group_height() + if has_legend { LEGEND_HEIGHT } else { 0.0 }
}

fn group_height() -> f32 {
    text_size::CAPTION + spacing::XS + size::MIN_CLICK_HEIGHT
}

fn group(
    ui: &mut egui::Ui,
    palette: &Palette,
    caption: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = spacing::XS;
        ui.label(
            egui::RichText::new(caption)
                .font(fonts::ui_medium(text_size::CAPTION))
                .color(palette.text_muted),
        );
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = size::SEGMENT_GAP;
            add_contents(ui);
        });
    });
}

pub fn show(
    ui: &mut egui::Ui,
    palette: &Palette,
    selected: ViewKind,
    upper: bool,
    legend: &[LegendItem<'_>],
    is_mac: bool,
) -> Vec<ViewCommand> {
    let mut commands = Vec::new();
    let view_button = |ui: &mut egui::Ui, kind: ViewKind, width: f32| -> Option<ViewCommand> {
        let label = view_label(kind);
        let response = toggle_button(ui, palette, label, selected == kind, Some(width));
        let response = if upper {
            response
                .accessible_name(text::upper_view_label(label))
                .on_hover_text(view_hint(kind))
        } else {
            response.on_hover_text(text::with_shortcut(
                view_hint(kind),
                view_action(kind).shortcut_text(is_mac).as_deref(),
            ))
        };
        (response.clicked() && selected != kind).then_some(if upper {
            ViewCommand::Upper(kind)
        } else {
            ViewCommand::Main(kind)
        })
    };
    let row_height = group_height();
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), row_height),
        Layout::left_to_right(Align::Max),
        |ui| {
            ui.spacing_mut().item_spacing.x = GROUP_GAP;
            if selected != ViewKind::VirtualEq && full_view_button(ui, palette, upper).clicked() {
                commands.push(ViewCommand::FullView(selected));
            }
            group(ui, palette, text::GROUP_TRANSFER_FUNCTION, |ui| {
                for kind in [ViewKind::Impulse, ViewKind::Magnitude, ViewKind::Phase] {
                    commands.extend(view_button(ui, kind, VIEW_BUTTON_WIDTH));
                }
            });
            group(ui, palette, text::GROUP_EXTERNAL, |ui| {
                commands.extend(view_button(ui, ViewKind::VirtualEq, MODULE_BUTTON_WIDTH));
            });
        },
    );
    if !legend.is_empty() {
        legend_row(ui, palette, legend);
    }
    commands
}

fn full_view_button(ui: &mut egui::Ui, palette: &Palette, upper: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        vec2(size::MIN_CLICK_HEIGHT, size::MIN_CLICK_HEIGHT),
        Sense::click(),
    );
    let label = if upper {
        text::upper_view_label(text::FULL_VIEW)
    } else {
        text::FULL_VIEW.to_owned()
    };
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, &label));
    if ui.is_rect_visible(rect) {
        let hovered = response.hovered();
        ui.painter().rect(
            rect,
            CornerRadius::same(radius::SMALL),
            if hovered {
                palette.surface_hover
            } else {
                palette.surface_raised
            },
            Stroke::new(stroke::BORDER, palette.border),
            StrokeKind::Inside,
        );
        let color = if hovered {
            palette.text_primary
        } else {
            palette.text_muted
        };
        let line = Stroke::new(stroke::FOCUS * 0.75, color);
        let center = rect.center();
        for direction in [
            vec2(1.0, 0.0),
            vec2(-1.0, 0.0),
            vec2(0.0, 1.0),
            vec2(0.0, -1.0),
        ] {
            let tip = center + direction * ARROW_REACH;
            let side = vec2(-direction.y, direction.x) * ARROW_HEAD;
            ui.painter().line_segment([center, tip], line);
            ui.painter()
                .line_segment([tip, tip - direction * ARROW_HEAD + side], line);
            ui.painter()
                .line_segment([tip, tip - direction * ARROW_HEAD - side], line);
        }
    }
    response
        .on_hover_text(text::FULL_VIEW_HINT)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn legend_row(ui: &mut egui::Ui, palette: &Palette, legend: &[LegendItem<'_>]) {
    let (rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), LEGEND_HEIGHT), Sense::hover());
    let painter = ui.painter_at(rect);
    let mut x = rect.left();
    for item in legend {
        if x >= rect.right() {
            break;
        }
        let swatch = Rect::from_center_size(
            pos2(x + LEGEND_SWATCH / 2.0, rect.center().y),
            vec2(LEGEND_SWATCH, LEGEND_SWATCH),
        );
        painter.rect_filled(swatch, CornerRadius::same(2), item.color);
        let galley = painter.layout_no_wrap(
            item.label.to_owned(),
            fonts::ui_medium(text_size::CAPTION),
            palette.text_primary,
        );
        let text_left = swatch.right() + spacing::XS;
        let width = galley.size().x;
        painter.galley(
            pos2(text_left, rect.center().y - galley.size().y / 2.0),
            galley,
            palette.text_primary,
        );
        x = text_left + width + spacing::MD;
    }
}
