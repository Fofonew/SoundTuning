use eframe::egui::{
    self, Align2, CornerRadius, Response, Role, Sense, Stroke, StrokeKind, Ui, WidgetInfo, vec2,
};

use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{radius, size, stroke};

pub const ICON_SIZE: f32 = 18.0;

pub const COMMAND: &str = "\u{E1C4}";
pub const SLIDERS: &str = "\u{E434}";
pub const SIDEBAR: &str = "\u{EC24}";
pub const STACK: &str = "\u{E5A2}";
pub const FULLSCREEN: &str = "\u{E1D0}";
pub const EXIT_FULLSCREEN: &str = "\u{E1CE}";
pub const CLOSE: &str = "\u{E4F6}";

pub fn icon_button_width() -> f32 {
    size::BUTTON_HEIGHT
}

pub fn icon_button(
    ui: &mut Ui,
    palette: &Palette,
    glyph: &str,
    label: &str,
    tooltip: &str,
    selected: bool,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        vec2(icon_button_width(), size::BUTTON_HEIGHT),
        Sense::click(),
    );
    response.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, label));
    if ui.is_rect_visible(rect) {
        let hovered = response.hovered();
        let fill = if selected {
            palette.surface_hover
        } else if hovered {
            palette.surface_hover.gamma_multiply(0.6)
        } else {
            egui::Color32::TRANSPARENT
        };
        let border = if selected || hovered {
            palette.border
        } else {
            egui::Color32::TRANSPARENT
        };
        ui.painter().rect(
            rect,
            CornerRadius::same(radius::MEDIUM),
            fill,
            Stroke::new(stroke::BORDER, border),
            StrokeKind::Inside,
        );
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            glyph,
            fonts::icon(ICON_SIZE),
            if selected {
                palette.accent
            } else {
                palette.text_primary
            },
        );
    }
    response.on_hover_text(tooltip)
}
