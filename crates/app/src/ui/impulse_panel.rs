use eframe::egui::{
    self, Align2, CornerRadius, Pos2, Rect, Role, Sense, Shape, Stroke, StrokeKind, WidgetInfo,
    pos2, vec2,
};

use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{radius, spacing, stroke, text_size};
use crate::plot::TraceStyle;
use crate::text;

const LEFT_AXIS_WIDTH: f32 = 40.0;
const BOTTOM_AXIS_HEIGHT: f32 = 20.0;
const PLOT_PADDING: f32 = 8.0;
const PLOT_TOP_PADDING: f32 = 24.0;
const MIN_PLOT_SIZE: f32 = 80.0;
const DEFAULT_START_MS: f64 = -5.0;
const DEFAULT_END_MS: f64 = 45.0;
const MIN_SPAN_MS: f64 = 0.5;
const MAX_SPAN_MS: f64 = 700.0;
const TIME_LIMIT_MS: f64 = 400.0;
const TIME_STEP_CHOICES_MS: [f64; 11] =
    [0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0];
const MAX_TIME_LINES: f64 = 10.0;
const LINEAR_LIMIT: f64 = 1.1;
const LINEAR_TICKS: [f64; 5] = [-1.0, -0.5, 0.0, 0.5, 1.0];
const DB_FLOOR: f64 = -60.0;
const DB_CEILING: f64 = 3.0;
const DB_TICK_STEP: f64 = 10.0;
const MIN_RELATIVE_AMPLITUDE: f64 = 1.0e-6;
const SCROLL_ZOOM_SENSITIVITY: f64 = 0.0025;
const SAMPLES_PER_COLUMN_FOR_ENVELOPE: f64 = 2.0;
const PEAK_MARKER_RADIUS: f32 = 4.0;
const READOUT_OFFSET: f32 = 14.0;
const MILLISECONDS_PER_SECOND: f64 = 1000.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImpulseView {
    pub start_ms: f64,
    pub end_ms: f64,
}

impl Default for ImpulseView {
    fn default() -> Self {
        Self {
            start_ms: DEFAULT_START_MS,
            end_ms: DEFAULT_END_MS,
        }
    }
}

impl ImpulseView {
    fn span_ms(&self) -> f64 {
        self.end_ms - self.start_ms
    }

    pub fn time_to_x(&self, rect: Rect, time_ms: f64) -> f32 {
        let fraction = (time_ms - self.start_ms) / self.span_ms();
        rect.left() + fraction as f32 * rect.width()
    }

    pub fn x_to_time(&self, rect: Rect, x: f32) -> f64 {
        let fraction = f64::from((x - rect.left()) / rect.width().max(1.0));
        self.start_ms + fraction * self.span_ms()
    }

    pub fn zoom(&mut self, anchor_ms: f64, factor: f64) {
        if !(factor.is_finite() && factor > 0.0 && anchor_ms.is_finite()) {
            return;
        }
        let start = anchor_ms + (self.start_ms - anchor_ms) * factor;
        let end = anchor_ms + (self.end_ms - anchor_ms) * factor;
        if (MIN_SPAN_MS..=MAX_SPAN_MS).contains(&(end - start)) {
            self.start_ms = start;
            self.end_ms = end;
            self.clamp();
        }
    }

    pub fn pan(&mut self, rect: Rect, delta_x: f32) {
        let shift = -f64::from(delta_x) * self.span_ms() / f64::from(rect.width().max(1.0));
        self.start_ms += shift;
        self.end_ms += shift;
        self.clamp();
    }

    fn clamp(&mut self) {
        let span = self.span_ms();
        if self.start_ms < -TIME_LIMIT_MS {
            self.start_ms = -TIME_LIMIT_MS;
            self.end_ms = self.start_ms + span;
        }
        if self.end_ms > TIME_LIMIT_MS {
            self.end_ms = TIME_LIMIT_MS;
            self.start_ms = self.end_ms - span;
        }
    }
}

pub struct ImpulseCurve<'a> {
    pub label: &'a str,
    pub samples: &'a [f32],
    pub start_ms: f64,
    pub sample_rate_hz: u32,
    pub style: TraceStyle,
}

impl ImpulseCurve<'_> {
    fn sample_period_ms(&self) -> f64 {
        MILLISECONDS_PER_SECOND / f64::from(self.sample_rate_hz.max(1))
    }

    fn time_ms(&self, index: usize) -> f64 {
        self.start_ms + index as f64 * self.sample_period_ms()
    }

    fn index_at(&self, time_ms: f64) -> f64 {
        (time_ms - self.start_ms) / self.sample_period_ms()
    }

    fn peak(&self) -> Option<(usize, f32)> {
        self.samples
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, sample)| sample.is_finite())
            .max_by(|left, right| left.1.abs().total_cmp(&right.1.abs()))
    }

    fn value_at(&self, time_ms: f64) -> Option<f32> {
        let index = self.index_at(time_ms).round();
        (index >= 0.0)
            .then(|| self.samples.get(index as usize).copied())
            .flatten()
    }
}

pub struct ImpulseInputs<'a> {
    pub live: Option<ImpulseCurve<'a>>,
    pub overlays: &'a [ImpulseCurve<'a>],
    pub in_db: bool,
    pub speed_of_sound_m_per_s: f64,
    pub warming_up: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ImpulseEvents {
    pub plot_rect: Option<Rect>,
}

#[derive(Debug, Default)]
pub struct ImpulsePanel {
    pub view: ImpulseView,
    points: Vec<Pos2>,
}

struct Scale {
    reference: f64,
    in_db: bool,
}

impl Scale {
    fn new(inputs: &ImpulseInputs<'_>) -> Self {
        let reference = inputs
            .live
            .iter()
            .chain(inputs.overlays)
            .filter_map(ImpulseCurve::peak)
            .map(|(_, value)| f64::from(value.abs()))
            .fold(0.0_f64, f64::max);
        Self {
            reference: if reference > 0.0 { reference } else { 1.0 },
            in_db: inputs.in_db,
        }
    }

    fn range(&self) -> (f64, f64) {
        if self.in_db {
            (DB_FLOOR, DB_CEILING)
        } else {
            (-LINEAR_LIMIT, LINEAR_LIMIT)
        }
    }

    fn level(&self, sample: f64) -> f64 {
        let relative = sample / self.reference;
        if self.in_db {
            20.0 * relative.abs().max(MIN_RELATIVE_AMPLITUDE).log10()
        } else {
            relative
        }
    }

    fn to_y(&self, rect: Rect, level: f64) -> f32 {
        let (low, high) = self.range();
        let fraction = ((level - low) / (high - low)).clamp(0.0, 1.0);
        rect.bottom() - fraction as f32 * rect.height()
    }
}

impl ImpulsePanel {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        palette: &Palette,
        inputs: &ImpulseInputs<'_>,
    ) -> ImpulseEvents {
        let outer = ui.available_rect_before_wrap();
        let plot_left = outer.left() + LEFT_AXIS_WIDTH;
        let plot_top = outer.top() + PLOT_TOP_PADDING;
        let plot_rect = Rect::from_min_max(
            pos2(plot_left, plot_top),
            pos2(
                (outer.right() - PLOT_PADDING).max(plot_left + MIN_PLOT_SIZE),
                (outer.bottom() - BOTTOM_AXIS_HEIGHT).max(plot_top + MIN_PLOT_SIZE),
            ),
        );
        ui.allocate_rect(outer, Sense::hover());
        let response = ui.interact(
            plot_rect,
            ui.make_persistent_id("impulse-plot"),
            Sense::click_and_drag(),
        );
        response.widget_info(|| {
            WidgetInfo::labeled(Role::Figure, true, text::IMPULSE_PLOT_ACCESSIBLE_LABEL)
        });
        self.handle_interaction(ui, &response, plot_rect);

        let scale = Scale::new(inputs);
        let painter = ui.painter_at(outer);
        painter.rect_filled(
            plot_rect,
            CornerRadius::same(radius::SMALL),
            palette.plot_background,
        );
        self.paint_grid(&painter, palette, plot_rect, &scale);
        let clipped = ui.painter_at(plot_rect);
        for overlay in inputs.overlays {
            self.paint_curve(&clipped, plot_rect, &scale, overlay, overlay.style);
        }
        if let Some(live) = &inputs.live {
            self.paint_curve(
                &clipped,
                plot_rect,
                &scale,
                live,
                TraceStyle {
                    color: palette.trace_live,
                    width: stroke::TRACE_LIVE,
                    halo: None,
                },
            );
            self.paint_peak(&painter, palette, plot_rect, &scale, live);
        }
        painter.rect_stroke(
            plot_rect,
            CornerRadius::same(radius::SMALL),
            Stroke::new(stroke::BORDER, palette.border),
            StrokeKind::Inside,
        );
        if let Some(hover) = response.hover_pos() {
            self.paint_crosshair(&painter, palette, plot_rect, hover, inputs);
        }
        if inputs.live.is_none() && inputs.overlays.is_empty() {
            painter.text(
                plot_rect.center(),
                Align2::CENTER_CENTER,
                if inputs.warming_up {
                    text::PLOT_WARMING_UP
                } else {
                    text::IMPULSE_EMPTY
                },
                fonts::ui_medium(text_size::LABEL),
                palette.text_muted,
            );
        }
        ImpulseEvents {
            plot_rect: Some(plot_rect),
        }
    }

    fn handle_interaction(&mut self, ui: &egui::Ui, response: &egui::Response, plot_rect: Rect) {
        if response.double_clicked() {
            self.view = ImpulseView::default();
            return;
        }
        if response.dragged() {
            self.view.pan(plot_rect, response.drag_delta().x);
        }
        if let Some(hover) = response.hover_pos() {
            let (scroll, zoom) = ui.input(|input| (input.smooth_scroll_delta, input.zoom_delta()));
            let anchor_ms = self.view.x_to_time(plot_rect, hover.x);
            if (zoom - 1.0).abs() > f32::EPSILON {
                self.view.zoom(anchor_ms, 1.0 / f64::from(zoom));
            }
            if scroll.y.abs() > f32::EPSILON {
                self.view.zoom(
                    anchor_ms,
                    (-f64::from(scroll.y) * SCROLL_ZOOM_SENSITIVITY).exp(),
                );
            }
            if scroll.x.abs() > f32::EPSILON {
                self.view.pan(plot_rect, scroll.x);
            }
        }
    }

    fn paint_curve(
        &mut self,
        painter: &egui::Painter,
        plot_rect: Rect,
        scale: &Scale,
        curve: &ImpulseCurve<'_>,
        style: TraceStyle,
    ) {
        self.points.clear();
        let columns = plot_rect.width().max(1.0) as usize;
        let samples_per_column = self.view.span_ms() / curve.sample_period_ms() / columns as f64;
        if samples_per_column > SAMPLES_PER_COLUMN_FOR_ENVELOPE {
            for column in 0..columns {
                let x = plot_rect.left() + column as f32;
                let first = curve.index_at(self.view.x_to_time(plot_rect, x)).ceil();
                let last = curve
                    .index_at(self.view.x_to_time(plot_rect, x + 1.0))
                    .floor();
                if last < 0.0 || first > last {
                    continue;
                }
                let first = first.max(0.0) as usize;
                let last = (last as usize).min(curve.samples.len().saturating_sub(1));
                let Some(window) = curve.samples.get(first..=last) else {
                    continue;
                };
                let (low, high) = window
                    .iter()
                    .fold((f32::INFINITY, f32::NEG_INFINITY), |(low, high), sample| {
                        (low.min(*sample), high.max(*sample))
                    });
                if !(low.is_finite() && high.is_finite()) {
                    continue;
                }
                if scale.in_db {
                    let peak = f64::from(low.abs().max(high.abs()));
                    self.points
                        .push(pos2(x, scale.to_y(plot_rect, scale.level(peak))));
                } else {
                    self.points
                        .push(pos2(x, scale.to_y(plot_rect, scale.level(f64::from(high)))));
                    self.points
                        .push(pos2(x, scale.to_y(plot_rect, scale.level(f64::from(low)))));
                }
            }
        } else {
            let first = curve.index_at(self.view.start_ms).floor().max(0.0) as usize;
            let last = (curve.index_at(self.view.end_ms).ceil().max(0.0) as usize)
                .min(curve.samples.len().saturating_sub(1));
            for index in first..=last {
                let Some(sample) = curve.samples.get(index).copied() else {
                    break;
                };
                self.points.push(pos2(
                    self.view.time_to_x(plot_rect, curve.time_ms(index)),
                    scale.to_y(plot_rect, scale.level(f64::from(sample))),
                ));
            }
        }
        if self.points.len() > 1 {
            painter.add(Shape::line(
                std::mem::take(&mut self.points),
                Stroke::new(style.width, style.color),
            ));
        }
    }

    fn paint_peak(
        &self,
        painter: &egui::Painter,
        palette: &Palette,
        plot_rect: Rect,
        scale: &Scale,
        live: &ImpulseCurve<'_>,
    ) {
        let Some((index, value)) = live.peak() else {
            return;
        };
        let time_ms = live.time_ms(index);
        let x = self.view.time_to_x(plot_rect, time_ms);
        if (plot_rect.left()..=plot_rect.right()).contains(&x) {
            painter.circle_filled(
                pos2(x, scale.to_y(plot_rect, scale.level(f64::from(value)))),
                PEAK_MARKER_RADIUS,
                palette.trace_live,
            );
        }
        painter.text(
            plot_rect.right_top() + vec2(-spacing::MD, spacing::MD),
            Align2::RIGHT_TOP,
            text::impulse_peak(time_ms),
            fonts::value_medium(text_size::LABEL),
            palette.text_primary,
        );
    }

    fn paint_grid(
        &self,
        painter: &egui::Painter,
        palette: &Palette,
        plot_rect: Rect,
        scale: &Scale,
    ) {
        for time_ms in time_ticks(self.view.start_ms, self.view.end_ms) {
            let x = self.view.time_to_x(plot_rect, time_ms);
            let is_zero = time_ms.abs() < f64::EPSILON;
            painter.line_segment(
                [pos2(x, plot_rect.top()), pos2(x, plot_rect.bottom())],
                Stroke::new(
                    if is_zero {
                        stroke::GRID_MAJOR * 1.5
                    } else {
                        stroke::GRID_MAJOR
                    },
                    if is_zero {
                        palette.border
                    } else {
                        palette.grid_major
                    },
                ),
            );
            painter.text(
                pos2(x, plot_rect.bottom() + spacing::XS),
                Align2::CENTER_TOP,
                text::format_axis_milliseconds(time_ms),
                fonts::value(text_size::CAPTION),
                palette.text_muted,
            );
        }
        let levels: Vec<f64> = if scale.in_db {
            let mut level = 0.0;
            let mut levels = Vec::new();
            while level >= DB_FLOOR {
                levels.push(level);
                level -= DB_TICK_STEP;
            }
            levels
        } else {
            LINEAR_TICKS.to_vec()
        };
        for level in levels {
            let y = scale.to_y(plot_rect, level);
            let is_zero = !scale.in_db && level.abs() < f64::EPSILON;
            painter.line_segment(
                [pos2(plot_rect.left(), y), pos2(plot_rect.right(), y)],
                Stroke::new(
                    stroke::GRID_MAJOR,
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
                if scale.in_db {
                    text::format_axis_db(level)
                } else {
                    text::format_axis_amplitude(level)
                },
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
            if scale.in_db {
                text::AXIS_DB
            } else {
                text::AXIS_AMPLITUDE
            },
            fonts::caption(),
            palette.text_muted,
        );
        painter.text(
            pos2(plot_rect.right(), plot_rect.top() - spacing::LG),
            Align2::RIGHT_CENTER,
            text::AXIS_MILLISECONDS,
            fonts::caption(),
            palette.text_muted,
        );
    }

    fn paint_crosshair(
        &self,
        painter: &egui::Painter,
        palette: &Palette,
        plot_rect: Rect,
        hover: Pos2,
        inputs: &ImpulseInputs<'_>,
    ) {
        painter.line_segment(
            [
                pos2(hover.x, plot_rect.top()),
                pos2(hover.x, plot_rect.bottom()),
            ],
            Stroke::new(stroke::CROSSHAIR, palette.text_muted.gamma_multiply(0.8)),
        );
        let time_ms = self.view.x_to_time(plot_rect, hover.x);
        let mut lines = vec![text::impulse_readout(
            time_ms,
            time_ms / MILLISECONDS_PER_SECOND * inputs.speed_of_sound_m_per_s,
        )];
        let scale = Scale::new(inputs);
        for curve in inputs.live.iter().chain(inputs.overlays) {
            if let Some(value) = curve.value_at(time_ms) {
                let level = scale.level(f64::from(value));
                lines.push(text::labelled_value(
                    curve.label,
                    &if scale.in_db {
                        text::format_db(level)
                    } else {
                        text::format_amplitude(level)
                    },
                ));
            }
        }
        let galley = painter.layout_no_wrap(
            lines.join("\n"),
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

pub fn time_ticks(start_ms: f64, end_ms: f64) -> Vec<f64> {
    let span = end_ms - start_ms;
    let step = TIME_STEP_CHOICES_MS
        .iter()
        .copied()
        .find(|step| span / step <= MAX_TIME_LINES)
        .unwrap_or(TIME_STEP_CHOICES_MS[TIME_STEP_CHOICES_MS.len() - 1]);
    let first = (start_ms / step).ceil() as i64;
    let last = (end_ms / step).floor() as i64;
    (first..=last).map(|index| index as f64 * step).collect()
}

#[cfg(test)]
mod tests {
    use eframe::egui::Color32;

    use super::*;

    fn curve(samples: &[f32]) -> ImpulseCurve<'_> {
        ImpulseCurve {
            label: "test",
            samples,
            start_ms: -1.0,
            sample_rate_hz: 1000,
            style: TraceStyle {
                color: Color32::WHITE,
                width: 1.0,
                halo: None,
            },
        }
    }

    #[test]
    fn time_ticks_use_a_readable_step() {
        assert_eq!(
            time_ticks(-5.0, 45.0),
            vec![
                -5.0, 0.0, 5.0, 10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0, 45.0
            ]
        );
        assert_eq!(time_ticks(0.0, 1.0).len(), 11);
    }

    #[test]
    fn zoom_keeps_the_anchor_and_respects_limits() {
        let mut view = ImpulseView::default();
        view.zoom(10.0, 0.5);
        assert!((view.start_ms - 2.5).abs() < 1.0e-9);
        assert!((view.end_ms - 27.5).abs() < 1.0e-9);
        view.zoom(10.0, 1.0e-6);
        assert!(view.span_ms() >= MIN_SPAN_MS);
        view.zoom(f64::NAN, 0.5);
        assert!(view.start_ms.is_finite());
    }

    #[test]
    fn panning_stays_inside_the_measured_window() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 200.0));
        let mut view = ImpulseView::default();
        view.pan(rect, 1.0e6);
        assert!(view.start_ms >= -TIME_LIMIT_MS);
        assert!((view.span_ms() - 50.0).abs() < 1.0e-9);
    }

    #[test]
    fn curves_report_their_peak_and_values_in_time() {
        let samples = [0.0, 0.2, -0.9, 0.1];
        let curve = curve(&samples);
        assert_eq!(curve.peak(), Some((2, -0.9)));
        assert!((curve.time_ms(2) - 1.0).abs() < 1.0e-9);
        assert_eq!(curve.value_at(1.0), Some(-0.9));
        assert_eq!(curve.value_at(-5.0), None);
    }

    #[test]
    fn the_scale_normalises_to_the_largest_peak() {
        let live = [0.0_f32, 0.5];
        let inputs = ImpulseInputs {
            live: Some(curve(&live)),
            overlays: &[],
            in_db: true,
            speed_of_sound_m_per_s: 343.0,
            warming_up: false,
        };
        let scale = Scale::new(&inputs);
        assert!((scale.level(0.5)).abs() < 1.0e-9);
        assert!((scale.level(0.05) + 20.0).abs() < 1.0e-9);
        assert!(scale.level(0.0).is_finite());
    }
}
