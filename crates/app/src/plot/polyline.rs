use std::ops::Range;

use eframe::egui::{Pos2, Rect};

use crate::plot::view::PlotView;

const MAX_ABS_SCREEN_COORDINATE: f32 = 1.0e6;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Polyline {
    pub points: Vec<Pos2>,
    pub segments: Vec<Range<usize>>,
}

impl Polyline {
    pub fn with_capacity(points: usize) -> Self {
        Self {
            points: Vec::with_capacity(points),
            segments: Vec::with_capacity(points / 2 + 1),
        }
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.segments.clear();
    }

    pub fn segment_points(&self) -> impl Iterator<Item = &[Pos2]> {
        self.segments
            .iter()
            .filter_map(|range| self.points.get(range.clone()))
    }
}

pub fn build_polyline(
    view: &PlotView,
    rect: Rect,
    frequencies_hz: &[f64],
    level_at: impl Fn(usize) -> Option<f64>,
    output: &mut Polyline,
) {
    output.clear();
    let visible_low = view.low_hz;
    let visible_high = view.high_hz;
    let mut segment_start: Option<usize> = None;
    for (index, frequency_hz) in frequencies_hz.iter().enumerate() {
        let neighbours_visible = frequencies_hz
            .get(index + 1)
            .is_some_and(|next| *next >= visible_low)
            && (index == 0
                || frequencies_hz
                    .get(index - 1)
                    .is_some_and(|previous| *previous <= visible_high));
        let point = level_at(index)
            .filter(|_| neighbours_visible || (visible_low..=visible_high).contains(frequency_hz))
            .map(|level_db| view.to_screen(rect, *frequency_hz, level_db))
            .filter(|point| {
                point.x.abs() < MAX_ABS_SCREEN_COORDINATE
                    && point.y.abs() < MAX_ABS_SCREEN_COORDINATE
            });
        match point {
            Some(point) => {
                if segment_start.is_none() {
                    segment_start = Some(output.points.len());
                }
                output.points.push(point);
            }
            None => {
                if let Some(start) = segment_start.take() {
                    output.segments.push(start..output.points.len());
                }
            }
        }
    }
    if let Some(start) = segment_start {
        output.segments.push(start..output.points.len());
    }
}

#[cfg(test)]
mod tests {
    use eframe::egui::{pos2, vec2};

    use super::*;

    #[test]
    fn invalid_points_split_the_curve() {
        let view = PlotView::default();
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 400.0));
        let frequencies = [100.0, 200.0, 300.0, 400.0, 500.0];
        let levels = [Some(0.0), Some(1.0), None, Some(2.0), Some(3.0)];
        let mut polyline = Polyline::default();
        build_polyline(
            &view,
            rect,
            &frequencies,
            |index| levels[index],
            &mut polyline,
        );
        assert_eq!(polyline.segments, vec![0..2, 2..4]);
        assert_eq!(polyline.segment_points().count(), 2);
    }

    #[test]
    fn points_far_outside_the_view_are_dropped() {
        let view = PlotView::default().with_preset(crate::plot::view::RangePreset::Sub);
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 400.0));
        let frequencies = [50.0, 100.0, 250.0, 5000.0, 10_000.0];
        let mut polyline = Polyline::default();
        build_polyline(&view, rect, &frequencies, |_| Some(0.0), &mut polyline);
        assert_eq!(polyline.points.len(), 3);
    }

    #[test]
    fn rebuilding_reuses_the_buffers() {
        let view = PlotView::default();
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 400.0));
        let frequencies: Vec<f64> = (0..957)
            .map(|index| 20.0 * (index as f64 / 96.0).exp2())
            .collect();
        let mut polyline = Polyline::with_capacity(frequencies.len());
        build_polyline(&view, rect, &frequencies, |_| Some(0.0), &mut polyline);
        let capacity = polyline.points.capacity();
        build_polyline(&view, rect, &frequencies, |_| Some(1.0), &mut polyline);
        assert_eq!(polyline.points.capacity(), capacity);
        assert_eq!(polyline.points.len(), 957);
    }
}
