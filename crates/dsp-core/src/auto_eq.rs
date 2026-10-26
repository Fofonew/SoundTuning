use crate::Complex64;
use crate::biquad::{EqFilter, FilterKind, MAX_EQ_FILTERS, eq_response, q_from_bandwidth_octaves};
use crate::error::{DspError, check_length};

const MIN_RANGE_POINTS: usize = 16;
const SMOOTHING_HALF_WIDTH_POINTS: usize = 8;
const MIN_CORRECTION_DB: f64 = 0.5;
const MIN_FILTER_SPACING_OCTAVES: f64 = 1.0 / 6.0;
const NEGLIGIBLE_GAIN_DB: f64 = 0.2;
const MIN_AUTO_Q: f64 = 0.3;
const MAX_AUTO_Q: f64 = 10.0;
const MIN_BANDWIDTH_OCTAVES: f64 = 1.0 / 12.0;
const MAX_BANDWIDTH_OCTAVES: f64 = 3.0;
const BOOST_TOLERANCE_DB: f64 = 0.01;
const BOOST_SHRINK_MARGIN: f64 = 0.98;
const MAX_BOOST_SHRINK_ROUNDS: usize = 32;
const FREQUENCY_STEP_OCTAVES: f64 = 1.0 / 24.0;
const GAIN_STEP_DB: f64 = 0.5;
const Q_STEP_FACTOR: f64 = 1.25;
const REFINE_SCALES: [f64; 3] = [1.0, 0.5, 0.25];
const QUICK_REFINE_SCALES: [f64; 1] = [1.0];
const REFINE_PASSES: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AutoEqSettings {
    pub low_hz: f64,
    pub high_hz: f64,
    pub max_boost_db: f64,
    pub max_cut_db: f64,
    pub max_filters: usize,
    pub target_db: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AutoEqResult {
    pub filters: Vec<EqFilter>,
    pub target_db: f64,
    pub rms_error_db: f64,
}

struct Problem<'a> {
    frequencies_hz: Vec<f64>,
    levels_db: Vec<f64>,
    target_db: f64,
    sample_rate_hz: f64,
    settings: &'a AutoEqSettings,
    low_hz: f64,
    high_hz: f64,
    scratch: Vec<Complex64>,
}

impl Problem<'_> {
    fn eq_db(&mut self, filters: &[EqFilter]) -> Option<Vec<f64>> {
        eq_response(
            filters,
            &self.frequencies_hz,
            self.sample_rate_hz,
            &mut self.scratch,
        )
        .ok()?;
        Some(
            self.scratch
                .iter()
                .map(|value| 20.0 * value.norm().max(f64::MIN_POSITIVE).log10())
                .collect(),
        )
    }

    fn cost(&mut self, filters: &[EqFilter]) -> f64 {
        let Some(eq_db) = self.eq_db(filters) else {
            return f64::INFINITY;
        };
        let limit = self.boost_limit();
        if eq_db.iter().any(|eq| *eq > limit) {
            return f64::INFINITY;
        }
        let total: f64 = self
            .levels_db
            .iter()
            .zip(&eq_db)
            .map(|(level, eq)| (level + eq - self.target_db).powi(2))
            .sum();
        (total / self.levels_db.len() as f64).sqrt()
    }

    fn boost_limit(&self) -> f64 {
        self.settings.max_boost_db.max(0.0) + BOOST_TOLERANCE_DB
    }

    fn enforce_boost_limit(&mut self, filters: &mut [EqFilter]) {
        let limit = self.settings.max_boost_db.max(0.0);
        for _ in 0..MAX_BOOST_SHRINK_ROUNDS {
            let Some(peak) = self
                .eq_db(filters)
                .and_then(|eq_db| eq_db.into_iter().reduce(f64::max))
            else {
                return;
            };
            if peak <= limit + BOOST_TOLERANCE_DB {
                return;
            }
            let factor = if peak > 0.0 {
                (limit / peak).clamp(0.0, 1.0) * BOOST_SHRINK_MARGIN
            } else {
                0.0
            };
            for filter in filters.iter_mut().filter(|filter| filter.gain_db > 0.0) {
                filter.gain_db *= factor;
            }
        }
        for filter in filters.iter_mut().filter(|filter| filter.gain_db > 0.0) {
            filter.gain_db = 0.0;
        }
    }

    fn rms_error(&mut self, filters: &[EqFilter]) -> f64 {
        let Some(eq_db) = self.eq_db(filters) else {
            return f64::INFINITY;
        };
        let total: f64 = self
            .levels_db
            .iter()
            .zip(&eq_db)
            .map(|(level, eq)| (level + eq - self.target_db).powi(2))
            .sum();
        (total / self.levels_db.len() as f64).sqrt()
    }

    fn clamp(&self, filter: &mut EqFilter) {
        filter.frequency_hz = filter.frequency_hz.clamp(self.low_hz, self.high_hz);
        filter.gain_db = filter.gain_db.clamp(
            -self.settings.max_cut_db.max(0.0),
            self.settings.max_boost_db.max(0.0),
        );
        filter.q = filter.q.clamp(MIN_AUTO_Q, MAX_AUTO_Q);
    }

    fn smoothed_residual(&mut self, filters: &[EqFilter]) -> Option<Vec<f64>> {
        let eq_db = self.eq_db(filters)?;
        let residual: Vec<f64> = self
            .levels_db
            .iter()
            .zip(&eq_db)
            .map(|(level, eq)| level + eq - self.target_db)
            .collect();
        let mut prefix = Vec::with_capacity(residual.len() + 1);
        prefix.push(0.0);
        for value in &residual {
            prefix.push(prefix.last().copied().unwrap_or(0.0) + value);
        }
        Some(
            (0..residual.len())
                .map(|index| {
                    let start = index.saturating_sub(SMOOTHING_HALF_WIDTH_POINTS);
                    let end = (index + SMOOTHING_HALF_WIDTH_POINTS + 1).min(residual.len());
                    (prefix[end] - prefix[start]) / (end - start) as f64
                })
                .collect(),
        )
    }

    fn achievable_gain(&self, residual_db: f64) -> f64 {
        (-residual_db).clamp(
            -self.settings.max_cut_db.max(0.0),
            self.settings.max_boost_db.max(0.0),
        )
    }

    fn next_filter(&mut self, filters: &[EqFilter]) -> Option<EqFilter> {
        let residual = self.smoothed_residual(filters)?;
        let frequencies_hz = &self.frequencies_hz;
        let free = |index: usize| {
            filters.iter().all(|filter| {
                (frequencies_hz[index] / filter.frequency_hz).log2().abs()
                    >= MIN_FILTER_SPACING_OCTAVES
            })
        };
        let (peak, gain) = residual
            .iter()
            .enumerate()
            .filter(|(index, _)| free(*index))
            .map(|(index, value)| (index, self.achievable_gain(*value)))
            .max_by(|left, right| left.1.abs().total_cmp(&right.1.abs()))?;
        if gain.abs() < MIN_CORRECTION_DB {
            return None;
        }
        let half = residual[peak] / 2.0;
        let same_side = |value: f64| value.signum() == half.signum() && value.abs() >= half.abs();
        let mut left = peak;
        while left > 0 && same_side(residual[left - 1]) {
            left -= 1;
        }
        let mut right = peak;
        while right + 1 < residual.len() && same_side(residual[right + 1]) {
            right += 1;
        }
        let bandwidth_octaves = (self.frequencies_hz[right] / self.frequencies_hz[left])
            .log2()
            .clamp(MIN_BANDWIDTH_OCTAVES, MAX_BANDWIDTH_OCTAVES);
        let mut filter = EqFilter {
            kind: FilterKind::Bell,
            frequency_hz: self.frequencies_hz[peak],
            gain_db: gain,
            q: q_from_bandwidth_octaves(bandwidth_octaves),
            bypassed: false,
        };
        self.clamp(&mut filter);
        Some(filter)
    }

    fn refine(&mut self, filters: &mut [EqFilter], scales: &[f64]) {
        let mut best = self.cost(filters);
        for scale in scales {
            for _ in 0..REFINE_PASSES {
                let mut improved = false;
                for index in 0..filters.len() {
                    for candidate in neighbours(filters[index], *scale) {
                        let mut candidate = candidate;
                        self.clamp(&mut candidate);
                        let crowded = filters.iter().enumerate().any(|(other, filter)| {
                            other != index
                                && (candidate.frequency_hz / filter.frequency_hz).log2().abs()
                                    < MIN_FILTER_SPACING_OCTAVES
                        });
                        if crowded {
                            continue;
                        }
                        let previous = filters[index];
                        filters[index] = candidate;
                        let cost = self.cost(filters);
                        if cost < best {
                            best = cost;
                            improved = true;
                        } else {
                            filters[index] = previous;
                        }
                    }
                }
                if !improved {
                    break;
                }
            }
        }
    }
}

fn neighbours(filter: EqFilter, scale: f64) -> [EqFilter; 6] {
    let frequency_factor = 2_f64.powf(FREQUENCY_STEP_OCTAVES * scale);
    let gain_step = GAIN_STEP_DB * scale;
    let q_factor = Q_STEP_FACTOR.powf(scale);
    [
        EqFilter {
            frequency_hz: filter.frequency_hz * frequency_factor,
            ..filter
        },
        EqFilter {
            frequency_hz: filter.frequency_hz / frequency_factor,
            ..filter
        },
        EqFilter {
            gain_db: filter.gain_db + gain_step,
            ..filter
        },
        EqFilter {
            gain_db: filter.gain_db - gain_step,
            ..filter
        },
        EqFilter {
            q: filter.q * q_factor,
            ..filter
        },
        EqFilter {
            q: filter.q / q_factor,
            ..filter
        },
    ]
}

fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        (sorted[middle - 1] + sorted[middle]) / 2.0
    } else {
        sorted[middle]
    }
}

pub fn auto_eq(
    response_db: &[Option<f64>],
    frequencies_hz: &[f64],
    sample_rate_hz: f64,
    settings: &AutoEqSettings,
) -> Result<Option<AutoEqResult>, DspError> {
    check_length("frequencies", response_db.len(), frequencies_hz.len())?;
    let low_hz = settings.low_hz.min(settings.high_hz);
    let high_hz = settings.low_hz.max(settings.high_hz);
    let (frequencies, levels): (Vec<f64>, Vec<f64>) = frequencies_hz
        .iter()
        .zip(response_db)
        .filter(|(frequency_hz, level)| {
            (low_hz..=high_hz).contains(*frequency_hz) && level.is_some_and(f64::is_finite)
        })
        .filter_map(|(frequency_hz, level)| level.map(|level| (*frequency_hz, level)))
        .unzip();
    let valid_sample_rate = sample_rate_hz.is_finite() && sample_rate_hz > 2.0 * high_hz;
    if levels.len() < MIN_RANGE_POINTS || !valid_sample_rate {
        return Ok(None);
    }
    let target_db = settings
        .target_db
        .filter(|target| target.is_finite())
        .unwrap_or_else(|| median(&levels));
    let point_count = levels.len();
    let mut problem = Problem {
        frequencies_hz: frequencies,
        levels_db: levels,
        target_db,
        sample_rate_hz,
        settings,
        low_hz,
        high_hz,
        scratch: vec![Complex64::new(1.0, 0.0); point_count],
    };
    let mut filters: Vec<EqFilter> = Vec::new();
    while filters.len() < settings.max_filters.min(MAX_EQ_FILTERS) {
        let Some(filter) = problem.next_filter(&filters) else {
            break;
        };
        filters.push(filter);
        problem.enforce_boost_limit(&mut filters);
        problem.refine(&mut filters, &QUICK_REFINE_SCALES);
    }
    problem.refine(&mut filters, &REFINE_SCALES);
    filters.retain(|filter| filter.gain_db.abs() >= NEGLIGIBLE_GAIN_DB);
    filters.sort_by(|left, right| left.frequency_hz.total_cmp(&right.frequency_hz));
    let rms_error_db = problem.rms_error(&filters);
    Ok(Some(AutoEqResult {
        filters,
        target_db,
        rms_error_db,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biquad::BiquadCoefficients;
    use crate::grid::DisplayGrid;

    const SAMPLE_RATE_HZ: f64 = 48_000.0;

    fn grid() -> Vec<f64> {
        DisplayGrid::new(20.0, 20_000.0, 96.0)
            .frequencies_hz()
            .to_vec()
    }

    fn settings(max_filters: usize, max_boost_db: f64) -> AutoEqSettings {
        AutoEqSettings {
            low_hz: 40.0,
            high_hz: 16_000.0,
            max_boost_db,
            max_cut_db: 12.0,
            max_filters,
            target_db: None,
        }
    }

    fn response(frequencies: &[f64], bells: &[(f64, f64, f64)]) -> Vec<Option<f64>> {
        frequencies
            .iter()
            .map(|frequency_hz| {
                let gain = bells
                    .iter()
                    .map(|(center_hz, q, gain_db)| {
                        BiquadCoefficients::bell(*center_hz, *q, *gain_db, SAMPLE_RATE_HZ)
                            .response_at(*frequency_hz, SAMPLE_RATE_HZ)
                    })
                    .fold(Complex64::new(1.0, 0.0), |product, value| product * value);
                Some(20.0 * gain.norm().log10() - 3.0)
            })
            .collect()
    }

    fn combined_db(filters: &[EqFilter], frequencies: &[f64]) -> Vec<f64> {
        let mut output = vec![Complex64::new(1.0, 0.0); frequencies.len()];
        eq_response(filters, frequencies, SAMPLE_RATE_HZ, &mut output).unwrap();
        output
            .iter()
            .map(|value| 20.0 * value.norm().log10())
            .collect()
    }

    #[test]
    fn a_single_resonance_is_cancelled() {
        let frequencies = grid();
        let measured = response(&frequencies, &[(1000.0, 2.0, 6.0)]);
        let result = auto_eq(&measured, &frequencies, SAMPLE_RATE_HZ, &settings(4, 6.0))
            .unwrap()
            .unwrap();
        assert!(result.rms_error_db < 0.3, "{result:?}");
        let main = result
            .filters
            .iter()
            .max_by(|left, right| left.gain_db.abs().total_cmp(&right.gain_db.abs()))
            .unwrap();
        assert!((main.frequency_hz / 1000.0 - 1.0).abs() < 0.1, "{main:?}");
        assert!((main.gain_db + 6.0).abs() < 1.0, "{main:?}");
    }

    #[test]
    fn the_filter_count_and_cut_limit_are_respected() {
        let frequencies = grid();
        let measured = response(
            &frequencies,
            &[
                (80.0, 4.0, 10.0),
                (250.0, 3.0, -6.0),
                (900.0, 2.0, 8.0),
                (2500.0, 5.0, 5.0),
                (6000.0, 3.0, -4.0),
            ],
        );
        let limits = AutoEqSettings {
            max_cut_db: 6.0,
            ..settings(3, 3.0)
        };
        let result = auto_eq(&measured, &frequencies, SAMPLE_RATE_HZ, &limits)
            .unwrap()
            .unwrap();
        assert!(!result.filters.is_empty() && result.filters.len() <= 3);
        assert!(
            result
                .filters
                .iter()
                .all(|filter| filter.gain_db >= -6.0 - 1.0e-9)
        );
        assert!(
            result
                .filters
                .iter()
                .all(|filter| filter.kind == FilterKind::Bell)
        );
    }

    #[test]
    fn boosts_never_exceed_the_limit_even_combined() {
        let frequencies = grid();
        let measured = response(&frequencies, &[(300.0, 1.0, -10.0), (400.0, 1.5, -8.0)]);
        let result = auto_eq(&measured, &frequencies, SAMPLE_RATE_HZ, &settings(6, 3.0))
            .unwrap()
            .unwrap();
        let in_range: Vec<f64> = frequencies
            .iter()
            .copied()
            .filter(|frequency_hz| (40.0..=16_000.0).contains(frequency_hz))
            .collect();
        for level in combined_db(&result.filters, &in_range) {
            assert!(level <= 3.0 + 0.05, "{level}");
        }
        let cut_only = auto_eq(&measured, &frequencies, SAMPLE_RATE_HZ, &settings(6, 0.0))
            .unwrap()
            .unwrap();
        assert!(cut_only.filters.iter().all(|filter| filter.gain_db <= 0.0));
    }

    #[test]
    fn a_flat_response_needs_no_filter_and_bad_input_gives_none() {
        let frequencies = grid();
        let flat = vec![Some(-3.0); frequencies.len()];
        let result = auto_eq(&flat, &frequencies, SAMPLE_RATE_HZ, &settings(8, 6.0))
            .unwrap()
            .unwrap();
        assert!(result.filters.is_empty());
        assert!(result.rms_error_db < 1.0e-9);
        let invalid = vec![None; frequencies.len()];
        assert_eq!(
            auto_eq(&invalid, &frequencies, SAMPLE_RATE_HZ, &settings(8, 6.0)).unwrap(),
            None
        );
        let mut with_nan = flat.clone();
        with_nan[400] = Some(f64::NAN);
        assert!(
            auto_eq(&with_nan, &frequencies, SAMPLE_RATE_HZ, &settings(8, 6.0))
                .unwrap()
                .is_some()
        );
        assert!(auto_eq(&flat[1..], &frequencies, SAMPLE_RATE_HZ, &settings(8, 6.0)).is_err());
    }

    #[test]
    fn filters_keep_a_sixth_of_an_octave_apart() {
        let frequencies = grid();
        let measured = response(
            &frequencies,
            &[(60.0, 1.0, 6.0), (70.0, 6.0, -5.0), (8000.0, 2.0, 4.0)],
        );
        let result = auto_eq(&measured, &frequencies, SAMPLE_RATE_HZ, &settings(8, 6.0))
            .unwrap()
            .unwrap();
        for pair in result.filters.windows(2) {
            let spacing = (pair[1].frequency_hz / pair[0].frequency_hz).log2();
            assert!(spacing >= 1.0 / 6.0 - 1.0e-9, "{pair:?}");
        }
    }

    #[test]
    fn sixteen_filters_are_computed_quickly() {
        let frequencies = grid();
        let bells: Vec<(f64, f64, f64)> = (0..16)
            .map(|index| {
                let sign = if index % 2 == 0 { 1.0 } else { -1.0 };
                (50.0 * 1.5_f64.powi(index), 4.0, sign * 6.0)
            })
            .collect();
        let measured = response(&frequencies, &bells);
        let started = std::time::Instant::now();
        let result = auto_eq(&measured, &frequencies, SAMPLE_RATE_HZ, &settings(16, 6.0))
            .unwrap()
            .unwrap();
        assert!(started.elapsed() < std::time::Duration::from_secs(2));
        assert!(result.filters.len() <= 16);
        assert!(result.rms_error_db < 2.0, "{}", result.rms_error_db);
    }
}
