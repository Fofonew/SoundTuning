use audio_io::StereoFrame;

#[derive(Clone, Debug)]
pub struct SignalHistory {
    reference: Vec<f64>,
    measurement: Vec<f64>,
    write_position: usize,
    contiguous_frames: usize,
}

impl SignalHistory {
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            reference: vec![0.0; capacity],
            measurement: vec![0.0; capacity],
            write_position: 0,
            contiguous_frames: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.reference.len()
    }

    pub fn contiguous_frames(&self) -> usize {
        self.contiguous_frames
    }

    pub fn mark_discontinuity(&mut self) {
        self.contiguous_frames = 0;
    }

    pub fn push(&mut self, frame: StereoFrame) {
        let capacity = self.capacity();
        if let (Some(reference), Some(measurement)) = (
            self.reference.get_mut(self.write_position),
            self.measurement.get_mut(self.write_position),
        ) {
            *reference = f64::from(frame.reference);
            *measurement = f64::from(frame.measurement);
        }
        self.write_position = (self.write_position + 1) % capacity;
        self.contiguous_frames = (self.contiguous_frames + 1).min(capacity);
    }

    pub fn copy_latest(
        &self,
        reference_out: &mut [f64],
        measurement_out: &mut [f64],
        reference_age: usize,
        measurement_age: usize,
    ) -> bool {
        let length = reference_out.len();
        if measurement_out.len() != length
            || length + reference_age.max(measurement_age) > self.contiguous_frames
        {
            return false;
        }
        copy_ending_before(
            &self.reference,
            self.write_position,
            reference_age,
            reference_out,
        );
        copy_ending_before(
            &self.measurement,
            self.write_position,
            measurement_age,
            measurement_out,
        );
        true
    }
}

fn copy_ending_before(source: &[f64], write_position: usize, age: usize, destination: &mut [f64]) {
    let capacity = source.len();
    let length = destination.len();
    let start = (write_position + 2 * capacity - age % capacity - length % capacity) % capacity;
    let first_part = (capacity - start).min(length);
    if let (Some(destination_first), Some(source_first)) = (
        destination.get_mut(..first_part),
        source.get(start..start + first_part),
    ) {
        destination_first.copy_from_slice(source_first);
    }
    if let (Some(destination_rest), Some(source_rest)) = (
        destination.get_mut(first_part..),
        source.get(..length - first_part),
    ) {
        destination_rest.copy_from_slice(source_rest);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(value: f32) -> StereoFrame {
        StereoFrame {
            reference: value,
            measurement: -value,
        }
    }

    #[test]
    fn latest_frames_are_copied_in_order_across_wraparound() {
        let mut history = SignalHistory::new(8);
        for value in 0..13 {
            history.push(frame(value as f32));
        }
        let mut reference = [0.0; 4];
        let mut measurement = [0.0; 4];
        assert!(history.copy_latest(&mut reference, &mut measurement, 0, 0));
        assert_eq!(reference, [9.0, 10.0, 11.0, 12.0]);
        assert_eq!(measurement, [-9.0, -10.0, -11.0, -12.0]);
    }

    #[test]
    fn ages_shift_each_channel_independently() {
        let mut history = SignalHistory::new(16);
        for value in 0..16 {
            history.push(frame(value as f32));
        }
        let mut reference = [0.0; 3];
        let mut measurement = [0.0; 3];
        assert!(history.copy_latest(&mut reference, &mut measurement, 5, 1));
        assert_eq!(reference, [8.0, 9.0, 10.0]);
        assert_eq!(measurement, [-12.0, -13.0, -14.0]);
    }

    #[test]
    fn copy_fails_until_enough_contiguous_frames() {
        let mut history = SignalHistory::new(16);
        for value in 0..6 {
            history.push(frame(value as f32));
        }
        let mut reference = [0.0; 4];
        let mut measurement = [0.0; 4];
        assert!(!history.copy_latest(&mut reference, &mut measurement, 3, 0));
        history.push(frame(6.0));
        assert!(history.copy_latest(&mut reference, &mut measurement, 3, 0));
        history.mark_discontinuity();
        assert!(!history.copy_latest(&mut reference, &mut measurement, 0, 0));
    }
}
