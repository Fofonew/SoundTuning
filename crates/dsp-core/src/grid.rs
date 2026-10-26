pub const DISPLAY_GRID_POINTS_PER_OCTAVE: f64 = 96.0;
const DISPLAY_GRID_LOW_HZ: f64 = 20.0;
const DISPLAY_GRID_HIGH_HZ: f64 = 20_000.0;
const MAX_POINTS_PER_OCTAVE: f64 = 1000.0;
const MAX_GRID_OCTAVES: f64 = 20.0;

#[derive(Clone, Debug, PartialEq)]
pub struct DisplayGrid {
    frequencies_hz: Vec<f64>,
}

impl Default for DisplayGrid {
    fn default() -> Self {
        Self::new(
            DISPLAY_GRID_LOW_HZ,
            DISPLAY_GRID_HIGH_HZ,
            DISPLAY_GRID_POINTS_PER_OCTAVE,
        )
    }
}

impl DisplayGrid {
    pub fn new(low_hz: f64, high_hz: f64, points_per_octave: f64) -> Self {
        let low_hz = if low_hz.is_finite() && low_hz > 0.0 {
            low_hz
        } else {
            DISPLAY_GRID_LOW_HZ
        };
        let high_hz = if high_hz.is_finite() {
            high_hz.max(low_hz)
        } else {
            DISPLAY_GRID_HIGH_HZ
        };
        let points_per_octave = if points_per_octave.is_finite() {
            points_per_octave.clamp(1.0, MAX_POINTS_PER_OCTAVE)
        } else {
            DISPLAY_GRID_POINTS_PER_OCTAVE
        };
        let octaves = (high_hz / low_hz).log2().min(MAX_GRID_OCTAVES);
        let last_index = (octaves * points_per_octave).floor() as usize;
        let frequencies_hz = (0..=last_index)
            .map(|index| low_hz * (index as f64 / points_per_octave).exp2())
            .collect();
        Self { frequencies_hz }
    }

    pub fn frequencies_hz(&self) -> &[f64] {
        &self.frequencies_hz
    }

    pub fn len(&self) -> usize {
        self.frequencies_hz.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frequencies_hz.is_empty()
    }

    pub fn index_range_for_band(&self, low_hz: f64, high_hz: f64) -> std::ops::Range<usize> {
        let start = self
            .frequencies_hz
            .partition_point(|frequency| *frequency < low_hz);
        let end = self
            .frequencies_hz
            .partition_point(|frequency| *frequency <= high_hz);
        start..end.max(start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_grid_has_about_957_points_from_20_hz_to_20_khz() {
        let grid = DisplayGrid::default();
        assert_eq!(grid.len(), 957);
        assert!((grid.frequencies_hz()[0] - 20.0).abs() < 1.0e-12);
        let last = *grid.frequencies_hz().last().unwrap();
        assert!(last <= 20_000.0 && last > 19_850.0);
    }

    #[test]
    fn neighbouring_points_are_one_96th_octave_apart() {
        let grid = DisplayGrid::default();
        let ratio = grid.frequencies_hz()[1] / grid.frequencies_hz()[0];
        assert!((ratio.log2() - 1.0 / 96.0).abs() < 1.0e-12);
    }

    #[test]
    fn invalid_parameters_give_a_bounded_grid() {
        for (low, high, density) in [
            (0.0, 20_000.0, 96.0),
            (f64::NAN, f64::INFINITY, f64::NAN),
            (20.0, 1.0e300, 1.0e9),
        ] {
            let grid = DisplayGrid::new(low, high, density);
            assert!(!grid.is_empty());
            assert!(grid.len() <= 20_001);
            assert!(
                grid.frequencies_hz()
                    .iter()
                    .all(|frequency| frequency.is_finite())
            );
        }
    }

    #[test]
    fn band_range_selects_inclusive_points() {
        let grid = DisplayGrid::default();
        let range = grid.index_range_for_band(60.0, 150.0);
        let frequencies = &grid.frequencies_hz()[range];
        assert!(frequencies.first().unwrap() >= &60.0);
        assert!(frequencies.last().unwrap() <= &150.0);
        assert!((frequencies.len() as i64 - 127).abs() <= 1);
    }
}
