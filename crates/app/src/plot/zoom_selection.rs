use eframe::egui::{CornerRadius, Painter, Pos2, Rect, Stroke};

use crate::design::Palette;
use crate::design::tokens::stroke;

const MIN_DRAG_PX: f32 = 8.0;
const AXIS_LOCK_RATIO: f32 = 0.35;
const FILL_ALPHA: f32 = 0.08;
const EDGE_ALPHA: f32 = 0.7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoomAxes {
    Horizontal,
    Vertical,
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZoomSelection {
    origin: Pos2,
    current: Pos2,
    vertical_allowed: bool,
}

impl ZoomSelection {
    pub fn new(origin: Pos2, vertical_allowed: bool) -> Self {
        Self {
            origin,
            current: origin,
            vertical_allowed,
        }
    }

    pub fn update(&mut self, plot_rect: Rect, position: Pos2) {
        self.current = position.clamp(plot_rect.min, plot_rect.max);
    }

    pub fn axes(&self, plot_rect: Rect) -> Option<ZoomAxes> {
        let width = (self.current.x - self.origin.x).abs();
        let height = (self.current.y - self.origin.y).abs();
        let wide = width >= MIN_DRAG_PX;
        let tall = height >= MIN_DRAG_PX && self.vertical_allowed;
        let width_fraction = width / plot_rect.width().max(1.0);
        let height_fraction = height / plot_rect.height().max(1.0);
        match (wide, tall) {
            (false, false) => None,
            (true, false) => Some(ZoomAxes::Horizontal),
            (false, true) => Some(ZoomAxes::Vertical),
            (true, true) if height_fraction < AXIS_LOCK_RATIO * width_fraction => {
                Some(ZoomAxes::Horizontal)
            }
            (true, true) if width_fraction < AXIS_LOCK_RATIO * height_fraction => {
                Some(ZoomAxes::Vertical)
            }
            (true, true) => Some(ZoomAxes::Both),
        }
    }

    pub fn x_range(&self) -> (f32, f32) {
        (
            self.origin.x.min(self.current.x),
            self.origin.x.max(self.current.x),
        )
    }

    pub fn y_range(&self) -> (f32, f32) {
        (
            self.origin.y.min(self.current.y),
            self.origin.y.max(self.current.y),
        )
    }

    pub fn region(&self, plot_rect: Rect) -> Option<Rect> {
        let (left, right) = self.x_range();
        let (top, bottom) = self.y_range();
        Some(match self.axes(plot_rect)? {
            ZoomAxes::Horizontal => Rect::from_x_y_ranges(left..=right, plot_rect.y_range()),
            ZoomAxes::Vertical => Rect::from_x_y_ranges(plot_rect.x_range(), top..=bottom),
            ZoomAxes::Both => Rect::from_x_y_ranges(left..=right, top..=bottom),
        })
    }

    pub fn paint(&self, painter: &Painter, palette: &Palette, plot_rect: Rect) {
        let Some(axes) = self.axes(plot_rect) else {
            return;
        };
        let Some(region) = self.region(plot_rect) else {
            return;
        };
        painter.rect_filled(
            region,
            CornerRadius::ZERO,
            palette.text_primary.gamma_multiply(FILL_ALPHA),
        );
        let edge = Stroke::new(
            stroke::CROSSHAIR,
            palette.text_primary.gamma_multiply(EDGE_ALPHA),
        );
        let vertical_edges = [
            [region.left_top(), region.left_bottom()],
            [region.right_top(), region.right_bottom()],
        ];
        let horizontal_edges = [
            [region.left_top(), region.right_top()],
            [region.left_bottom(), region.right_bottom()],
        ];
        let edges: &[[Pos2; 2]] = match axes {
            ZoomAxes::Horizontal => &vertical_edges,
            ZoomAxes::Vertical => &horizontal_edges,
            ZoomAxes::Both => &[
                vertical_edges[0],
                vertical_edges[1],
                horizontal_edges[0],
                horizontal_edges[1],
            ],
        };
        for edge_points in edges {
            painter.line_segment(*edge_points, edge);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{pos2, vec2};

    fn rect() -> Rect {
        Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 500.0))
    }

    fn selection(from: Pos2, to: Pos2, vertical_allowed: bool) -> ZoomSelection {
        let mut selection = ZoomSelection::new(from, vertical_allowed);
        selection.update(rect(), to);
        selection
    }

    #[test]
    fn a_wide_drag_zooms_frequency_only() {
        let wide = selection(pos2(100.0, 200.0), pos2(500.0, 230.0), true);
        assert_eq!(wide.axes(rect()), Some(ZoomAxes::Horizontal));
        assert_eq!(wide.region(rect()).unwrap().height(), 500.0);
    }

    #[test]
    fn a_tall_drag_zooms_level_only() {
        let tall = selection(pos2(400.0, 50.0), pos2(420.0, 300.0), true);
        assert_eq!(tall.axes(rect()), Some(ZoomAxes::Vertical));
        assert_eq!(tall.region(rect()).unwrap().width(), 1000.0);
    }

    #[test]
    fn a_diagonal_drag_zooms_both_axes() {
        let diagonal = selection(pos2(100.0, 100.0), pos2(400.0, 300.0), true);
        assert_eq!(diagonal.axes(rect()), Some(ZoomAxes::Both));
        assert_eq!(
            diagonal.region(rect()),
            Some(Rect::from_min_max(pos2(100.0, 100.0), pos2(400.0, 300.0)))
        );
    }

    #[test]
    fn short_drags_and_vertical_only_drags_on_time_plots_are_ignored() {
        assert_eq!(
            selection(pos2(100.0, 100.0), pos2(104.0, 103.0), true).axes(rect()),
            None
        );
        assert_eq!(
            selection(pos2(100.0, 100.0), pos2(102.0, 400.0), false).axes(rect()),
            None
        );
        assert_eq!(
            selection(pos2(100.0, 100.0), pos2(400.0, 400.0), false).axes(rect()),
            Some(ZoomAxes::Horizontal)
        );
    }

    #[test]
    fn the_selection_stays_inside_the_plot() {
        let outside = selection(pos2(900.0, 100.0), pos2(1500.0, 120.0), true);
        assert_eq!(outside.x_range(), (900.0, 1000.0));
    }
}
