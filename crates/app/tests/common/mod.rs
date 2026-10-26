#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use audio_io::{DeviceInfo, StereoFrame, write_stereo_wav};
use dsp_core::{BiquadCoefficients, BiquadState};
use eframe::egui;
use egui_kittest::Harness;
use soundtuning::app::{AppOptions, DeviceCatalog, SoundTuningApp};
use soundtuning::config::{AppConfig, ConfigStore};
use soundtuning::design::Theme;
use soundtuning::measurement::{FilePacing, SourceSpec};
use soundtuning::session::SessionStore;
use soundtuning::update::UpdateSource;

pub const SAMPLE_RATE_HZ: u32 = 48_000;
pub const WINDOW_SIZE: [f32; 2] = [1440.0, 900.0];
const FIXTURE_SECONDS: usize = 8;
const FAST_STEP_SECONDS: f32 = 1.0 / 60.0;
const FIXTURE_DELAY_SAMPLES: usize = 96;

fn pink_noise(length: usize, seed: u64) -> Vec<f64> {
    let mut state = seed.max(1);
    let mut b = [0.0_f64; 7];
    (0..length)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let white = (state >> 11) as f64 / (1_u64 << 53) as f64 * 2.0 - 1.0;
            b[0] = 0.99886 * b[0] + white * 0.0555179;
            b[1] = 0.99332 * b[1] + white * 0.0750759;
            b[2] = 0.96900 * b[2] + white * 0.1538520;
            b[3] = 0.86650 * b[3] + white * 0.3104856;
            b[4] = 0.55000 * b[4] + white * 0.5329522;
            b[5] = -0.7616 * b[5] - white * 0.0168980;
            let pink = b.iter().sum::<f64>() + white * 0.5362;
            b[6] = white * 0.115926;
            pink * 0.11
        })
        .collect()
}

pub fn fixture_path() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let path =
            std::env::temp_dir().join(format!("soundtuning-ui-fixture-{}.wav", std::process::id()));
        let length = SAMPLE_RATE_HZ as usize * FIXTURE_SECONDS;
        let reference = pink_noise(length, 2024);
        let room_noise = pink_noise(length, 77);
        let bell = BiquadCoefficients::bell(1000.0, 1.4, 5.0, f64::from(SAMPLE_RATE_HZ));
        let low_shelf = BiquadCoefficients::low_shelf(120.0, 0.7, 4.0, f64::from(SAMPLE_RATE_HZ));
        let high_cut = BiquadCoefficients::low_pass(14_000.0, 0.7, f64::from(SAMPLE_RATE_HZ));
        let mut states = [BiquadState::default(); 3];
        let frames: Vec<StereoFrame> = (0..length)
            .map(|index| {
                let source = if index >= FIXTURE_DELAY_SAMPLES {
                    reference[index - FIXTURE_DELAY_SAMPLES]
                } else {
                    0.0
                };
                let after_bell = states[0].process(&bell, source);
                let after_shelf = states[1].process(&low_shelf, after_bell);
                let shaped = states[2].process(&high_cut, after_shelf);
                StereoFrame {
                    reference: reference[index] as f32,
                    measurement: (0.5 * shaped + 0.03 * room_noise[index]) as f32,
                }
            })
            .collect();
        write_stereo_wav(&path, SAMPLE_RATE_HZ, &frames).unwrap();
        path
    })
}

pub fn fake_devices() -> Vec<DeviceInfo> {
    vec![
        DeviceInfo {
            id: "coreaudio:UAX22".into(),
            name: "UAX 2-2".into(),
            input_channel_count: 2,
            output_channel_count: 2,
            driver: "CoreAudio".into(),
            is_default_driver: true,
            input_sample_rates_hz: vec![44_100, 48_000, 96_000],
            output_sample_rates_hz: vec![44_100, 48_000, 96_000],
            is_default_input: false,
            is_default_output: false,
        },
        DeviceInfo {
            id: "coreaudio:BuiltInMicrophoneDevice".into(),
            name: "Micro MacBook Pro".into(),
            input_channel_count: 1,
            output_channel_count: 0,
            driver: "CoreAudio".into(),
            is_default_driver: true,
            input_sample_rates_hz: vec![48_000],
            output_sample_rates_hz: Vec::new(),
            is_default_input: true,
            is_default_output: false,
        },
    ]
}

pub fn file_options(theme: Theme, start: bool) -> AppOptions {
    AppOptions {
        source: SourceSpec::File {
            path: fixture_path().to_path_buf(),
            pacing: FilePacing::AsFastAsPossible,
        },
        store: ConfigStore::Memory,
        session_store: SessionStore::Memory,
        export_directory: std::env::temp_dir()
            .join(format!("soundtuning-exports-{}", std::process::id())),
        devices: DeviceCatalog::Fixed(fake_devices()),
        initial_config: Some(AppConfig {
            theme,
            wizard_completed: true,
            ..AppConfig::default()
        }),
        start_measurement: start,
        updates: UpdateSource::Offline,
    }
}

pub fn first_launch_options() -> AppOptions {
    AppOptions {
        source: SourceSpec::Live,
        store: ConfigStore::Memory,
        session_store: SessionStore::Memory,
        export_directory: std::env::temp_dir()
            .join(format!("soundtuning-exports-{}", std::process::id())),
        devices: DeviceCatalog::Fixed(fake_devices()),
        initial_config: Some(AppConfig::default()),
        start_measurement: false,
        updates: UpdateSource::Offline,
    }
}

pub fn harness(options: AppOptions) -> Harness<'static, SoundTuningApp> {
    Harness::builder()
        .with_size(egui::vec2(WINDOW_SIZE[0], WINDOW_SIZE[1]))
        .with_max_steps(64)
        .build_eframe(move |creation_context| {
            SoundTuningApp::new(&creation_context.egui_ctx, options)
        })
}

pub fn fast_clock_harness(options: AppOptions) -> Harness<'static, SoundTuningApp> {
    Harness::builder()
        .with_size(egui::vec2(WINDOW_SIZE[0], WINDOW_SIZE[1]))
        .with_max_steps(64)
        .with_step_dt(FAST_STEP_SECONDS)
        .build_eframe(move |creation_context| {
            SoundTuningApp::new(&creation_context.egui_ctx, options)
        })
}

pub fn rendering_harness(options: AppOptions) -> Harness<'static, SoundTuningApp> {
    Harness::builder()
        .with_size(egui::vec2(WINDOW_SIZE[0], WINDOW_SIZE[1]))
        .with_max_steps(64)
        .wgpu()
        .build_eframe(move |creation_context| {
            SoundTuningApp::new(&creation_context.egui_ctx, options)
        })
}

pub fn wait_until_running(harness: &mut Harness<'static, SoundTuningApp>) {
    step_until(harness, Duration::from_secs(20), |app| {
        app.measurement().source_label().is_some()
    });
}

pub fn step_until(
    harness: &mut Harness<'static, SoundTuningApp>,
    timeout: Duration,
    mut condition: impl FnMut(&SoundTuningApp) -> bool,
) {
    let started = Instant::now();
    loop {
        harness.step();
        if condition(harness.state()) {
            return;
        }
        assert!(
            started.elapsed() < timeout,
            "condition not reached in {timeout:?}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

pub fn wait_for_stable_measurement(harness: &mut Harness<'static, SoundTuningApp>) {
    step_until(harness, Duration::from_secs(60), |app| {
        let snapshot = app.measurement().snapshot();
        snapshot.averages > 30 && !snapshot.warming_up && snapshot.delay.latest_confident.is_some()
    });
}

pub fn snapshot_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
}

pub fn save_screenshot(harness: &mut Harness<'static, SoundTuningApp>, name: &str) -> PathBuf {
    let directory = snapshot_directory();
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join(format!("{name}.png"));
    let image = harness.render().expect("offscreen rendering");
    image.save(&path).unwrap();
    path
}
