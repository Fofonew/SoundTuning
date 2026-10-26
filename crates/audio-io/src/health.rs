use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamFault {
    None,
    DeviceDisconnected,
    PermissionDenied,
    DeviceBusy,
    StreamInvalidated,
    Other,
}

impl StreamFault {
    fn to_code(self) -> u8 {
        match self {
            Self::None => 0,
            Self::DeviceDisconnected => 1,
            Self::PermissionDenied => 2,
            Self::DeviceBusy => 3,
            Self::StreamInvalidated => 4,
            Self::Other => 5,
        }
    }

    fn from_code(code: u8) -> Self {
        match code {
            0 => Self::None,
            1 => Self::DeviceDisconnected,
            2 => Self::PermissionDenied,
            3 => Self::DeviceBusy,
            4 => Self::StreamInvalidated,
            _ => Self::Other,
        }
    }

    pub fn from_backend_error(kind: cpal::ErrorKind) -> Option<Self> {
        match kind {
            cpal::ErrorKind::DeviceNotAvailable => Some(Self::DeviceDisconnected),
            cpal::ErrorKind::PermissionDenied => Some(Self::PermissionDenied),
            cpal::ErrorKind::DeviceBusy => Some(Self::DeviceBusy),
            cpal::ErrorKind::StreamInvalidated | cpal::ErrorKind::DeviceChanged => {
                Some(Self::StreamInvalidated)
            }
            cpal::ErrorKind::Xrun | cpal::ErrorKind::RealtimeDenied => None,
            _ => Some(Self::Other),
        }
    }
}

#[derive(Debug, Default)]
pub struct StreamHealth {
    fault_code: AtomicU8,
    is_faulted: AtomicBool,
    xrun_count: AtomicU64,
    dropped_input_frames: AtomicU64,
    dropped_generator_frames: AtomicU64,
}

impl StreamHealth {
    pub fn report_backend_error(&self, kind: cpal::ErrorKind) {
        match StreamFault::from_backend_error(kind) {
            Some(fault) => {
                self.fault_code.store(fault.to_code(), Ordering::Relaxed);
                self.is_faulted.store(true, Ordering::Release);
            }
            None => {
                self.xrun_count.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn fault(&self) -> StreamFault {
        if self.is_faulted.load(Ordering::Acquire) {
            StreamFault::from_code(self.fault_code.load(Ordering::Relaxed))
        } else {
            StreamFault::None
        }
    }

    pub fn add_dropped_input_frames(&self, count: usize) {
        self.dropped_input_frames
            .fetch_add(count as u64, Ordering::Relaxed);
    }

    pub fn add_dropped_generator_frames(&self, count: usize) {
        self.dropped_generator_frames
            .fetch_add(count as u64, Ordering::Relaxed);
    }

    pub fn xrun_count(&self) -> u64 {
        self.xrun_count.load(Ordering::Relaxed)
    }

    pub fn dropped_input_frames(&self) -> u64 {
        self.dropped_input_frames.load(Ordering::Relaxed)
    }

    pub fn dropped_generator_frames(&self) -> u64 {
        self.dropped_generator_frames.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disconnection_is_reported_as_fault() {
        let health = StreamHealth::default();
        assert_eq!(health.fault(), StreamFault::None);
        health.report_backend_error(cpal::ErrorKind::DeviceNotAvailable);
        assert_eq!(health.fault(), StreamFault::DeviceDisconnected);
    }

    #[test]
    fn xruns_are_counted_without_fault() {
        let health = StreamHealth::default();
        health.report_backend_error(cpal::ErrorKind::Xrun);
        health.report_backend_error(cpal::ErrorKind::Xrun);
        assert_eq!(health.fault(), StreamFault::None);
        assert_eq!(health.xrun_count(), 2);
    }
}
