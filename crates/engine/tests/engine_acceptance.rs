#![allow(clippy::unwrap_used)]

mod common;

use std::sync::Arc;
use std::time::{Duration, Instant};

use audio_io::{
    FrameSource, GeneratorSample, InputFrame, InternalReferenceAligner, RecordingCursor,
    StereoFrame, StereoRecording, StreamClock,
};
use common::{NoiseSource, SAMPLE_RATE_HZ, circular_fractional_delay, rms, stereo_frames};
use dsp_core::{DEFAULT_DELAY_FFT_LENGTH, DisplayGrid};
use engine::{
    AnalysisSettings, DelayEstimator, DelayPolicy, EngineCommand, EngineConfig, EngineHandle,
    MeasurementCore, MeasurementSnapshot,
};

const MILLISECONDS_PER_SECOND: f64 = 1000.0;
const CHUNK_FRAMES: usize = 480;

fn ms_to_samples(milliseconds: f64) -> f64 {
    milliseconds / MILLISECONDS_PER_SECOND * f64::from(SAMPLE_RATE_HZ)
}

fn new_core() -> (MeasurementCore, DelayEstimator) {
    let core = MeasurementCore::new(
        SAMPLE_RATE_HZ,
        AnalysisSettings::default(),
        DelayPolicy::default(),
        Arc::new(DisplayGrid::default()),
        DEFAULT_DELAY_FFT_LENGTH,
    )
    .unwrap();
    let estimator = DelayEstimator::inline(DEFAULT_DELAY_FFT_LENGTH).unwrap();
    (core, estimator)
}

fn feed_source(
    core: &mut MeasurementCore,
    estimator: &mut DelayEstimator,
    source: &mut dyn FrameSource,
    total_frames: usize,
) -> MeasurementSnapshot {
    let mut snapshot =
        MeasurementSnapshot::empty(core.grid().len(), SAMPLE_RATE_HZ, core.settings());
    let mut frames = Vec::with_capacity(CHUNK_FRAMES);
    let mut consumed = 0;
    while consumed < total_frames {
        frames.clear();
        let report = source.read_frames(&mut frames, CHUNK_FRAMES);
        if report.discontinuity {
            core.note_discontinuity();
        }
        if frames.is_empty() {
            break;
        }
        consumed += frames.len();
        core.push_frames(&frames, estimator);
        core.poll_delay(estimator);
    }
    core.fill_snapshot(&mut snapshot).unwrap();
    snapshot
}

fn feed_frames(
    core: &mut MeasurementCore,
    estimator: &mut DelayEstimator,
    frames: &[StereoFrame],
) -> (MeasurementSnapshot, usize) {
    let mut snapshot =
        MeasurementSnapshot::empty(core.grid().len(), SAMPLE_RATE_HZ, core.settings());
    let mut hops = 0;
    for chunk in frames.chunks(CHUNK_FRAMES) {
        hops += core.push_frames(chunk, estimator);
        core.poll_delay(estimator);
    }
    core.fill_snapshot(&mut snapshot).unwrap();
    (snapshot, hops)
}

fn simulated_internal_reference(
    generator: &[f64],
    microphone: &[f64],
    playback_lead_frames: u64,
) -> InternalReferenceAligner {
    let capacity = generator.len() + 16;
    let (mut generator_producer, generator_consumer) = rtrb::RingBuffer::new(capacity);
    let (mut input_producer, input_consumer) = rtrb::RingBuffer::new(capacity);
    for (frame_index, value) in generator.iter().enumerate() {
        generator_producer
            .push(GeneratorSample {
                frame_index: frame_index as u64,
                value: *value as f32,
            })
            .unwrap();
    }
    let lead = playback_lead_frames as usize;
    for (frame_index, value) in microphone.iter().skip(lead).enumerate() {
        input_producer
            .push(InputFrame {
                frame_index: frame_index as u64,
                loopback_reference: 0.0,
                measurement: *value as f32,
            })
            .unwrap();
    }
    let playback_start_nanos = 5_000_000_000_u64;
    let lead_nanos =
        (playback_lead_frames as f64 / f64::from(SAMPLE_RATE_HZ) * 1.0e9).round() as u64;
    let input_clock = Arc::new(StreamClock::default());
    let output_clock = Arc::new(StreamClock::default());
    output_clock.record_first_frame_instant(playback_start_nanos);
    input_clock.record_first_frame_instant(playback_start_nanos + lead_nanos);
    InternalReferenceAligner::new(
        input_consumer,
        generator_consumer,
        input_clock,
        output_clock,
        SAMPLE_RATE_HZ,
    )
}

#[test]
fn internal_reference_reports_acoustic_delay_after_latency_calibration() {
    let system_latency_ms = 7.3;
    let acoustic_delay_ms = 3.2;
    let playback_lead_frames = 960;
    let length = 48_000 * 6;
    let mut noise = NoiseSource::new(2024);
    let generator = noise.pink_signal(length);

    let cable = circular_fractional_delay(&generator, ms_to_samples(system_latency_ms));
    let mut calibration_source =
        simulated_internal_reference(&generator, &cable, playback_lead_frames);
    let (mut calibration_core, mut calibration_estimator) = new_core();
    calibration_core
        .handle_command(EngineCommand::StartLatencyCalibration)
        .unwrap();
    let calibration = feed_source(
        &mut calibration_core,
        &mut calibration_estimator,
        &mut calibration_source,
        length,
    );
    let calibrated_latency_ms = calibration.delay.system_latency_ms.unwrap();
    assert!(
        (calibrated_latency_ms - system_latency_ms).abs() <= 0.02,
        "calibrated {calibrated_latency_ms} ms"
    );
    assert!(!calibration.delay.calibrating_latency);

    let delayed = circular_fractional_delay(
        &generator,
        ms_to_samples(system_latency_ms + acoustic_delay_ms),
    );
    let noise_gain = rms(&delayed) / 10.0 * 3.0_f64.sqrt();
    let uncorrelated: Vec<f64> = noise
        .white_signal(length)
        .iter()
        .map(|value| value * noise_gain)
        .collect();
    let microphone: Vec<f64> = delayed
        .iter()
        .zip(&uncorrelated)
        .map(|(signal, noise)| signal + noise)
        .collect();
    let mut measurement_source =
        simulated_internal_reference(&generator, &microphone, playback_lead_frames);
    let (mut core, mut estimator) = new_core();
    core.handle_command(EngineCommand::SetSystemLatencyMs(Some(
        calibrated_latency_ms,
    )))
    .unwrap();
    let snapshot = feed_source(&mut core, &mut estimator, &mut measurement_source, length);
    let acoustic_ms = snapshot.delay.applied_acoustic_ms();
    assert!(
        (acoustic_ms - acoustic_delay_ms).abs() <= 0.02,
        "acoustic delay {acoustic_ms} ms"
    );
    let latest = snapshot.delay.latest.unwrap();
    assert!((snapshot.delay.acoustic_ms(latest.total_ms) - acoustic_delay_ms).abs() <= 0.02);
}

#[test]
fn snapshots_update_more_than_twenty_times_per_second_at_48_khz() {
    let (mut core, mut estimator) = new_core();
    let mut noise = NoiseSource::new(8);
    let signal = noise.pink_signal(48_000 * 3);
    let frames = stereo_frames(&signal, &signal);
    let (warm_up, steady) = frames.split_at(48_000 * 2);
    feed_frames(&mut core, &mut estimator, warm_up);
    let (_, hops_in_one_second) = feed_frames(&mut core, &mut estimator, steady);
    assert!(hops_in_one_second >= 20, "{hops_in_one_second} updates/s");
}

#[test]
fn delay_found_at_start_is_applied_and_restores_coherence() {
    let delay_samples = 2000.4;
    let length = 48_000 * 8;
    let mut noise = NoiseSource::new(31);
    let reference = noise.pink_signal(length);
    let measurement = circular_fractional_delay(&reference, delay_samples);
    let frames = stereo_frames(&reference, &measurement);
    let (mut core, mut estimator) = new_core();
    let (snapshot, _) = feed_frames(&mut core, &mut estimator, &frames);

    let applied_samples =
        snapshot.delay.applied_ms / MILLISECONDS_PER_SECOND * f64::from(SAMPLE_RATE_HZ);
    assert!(
        (applied_samples - delay_samples).abs() < 0.5,
        "{applied_samples}"
    );
    assert!(!snapshot.warming_up);
    let grid = DisplayGrid::default();
    for (index, frequency_hz) in grid.frequencies_hz().iter().enumerate() {
        if !(200.0..=10_000.0).contains(frequency_hz) {
            continue;
        }
        let coherence = snapshot.coherence[index].unwrap();
        assert!(coherence > 0.98, "{frequency_hz} Hz coherence {coherence}");
        let magnitude = snapshot.magnitude_db[index].unwrap();
        assert!(
            magnitude.abs() < 0.2,
            "{frequency_hz} Hz magnitude {magnitude}"
        );
        let phase_degrees = snapshot.response[index].unwrap().arg().to_degrees();
        assert!(
            phase_degrees.abs() < 10.0,
            "{frequency_hz} Hz phase {phase_degrees}"
        );
    }
}

#[test]
fn fine_adjustment_and_manual_application_move_the_compensation() {
    let (mut core, _) = new_core();
    core.handle_command(EngineCommand::SetCompensationMs(10.0))
        .unwrap();
    core.handle_command(EngineCommand::AdjustCompensationMs(0.01))
        .unwrap();
    assert!((core.compensation_ms() - 10.01).abs() < 1.0e-9);
    core.handle_command(EngineCommand::SetCompensationMs(f64::NAN))
        .unwrap();
    assert!((core.compensation_ms() - 10.01).abs() < 1.0e-9);
    core.handle_command(EngineCommand::SetCompensationMs(1.0e9))
        .unwrap();
    assert!(core.compensation_ms() < 2000.0);
}

#[test]
fn discontinuity_restarts_the_window_without_losing_averages() {
    let (mut core, mut estimator) = new_core();
    let mut noise = NoiseSource::new(4);
    let signal = noise.pink_signal(48_000 * 2);
    let frames = stereo_frames(&signal, &signal);
    let (before, _) = feed_frames(&mut core, &mut estimator, &frames);
    assert!(!before.warming_up);
    let averages = before.averages;
    core.note_discontinuity();
    let (after, hops) = feed_frames(&mut core, &mut estimator, &frames[..4096]);
    assert_eq!(hops, 0);
    assert!(after.warming_up);
    assert_eq!(after.averages, averages);
    assert_eq!(after.discontinuities, 1);
}

#[test]
fn settings_change_rebuilds_the_analysis() {
    let (mut core, mut estimator) = new_core();
    let mut noise = NoiseSource::new(12);
    let signal = noise.pink_signal(48_000 * 2);
    let frames = stereo_frames(&signal, &signal);
    let settings = AnalysisSettings {
        fft_length: 8192,
        ..AnalysisSettings::default()
    };
    core.handle_command(EngineCommand::UpdateSettings(settings))
        .unwrap();
    let (snapshot, hops) = feed_frames(&mut core, &mut estimator, &frames);
    assert!(hops > 0);
    assert_eq!(snapshot.settings.fft_length, 8192);
    assert!(
        snapshot
            .magnitude_db
            .iter()
            .flatten()
            .all(|value| value.abs() < 0.01)
    );
}

#[test]
fn threaded_engine_publishes_snapshots_from_a_file_source() {
    let mut noise = NoiseSource::new(77);
    let signal = noise.pink_signal(48_000 * 4);
    let recording = Arc::new(StereoRecording {
        sample_rate_hz: SAMPLE_RATE_HZ,
        frames: stereo_frames(&signal, &signal),
    });
    let source = Box::new(RecordingCursor::new(recording, true));
    let mut engine = EngineHandle::spawn(
        source,
        EngineConfig::default(),
        Arc::new(DisplayGrid::default()),
    )
    .unwrap();
    let started = Instant::now();
    loop {
        let snapshot = engine.latest_snapshot();
        if snapshot.averages > 10 && snapshot.delay.latest.is_some() {
            assert!(snapshot.sequence > 0);
            break;
        }
        assert!(started.elapsed() < Duration::from_secs(20), "no snapshot");
        std::thread::sleep(Duration::from_millis(5));
    }
    engine.send(EngineCommand::ResetAverages);
    assert!(engine.is_running());
    drop(engine);
}

fn core_with_policy(policy: DelayPolicy) -> (MeasurementCore, DelayEstimator) {
    let core = MeasurementCore::new(
        SAMPLE_RATE_HZ,
        AnalysisSettings::default(),
        policy,
        Arc::new(DisplayGrid::default()),
        DEFAULT_DELAY_FFT_LENGTH,
    )
    .unwrap();
    (
        core,
        DelayEstimator::inline(DEFAULT_DELAY_FFT_LENGTH).unwrap(),
    )
}

#[test]
fn measurement_ahead_of_reference_is_compensated_with_a_negative_delay() {
    let lead_samples = 640.3;
    let length = 48_000 * 8;
    let mut noise = NoiseSource::new(55);
    let measurement = noise.pink_signal(length);
    let reference = circular_fractional_delay(&measurement, lead_samples);
    let (mut core, mut estimator) = new_core();
    let (snapshot, _) = feed_frames(
        &mut core,
        &mut estimator,
        &stereo_frames(&reference, &measurement),
    );
    let applied_samples =
        snapshot.delay.applied_ms / MILLISECONDS_PER_SECOND * f64::from(SAMPLE_RATE_HZ);
    assert!(
        (applied_samples + lead_samples).abs() < 0.5,
        "{applied_samples}"
    );
    let grid = DisplayGrid::default();
    for (index, frequency_hz) in grid.frequencies_hz().iter().enumerate() {
        if (200.0..=10_000.0).contains(frequency_hz) {
            assert!(snapshot.coherence[index].unwrap() > 0.98);
            let phase_degrees = snapshot.response[index].unwrap().arg().to_degrees();
            assert!(
                phase_degrees.abs() < 10.0,
                "{frequency_hz} Hz phase {phase_degrees}"
            );
        }
    }
}

#[test]
fn tracking_does_not_reset_averages_on_every_estimate() {
    let length = 48_000 * 10;
    let mut noise = NoiseSource::new(91);
    let reference = noise.pink_signal(length);
    let measurement = circular_fractional_delay(&reference, 300.3);
    let (mut core, mut estimator) = core_with_policy(DelayPolicy {
        tracking: true,
        ..DelayPolicy::default()
    });
    let (snapshot, _) = feed_frames(
        &mut core,
        &mut estimator,
        &stereo_frames(&reference, &measurement),
    );
    let hops_after_first_application = (length - 48_000 * 3) / AnalysisSettings::default().hop;
    assert!(
        snapshot.averages as usize > hops_after_first_application / 2,
        "only {} averages",
        snapshot.averages
    );
}

#[test]
fn apply_latest_delay_ignores_unreliable_estimates() {
    let length = 48_000 * 4;
    let mut noise = NoiseSource::new(17);
    let reference = noise.pink_signal(length);
    let measurement = circular_fractional_delay(&reference, 480.0);
    let (mut core, mut estimator) = core_with_policy(DelayPolicy {
        auto_apply_on_start: false,
        ..DelayPolicy::default()
    });
    feed_frames(
        &mut core,
        &mut estimator,
        &stereo_frames(&reference, &measurement),
    );
    let unrelated = noise.white_signal(length * 2);
    let later_reference = noise.pink_signal(length * 2);
    let (snapshot, _) = feed_frames(
        &mut core,
        &mut estimator,
        &stereo_frames(&later_reference, &unrelated),
    );
    assert!(snapshot.delay.latest.unwrap().confidence_db < 18.0);
    assert!((snapshot.delay.latest_confident.unwrap().total_samples - 480.0).abs() < 0.1);
    core.handle_command(EngineCommand::ApplyLatestDelay)
        .unwrap();
    assert!((core.compensation_ms() - 10.0).abs() < 0.01);
}

#[test]
fn manual_compensation_before_the_first_estimate_is_kept() {
    let length = 48_000 * 4;
    let mut noise = NoiseSource::new(23);
    let reference = noise.pink_signal(length);
    let measurement = circular_fractional_delay(&reference, 480.0);
    let (mut core, mut estimator) = new_core();
    core.handle_command(EngineCommand::SetCompensationMs(2.0))
        .unwrap();
    let (snapshot, _) = feed_frames(
        &mut core,
        &mut estimator,
        &stereo_frames(&reference, &measurement),
    );
    assert!(snapshot.delay.latest_confident.is_some());
    assert!((snapshot.delay.applied_ms - 2.0).abs() < 1.0e-9);
}

#[test]
fn latency_calibration_ignores_audio_from_before_the_command() {
    let length = 48_000 * 4;
    let mut noise = NoiseSource::new(61);
    let generator = noise.pink_signal(length);
    let before_cable = circular_fractional_delay(&generator, 900.0);
    let with_cable = circular_fractional_delay(&generator, 350.0);
    let (mut core, mut estimator) = new_core();
    let half = length / 2;
    feed_frames(
        &mut core,
        &mut estimator,
        &stereo_frames(&generator[..half], &before_cable[..half]),
    );
    core.handle_command(EngineCommand::StartLatencyCalibration)
        .unwrap();
    let (snapshot, _) = feed_frames(
        &mut core,
        &mut estimator,
        &stereo_frames(&generator, &with_cable),
    );
    let latency_samples = snapshot.delay.system_latency_ms.unwrap() / MILLISECONDS_PER_SECOND
        * f64::from(SAMPLE_RATE_HZ);
    assert!((latency_samples - 350.0).abs() < 0.1, "{latency_samples}");
}

#[test]
fn latency_calibration_gives_up_on_silence() {
    let (mut core, mut estimator) = new_core();
    core.handle_command(EngineCommand::StartLatencyCalibration)
        .unwrap();
    let silence = vec![StereoFrame::default(); 48_000 * 11];
    let (snapshot, _) = feed_frames(&mut core, &mut estimator, &silence);
    assert!(!snapshot.delay.calibrating_latency);
    assert!(snapshot.delay.calibration_failed);
    assert_eq!(snapshot.delay.system_latency_ms, None);
}

#[test]
fn looping_file_never_mixes_both_sides_of_the_seam_in_one_window() {
    let loop_length = 48_000 * 3;
    let mut noise = NoiseSource::new(70);
    let signal = noise.pink_signal(loop_length);
    let recording = Arc::new(StereoRecording {
        sample_rate_hz: SAMPLE_RATE_HZ,
        frames: stereo_frames(&signal, &signal),
    });
    let mut cursor = RecordingCursor::new(recording, true);
    let (mut core, mut estimator) = new_core();
    let snapshot = feed_source(&mut core, &mut estimator, &mut cursor, loop_length * 3);
    assert_eq!(snapshot.discontinuities, 2);
    assert!(
        snapshot
            .magnitude_db
            .iter()
            .flatten()
            .all(|value| value.abs() < 0.01)
    );
}

#[test]
fn finished_file_is_reported_in_the_snapshot() {
    let mut noise = NoiseSource::new(81);
    let signal = noise.pink_signal(48_000);
    let recording = Arc::new(StereoRecording {
        sample_rate_hz: SAMPLE_RATE_HZ,
        frames: stereo_frames(&signal, &signal),
    });
    let source = Box::new(RecordingCursor::new(recording, false));
    let mut engine = EngineHandle::spawn(
        source,
        EngineConfig::default(),
        Arc::new(DisplayGrid::default()),
    )
    .unwrap();
    let started = Instant::now();
    while !engine.latest_snapshot().source_finished {
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "finish not published"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

struct RealTimeSource {
    cursor: RecordingCursor,
    started: Instant,
    delivered_frames: usize,
}

impl FrameSource for RealTimeSource {
    fn sample_rate_hz(&self) -> u32 {
        SAMPLE_RATE_HZ
    }

    fn read_frames(
        &mut self,
        destination: &mut Vec<StereoFrame>,
        max_frames: usize,
    ) -> audio_io::ReadReport {
        let due = (self.started.elapsed().as_secs_f64() * f64::from(SAMPLE_RATE_HZ)) as usize;
        let allowed = due.saturating_sub(self.delivered_frames).min(max_frames);
        let report = self.cursor.read_frames(destination, allowed);
        self.delivered_frames += report.frames_read;
        report
    }
}

#[test]
fn threaded_engine_publishes_at_least_twenty_snapshots_per_second_in_real_time() {
    let mut noise = NoiseSource::new(33);
    let signal = noise.pink_signal(48_000 * 10);
    let recording = Arc::new(StereoRecording {
        sample_rate_hz: SAMPLE_RATE_HZ,
        frames: stereo_frames(&signal, &signal),
    });
    let mut warm_cursor = RecordingCursor::new(recording, true);
    let mut warm_up = Vec::new();
    warm_cursor.read_frames(&mut warm_up, 48_000);
    let source = Box::new(RealTimeSource {
        cursor: warm_cursor,
        started: Instant::now() - Duration::from_secs(1),
        delivered_frames: 0,
    });
    let mut engine = EngineHandle::spawn(
        source,
        EngineConfig::default(),
        Arc::new(DisplayGrid::default()),
    )
    .unwrap();
    std::thread::sleep(Duration::from_millis(1500));
    let first_sequence = engine.latest_snapshot().sequence;

    let measured_window = Duration::from_secs(2);
    std::thread::sleep(measured_window);
    let last_sequence = engine.latest_snapshot().sequence;
    let per_second = (last_sequence - first_sequence) as f64 / measured_window.as_secs_f64();
    assert!(per_second >= 20.0, "{per_second} snapshots/s");
}
