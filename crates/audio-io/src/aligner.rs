use std::sync::Arc;

use rtrb::Consumer;

use crate::clock::{StreamClock, playback_to_capture_offset_frames};
use crate::frames::{FrameSource, GeneratorSample, InputFrame, ReadReport, StereoFrame};

pub struct LoopbackFrameSource {
    input: Consumer<InputFrame>,
    sample_rate_hz: u32,
    expected_next_index: Option<u64>,
}

impl LoopbackFrameSource {
    pub fn new(input: Consumer<InputFrame>, sample_rate_hz: u32) -> Self {
        Self {
            input,
            sample_rate_hz,
            expected_next_index: None,
        }
    }
}

impl FrameSource for LoopbackFrameSource {
    fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    fn read_frames(&mut self, destination: &mut Vec<StereoFrame>, max_frames: usize) -> ReadReport {
        let available = self.input.slots().min(max_frames);
        let Ok(chunk) = self.input.read_chunk(available) else {
            return ReadReport::default();
        };
        let mut report = ReadReport::default();
        let (first, second) = chunk.as_slices();
        for frame in first.iter().chain(second.iter()) {
            let follows_gap = self
                .expected_next_index
                .is_some_and(|expected| expected != frame.frame_index);
            if follows_gap {
                if report.frames_read > 0 {
                    break;
                }
                report.discontinuity = true;
            }
            self.expected_next_index = Some(frame.frame_index + 1);
            destination.push(StereoFrame {
                reference: frame.loopback_reference,
                measurement: frame.measurement,
            });
            report.frames_read += 1;
        }
        chunk.commit(report.frames_read);
        report
    }
}

pub struct InternalReferenceAligner {
    input: Consumer<InputFrame>,
    generator: Consumer<GeneratorSample>,
    input_clock: Arc<StreamClock>,
    output_clock: Arc<StreamClock>,
    sample_rate_hz: u32,
    generator_index_minus_input_index: Option<i64>,
    last_paired_input_index: Option<u64>,
}

impl InternalReferenceAligner {
    pub fn new(
        input: Consumer<InputFrame>,
        generator: Consumer<GeneratorSample>,
        input_clock: Arc<StreamClock>,
        output_clock: Arc<StreamClock>,
        sample_rate_hz: u32,
    ) -> Self {
        Self {
            input,
            generator,
            input_clock,
            output_clock,
            sample_rate_hz,
            generator_index_minus_input_index: None,
            last_paired_input_index: None,
        }
    }

    pub fn stream_offset_frames(&self) -> Option<i64> {
        self.generator_index_minus_input_index
    }

    fn resolve_offset(&mut self) -> Option<i64> {
        if self.generator_index_minus_input_index.is_none() {
            let capture_nanos = self.input_clock.first_frame_instant_nanos()?;
            let playback_nanos = self.output_clock.first_frame_instant_nanos()?;
            self.generator_index_minus_input_index = Some(playback_to_capture_offset_frames(
                capture_nanos,
                playback_nanos,
                self.sample_rate_hz,
            ));
        }
        self.generator_index_minus_input_index
    }
}

impl FrameSource for InternalReferenceAligner {
    fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    fn read_frames(&mut self, destination: &mut Vec<StereoFrame>, max_frames: usize) -> ReadReport {
        let Some(offset) = self.resolve_offset() else {
            return ReadReport::default();
        };
        let input_available = self.input.slots();
        let generator_available = self.generator.slots();
        let (Ok(input_chunk), Ok(generator_chunk)) = (
            self.input.read_chunk(input_available),
            self.generator.read_chunk(generator_available),
        ) else {
            return ReadReport::default();
        };

        let (input_first, input_second) = input_chunk.as_slices();
        let (generator_first, generator_second) = generator_chunk.as_slices();
        let mut inputs = input_first.iter().chain(input_second.iter()).peekable();
        let mut generated = generator_first
            .iter()
            .chain(generator_second.iter())
            .peekable();
        let mut consumed_input = 0;
        let mut consumed_generator = 0;
        let mut frames_read = 0;
        let mut discontinuity = false;

        while frames_read < max_frames {
            let (Some(input_frame), Some(generator_sample)) = (inputs.peek(), generated.peek())
            else {
                break;
            };
            let wanted_generator_index = input_frame.frame_index as i64 + offset;
            let generator_index = generator_sample.frame_index as i64;
            let pairs_are_misaligned = generator_index != wanted_generator_index;
            let follows_gap = self
                .last_paired_input_index
                .is_some_and(|last| last + 1 != input_frame.frame_index);
            if (pairs_are_misaligned || follows_gap) && self.last_paired_input_index.is_some() {
                if frames_read > 0 {
                    break;
                }
                discontinuity = true;
            }
            if generator_index < wanted_generator_index {
                generated.next();
                consumed_generator += 1;
                continue;
            }
            if generator_index > wanted_generator_index {
                inputs.next();
                consumed_input += 1;
                continue;
            }
            self.last_paired_input_index = Some(input_frame.frame_index);
            destination.push(StereoFrame {
                reference: generator_sample.value,
                measurement: input_frame.measurement,
            });
            inputs.next();
            generated.next();
            consumed_input += 1;
            consumed_generator += 1;
            frames_read += 1;
        }

        input_chunk.commit(consumed_input);
        generator_chunk.commit(consumed_generator);
        ReadReport {
            frames_read,
            discontinuity,
            finished: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use rtrb::RingBuffer;

    use super::*;

    const SAMPLE_RATE_HZ: u32 = 48_000;
    const NANOS_PER_FRAME_AT_48K: u64 = 20_833;

    fn read_all(source: &mut impl FrameSource) -> (Vec<StereoFrame>, bool) {
        let mut frames = Vec::new();
        let mut discontinuity = false;
        for _ in 0..8 {
            discontinuity |= source.read_frames(&mut frames, usize::MAX).discontinuity;
        }
        (frames, discontinuity)
    }

    fn input_frame(frame_index: u64) -> InputFrame {
        InputFrame {
            frame_index,
            loopback_reference: 0.0,
            measurement: frame_index as f32,
        }
    }

    fn generator_sample(frame_index: u64) -> GeneratorSample {
        GeneratorSample {
            frame_index,
            value: frame_index as f32,
        }
    }

    fn clocks(capture_minus_playback_frames: i64) -> (Arc<StreamClock>, Arc<StreamClock>) {
        let base_nanos = 1_000_000_000_u64;
        let shift_nanos = capture_minus_playback_frames.unsigned_abs() * NANOS_PER_FRAME_AT_48K;
        let (capture, playback) = if capture_minus_playback_frames >= 0 {
            (base_nanos + shift_nanos, base_nanos)
        } else {
            (base_nanos, base_nanos + shift_nanos)
        };
        let input_clock = Arc::new(StreamClock::default());
        let output_clock = Arc::new(StreamClock::default());
        input_clock.record_first_frame_instant(capture);
        output_clock.record_first_frame_instant(playback);
        (input_clock, output_clock)
    }

    #[test]
    fn loopback_passes_input_channels_through() {
        let (mut producer, consumer) = RingBuffer::new(16);
        for index in 0..4 {
            let _ = producer.push(InputFrame {
                frame_index: index,
                loopback_reference: index as f32,
                measurement: -(index as f32),
            });
        }
        let mut source = LoopbackFrameSource::new(consumer, SAMPLE_RATE_HZ);
        let (frames, discontinuity) = read_all(&mut source);
        assert_eq!(frames.len(), 4);
        assert!(!discontinuity);
        assert_eq!(
            frames[3],
            StereoFrame {
                reference: 3.0,
                measurement: -3.0
            }
        );
    }

    #[test]
    fn loopback_stops_before_a_gap_and_flags_the_next_read() {
        let (mut producer, consumer) = RingBuffer::new(16);
        for index in [0, 1, 5, 6] {
            let _ = producer.push(input_frame(index));
        }
        let mut source = LoopbackFrameSource::new(consumer, SAMPLE_RATE_HZ);
        let mut frames = Vec::new();
        let before = source.read_frames(&mut frames, usize::MAX);
        assert_eq!(before.frames_read, 2);
        assert!(!before.discontinuity);
        let after = source.read_frames(&mut frames, usize::MAX);
        assert_eq!(after.frames_read, 2);
        assert!(after.discontinuity);
    }

    #[test]
    fn internal_reference_pairs_samples_played_at_capture_time() {
        let latency_frames = 100_i64;
        let (mut input_producer, input_consumer) = RingBuffer::new(4096);
        let (mut generator_producer, generator_consumer) = RingBuffer::new(4096);
        for index in 0..1000 {
            let _ = generator_producer.push(generator_sample(index));
        }
        for index in 0..500 {
            let _ = input_producer.push(input_frame(index));
        }
        let (input_clock, output_clock) = clocks(latency_frames);
        let mut aligner = InternalReferenceAligner::new(
            input_consumer,
            generator_consumer,
            input_clock,
            output_clock,
            SAMPLE_RATE_HZ,
        );
        let (frames, discontinuity) = read_all(&mut aligner);
        assert_eq!(aligner.stream_offset_frames(), Some(latency_frames));
        assert_eq!(frames.len(), 500);
        assert!(!discontinuity);
        assert!(
            frames
                .iter()
                .all(|frame| frame.reference as i64 == frame.measurement as i64 + latency_frames)
        );
    }

    #[test]
    fn internal_reference_drops_input_captured_before_generator_started() {
        let (mut input_producer, input_consumer) = RingBuffer::new(4096);
        let (mut generator_producer, generator_consumer) = RingBuffer::new(4096);
        for index in 0..300 {
            let _ = generator_producer.push(generator_sample(index));
            let _ = input_producer.push(input_frame(index));
        }
        let (input_clock, output_clock) = clocks(-50);
        let mut aligner = InternalReferenceAligner::new(
            input_consumer,
            generator_consumer,
            input_clock,
            output_clock,
            SAMPLE_RATE_HZ,
        );
        let (frames, discontinuity) = read_all(&mut aligner);
        assert_eq!(frames.len(), 250);
        assert!(!discontinuity);
        assert_eq!(
            frames[0],
            StereoFrame {
                reference: 0.0,
                measurement: 50.0
            }
        );
    }

    #[test]
    fn lost_frames_keep_pairs_aligned_and_are_reported() {
        let (mut input_producer, input_consumer) = RingBuffer::new(4096);
        let (mut generator_producer, generator_consumer) = RingBuffer::new(4096);
        for index in (0..400).filter(|index| !(100..130).contains(index)) {
            let _ = generator_producer.push(generator_sample(index));
        }
        for index in (0..400).filter(|index| !(200..260).contains(index)) {
            let _ = input_producer.push(input_frame(index));
        }
        let (input_clock, output_clock) = clocks(0);
        let mut aligner = InternalReferenceAligner::new(
            input_consumer,
            generator_consumer,
            input_clock,
            output_clock,
            SAMPLE_RATE_HZ,
        );
        let mut frames = Vec::new();
        let mut reports = Vec::new();
        for _ in 0..8 {
            let report = aligner.read_frames(&mut frames, usize::MAX);
            if report.frames_read > 0 {
                reports.push(report);
            }
        }
        assert_eq!(
            reports
                .iter()
                .map(|report| report.frames_read)
                .collect::<Vec<_>>(),
            vec![100, 70, 140]
        );
        assert_eq!(
            reports
                .iter()
                .map(|report| report.discontinuity)
                .collect::<Vec<_>>(),
            vec![false, true, true]
        );
        assert_eq!(frames.len(), 400 - 30 - 60);
        assert!(
            frames
                .iter()
                .all(|frame| frame.reference == frame.measurement)
        );
    }

    #[test]
    fn nothing_is_emitted_before_both_clocks_start() {
        let (_input_producer, input_consumer) = RingBuffer::<InputFrame>::new(16);
        let (_generator_producer, generator_consumer) = RingBuffer::<GeneratorSample>::new(16);
        let mut aligner = InternalReferenceAligner::new(
            input_consumer,
            generator_consumer,
            Arc::new(StreamClock::default()),
            Arc::new(StreamClock::default()),
            SAMPLE_RATE_HZ,
        );
        assert!(read_all(&mut aligner).0.is_empty());
    }
}
