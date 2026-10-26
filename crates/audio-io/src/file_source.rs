use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use rtrb::{Consumer, RingBuffer};

use crate::error::AudioIoError;
use crate::frames::{FrameSource, ReadReport, StereoFrame};
use crate::meter::MeterBank;

const PLAYBACK_BLOCK_FRAMES: usize = 512;
const PLAYBACK_BUFFER_SECONDS: usize = 2;
const BACKPRESSURE_WAIT: Duration = Duration::from_millis(2);

#[derive(Clone, Debug, PartialEq)]
pub struct StereoRecording {
    pub sample_rate_hz: u32,
    pub frames: Vec<StereoFrame>,
}

impl StereoRecording {
    pub fn read_wav(path: &Path) -> Result<Self, AudioIoError> {
        let wav_error = |source| AudioIoError::Wav {
            path: path.to_path_buf(),
            source,
        };
        let mut reader = hound::WavReader::open(path).map_err(wav_error)?;
        let spec = reader.spec();
        if spec.channels < 2 {
            return Err(AudioIoError::WavChannelCount {
                path: path.to_path_buf(),
                channels: spec.channels,
            });
        }
        let interleaved: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Float => reader
                .samples::<f32>()
                .collect::<Result<_, _>>()
                .map_err(wav_error)?,
            hound::SampleFormat::Int => {
                let full_scale = (1_i64 << (spec.bits_per_sample - 1)) as f32;
                reader
                    .samples::<i32>()
                    .map(|sample| sample.map(|value| value as f32 / full_scale))
                    .collect::<Result<_, _>>()
                    .map_err(wav_error)?
            }
        };
        let frames: Vec<StereoFrame> = interleaved
            .chunks_exact(usize::from(spec.channels))
            .map(|frame| StereoFrame {
                reference: frame[0],
                measurement: frame[1],
            })
            .collect();
        if frames.is_empty() {
            return Err(AudioIoError::WavEmpty {
                path: path.to_path_buf(),
            });
        }
        Ok(Self {
            sample_rate_hz: spec.sample_rate,
            frames,
        })
    }

    pub fn duration_seconds(&self) -> f64 {
        self.frames.len() as f64 / f64::from(self.sample_rate_hz)
    }
}

pub struct RecordingCursor {
    recording: Arc<StereoRecording>,
    position: usize,
    looping: bool,
}

impl RecordingCursor {
    pub fn new(recording: Arc<StereoRecording>, looping: bool) -> Self {
        Self {
            recording,
            position: 0,
            looping,
        }
    }

    pub fn is_finished(&self) -> bool {
        !self.looping && self.position >= self.recording.frames.len()
    }
}

impl FrameSource for RecordingCursor {
    fn sample_rate_hz(&self) -> u32 {
        self.recording.sample_rate_hz
    }

    fn read_frames(&mut self, destination: &mut Vec<StereoFrame>, max_frames: usize) -> ReadReport {
        let total = self.recording.frames.len();
        let mut report = ReadReport::default();
        while report.frames_read < max_frames {
            if self.position >= total {
                if self.looping && total > 0 {
                    self.position = 0;
                    report.discontinuity = true;
                } else {
                    report.finished = true;
                    break;
                }
            }
            let count = (total - self.position).min(max_frames - report.frames_read);
            destination
                .extend_from_slice(&self.recording.frames[self.position..self.position + count]);
            self.position += count;
            report.frames_read += count;
        }
        report.finished |= self.is_finished();
        report
    }
}

pub struct FilePlayback {
    consumer: Consumer<StereoFrame>,
    sample_rate_hz: u32,
    source_path: PathBuf,
    stop_requested: Arc<AtomicBool>,
    loop_count: Arc<AtomicU64>,
    loop_count_seen: u64,
    worker: Option<JoinHandle<()>>,
}

impl FilePlayback {
    pub fn start(path: &Path, meters: Arc<MeterBank>) -> Result<Self, AudioIoError> {
        let recording = Arc::new(StereoRecording::read_wav(path)?);
        let sample_rate_hz = recording.sample_rate_hz;
        let capacity = sample_rate_hz as usize * PLAYBACK_BUFFER_SECONDS;
        let (mut producer, consumer) = RingBuffer::new(capacity);
        let stop_requested = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop_requested);
        let loop_count = Arc::new(AtomicU64::new(0));
        let worker_loop_count = Arc::clone(&loop_count);

        let worker = std::thread::Builder::new()
            .name("file-playback".into())
            .spawn(move || {
                let mut cursor = RecordingCursor::new(recording, true);
                let mut block = Vec::with_capacity(PLAYBACK_BLOCK_FRAMES);
                let block_duration = Duration::from_secs_f64(
                    PLAYBACK_BLOCK_FRAMES as f64 / f64::from(sample_rate_hz),
                );
                let started = Instant::now();
                let mut blocks_sent: u32 = 0;
                while !worker_stop.load(Ordering::Relaxed) && !producer.is_abandoned() {
                    block.clear();
                    if cursor
                        .read_frames(&mut block, PLAYBACK_BLOCK_FRAMES)
                        .discontinuity
                    {
                        worker_loop_count.fetch_add(1, Ordering::Relaxed);
                    }
                    meters
                        .reference
                        .record_samples(block.iter().map(|frame| frame.reference));
                    meters
                        .measurement
                        .record_samples(block.iter().map(|frame| frame.measurement));
                    let mut unsent: &[StereoFrame] = &block;
                    while !unsent.is_empty()
                        && !worker_stop.load(Ordering::Relaxed)
                        && !producer.is_abandoned()
                    {
                        let (_, remaining) = producer.push_partial_slice(unsent);
                        unsent = remaining;
                        if !unsent.is_empty() {
                            std::thread::sleep(BACKPRESSURE_WAIT);
                        }
                    }
                    blocks_sent = blocks_sent.saturating_add(1);
                    let due = started + block_duration * blocks_sent;
                    if let Some(wait) = due.checked_duration_since(Instant::now()) {
                        std::thread::sleep(wait);
                    }
                }
            })
            .map_err(AudioIoError::PlaybackThread)?;

        Ok(Self {
            consumer,
            sample_rate_hz,
            source_path: path.to_path_buf(),
            stop_requested,
            loop_count,
            loop_count_seen: 0,
            worker: Some(worker),
        })
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }
}

impl FrameSource for FilePlayback {
    fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    fn read_frames(&mut self, destination: &mut Vec<StereoFrame>, max_frames: usize) -> ReadReport {
        let available = self.consumer.slots().min(max_frames);
        let Ok(chunk) = self.consumer.read_chunk(available) else {
            return ReadReport::default();
        };
        let (first, second) = chunk.as_slices();
        destination.extend_from_slice(first);
        destination.extend_from_slice(second);
        chunk.commit_all();
        let loop_count = self.loop_count.load(Ordering::Relaxed);
        let discontinuity = loop_count != self.loop_count_seen;
        self.loop_count_seen = loop_count;
        ReadReport {
            frames_read: available,
            discontinuity,
            finished: false,
        }
    }
}

impl Drop for FilePlayback {
    fn drop(&mut self) {
        self.stop_requested.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wav_writer::write_stereo_wav;

    fn temporary_wav_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("soundtuning-{}-{name}.wav", std::process::id()))
    }

    fn ramp_frames(count: usize) -> Vec<StereoFrame> {
        (0..count)
            .map(|index| StereoFrame {
                reference: index as f32 / count as f32,
                measurement: -(index as f32) / count as f32,
            })
            .collect()
    }

    #[test]
    fn wav_round_trip_preserves_both_channels() {
        let path = temporary_wav_path("round-trip");
        let frames = ramp_frames(1000);
        write_stereo_wav(&path, 48_000, &frames).unwrap();
        let recording = StereoRecording::read_wav(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(recording.sample_rate_hz, 48_000);
        assert_eq!(recording.frames, frames);
    }

    #[test]
    fn integer_wav_is_scaled_to_unit_range() {
        let path = temporary_wav_path("int16");
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 44_100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(&path, spec).unwrap();
        for sample in [16_384_i16, -32_768, 0, 8192] {
            writer.write_sample(sample).unwrap();
        }
        writer.finalize().unwrap();
        let recording = StereoRecording::read_wav(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(
            recording.frames[0],
            StereoFrame {
                reference: 0.5,
                measurement: -1.0
            }
        );
        assert_eq!(
            recording.frames[1],
            StereoFrame {
                reference: 0.0,
                measurement: 0.25
            }
        );
    }

    #[test]
    fn mono_wav_is_rejected() {
        let path = temporary_wav_path("mono");
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 48_000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(&path, spec).unwrap();
        writer.write_sample(0.0_f32).unwrap();
        writer.finalize().unwrap();
        let result = StereoRecording::read_wav(&path);
        std::fs::remove_file(&path).unwrap();
        assert!(matches!(
            result,
            Err(AudioIoError::WavChannelCount { channels: 1, .. })
        ));
    }

    #[test]
    fn looping_cursor_wraps_around() {
        let recording = Arc::new(StereoRecording {
            sample_rate_hz: 48_000,
            frames: ramp_frames(10),
        });
        let mut cursor = RecordingCursor::new(Arc::clone(&recording), true);
        let mut frames = Vec::new();
        let report = cursor.read_frames(&mut frames, 25);
        assert_eq!(report.frames_read, 25);
        assert!(report.discontinuity);
        assert_eq!(frames[10], recording.frames[0]);
        assert_eq!(frames[24], recording.frames[4]);
    }

    #[test]
    fn non_looping_cursor_stops_at_end() {
        let recording = Arc::new(StereoRecording {
            sample_rate_hz: 48_000,
            frames: ramp_frames(10),
        });
        let mut cursor = RecordingCursor::new(recording, false);
        let mut frames = Vec::new();
        let report = cursor.read_frames(&mut frames, 25);
        assert_eq!(report.frames_read, 10);
        assert!(report.finished);
        assert!(cursor.is_finished());
    }

    #[test]
    fn file_playback_streams_frames_and_feeds_meters() {
        let path = temporary_wav_path("playback");
        write_stereo_wav(&path, 48_000, &ramp_frames(4800)).unwrap();
        let meters = Arc::new(MeterBank::default());
        let mut playback = FilePlayback::start(&path, Arc::clone(&meters)).unwrap();
        std::thread::sleep(Duration::from_millis(60));
        let mut frames = Vec::new();
        playback.read_frames(&mut frames, usize::MAX);
        drop(playback);
        std::fs::remove_file(&path).unwrap();
        assert!(frames.len() >= PLAYBACK_BLOCK_FRAMES);
        assert!(meters.reference.take_snapshot().peak_dbfs > -60.0);
    }
}
