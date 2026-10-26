use eframe::egui::{Color32, Rect};
use engine::MeasurementSnapshot;

use crate::plot::polyline::{Polyline, build_polyline, build_polyline_with_breaks};

const PHASE_WRAP_BREAK_DEG: f64 = 180.0;
use crate::plot::view::PlotView;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TraceStyle {
    pub color: Color32,
    pub width: f32,
    pub halo: Option<(f32, f32)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoherenceColumn {
    pub left: f32,
    pub right: f32,
    pub coherence: Option<f64>,
}

#[derive(Clone, Debug, Default)]
pub struct PlotFrame {
    pub live: Polyline,
    pub coherence_columns: Vec<CoherenceColumn>,
}

impl PlotFrame {
    pub fn with_capacity(points: usize) -> Self {
        Self {
            live: Polyline::with_capacity(points),
            coherence_columns: Vec::with_capacity(points),
        }
    }

    pub fn prepare_live(
        &mut self,
        view: &PlotView,
        plot_rect: Rect,
        frequencies_hz: &[f64],
        snapshot: &MeasurementSnapshot,
    ) {
        let threshold = snapshot.settings.coherence_threshold;
        build_polyline(
            view,
            plot_rect,
            frequencies_hz,
            |index| {
                let coherent = snapshot
                    .coherence
                    .get(index)
                    .copied()
                    .flatten()
                    .is_some_and(|value| value >= threshold);
                snapshot
                    .magnitude_db
                    .get(index)
                    .copied()
                    .flatten()
                    .filter(|_| coherent)
            },
            &mut self.live,
        );
        self.prepare_coherence(view, plot_rect, frequencies_hz, &snapshot.coherence);
    }

    pub fn prepare_live_phase(
        &mut self,
        view: &PlotView,
        plot_rect: Rect,
        frequencies_hz: &[f64],
        snapshot: &MeasurementSnapshot,
    ) {
        build_polyline_with_breaks(
            view,
            plot_rect,
            frequencies_hz,
            |index| live_phase_deg(snapshot, index),
            PHASE_WRAP_BREAK_DEG,
            &mut self.live,
        );
        self.coherence_columns.clear();
    }

    fn prepare_coherence(
        &mut self,
        view: &PlotView,
        plot_rect: Rect,
        frequencies_hz: &[f64],
        coherence: &[Option<f64>],
    ) {
        self.coherence_columns.clear();
        let to_x = |frequency_hz: f64| view.frequency_to_x(plot_rect, frequency_hz);
        for (index, frequency_hz) in frequencies_hz.iter().enumerate() {
            if *frequency_hz < view.low_hz || *frequency_hz > view.high_hz {
                continue;
            }
            let previous = index
                .checked_sub(1)
                .and_then(|previous| frequencies_hz.get(previous))
                .map_or(*frequency_hz, |previous| (previous * frequency_hz).sqrt());
            let next = frequencies_hz
                .get(index + 1)
                .map_or(*frequency_hz, |next| (next * frequency_hz).sqrt());
            self.coherence_columns.push(CoherenceColumn {
                left: to_x(previous).max(plot_rect.left()),
                right: to_x(next).min(plot_rect.right()),
                coherence: coherence.get(index).copied().flatten(),
            });
        }
    }
}

pub fn live_phase_deg(snapshot: &MeasurementSnapshot, index: usize) -> Option<f64> {
    snapshot
        .response
        .get(index)
        .copied()
        .flatten()
        .filter(|_| snapshot.is_coherent(index))
        .map(|response| response.arg().to_degrees())
}

#[cfg(test)]
mod tests {
    use dsp_core::DisplayGrid;
    use eframe::egui::{pos2, vec2};
    use engine::AnalysisSettings;

    use super::*;

    #[test]
    fn incoherent_points_are_not_drawn() {
        let grid = DisplayGrid::default();
        let mut snapshot =
            MeasurementSnapshot::empty(grid.len(), 48_000, AnalysisSettings::default());
        snapshot.magnitude_db = vec![Some(0.0); grid.len()];
        snapshot.coherence = vec![Some(0.9); grid.len()];
        snapshot.coherence[500] = Some(0.1);
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 500.0));
        let mut frame = PlotFrame::with_capacity(grid.len());
        frame.prepare_live(&PlotView::default(), rect, grid.frequencies_hz(), &snapshot);
        assert_eq!(frame.live.segments.len(), 2);
        assert_eq!(frame.coherence_columns.len(), grid.len());
        let covered: f32 = frame
            .coherence_columns
            .iter()
            .map(|column| column.right - column.left)
            .sum();
        assert!((covered - rect.width()).abs() < 1.0);
    }
}
