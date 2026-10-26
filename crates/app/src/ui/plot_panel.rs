use eframe::egui::{
    self, Align2, Color32, CornerRadius, Pos2, Rect, Role, Sense, Shape, Stroke, StrokeKind,
    WidgetInfo, pos2, vec2,
};
use engine::MeasurementSnapshot;

use crate::actions::Action;
use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{radius, size, spacing, stroke, text_size};
use crate::design::widgets::{Emphasis, action_button};
use crate::plot::axis::{db_ticks, frequency_ticks};
use crate::plot::{PlotFrame, PlotView, Polyline, TraceStyle, build_polyline};
use crate::text;

const LEFT_AXIS_WIDTH: f32 = 40.0;
const BOTTOM_AXIS_HEIGHT: f32 = 20.0;
const PLOT_PADDING: f32 = 8.0;
const PLOT_TOP_PADDING: f32 = 24.0;
const EDGE_LABEL_MARGIN: f32 = 16.0;
const COHERENCE_FILL_ALPHA_SOFT: f32 = 0.38;
const SCROLL_ZOOM_SENSITIVITY: f64 = 0.0025;
const GRID_POINTS_PER_OCTAVE: f64 = 96.0;
const GRID_LOW_HZ: f64 = 20.0;
const READOUT_OFFSET: f32 = 14.0;
const COHERENCE_FILL_ALPHA: f32 = 0.75;
const COHERENCE_LOW_ALPHA: f32 = 0.55;
const EMPTY_STATE_WIDTH: f32 = 360.0;
const MIN_PLOT_SIZE: f32 = 80.0;

pub struct OverlayTrace<'a> {
    pub label: &'a str,
    pub values_db: &'a [Option<f64>],
    pub style: TraceStyle,
}

pub struct PlotInputs<'a> {
    pub frequencies_hz: &'a [f64],
    pub snapshot: &'a MeasurementSnapshot,
    pub show_live: bool,
    pub running: bool,
    pub overlays: &'a [OverlayTrace<'a>],
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlotEvents {
    pub double_click_hz_db: Option<(f64, f64)>,
    pub plot_rect: Option<Rect>,
}

#[derive(Debug, Default)]
pub struct PlotPanel {
    pub view: PlotView,
    frame: PlotFrame,
    overlay_polylines: Vec<Polyline>,
}

impl PlotPanel {
    pub fn new(view: PlotView) -> Self {
        Self {
            view,
            frame: PlotFrame::with_capacity(1024),
            overlay_polylines: Vec::new(),
        }
    }

    pub fn prepare(&mut self, plot_rect: Rect, inputs: &PlotInputs<'_>) {
        if inputs.show_live {
            self.frame.prepare_live(
                &self.view,
                plot_rect,
                inputs.frequencies_hz,
                inputs.snapshot,
            );
        } else {
            self.frame.live.clear();
            self.frame.coherence_columns.clear();
        }
        self.overlay_polylines
            .resize_with(inputs.overlays.len(), || Polyline::with_capacity(1024));
        for (polyline, overlay) in self.overlay_polylines.iter_mut().zip(inputs.overlays) {
            build_polyline(
                &self.view,
                plot_rect,
                inputs.frequencies_hz,
                |index| overlay.values_db.get(index).copied().flatten(),
                polyline,
            );
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        palette: &Palette,
        inputs: &PlotInputs<'_>,
        is_mac: bool,
        actions: &mut Vec<Action>,
    ) -> PlotEvents {
        let outer = ui.available_rect_before_wrap();
        let band_height = size::COHERENCE_BAND_HEIGHT;
        let plot_left = outer.left() + LEFT_AXIS_WIDTH;
        let plot_top = outer.top() + PLOT_TOP_PADDING;
        let plot_rect = Rect::from_min_max(
            pos2(plot_left, plot_top),
            pos2(
                (outer.right() - PLOT_PADDING).max(plot_left + MIN_PLOT_SIZE),
                (outer.bottom() - band_height - BOTTOM_AXIS_HEIGHT - spacing::SM)
                    .max(plot_top + MIN_PLOT_SIZE),
            ),
        );
        let band_rect = Rect::from_min_max(
            pos2(plot_rect.left(), outer.bottom() - band_height),
            pos2(plot_rect.right(), outer.bottom()),
        );
        ui.allocate_rect(outer, Sense::hover());
        let response = ui.interact(
            plot_rect,
            ui.make_persistent_id("magnitude-plot"),
            Sense::click_and_drag(),
        );
        response
            .widget_info(|| WidgetInfo::labeled(Role::Figure, true, text::PLOT_ACCESSIBLE_LABEL));
        let mut events = PlotEvents {
            plot_rect: Some(plot_rect),
            ..PlotEvents::default()
        };
        self.handle_interaction(ui, &response, plot_rect, &mut events);
        self.prepare(plot_rect, inputs);

        let painter = ui.painter_at(outer);
        painter.rect_filled(
            plot_rect,
            CornerRadius::same(radius::SMALL),
            palette.plot_background,
        );
        self.paint_grid(&painter, palette, plot_rect);
        for (polyline, overlay) in self.overlay_polylines.iter().zip(inputs.overlays) {
            paint_polyline(&painter, polyline, overlay.style);
        }
        if inputs.show_live {
            paint_polyline(
                &painter,
                &self.frame.live,
                TraceStyle {
                    color: palette.trace_live,
                    width: stroke::TRACE_LIVE,
                    halo: Some((stroke::TRACE_LIVE_HALO, stroke::TRACE_LIVE_HALO_ALPHA)),
                },
            );
        }
        painter.rect_stroke(
            plot_rect,
            CornerRadius::same(radius::SMALL),
            Stroke::new(stroke::BORDER, palette.border),
            StrokeKind::Inside,
        );
        self.paint_coherence_band(&painter, palette, band_rect, inputs);

        if let Some(hover) = response.hover_pos() {
            self.paint_crosshair(&painter, palette, plot_rect, hover, inputs);
        }
        let has_data = inputs.snapshot.averages > 0 || !inputs.overlays.is_empty();
        if !inputs.running && !has_data {
            paint_empty_state(ui, palette, plot_rect, is_mac, actions);
        } else if inputs.running && inputs.snapshot.warming_up {
            painter.text(
                plot_rect.left_top() + vec2(spacing::MD, spacing::MD),
                Align2::LEFT_TOP,
                text::PLOT_WARMING_UP,
                fonts::ui_medium(text_size::LABEL),
                palette.text_muted,
            );
        }
        events
    }

    fn handle_interaction(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        plot_rect: Rect,
        events: &mut PlotEvents,
    ) {
        if response.double_clicked()
            && let Some(position) = response.interact_pointer_pos()
        {
            events.double_click_hz_db = Some((
                self.view.x_to_frequency(plot_rect, position.x),
                self.view.y_to_db(plot_rect, position.y),
            ));
        }
        if response.dragged() {
            let delta = response.drag_delta();
            self.view.pan(plot_rect, delta.x, delta.y);
        }
        if let Some(hover) = response.hover_pos() {
            let (scroll, zoom, alternate) = ui.input(|input| {
                (
                    input.smooth_scroll_delta,
                    input.zoom_delta(),
                    input.modifiers.alt,
                )
            });
            let anchor_hz = self.view.x_to_frequency(plot_rect, hover.x);
            let anchor_db = self.view.y_to_db(plot_rect, hover.y);
            if (zoom - 1.0).abs() > f32::EPSILON {
                let factor = 1.0 / f64::from(zoom);
                self.view.zoom_frequency(anchor_hz, factor);
                self.view.zoom_level(anchor_db, factor);
            }
            if scroll.y.abs() > f32::EPSILON {
                let factor = (-f64::from(scroll.y) * SCROLL_ZOOM_SENSITIVITY).exp();
                if alternate {
                    self.view.zoom_level(anchor_db, factor);
                } else {
                    self.view.zoom_frequency(anchor_hz, factor);
                }
            }
            if scroll.x.abs() > f32::EPSILON {
                self.view.pan(plot_rect, scroll.x, 0.0);
            }
        }
    }

    fn paint_grid(&self, painter: &egui::Painter, palette: &Palette, plot_rect: Rect) {
        for tick in frequency_ticks(self.view.low_hz, self.view.high_hz) {
            let x = self.view.frequency_to_x(plot_rect, tick.frequency_hz);
            let (color, width) = if tick.is_major {
                (palette.grid_major, stroke::GRID_MAJOR)
            } else {
                (palette.grid_minor, stroke::GRID_MINOR)
            };
            painter.line_segment(
                [pos2(x, plot_rect.top()), pos2(x, plot_rect.bottom())],
                Stroke::new(width, color),
            );
            if tick.is_major {
                let anchor = if x > plot_rect.right() - EDGE_LABEL_MARGIN {
                    Align2::RIGHT_TOP
                } else if x < plot_rect.left() + EDGE_LABEL_MARGIN {
                    Align2::LEFT_TOP
                } else {
                    Align2::CENTER_TOP
                };
                painter.text(
                    pos2(x, plot_rect.bottom() + spacing::XS),
                    anchor,
                    text::format_axis_frequency(tick.frequency_hz),
                    fonts::value(text_size::CAPTION),
                    palette.text_muted,
                );
            }
        }
        for level_db in db_ticks(self.view.low_db, self.view.high_db) {
            let y = self.view.db_to_y(plot_rect, level_db);
            let is_zero = level_db.abs() < f64::EPSILON;
            painter.line_segment(
                [pos2(plot_rect.left(), y), pos2(plot_rect.right(), y)],
                Stroke::new(
                    if is_zero {
                        stroke::GRID_MAJOR * 1.5
                    } else {
                        stroke::GRID_MAJOR
                    },
                    if is_zero {
                        palette.border
                    } else {
                        palette.grid_major.gamma_multiply(0.8)
                    },
                ),
            );
            painter.text(
                pos2(plot_rect.left() - spacing::SM, y),
                Align2::RIGHT_CENTER,
                text::format_axis_db(level_db),
                fonts::value(text_size::CAPTION),
                palette.text_muted,
            );
        }
        painter.text(
            pos2(
                plot_rect.left() - spacing::SM,
                plot_rect.top() - spacing::LG,
            ),
            Align2::RIGHT_CENTER,
            text::AXIS_DB,
            fonts::caption(),
            palette.text_muted,
        );
    }

    fn paint_coherence_band(
        &self,
        painter: &egui::Painter,
        palette: &Palette,
        band_rect: Rect,
        inputs: &PlotInputs<'_>,
    ) {
        painter.rect_filled(
            band_rect,
            CornerRadius::same(radius::SMALL),
            palette.plot_background,
        );
        let threshold = inputs.snapshot.settings.coherence_threshold;
        let mut mesh = egui::Mesh::default();
        let mut outline: Vec<Pos2> = Vec::with_capacity(self.frame.coherence_columns.len());
        for column in &self.frame.coherence_columns {
            let Some(coherence) = column.coherence else {
                continue;
            };
            let height = band_rect.height() * coherence.clamp(0.0, 1.0) as f32;
            let color = if coherence >= threshold {
                palette
                    .coherence_band
                    .gamma_multiply(COHERENCE_FILL_ALPHA_SOFT)
            } else {
                palette.warning.gamma_multiply(COHERENCE_LOW_ALPHA)
            };
            outline.push(pos2(
                (column.left + column.right) / 2.0,
                band_rect.bottom() - height,
            ));
            mesh.add_colored_rect(
                Rect::from_min_max(
                    pos2(column.left, band_rect.bottom() - height),
                    pos2(column.right.max(column.left + 0.5), band_rect.bottom()),
                ),
                color,
            );
        }
        painter.add(Shape::mesh(mesh));
        if outline.len() > 1 {
            painter.add(Shape::line(
                outline,
                Stroke::new(
                    stroke::TRACE_STORED,
                    palette.coherence_band.gamma_multiply(COHERENCE_FILL_ALPHA),
                ),
            ));
        }
        let threshold_y = band_rect.bottom() - band_rect.height() * threshold as f32;
        painter.add(Shape::dashed_line(
            &[
                pos2(band_rect.left(), threshold_y),
                pos2(band_rect.right(), threshold_y),
            ],
            Stroke::new(stroke::GRID_MAJOR, palette.text_muted),
            4.0,
            4.0,
        ));
        painter.rect_stroke(
            band_rect,
            CornerRadius::same(radius::SMALL),
            Stroke::new(stroke::BORDER, palette.border),
            StrokeKind::Inside,
        );
        let label = painter.layout_no_wrap(
            text::COHERENCE_LABEL.to_owned(),
            fonts::ui_medium(text_size::CAPTION),
            palette.text_muted,
        );
        let label_rect = Rect::from_min_size(
            band_rect.left_top() + vec2(spacing::SM, spacing::XS),
            label.size() + vec2(2.0 * spacing::SM, spacing::XS),
        );
        painter.rect_filled(
            label_rect,
            CornerRadius::same(radius::SMALL),
            palette.surface.gamma_multiply(0.9),
        );
        painter.galley(
            label_rect.left_top() + vec2(spacing::SM, spacing::XS / 2.0),
            label,
            palette.text_muted,
        );
        painter.text(
            pos2(band_rect.left() - spacing::SM, band_rect.top()),
            Align2::RIGHT_TOP,
            text::COHERENCE_AXIS_MAX,
            fonts::value(text_size::CAPTION),
            palette.text_muted,
        );
        painter.text(
            pos2(band_rect.left() - spacing::SM, band_rect.bottom()),
            Align2::RIGHT_BOTTOM,
            text::COHERENCE_AXIS_MIN,
            fonts::value(text_size::CAPTION),
            palette.text_muted,
        );
    }

    fn paint_crosshair(
        &self,
        painter: &egui::Painter,
        palette: &Palette,
        plot_rect: Rect,
        hover: Pos2,
        inputs: &PlotInputs<'_>,
    ) {
        let stroke = Stroke::new(stroke::CROSSHAIR, palette.text_muted.gamma_multiply(0.8));
        painter.line_segment(
            [
                pos2(hover.x, plot_rect.top()),
                pos2(hover.x, plot_rect.bottom()),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(plot_rect.left(), hover.y),
                pos2(plot_rect.right(), hover.y),
            ],
            stroke,
        );
        let frequency_hz = self.view.x_to_frequency(plot_rect, hover.x);
        let level_db = self.view.y_to_db(plot_rect, hover.y);
        let mut lines = vec![text::crosshair_readout(frequency_hz, level_db)];
        if let Some((label, value, color)) =
            nearest_trace_value(frequency_hz, level_db, inputs, palette)
        {
            lines.push(format!("{label} : {}", text::format_db(value)));
            painter.circle_filled(
                self.view.to_screen(plot_rect, frequency_hz, value),
                4.0,
                color,
            );
        }
        let readout = lines.join("\n");
        let galley = painter.layout_no_wrap(
            readout,
            fonts::value(text_size::LABEL),
            palette.text_primary,
        );
        let mut position = hover + vec2(READOUT_OFFSET, READOUT_OFFSET);
        if position.x + galley.size().x + 2.0 * spacing::SM > plot_rect.right() {
            position.x = hover.x - READOUT_OFFSET - galley.size().x - 2.0 * spacing::SM;
        }
        if position.y + galley.size().y + 2.0 * spacing::SM > plot_rect.bottom() {
            position.y = hover.y - READOUT_OFFSET - galley.size().y - 2.0 * spacing::SM;
        }
        let box_rect = Rect::from_min_size(
            position,
            galley.size() + vec2(2.0 * spacing::SM, 2.0 * spacing::SM),
        );
        painter.rect(
            box_rect,
            CornerRadius::same(radius::MEDIUM),
            palette.surface_raised,
            Stroke::new(stroke::BORDER, palette.border),
            StrokeKind::Inside,
        );
        painter.galley(
            position + vec2(spacing::SM, spacing::SM),
            galley,
            palette.text_primary,
        );
    }
}

fn grid_index(frequency_hz: f64, length: usize) -> Option<usize> {
    let index = (GRID_POINTS_PER_OCTAVE * (frequency_hz / GRID_LOW_HZ).log2()).round();
    (index >= 0.0 && (index as usize) <= length && length > 0)
        .then(|| (index as usize).min(length - 1))
}

fn nearest_trace_value(
    frequency_hz: f64,
    cursor_db: f64,
    inputs: &PlotInputs<'_>,
    palette: &Palette,
) -> Option<(String, f64, Color32)> {
    let index = grid_index(frequency_hz, inputs.frequencies_hz.len())?;
    let live = inputs
        .show_live
        .then(|| {
            inputs
                .snapshot
                .magnitude_db
                .get(index)
                .copied()
                .flatten()
                .filter(|_| inputs.snapshot.is_coherent(index))
                .map(|value| (text::LIVE_TRACE_LABEL.to_owned(), value, palette.trace_live))
        })
        .flatten();
    live.into_iter()
        .chain(inputs.overlays.iter().filter_map(|overlay| {
            overlay
                .values_db
                .get(index)
                .copied()
                .flatten()
                .map(|value| (overlay.label.to_owned(), value, overlay.style.color))
        }))
        .min_by(|left, right| {
            (left.1 - cursor_db)
                .abs()
                .total_cmp(&(right.1 - cursor_db).abs())
        })
}

fn paint_polyline(painter: &egui::Painter, polyline: &Polyline, style: TraceStyle) {
    for points in polyline.segment_points() {
        if points.len() < 2 {
            if let Some(point) = points.first() {
                painter.circle_filled(*point, style.width / 2.0, style.color);
            }
            continue;
        }
        if let Some((halo_width, halo_alpha)) = style.halo {
            painter.add(Shape::line(
                points.to_vec(),
                Stroke::new(halo_width, style.color.gamma_multiply(halo_alpha)),
            ));
        }
        painter.add(Shape::line(
            points.to_vec(),
            Stroke::new(style.width, style.color),
        ));
    }
}

fn paint_empty_state(
    ui: &mut egui::Ui,
    palette: &Palette,
    plot_rect: Rect,
    is_mac: bool,
    actions: &mut Vec<Action>,
) {
    let card = Rect::from_center_size(plot_rect.center(), vec2(EMPTY_STATE_WIDTH, 132.0));
    ui.painter().rect(
        card,
        CornerRadius::same(radius::LARGE),
        palette.surface.gamma_multiply(0.92),
        Stroke::new(stroke::BORDER, palette.border),
        StrokeKind::Inside,
    );
    ui.painter().text(
        card.center_top() + vec2(0.0, spacing::LG),
        Align2::CENTER_TOP,
        text::PLOT_EMPTY_TITLE,
        fonts::ui_semibold(text_size::HEADING),
        palette.text_primary,
    );
    ui.painter().text(
        card.center_top() + vec2(0.0, spacing::LG + 30.0),
        Align2::CENTER_TOP,
        text::PLOT_EMPTY_BODY,
        fonts::body(),
        palette.text_muted,
    );
    let button_rect = Rect::from_center_size(
        card.center_bottom() - vec2(0.0, spacing::LG + size::BUTTON_HEIGHT / 2.0),
        vec2(160.0, size::BUTTON_HEIGHT),
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(button_rect)
            .layout(egui::Layout::top_down(egui::Align::Center)),
    );
    if action_button(
        &mut child,
        palette,
        text::BUTTON_START,
        Action::ToggleMeasurement.shortcut_text(is_mac).as_deref(),
        Emphasis::Primary,
        true,
    )
    .clicked()
    {
        actions.push(Action::ToggleMeasurement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_index_matches_the_96th_octave_grid() {
        assert_eq!(grid_index(20.0, 957), Some(0));
        assert_eq!(grid_index(40.0, 957), Some(96));
        assert_eq!(grid_index(19_999.0, 957), Some(956));
        assert_eq!(grid_index(25_000.0, 957), None);
        assert_eq!(grid_index(10.0, 957), None);
    }
}
