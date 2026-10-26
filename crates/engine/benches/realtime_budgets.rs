#![allow(clippy::unwrap_used)]

use std::hint::black_box;
use std::sync::Arc;

use audio_io::StereoFrame;
use criterion::{Criterion, criterion_group, criterion_main};
use dsp_core::{
    AlignmentSearch, Complex64, DEFAULT_DELAY_FFT_LENGTH, DelayFinder, DisplayGrid,
    find_optimal_alignment,
};
use engine::{AnalysisSettings, DelayEstimator, DelayPolicy, MeasurementCore, MeasurementSnapshot};

const SAMPLE_RATE_HZ: u32 = 48_000;

fn noise(length: usize, seed: u64) -> Vec<f64> {
    let mut state = seed.max(1);
    (0..length)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 11) as f64 / (1_u64 << 53) as f64 - 0.5
        })
        .collect()
}

fn frames(length: usize) -> Vec<StereoFrame> {
    noise(length, 3)
        .into_iter()
        .zip(noise(length, 5))
        .map(|(reference, measurement)| StereoFrame {
            reference: reference as f32,
            measurement: (reference + 0.1 * measurement) as f32,
        })
        .collect()
}

fn engine_update(criterion: &mut Criterion) {
    let settings = AnalysisSettings::default();
    let policy = DelayPolicy {
        interval_s: 1.0e9,
        ..DelayPolicy::default()
    };
    let grid = Arc::new(DisplayGrid::default());
    let mut core = MeasurementCore::new(
        SAMPLE_RATE_HZ,
        settings,
        policy,
        Arc::clone(&grid),
        DEFAULT_DELAY_FFT_LENGTH,
    )
    .unwrap();
    let mut estimator = DelayEstimator::inline(DEFAULT_DELAY_FFT_LENGTH).unwrap();
    core.push_frames(&frames(48_000 * 3), &mut estimator);
    let hop = frames(settings.hop);
    let mut snapshot = MeasurementSnapshot::empty(grid.len(), SAMPLE_RATE_HZ, settings);
    criterion.bench_function("engine_update", |bencher| {
        bencher.iter(|| {
            core.push_frames(black_box(&hop), &mut estimator);
            core.fill_snapshot(&mut snapshot).unwrap();
            black_box(snapshot.sequence)
        });
    });
}

fn delay_search(criterion: &mut Criterion) {
    let reference = noise(DEFAULT_DELAY_FFT_LENGTH, 7);
    let measurement: Vec<f64> = (0..DEFAULT_DELAY_FFT_LENGTH)
        .map(|index| reference[(index + DEFAULT_DELAY_FFT_LENGTH - 500) % DEFAULT_DELAY_FFT_LENGTH])
        .collect();
    let mut finder = DelayFinder::new(DEFAULT_DELAY_FFT_LENGTH).unwrap();
    criterion.bench_function("delay_search", |bencher| {
        bencher.iter(|| {
            finder
                .estimate(black_box(&reference), black_box(&measurement))
                .unwrap()
        });
    });
}

fn optimal_alignment(criterion: &mut Criterion) {
    let grid = DisplayGrid::default();
    let frequencies = grid.frequencies_hz();
    let trace_a: Vec<Option<Complex64>> = frequencies
        .iter()
        .map(|frequency| {
            Some(Complex64::from_polar(
                1.0,
                -std::f64::consts::TAU * frequency * 0.004,
            ))
        })
        .collect();
    let trace_b: Vec<Option<Complex64>> = frequencies
        .iter()
        .map(|frequency| {
            Some(Complex64::from_polar(
                0.8,
                -std::f64::consts::TAU * frequency * 0.0055,
            ))
        })
        .collect();
    let search = AlignmentSearch::default();
    criterion.bench_function("optimal_alignment", |bencher| {
        bencher.iter(|| {
            find_optimal_alignment(
                black_box(&trace_a),
                black_box(&trace_b),
                frequencies,
                &search,
            )
            .unwrap()
        });
    });
}

criterion_group!(benches, engine_update, delay_search, optimal_alignment);
criterion_main!(benches);
