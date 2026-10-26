use eframe::egui::{
    self, CornerRadius, CursorIcon, Rect, Role, Sense, Stroke, WidgetInfo, pos2, vec2,
};

use crate::design::Palette;
use crate::design::tokens::stroke;
use crate::plot::{DEFAULT_UPPER_PANE_FRACTION, upper_fraction_at};
use crate::text;

const HIT_MARGIN: f32 = 3.0;
const GRIP_WIDTH: f32 = 48.0;
const GRIP_THICKNESS: f32 = 3.0;
const GRIP_RADIUS: u8 = 2;
const ACTIVE_LINE_ALPHA: f32 = 0.4;

pub struct SplitArea {
    pub top: f32,
    pub available_height: f32,
}

pub fn show(ui: &mut egui::Ui, palette: &Palette, gap: Rect, area: SplitArea) -> Option<f32> {
    let response = ui
        .interact(
            gap.expand2(vec2(0.0, HIT_MARGIN)),
            ui.make_persistent_id("pane-splitter"),
            Sense::click_and_drag(),
        )
        .on_hover_text(text::PANE_SPLITTER_HINT);
    response.widget_info(|| WidgetInfo::labeled(Role::Splitter, true, text::PANE_SPLITTER_LABEL));
    let active = response.hovered() || response.dragged();
    if active {
        ui.ctx().set_cursor_icon(CursorIcon::ResizeVertical);
    }
    let grip = Rect::from_center_size(gap.center(), vec2(GRIP_WIDTH, GRIP_THICKNESS));
    ui.painter().rect_filled(
        grip,
        CornerRadius::same(GRIP_RADIUS),
        if active {
            palette.accent
        } else {
            palette.border
        },
    );
    if active {
        ui.painter().line_segment(
            [
                pos2(gap.left(), gap.center().y),
                pos2(gap.right(), gap.center().y),
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
