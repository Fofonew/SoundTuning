use std::time::{Duration, Instant};

use anyhow::{Context, bail};
use audio_io::{
    FrameSource, GeneratorSettings, LiveSession, NoiseKind, ReferenceMode, StereoFrame,
    StreamSettings, list_devices, write_stereo_wav,
};

use crate::cli::RecordArguments;
use crate::text;

const CAPTURE_POLL_INTERVAL: Duration = Duration::from_millis(10);
const CAPTURE_TIMEOUT_MARGIN: Duration = Duration::from_secs(3);

pub fn print_devices() -> anyhow::Result<()> {
    let devices = list_devices().context("listing audio devices")?;
    if devices.is_empty() {
        println!("{}", text::DEVICES_NONE);
        return Ok(());
    }
    for device in devices {
        let mut marks = Vec::new();
        if device.is_default_input {
            marks.push(text::DEFAULT_INPUT_MARK);
        }
        if device.is_default_output {
            marks.push(text::DEFAULT_OUTPUT_MARK);
        }
        println!(
            "{}\n",
            text::device_line(
                &device.id,
                &text::device_name(&device),
                device.input_channel_count,
                device.output_channel_count,
                &device.sample_rates_hz(),
                &marks,
            )
        );
    }
    Ok(())
}

pub fn stream_settings_for_recording(arguments: &RecordArguments) -> StreamSettings {
    let reference = match arguments.loopback_channel {
        Some(channel) => ReferenceMode::Loopback {
            reference_channel_index: usize::from(channel) - 1,
        },
        None => ReferenceMode::Internal,
    };
    StreamSettings {
        input_device_id: arguments.device.clone(),
        output_device_id: arguments.output_device.clone(),
        sample_rate_hz: arguments.sample_rate,
        buffer_frames: None,
        measurement_channel_index: usize::from(arguments.measurement_channel) - 1,
        generator: GeneratorSettings {
            enabled: matches!(reference, ReferenceMode::Internal),
            kind: NoiseKind::Pink,
            level_dbfs: arguments.level_dbfs,
            output_channel_indices: vec![usize::from(arguments.generator_channel) - 1],
        },
        reference,
    }
}

pub fn capture_frames(
    source: &mut impl FrameSource,
    frame_count: usize,
    timeout: Duration,
) -> Vec<StereoFrame> {
    let mut frames = Vec::with_capacity(frame_count);
    let started = Instant::now();
    while frames.len() < frame_count && started.elapsed() < timeout {
        let remaining = frame_count - frames.len();
        let report = source.read_frames(&mut frames, remaining);
        if report.finished {
            break;
        }
        if report.frames_read == 0 {
            std::thread::sleep(CAPTURE_POLL_INTERVAL);
        }
    }
    frames
}

pub fn record(arguments: &RecordArguments) -> anyhow::Result<()> {
    if !(arguments.seconds.is_finite() && arguments.seconds > 0.0) {
        bail!("recording duration must be positive");
    }
    let settings = stream_settings_for_recording(arguments);
    let mut session = LiveSession::open(&settings)
        .map_err(|error| anyhow::anyhow!(text::audio_error(&error)))
        .context("opening audio streams")?;
    if session.sample_rate_hz() != settings.sample_rate_hz {
        println!(
            "{}",
            text::sample_rate_adjusted(session.sample_rate_hz(), settings.sample_rate_hz)
        );
    }
    let mut source = session
        .take_frame_source()
        .context("audio frame source already taken")?;
    let sample_rate_hz = source.sample_rate_hz();
    let expected_frames = (arguments.seconds * f64::from(sample_rate_hz)).round() as usize;
    let output_display = arguments.output.display().to_string();

    println!(
        "{}",
        text::record_started(arguments.seconds, &output_display)
    );
    let timeout = Duration::from_secs_f64(arguments.seconds) + CAPTURE_TIMEOUT_MARGIN;
    let frames = capture_frames(&mut source, expected_frames, timeout);
    drop(session);

    write_stereo_wav(&arguments.output, sample_rate_hz, &frames)
        .with_context(|| format!("writing {output_display}"))?;
    if frames.len() < expected_frames {
        println!("{}", text::record_incomplete(frames.len(), expected_frames));
    }
    println!(
        "{}",
        text::record_finished(frames.len(), sample_rate_hz, &output_display)
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use audio_io::{RecordingCursor, StereoRecording};

    use super::*;

    fn record_arguments(loopback_channel: Option<u16>) -> RecordArguments {
        RecordArguments {
            seconds: 10.0,
            output: PathBuf::from("out.wav"),
            device: Some("coreaudio:uax".into()),
            output_device: None,
            sample_rate: 48_000,
            measurement_channel: 1,
            loopback_channel,
            generator_channel: 2,
            level_dbfs: -18.0,
        }
    }

    #[test]
    fn internal_reference_recording_enables_generator_on_chosen_output() {
        let settings = stream_settings_for_recording(&record_arguments(None));
        assert_eq!(settings.reference, ReferenceMode::Internal);
        assert!(settings.generator.enabled);
        assert_eq!(settings.generator.output_channel_indices, vec![1]);
        assert_eq!(settings.measurement_channel_index, 0);
    }

    #[test]
    fn loopback_recording_converts_one_based_channels() {
        let settings = stream_settings_for_recording(&record_arguments(Some(2)));
        assert_eq!(
            settings.reference,
            ReferenceMode::Loopback {
                reference_channel_index: 1
            }
        );
        assert!(!settings.generator.enabled);
    }

    #[test]
    fn capture_collects_requested_frame_count_from_file_source() {
        let recording = Arc::new(StereoRecording {
            sample_rate_hz: 48_000,
            frames: vec![StereoFrame::default(); 1000],
        });
        let mut cursor = RecordingCursor::new(recording, true);
        let frames = capture_frames(&mut cursor, 4800, Duration::from_secs(1));
        assert_eq!(frames.len(), 4800);
    }

    #[test]
    fn capture_stops_when_source_is_finished() {
        let recording = Arc::new(StereoRecording {
            sample_rate_hz: 48_000,
            frames: vec![StereoFrame::default(); 100],
        });
        let mut cursor = RecordingCursor::new(recording, false);
        let frames = capture_frames(&mut cursor, 4800, Duration::from_secs(5));
        assert_eq!(frames.len(), 100);
    }
}
