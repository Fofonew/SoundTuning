use eframe::egui::{
    self, CornerRadius, CursorIcon, Rect, Role, Sense, Stroke, WidgetInfo, pos2, vec2,
};

use crate::design::Palette;
use crate::design::tokens::stroke;
use crate::plot::{DEFAULT_UPPER_PANE_FRACTION, upper_fraction_at};
use crate::text;

const GRIP_LENGTH: f32 = 20.0;
const GRIP_THICKNESS: f32 = 2.0;
const GRIP_SPACING: f32 = 4.0;
const GRIP_RADIUS: u8 = 1;
const ACTIVE_LINE_ALPHA: f32 = 0.4;

pub struct SplitArea {
    pub top: f32,
    pub available_height: f32,
}

pub fn show(
    ui: &mut egui::Ui,
    palette: &Palette,
    gap: Rect,
    grip_center: egui::Pos2,
    area: SplitArea,
) -> Option<f32> {
    let response = ui
        .interact(
            gap,
            ui.make_persistent_id("pane-splitter"),
            Sense::click_and_drag(),
        )
        .on_hover_text(text::PANE_SPLITTER_HINT);
    response.widget_info(|| WidgetInfo::labeled(Role::Splitter, true, text::PANE_SPLITTER_LABEL));
    let active = response.hovered() || response.dragged();
    if active {
        ui.ctx().set_cursor_icon(CursorIcon::ResizeVertical);
    }
    for offset in [-GRIP_SPACING / 2.0, GRIP_SPACING / 2.0] {
        let grip = Rect::from_center_size(
            grip_center + vec2(0.0, offset),
            vec2(GRIP_LENGTH, GRIP_THICKNESS),
        );
        ui.painter().rect_filled(
            grip,
            CornerRadius::same(GRIP_RADIUS),
            if active {
                palette.accent
            } else {
                palette.text_muted
            },
        );
    }
    if active {
        ui.painter().line_segment(
            [pos2(gap.left(), gap.top()), pos2(gap.right(), gap.top())],
            Stroke::new(
                stroke::BORDER,
                palette.accent.gamma_multiply(ACTIVE_LINE_ALPHA),
            ),
        );
        ui.painter().line_segment(
            [
                pos2(gap.left(), gap.bottom()),
                pos2(gap.right(), gap.bottom()),
            ],
            Stroke::new(
                stroke::BORDER,
                palette.accent.gamma_multiply(ACTIVE_LINE_ALPHA),
            ),
        );
    }
    if response.double_clicked() {
        return Some(DEFAULT_UPPER_PANE_FRACTION);
    }
    let delta_y = response.drag_delta().y;
    if response.dragged() && delta_y != 0.0 {
        let upper_height = gap.top() - area.top + delta_y;
        return Some(upper_fraction_at(area.available_height, upper_height));
    }
    None
}
