#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::{Duration, Instant};

use audio_io::{StereoFrame, write_stereo_wav};
use criterion::{Criterion, criterion_group, criterion_main};
use eframe::egui;
use egui_kittest::Harness;
use soundtuning::actions::Action;
use soundtuning::app::{AppOptions, DeviceCatalog, SoundTuningApp};
use soundtuning::config::{AppConfig, ConfigStore};
use soundtuning::eq_state::{StoredFilter, StoredFilterKind};
use soundtuning::measurement::{FilePacing, SourceSpec};
use soundtuning::session::SessionStore;
use soundtuning::ui::alignment_panel::AlignmentCommand;
use soundtuning::ui::eq_panel::EqCommand;
use soundtuning::ui::side_panel::SideTab;
use soundtuning::update::UpdateSource;

const SAMPLE_RATE_HZ: u32 = 48_000;
const FIXTURE_SECONDS: usize = 6;
const DELAY_SAMPLES: usize = 96;
const STORED_TRACES: usize = 6;
const EQ_FILTERS: usize = 8;
const WINDOW: [f32; 2] = [1440.0, 900.0];

fn fixture() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("soundtuning-bench-{}.wav", std::process::id()));
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    let reference: Vec<f32> = (0..SAMPLE_RATE_HZ as usize * FIXTURE_SECONDS)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            ((state >> 11) as f64 / (1_u64 << 53) as f64 * 0.5 - 0.25) as f32
        })
        .collect();
    let frames: Vec<StereoFrame> = reference
        .iter()
        .enumerate()
        .map(|(index, sample)| StereoFrame {
            reference: *sample,
            measurement: index
                .checked_sub(DELAY_SAMPLES)
                .map_or(0.0, |earlier| 0.5 * reference[earlier]),
        })
        .collect();
    write_stereo_wav(&path, SAMPLE_RATE_HZ, &frames).unwrap();
    path
}

fn harness(path: &std::path::Path) -> Harness<'static, SoundTuningApp> {
    let options = AppOptions {
        source: SourceSpec::File {
            path: path.to_path_buf(),
            pacing: FilePacing::AsFastAsPossible,
        },
        store: ConfigStore::Memory,
        session_store: SessionStore::Memory,
        export_directory: std::env::temp_dir(),
        devices: DeviceCatalog::Fixed(Vec::new()),
        initial_config: Some(AppConfig {
            wizard_completed: true,
            ..AppConfig::default()
        }),
        start_measurement: true,
        updates: UpdateSource::Offline,
    };
    let mut harness = Harness::builder()
        .with_size(egui::vec2(WINDOW[0], WINDOW[1]))
        .build_eframe(move |context| SoundTuningApp::new(&context.egui_ctx, options));
    let started = Instant::now();
    while harness.state().measurement().snapshot().averages < 20 {
        harness.step();
        assert!(started.elapsed() < Duration::from_secs(30));
        std::thread::sleep(Duration::from_millis(5));
    }
    for _ in 0..STORED_TRACES {
        harness.key_press(egui::Key::S);
        harness.step();
    }
    harness.key_press(egui::Key::F);
    harness.step();
    harness.hover_at(egui::pos2(600.0, 400.0));
    harness
}

fn ui_frame(criterion: &mut Criterion) {
    let path = fixture();
    let mut harness = harness(&path);
    let mut group = criterion.benchmark_group("ui_frame_cpu_frozen");
    group.bench_function("traces", |bencher| bencher.iter(|| harness.step()));

    harness.state_mut().set_side_tab(SideTab::Alignment);
    let ids: Vec<u64> = harness
        .state()
        .session()
        .traces
        .iter()
        .map(|trace| trace.id)
        .collect();
    let mut alignment = harness.state().session().alignment.clone();
    alignment.reference_trace = ids.first().copied();
    alignment.delayed_trace = ids.get(1).copied();
    harness
        .state_mut()
        .apply_test_alignment(AlignmentCommand::Update(alignment));
    group.bench_function("alignment", |bencher| bencher.iter(|| harness.step()));

    harness.state_mut().set_side_tab(SideTab::Eq);
    let mut eq = harness.state().session().eq.clone();
    for index in 0..EQ_FILTERS {
        eq.add_filter(StoredFilter::new(
            StoredFilterKind::Bell,
            50.0 * 2_f64.powi(index as i32),
            -3.0,
        ));
    }
    harness.state_mut().apply_test_eq(EqCommand::Update(eq));
    group.bench_function("eq", |bencher| bencher.iter(|| harness.step()));

    harness.state_mut().set_side_tab(SideTab::Traces);
    harness
        .state_mut()
        .handle_action(Action::ToggleStackedViews);
    group.bench_function("phase_over_magnitude", |bencher| {
        bencher.iter(|| harness.step())
    });
    harness.state_mut().handle_action(Action::ViewImpulse);
    group.bench_function("magnitude_over_impulse", |bencher| {
        bencher.iter(|| harness.step())
    });
    group.finish();
    drop(harness);
    let _ = std::fs::remove_file(path);
}

criterion_group!(benches, ui_frame);
criterion_main!(benches);
