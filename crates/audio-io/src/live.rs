use std::sync::Arc;

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{FromSample, I24, SampleFormat, SizedSample, StreamInstant};
use rtrb::{Consumer, Producer, RingBuffer};

use crate::aligner::{InternalReferenceAligner, LoopbackFrameSource};
use crate::clock::StreamClock;
use crate::devices::{find_device, sample_format_preference, usable_sample_rates};
use crate::error::AudioIoError;
use crate::frames::{FrameSource, GeneratorSample, InputFrame, ReadReport, StereoFrame};
use crate::generator::{GeneratorControls, NoiseGenerator};
use crate::health::StreamHealth;
use crate::meter::MeterBank;
use crate::sample_rate::negotiate_sample_rate;
use crate::settings::{ReferenceMode, StreamSettings};

const RING_BUFFER_SECONDS: usize = 2;
const GENERATOR_SCRATCH_FRAMES: usize = 512;
const MAX_ROUTABLE_OUTPUT_CHANNELS: usize = u64::BITS as usize;
const INPUT_DIRECTION: &str = "input";
const OUTPUT_DIRECTION: &str = "output";

struct StreamConfigChoice {
    config: cpal::StreamConfig,
    sample_format: SampleFormat,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioRoute {
    pub input_device_id: Option<String>,
    pub output_device_id: Option<String>,
    pub sample_rate_hz: u32,
}

pub enum LiveFrameSource {
    Loopback(LoopbackFrameSource),
    Internal(InternalReferenceAligner),
}

impl FrameSource for LiveFrameSource {
    fn sample_rate_hz(&self) -> u32 {
        match self {
            Self::Loopback(source) => source.sample_rate_hz(),
            Self::Internal(source) => source.sample_rate_hz(),
        }
    }

    fn read_frames(&mut self, destination: &mut Vec<StereoFrame>, max_frames: usize) -> ReadReport {
        match self {
            Self::Loopback(source) => source.read_frames(destination, max_frames),
            Self::Internal(source) => source.read_frames(destination, max_frames),
        }
    }
}

pub struct LiveSession {
    _input_stream: cpal::Stream,
    _output_stream: Option<cpal::Stream>,
    frame_source: Option<LiveFrameSource>,
    meters: Arc<MeterBank>,
    health: Arc<StreamHealth>,
    generator: Arc<GeneratorControls>,
    route: AudioRoute,
    input_device_name: String,
    output_device_name: Option<String>,
    generator_error: Option<String>,
    uses_separate_clocks: bool,
}

struct SharedStreamState {
    meters: Arc<MeterBank>,
    health: Arc<StreamHealth>,
    controls: Arc<GeneratorControls>,
    input_clock: Arc<StreamClock>,
    output_clock: Arc<StreamClock>,
}

impl LiveSession {
    pub fn open(settings: &StreamSettings) -> Result<Self, AudioIoError> {
        validate_generator_channels(settings)?;
        let shared = SharedStreamState {
            meters: Arc::new(MeterBank::default()),
            health: Arc::new(StreamHealth::default()),
            controls: Arc::new(GeneratorControls::new(
                settings.generator.enabled,
                settings.generator.kind,
                settings.generator.level_dbfs,
            )),
            input_clock: Arc::new(StreamClock::default()),
            output_clock: Arc::new(StreamClock::default()),
        };

        let input_device = find_device(settings.input_device_id.as_deref(), true)?;
        let routing = InputRouting {
            measurement_channel_index: settings.measurement_channel_index,
            loopback_channel_index: match settings.reference {
                ReferenceMode::Loopback {
                    reference_channel_index,
                } => Some(reference_channel_index),
                ReferenceMode::Internal => None,
            },
        };
        let input_ranges: Vec<_> = input_device.supported_input_configs()?.collect();
        let input_rates = usable_sample_rates(&input_ranges, routing.required_channels());

        let is_internal = matches!(settings.reference, ReferenceMode::Internal);
        let wants_output = is_internal || settings.generator.enabled;
        let mut output = None;
        let mut generator_error = None;
        if wants_output {
            match OutputCandidate::resolve(settings, &input_device) {
                Ok(candidate) => output = Some(candidate),
                Err(error) if !is_internal => {
                    tracing::warn!(%error, "generator output unavailable in loopback mode");
                    generator_error = Some(error.to_string());
                }
                Err(error) => return Err(error),
            }
        }

        let shared_output_rates = output
            .as_ref()
            .filter(|_| is_internal)
            .map(|candidate| candidate.rates.as_slice());
        let sample_rate_hz = match (
            negotiate_sample_rate(settings.sample_rate_hz, &input_rates, shared_output_rates),
            shared_output_rates,
        ) {
            (Some(rate), _) => rate,
            (None, Some(output_rates)) if !input_rates.is_empty() && !output_rates.is_empty() => {
                return Err(AudioIoError::NoCommonSampleRate {
                    input_rates_hz: input_rates,
                    output_rates_hz: output_rates.to_vec(),
                });
            }
            (None, _) => settings.sample_rate_hz,
        };
        let ring_capacity = sample_rate_hz as usize * RING_BUFFER_SECONDS;
        let input_config = choose_stream_config(
            &input_ranges,
            sample_rate_hz,
            routing.required_channels(),
            settings.buffer_frames,
            INPUT_DIRECTION,
        )?;
        let (input_producer, input_consumer) = pretouched_ring_buffer(ring_capacity);
        let input_stream = build_input_stream(
            &input_device,
            &input_config,
            routing,
            input_producer,
            &shared,
        )?;

        let mut opened_output = None;
        if let Some(candidate) = output {
            let output_rate_hz = if candidate.rates.contains(&sample_rate_hz) {
                sample_rate_hz
            } else {
                negotiate_sample_rate(settings.sample_rate_hz, &candidate.rates, None)
                    .unwrap_or(sample_rate_hz)
            };
            match open_generator_output(candidate, output_rate_hz, settings, ring_capacity, &shared)
            {
                Ok(opened) => opened_output = Some(opened),
                Err(error) if !is_internal => {
                    tracing::warn!(%error, "generator output unavailable in loopback mode");
                    generator_error = Some(error.to_string());
                }
                Err(error) => return Err(error),
            }
        }

        let input_device_id = input_device.id().ok();
        let route = AudioRoute {
            input_device_id: input_device_id.as_ref().map(ToString::to_string),
            output_device_id: opened_output
                .as_ref()
                .and_then(|opened| opened.device_id.as_ref())
                .map(ToString::to_string),
            sample_rate_hz,
        };
        let uses_separate_clocks = is_internal
            && opened_output
                .as_ref()
                .is_some_and(|opened| opened.device_id != input_device_id);
        let output_device_name = opened_output
            .as_ref()
            .map(|opened| opened.device_name.clone());
        let (output_stream, generator_consumer) = match opened_output {
            Some(opened) => (Some(opened.stream), opened.reference_consumer),
            None => (None, None),
        };

        let frame_source = match (is_internal, generator_consumer) {
            (true, Some(generator_consumer)) => {
                LiveFrameSource::Internal(InternalReferenceAligner::new(
                    input_consumer,
                    generator_consumer,
                    Arc::clone(&shared.input_clock),
                    Arc::clone(&shared.output_clock),
                    sample_rate_hz,
                ))
            }
            (true, None) => return Err(AudioIoError::MissingGeneratorOutput),
            (false, _) => {
                LiveFrameSource::Loopback(LoopbackFrameSource::new(input_consumer, sample_rate_hz))
            }
        };

        input_stream.play()?;
        if let Some(stream) = &output_stream {
            stream.play()?;
        }

        let input_device_name = device_display_name(&input_device);
        tracing::info!(
            input = %input_device_name,
            output = ?output_device_name,
            requested_sample_rate_hz = settings.sample_rate_hz,
            sample_rate_hz,
            uses_separate_clocks,
            "audio streams started"
        );
        Ok(Self {
            _input_stream: input_stream,
            _output_stream: output_stream,
            frame_source: Some(frame_source),
            meters: shared.meters,
            health: shared.health,
            generator: shared.controls,
            route,
            input_device_name,
            output_device_name,
            generator_error,
            uses_separate_clocks,
        })
    }

    pub fn take_frame_source(&mut self) -> Option<LiveFrameSource> {
        self.frame_source.take()
    }

    pub fn meters(&self) -> &Arc<MeterBank> {
        &self.meters
    }

    pub fn health(&self) -> &Arc<StreamHealth> {
        &self.health
    }

    pub fn generator(&self) -> &Arc<GeneratorControls> {
        &self.generator
    }

    pub fn sample_rate_hz(&self) -> u32 {
        self.route.sample_rate_hz
    }

    pub fn route(&self) -> &AudioRoute {
        &self.route
    }

    pub fn input_device_name(&self) -> &str {
        &self.input_device_name
    }

    pub fn output_device_name(&self) -> Option<&str> {
        self.output_device_name.as_deref()
    }

    pub fn generator_error(&self) -> Option<&str> {
        self.generator_error.as_deref()
    }

    pub fn uses_separate_clocks(&self) -> bool {
        self.uses_separate_clocks
    }
}

fn validate_generator_channels(settings: &StreamSettings) -> Result<(), AudioIoError> {
    let channels = &settings.generator.output_channel_indices;
    let needs_generator =
        matches!(settings.reference, ReferenceMode::Internal) || settings.generator.enabled;
    if needs_generator && channels.is_empty() {
        return Err(AudioIoError::NoGeneratorChannel);
    }
    match channels
        .iter()
        .find(|index| **index >= MAX_ROUTABLE_OUTPUT_CHANNELS)
    {
        Some(index) => Err(AudioIoError::ChannelOutOfRange {
            direction: OUTPUT_DIRECTION,
            channel_index: *index,
            available: MAX_ROUTABLE_OUTPUT_CHANNELS,
        }),
        None => Ok(()),
    }
}

fn device_display_name(device: &cpal::Device) -> String {
    device
        .description()
        .map(|description| description.name().to_owned())
        .unwrap_or_else(|_| device.to_string())
}

fn choose_stream_config(
    ranges: &[cpal::SupportedStreamConfigRange],
    sample_rate_hz: u32,
    minimum_channels: usize,
    buffer_frames: Option<u32>,
    direction: &'static str,
) -> Result<StreamConfigChoice, AudioIoError> {
    let ranges: Vec<_> = ranges
        .iter()
        .filter(|range| sample_format_preference(range.sample_format()) > 0)
        .filter(|range| range.contains_rate(sample_rate_hz))
        .collect();
    let best = ranges
        .iter()
        .filter(|range| usize::from(range.channels()) >= minimum_channels)
        .min_by_key(|range| {
            (
                range.channels(),
                std::cmp::Reverse(sample_format_preference(range.sample_format())),
            )
        });
    let Some(best) = best else {
        let available = ranges
            .iter()
            .map(|range| usize::from(range.channels()))
            .max();
        return Err(match available {
            Some(available) => AudioIoError::ChannelOutOfRange {
                direction,
                channel_index: minimum_channels.saturating_sub(1),
                available,
            },
            None => AudioIoError::UnsupportedSampleRate {
                direction,
                sample_rate_hz,
            },
        });
    };
    let supported = (**best).try_with_sample_rate(sample_rate_hz).ok_or(
        AudioIoError::UnsupportedSampleRate {
            direction,
            sample_rate_hz,
        },
    )?;
    let mut config = supported.config();
    if let (Some(frames), cpal::SupportedBufferSize::Range { min, max }) =
        (buffer_frames, supported.buffer_size())
        && (*min..=*max).contains(&frames)
    {
        config.buffer_size = cpal::BufferSize::Fixed(frames);
    }
    Ok(StreamConfigChoice {
        config,
        sample_format: supported.sample_format(),
    })
}

fn pretouched_ring_buffer<T: Copy + Default>(capacity: usize) -> (Producer<T>, Consumer<T>) {
    let (mut producer, mut consumer) = RingBuffer::new(capacity);
    while producer.push(T::default()).is_ok() {}
    while consumer.pop().is_ok() {}
    (producer, consumer)
}

fn instant_nanos(instant: StreamInstant) -> Option<u64> {
    u64::try_from(instant.as_nanos()).ok()
}

#[derive(Clone, Copy)]
struct InputRouting {
    measurement_channel_index: usize,
    loopback_channel_index: Option<usize>,
}

impl InputRouting {
    fn required_channels(self) -> usize {
        self.measurement_channel_index
            .max(self.loopback_channel_index.unwrap_or(0))
            + 1
    }
}

fn build_input_stream(
    device: &cpal::Device,
    choice: &StreamConfigChoice,
    routing: InputRouting,
    producer: Producer<InputFrame>,
    shared: &SharedStreamState,
) -> Result<cpal::Stream, AudioIoError> {
    match choice.sample_format {
        SampleFormat::F32 => typed_input_stream::<f32>(device, choice, routing, producer, shared),
        SampleFormat::F64 => typed_input_stream::<f64>(device, choice, routing, producer, shared),
        SampleFormat::I32 => typed_input_stream::<i32>(device, choice, routing, producer, shared),
        SampleFormat::I24 => typed_input_stream::<I24>(device, choice, routing, producer, shared),
        SampleFormat::I16 => typed_input_stream::<i16>(device, choice, routing, producer, shared),
        other => Err(AudioIoError::UnsupportedSampleFormat(other.to_string())),
    }
}

fn typed_input_stream<T>(
    device: &cpal::Device,
    choice: &StreamConfigChoice,
    routing: InputRouting,
    mut producer: Producer<InputFrame>,
    shared: &SharedStreamState,
) -> Result<cpal::Stream, AudioIoError>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let channel_count = usize::from(choice.config.channels).max(1);
    let meters = Arc::clone(&shared.meters);
    let clock = Arc::clone(&shared.input_clock);
    let health = Arc::clone(&shared.health);
    let error_health = Arc::clone(&shared.health);
    let mut next_frame_index: u64 = 0;
    let read_channel = move |frame: &[T], channel_index: usize| -> f32 {
        frame
            .get(channel_index)
            .map_or(0.0, |sample| sample.to_sample::<f32>())
    };
    let stream = device.build_input_stream::<T, _, _>(
        choice.config,
        move |data: &[T], info: &cpal::InputCallbackInfo| {
            if let Some(capture_nanos) = instant_nanos(info.timestamp().capture) {
                clock.record_first_frame_instant(capture_nanos);
            }
            let frames = data.chunks_exact(channel_count);
            let frame_count = frames.len();

            meters.measurement.record_samples(
                frames
                    .clone()
                    .map(|frame| read_channel(frame, routing.measurement_channel_index)),
            );
            if let Some(loopback_index) = routing.loopback_channel_index {
                meters.reference.record_samples(
                    frames
                        .clone()
                        .map(|frame| read_channel(frame, loopback_index)),
                );
            }

            let first_index = next_frame_index;
            next_frame_index += frame_count as u64;
            let writable = producer.slots().min(frame_count);
            if let Ok(chunk) = producer.write_chunk_uninit(writable) {
                chunk.fill_from_iter(frames.zip(first_index..).map(|(frame, frame_index)| {
                    InputFrame {
                        frame_index,
                        loopback_reference: routing
                            .loopback_channel_index
                            .map_or(0.0, |index| read_channel(frame, index)),
                        measurement: read_channel(frame, routing.measurement_channel_index),
                    }
                }));
            }
            if writable < frame_count {
                health.add_dropped_input_frames(frame_count - writable);
            }
        },
        move |error: cpal::Error| error_health.report_backend_error(error.kind()),
        None,
    )?;
    Ok(stream)
}

struct OpenedGeneratorOutput {
    stream: cpal::Stream,
    reference_consumer: Option<Consumer<GeneratorSample>>,
    device_name: String,
    device_id: Option<cpal::DeviceId>,
}

struct OutputCandidate {
    device: cpal::Device,
    ranges: Vec<cpal::SupportedStreamConfigRange>,
    rates: Vec<u32>,
    required_channels: usize,
}

impl OutputCandidate {
    fn resolve(
        settings: &StreamSettings,
        input_device: &cpal::Device,
    ) -> Result<Self, AudioIoError> {
        let required_channels = settings.required_output_channels();
        if settings.output_device_id.is_none() {
            let ranges: Vec<_> = input_device
                .supported_output_configs()
                .map(Iterator::collect)
                .unwrap_or_default();
            let rates = usable_sample_rates(&ranges, required_channels);
            if !rates.is_empty() {
                return Ok(Self {
                    device: input_device.clone(),
                    ranges,
                    rates,
                    required_channels,
                });
            }
        }
        let device = find_device(settings.output_device_id.as_deref(), false)?;
        let ranges: Vec<_> = device.supported_output_configs()?.collect();
        let rates = usable_sample_rates(&ranges, required_channels);
        Ok(Self {
            device,
            ranges,
            rates,
            required_channels,
        })
    }
}

fn open_generator_output(
    candidate: OutputCandidate,
    sample_rate_hz: u32,
    settings: &StreamSettings,
    ring_capacity: usize,
    shared: &SharedStreamState,
) -> Result<OpenedGeneratorOutput, AudioIoError> {
    let device = candidate.device;
    let choice = choose_stream_config(
        &candidate.ranges,
        sample_rate_hz,
        candidate.required_channels,
        settings.buffer_frames,
        OUTPUT_DIRECTION,
    )?;
    let publishes_reference = matches!(settings.reference, ReferenceMode::Internal);
    let (reference_producer, reference_consumer) = if publishes_reference {
        let (producer, consumer) = pretouched_ring_buffer(ring_capacity);
        (Some(producer), Some(consumer))
    } else {
        (None, None)
    };
    let routing = OutputRouting {
        channel_mask: settings.generator_channel_mask(),
        meters_reference: publishes_reference,
    };
    let stream = match choice.sample_format {
        SampleFormat::F32 => {
            typed_output_stream::<f32>(&device, &choice, routing, reference_producer, shared)
        }
        SampleFormat::F64 => {
            typed_output_stream::<f64>(&device, &choice, routing, reference_producer, shared)
        }
        SampleFormat::I32 => {
            typed_output_stream::<i32>(&device, &choice, routing, reference_producer, shared)
        }
        SampleFormat::I24 => {
            typed_output_stream::<I24>(&device, &choice, routing, reference_producer, shared)
        }
        SampleFormat::I16 => {
            typed_output_stream::<i16>(&device, &choice, routing, reference_producer, shared)
        }
        other => Err(AudioIoError::UnsupportedSampleFormat(other.to_string())),
    }?;
    Ok(OpenedGeneratorOutput {
        stream,
        reference_consumer,
        device_name: device_display_name(&device),
        device_id: device.id().ok(),
    })
}

#[derive(Clone, Copy)]
struct OutputRouting {
    channel_mask: u64,
    meters_reference: bool,
}

fn channel_is_routed(channel_mask: u64, channel_index: usize) -> bool {
    u32::try_from(channel_index)
        .ok()
        .and_then(|shift| 1_u64.checked_shl(shift))
        .is_some_and(|bit| channel_mask & bit != 0)
}

fn typed_output_stream<T>(
    device: &cpal::Device,
    choice: &StreamConfigChoice,
    routing: OutputRouting,
    mut reference_producer: Option<Producer<GeneratorSample>>,
    shared: &SharedStreamState,
) -> Result<cpal::Stream, AudioIoError>
where
    T: SizedSample + FromSample<f32>,
{
    let channel_count = usize::from(choice.config.channels).max(1);
    let meters = Arc::clone(&shared.meters);
    let clock = Arc::clone(&shared.output_clock);
    let health = Arc::clone(&shared.health);
    let controls = Arc::clone(&shared.controls);
    let error_health = Arc::clone(&shared.health);
    let mut generator = NoiseGenerator::default();
    let mut scratch = [0.0_f32; GENERATOR_SCRATCH_FRAMES];
    let mut next_frame_index: u64 = 0;
    let stream = device.build_output_stream::<T, _, _>(
        choice.config,
        move |data: &mut [T], info: &cpal::OutputCallbackInfo| {
            if let Some(playback_nanos) = instant_nanos(info.timestamp().playback) {
                clock.record_first_frame_instant(playback_nanos);
            }
            let kind = controls.kind();
            let target_gain = controls.target_gain();
            for block in data.chunks_mut(channel_count * GENERATOR_SCRATCH_FRAMES) {
                let frame_count = block.len() / channel_count;
                let Some(samples) = scratch.get_mut(..frame_count) else {
                    continue;
                };
                generator.fill_block(kind, target_gain, samples);
                for (frame, sample) in block.chunks_exact_mut(channel_count).zip(samples.iter()) {
                    for (channel_index, output) in frame.iter_mut().enumerate() {
                        *output = if channel_is_routed(routing.channel_mask, channel_index) {
                            T::from_sample(*sample)
                        } else {
                            T::EQUILIBRIUM
                        };
                    }
                }
                if routing.meters_reference {
                    meters.reference.record_samples(samples.iter().copied());
                }
                let first_index = next_frame_index;
                next_frame_index += frame_count as u64;
                if let Some(producer) = reference_producer.as_mut() {
                    let writable = producer.slots().min(frame_count);
                    if let Ok(chunk) = producer.write_chunk_uninit(writable) {
                        chunk.fill_from_iter(samples.iter().zip(first_index..).map(
                            |(value, frame_index)| GeneratorSample {
                                frame_index,
                                value: *value,
                            },
                        ));
                    }
                    if writable < frame_count {
                        health.add_dropped_generator_frames(frame_count - writable);
                    }
                }
            }
        },
        move |error: cpal::Error| error_health.report_backend_error(error.kind()),
        None,
    )?;
    Ok(stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pretouched_ring_buffer_starts_empty_with_full_capacity() {
        let (producer, consumer) = pretouched_ring_buffer::<f32>(64);
        assert_eq!(producer.slots(), 64);
        assert_eq!(consumer.slots(), 0);
    }

    #[test]
    fn channel_routing_ignores_out_of_range_indices() {
        assert!(channel_is_routed(0b10, 1));
        assert!(!channel_is_routed(0b10, 0));
        assert!(!channel_is_routed(u64::MAX, 64));
        assert!(!channel_is_routed(u64::MAX, usize::MAX));
    }

    #[test]
    fn internal_reference_without_generator_channel_is_rejected() {
        let mut settings = StreamSettings::default();
        settings.generator.output_channel_indices.clear();
        assert!(matches!(
            validate_generator_channels(&settings),
            Err(AudioIoError::NoGeneratorChannel)
        ));
    }

    #[test]
    fn unroutable_generator_channel_is_rejected() {
        let mut settings = StreamSettings::default();
        settings.generator.output_channel_indices = vec![0, 64];
        assert!(matches!(
            validate_generator_channels(&settings),
            Err(AudioIoError::ChannelOutOfRange {
                channel_index: 64,
                ..
            })
        ));
    }

    #[test]
    fn loopback_without_generator_accepts_empty_channels() {
        let mut settings = StreamSettings {
            reference: ReferenceMode::Loopback {
                reference_channel_index: 1,
            },
            ..StreamSettings::default()
        };
        settings.generator.enabled = false;
        settings.generator.output_channel_indices.clear();
        assert!(validate_generator_channels(&settings).is_ok());
    }

    #[test]
    fn input_routing_requires_highest_used_channel() {
        let routing = InputRouting {
            measurement_channel_index: 0,
            loopback_channel_index: Some(1),
        };
        assert_eq!(routing.required_channels(), 2);
    }
}
