use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug)]
pub(crate) struct AtomicF32(AtomicU32);

impl AtomicF32 {
    pub(crate) fn new(value: f32) -> Self {
        Self(AtomicU32::new(value.to_bits()))
    }

    pub(crate) fn load(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }

    pub(crate) fn store(&self, value: f32) {
        self.0.store(value.to_bits(), Ordering::Relaxed);
    }

    pub(crate) fn swap(&self, value: f32) -> f32 {
        f32::from_bits(self.0.swap(value.to_bits(), Ordering::Relaxed))
    }

    pub(crate) fn fetch_max(&self, value: f32) {
        let mut current_bits = self.0.load(Ordering::Relaxed);
        while value > f32::from_bits(current_bits) {
            match self.0.compare_exchange_weak(
                current_bits,
                value.to_bits(),
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(observed_bits) => current_bits = observed_bits,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_max_keeps_largest_value() {
        let value = AtomicF32::new(0.25);
        value.fetch_max(0.5);
        value.fetch_max(0.1);
        assert_eq!(value.load(), 0.5);
    }

    #[test]
    fn swap_returns_previous_value() {
        let value = AtomicF32::new(1.5);
        assert_eq!(value.swap(0.0), 1.5);
        assert_eq!(value.load(), 0.0);
    }
}
