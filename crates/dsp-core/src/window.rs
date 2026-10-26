use std::f64::consts::TAU;

pub fn hann_window(length: usize) -> Vec<f64> {
    if length == 0 {
        return Vec::new();
    }
    (0..length)
        .map(|index| 0.5 - 0.5 * (TAU * index as f64 / length as f64).cos())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn periodic_hann_starts_at_zero_and_peaks_at_center() {
        let window = hann_window(8);
        assert_eq!(window[0], 0.0);
        assert!((window[4] - 1.0).abs() < 1.0e-12);
        assert!((window[2] - window[6]).abs() < 1.0e-12);
    }

    #[test]
    fn periodic_hann_has_half_mean() {
        let window = hann_window(1024);
        let mean: f64 = window.iter().sum::<f64>() / window.len() as f64;
        assert!((mean - 0.5).abs() < 1.0e-12);
    }

    #[test]
    fn empty_window_is_empty() {
        assert!(hann_window(0).is_empty());
    }
}
