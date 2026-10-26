use std::sync::atomic::{AtomicU64, Ordering};

const UNSET_INSTANT: u64 = u64::MAX;
const NANOS_PER_SECOND: f64 = 1.0e9;

#[derive(Debug)]
pub struct StreamClock {
    first_frame_instant_nanos: AtomicU64,
}

impl Default for StreamClock {
    fn default() -> Self {
        Self {
            first_frame_instant_nanos: AtomicU64::new(UNSET_INSTANT),
        }
    }
}

impl StreamClock {
    pub fn record_first_frame_instant(&self, instant_nanos: u64) {
        let _ = self.first_frame_instant_nanos.compare_exchange(
            UNSET_INSTANT,
            instant_nanos,
            Ordering::Release,
            Ordering::Relaxed,
        );
    }

    pub fn first_frame_instant_nanos(&self) -> Option<u64> {
        let value = self.first_frame_instant_nanos.load(Ordering::Acquire);
        (value != UNSET_INSTANT).then_some(value)
    }
}

pub fn playback_to_capture_offset_frames(
    first_capture_nanos: u64,
    first_playback_nanos: u64,
    sample_rate_hz: u32,
) -> i64 {
    let difference_nanos = i128::from(first_capture_nanos) - i128::from(first_playback_nanos);
    (difference_nanos as f64 / NANOS_PER_SECOND * f64::from(sample_rate_hz)).round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_instant_is_recorded_once() {
        let clock = StreamClock::default();
        assert_eq!(clock.first_frame_instant_nanos(), None);
        clock.record_first_frame_instant(10);
        clock.record_first_frame_instant(20);
        assert_eq!(clock.first_frame_instant_nanos(), Some(10));
    }

    #[test]
    fn offset_converts_time_difference_to_frames() {
        let ten_ms_nanos = 10_000_000;
        assert_eq!(
            playback_to_capture_offset_frames(1_000_000_000 + ten_ms_nanos, 1_000_000_000, 48_000),
            480
        );
        assert_eq!(
            playback_to_capture_offset_frames(1_000_000_000, 1_000_000_000 + ten_ms_nanos, 48_000),
            -480
        );
    }
}
