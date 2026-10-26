#![allow(clippy::unwrap_used)]

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use dsp_core::DisplayGrid;
use eframe::egui::{Rect, pos2, vec2};
use engine::{AnalysisSettings, MeasurementSnapshot};
use soundtuning::plot::{PlotFrame, PlotView, Polyline, build_polyline};

const STORED_TRACES: usize = 8;

fn plot_frame_prep(criterion: &mut Criterion) {
    let grid = DisplayGrid::default();
    let mut snapshot = MeasurementSnapshot::empty(grid.len(), 48_000, AnalysisSettings::default());
    snapshot.magnitude_db = grid
        .frequencies_hz()
        .iter()
        .map(|frequency| Some(6.0 * (frequency / 1000.0).log2().sin()))
        .collect();
    snapshot.coherence = grid
        .frequencies_hz()
        .iter()
        .enumerate()
        .map(|(index, _)| Some(if index % 97 == 0 { 0.2 } else { 0.95 }))
        .collect();
    let stored: Vec<Vec<Option<f64>>> = (0..STORED_TRACES)
        .map(|offset| {
            snapshot
                .magnitude_db
                .iter()
                .map(|value| value.map(|level| level + offset as f64))
                .collect()
        })
        .collect();
    let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(1100.0, 700.0));
    let view = PlotView::default();
    let mut frame = PlotFrame::with_capacity(grid.len());
    let mut stored_polylines = vec![Polyline::with_capacity(grid.len()); STORED_TRACES];
    criterion.bench_function("plot_frame_prep", |bencher| {
        bencher.iter(|| {
            frame.prepare_live(&view, rect, grid.frequencies_hz(), black_box(&snapshot));
            for (polyline, values) in stored_polylines.iter_mut().zip(&stored) {
                build_polyline(
                    &view,
                    rect,
                    grid.frequencies_hz(),
                    |index| values[index],
                    polyline,
                );
            }
            black_box(frame.live.points.len())
        });
    });
}

criterion_group!(benches, plot_frame_prep);
criterion_main!(benches);
