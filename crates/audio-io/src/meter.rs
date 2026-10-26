use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::atomic_f32::AtomicF32;

const CLIP_THRESHOLD: f32 = 0.999;
const MIN_LEVEL_FOR_DB: f32 = 1.0e-10;
const SILENCE_DBFS: f32 = -200.0;

#[derive(Debug)]
pub struct LevelMeter {
    peak_since_last_read: AtomicF32,
    last_block_mean_square: AtomicF32,
    clipped: AtomicBool,
    consecutive_silent_frames: AtomicU64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterSnapshot {
    pub peak_dbfs: f32,
    pub rms_dbfs: f32,
    pub clipped: bool,
    pub consecutive_silent_frames: u64,
}

impl Default for LevelMeter {
    fn default() -> Self {
        Self {
            peak_since_last_read: AtomicF32::new(0.0),
            last_block_mean_square: AtomicF32::new(0.0),
            clipped: AtomicBool::new(false),
            consecutive_silent_frames: AtomicU64::new(0),
        }
    }
}

impl LevelMeter {
    pub fn record_block(&self, block_peak: f32, block_sum_of_squares: f32, frame_count: usize) {
        if frame_count == 0 {
            return;
        }
        self.peak_since_last_read.fetch_max(block_peak);
        self.last_block_mean_square
            .store(block_sum_of_squares / frame_count as f32);
        if block_peak >= CLIP_THRESHOLD {
            self.clipped.store(true, Ordering::Relaxed);
        }
        if block_peak == 0.0 {
            self.consecutive_silent_frames
                .fetch_add(frame_count as u64, Ordering::Relaxed);
        } else {
            self.consecutive_silent_frames.store(0, Ordering::Relaxed);
        }
    }

    pub fn record_samples(&self, samples: impl Iterator<Item = f32>) {
        let mut peak = 0.0_f32;
        let mut sum_of_squares = 0.0_f32;
        let mut frame_count = 0_usize;
        for sample in samples {
            peak = peak.max(sample.abs());
            sum_of_squares += sample * sample;
            frame_count += 1;
        }
        self.record_block(peak, sum_of_squares, frame_count);
    }

    pub fn take_snapshot(&self) -> MeterSnapshot {
        let peak = self.peak_since_last_read.swap(0.0);
        let mean_square = self.last_block_mean_square.load();
        MeterSnapshot {
            peak_dbfs: amplitude_to_dbfs(peak),
            rms_dbfs: amplitude_to_dbfs(mean_square.max(0.0).sqrt()),
            clipped: self.clipped.load(Ordering::Relaxed),
            consecutive_silent_frames: self.consecutive_silent_frames.load(Ordering::Relaxed),
        }
    }

    pub fn reset_clip(&self) {
        self.clipped.store(false, Ordering::Relaxed);
    }
}

pub fn amplitude_to_dbfs(amplitude: f32) -> f32 {
    if amplitude < MIN_LEVEL_FOR_DB {
        SILENCE_DBFS
    } else {
        20.0 * amplitude.log10()
    }
}

#[derive(Debug, Default)]
pub struct MeterBank {
    pub reference: LevelMeter,
    pub measurement: LevelMeter,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_scale_sine_reads_zero_dbfs_peak_and_minus_three_rms() {
        let meter = LevelMeter::default();
        let samples = (0..4800).map(|index| (index as f32 * 0.1).sin());
        meter.record_samples(samples);
        let snapshot = meter.take_snapshot();
        assert!(snapshot.peak_dbfs > -0.01 && snapshot.peak_dbfs <= 0.0);
        assert!((snapshot.rms_dbfs + 3.01).abs() < 0.05);
        assert!(snapshot.clipped);
    }

    #[test]
    fn peak_resets_after_read_but_clip_is_sticky() {
        let meter = LevelMeter::default();
        meter.record_samples([1.0_f32, -1.0].into_iter());
        let _ = meter.take_snapshot();
        meter.record_samples([0.1_f32].into_iter());
        let snapshot = meter.take_snapshot();
        assert!((snapshot.peak_dbfs + 20.0).abs() < 0.01);
        assert!(snapshot.clipped);
        meter.reset_clip();
        assert!(!meter.take_snapshot().clipped);
    }

    #[test]
    fn silent_frames_accumulate_until_signal() {
        let meter = LevelMeter::default();
        meter.record_samples([0.0_f32; 100].into_iter());
        meter.record_samples([0.0_f32; 50].into_iter());
        assert_eq!(meter.take_snapshot().consecutive_silent_frames, 150);
        meter.record_samples([0.01_f32].into_iter());
        assert_eq!(meter.take_snapshot().consecutive_silent_frames, 0);
    }

    #[test]
    fn silence_is_finite() {
        let meter = LevelMeter::default();
        let snapshot = meter.take_snapshot();
        assert!(snapshot.peak_dbfs.is_finite());
        assert!(snapshot.rms_dbfs.is_finite());
    }
}
