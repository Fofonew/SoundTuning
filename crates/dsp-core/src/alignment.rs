use std::f64::consts::TAU;

use crate::Complex64;
use crate::decibel::MIN_POWER_FOR_DB;
use crate::error::{DspError, check_length};

pub const DEFAULT_ALIGNMENT_BAND_HZ: (f64, f64) = (60.0, 150.0);
const DEFAULT_SEARCH_RANGE_MS: f64 = 50.0;
const MAX_SEARCH_RANGE_MS: f64 = 1000.0;
const FINE_STEP_MS: f64 = 0.01;
const FINE_STEPS_PER_COARSE_STEP: i64 = 10;
const MILLISECONDS_PER_SECOND: f64 = 1000.0;
const MIN_BAND_POINTS: usize = 2;
const DESTRUCTIVE_SUM_SCORE_OFFSET: f64 = -1.0e9;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Polarity {
    #[default]
    Normal,
    Inverted,
}

impl Polarity {
    pub fn sign(self) -> f64 {
        match self {
            Self::Normal => 1.0,
            Self::Inverted => -1.0,
        }
    }

    pub fn toggled(self) -> Self {
        match self {
            Self::Normal => Self::Inverted,
            Self::Inverted => Self::Normal,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AlignmentCriterion {
    #[default]
    MaximumLevel,
    Flattest,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlignmentSearch {
    pub band_low_hz: f64,
    pub band_high_hz: f64,
    pub criterion: AlignmentCriterion,
    pub range_ms: f64,
}

impl Default for AlignmentSearch {
    fn default() -> Self {
        Self {
            band_low_hz: DEFAULT_ALIGNMENT_BAND_HZ.0,
            band_high_hz: DEFAULT_ALIGNMENT_BAND_HZ.1,
            criterion: AlignmentCriterion::default(),
            range_ms: DEFAULT_SEARCH_RANGE_MS,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlignmentResult {
    pub delay_ms: f64,
    pub polarity: Polarity,
    pub mean_level_db: f64,
    pub level_spread_db: f64,
}

fn summed_power(
    a: Complex64,
    b: Complex64,
    angular_frequency: f64,
    delay_s: f64,
    sign: f64,
) -> f64 {
    (a + b * Complex64::from_polar(sign, -angular_frequency * delay_s)).norm_sqr()
}

fn floored_db(power: f64) -> f64 {
    10.0 * power.max(MIN_POWER_FOR_DB).log10()
}

pub fn summed_response_db(
    response_a: &[Option<Complex64>],
    response_b: &[Option<Complex64>],
    frequencies_hz: &[f64],
    delay_ms: f64,
    polarity: Polarity,
    output: &mut [Option<f64>],
) -> Result<(), DspError> {
    check_length("trace B", response_a.len(), response_b.len())?;
    check_length("frequencies", response_a.len(), frequencies_hz.len())?;
    check_length("summation output", response_a.len(), output.len())?;
    let delay_s = delay_ms / MILLISECONDS_PER_SECOND;
    let sign = polarity.sign();
    for (((slot, a), b), frequency_hz) in output
        .iter_mut()
        .zip(response_a)
        .zip(response_b)
        .zip(frequencies_hz)
    {
        *slot = match (a, b) {
            (Some(a), Some(b)) => Some(floored_db(summed_power(
                *a,
                *b,
                TAU * frequency_hz,
                delay_s,
                sign,
            ))),
            _ => None,
        };
    }
    Ok(())
}

struct BandPoint {
    angular_frequency: f64,
    a: Complex64,
    b: Complex64,
}

#[derive(Clone, Copy)]
struct Candidate {
    step: i64,
    polarity: Polarity,
    score: f64,
    mean_level_db: f64,
    level_spread_db: f64,
}

fn evaluate(
    points: &[BandPoint],
    step: i64,
    criterion: AlignmentCriterion,
    louder_trace_mean_db: f64,
    levels: &mut Vec<[f64; 2]>,
) -> [Candidate; 2] {
    let delay_s = step as f64 * FINE_STEP_MS / MILLISECONDS_PER_SECOND;
    levels.clear();
    levels.extend(points.iter().map(|point| {
        let rotated_b = point.b * Complex64::from_polar(1.0, -point.angular_frequency * delay_s);
        [
            floored_db((point.a + rotated_b).norm_sqr()),
            floored_db((point.a - rotated_b).norm_sqr()),
        ]
    }));
    let count = levels.len() as f64;
    [Polarity::Normal, Polarity::Inverted].map(|polarity| {
        let column = usize::from(polarity == Polarity::Inverted);
        let mean = levels.iter().map(|level| level[column]).sum::<f64>() / count;
        let variance = levels
            .iter()
            .map(|level| (level[column] - mean).powi(2))
            .sum::<f64>()
            / count;
        let spread = variance.max(0.0).sqrt();
        Candidate {
            step,
            polarity,
            score: match criterion {
                AlignmentCriterion::MaximumLevel => mean,
                AlignmentCriterion::Flattest if mean >= louder_trace_mean_db => -spread,
                AlignmentCriterion::Flattest => DESTRUCTIVE_SUM_SCORE_OFFSET + mean,
            },
            mean_level_db: mean,
            level_spread_db: spread,
        }
    })
}

fn better(left: Candidate, right: Candidate) -> Candidate {
    let left_is_better = left.score > right.score
        || (left.score == right.score && left.step.abs() < right.step.abs());
    if left_is_better { left } else { right }
}

pub fn find_optimal_alignment(
    response_a: &[Option<Complex64>],
    response_b: &[Option<Complex64>],
    frequencies_hz: &[f64],
    search: &AlignmentSearch,
) -> Result<Option<AlignmentResult>, DspError> {
    check_length("trace B", response_a.len(), response_b.len())?;
    check_length("frequencies", response_a.len(), frequencies_hz.len())?;
    let band_low_hz = search.band_low_hz.min(search.band_high_hz);
    let band_high_hz = search.band_low_hz.max(search.band_high_hz);
    let points: Vec<BandPoint> = response_a
        .iter()
        .zip(response_b)
        .zip(frequencies_hz)
        .filter(|(_, frequency_hz)| (band_low_hz..=band_high_hz).contains(*frequency_hz))
        .filter_map(|((a, b), frequency_hz)| match (a, b) {
            (Some(a), Some(b)) => Some(BandPoint {
                angular_frequency: TAU * frequency_hz,
                a: *a,
                b: *b,
            }),
            _ => None,
        })
        .collect();
    if points.len() < MIN_BAND_POINTS || !search.range_ms.is_finite() {
        return Ok(None);
    }

    let max_step = (search.range_ms.abs().min(MAX_SEARCH_RANGE_MS) / FINE_STEP_MS).round() as i64;
    let mean_db_of = |pick: fn(&BandPoint) -> Complex64| {
        points
            .iter()
            .map(|point| floored_db(pick(point).norm_sqr()))
            .sum::<f64>()
            / points.len() as f64
    };
    let louder_trace_mean_db = mean_db_of(|point| point.a).max(mean_db_of(|point| point.b));
    let mut levels = Vec::with_capacity(points.len());
    let mut best_per_polarity: [Option<Candidate>; 2] = [None, None];
    let mut step = -max_step;
    loop {
        for (slot, candidate) in best_per_polarity.iter_mut().zip(evaluate(
            &points,
            step,
            search.criterion,
            louder_trace_mean_db,
            &mut levels,
        )) {
            *slot = Some(slot.map_or(candidate, |current| better(candidate, current)));
        }
        if step >= max_step {
            break;
        }
        step = (step + FINE_STEPS_PER_COARSE_STEP).min(max_step);
    }

    let mut best: Option<Candidate> = None;
    for coarse in best_per_polarity.into_iter().flatten() {
        let polarity_column = usize::from(coarse.polarity == Polarity::Inverted);
        let mut refined = coarse;
        let low = (coarse.step - FINE_STEPS_PER_COARSE_STEP).max(-max_step);
        let high = (coarse.step + FINE_STEPS_PER_COARSE_STEP).min(max_step);
        for fine_step in low..=high {
            let candidates = evaluate(
                &points,
                fine_step,
                search.criterion,
                louder_trace_mean_db,
                &mut levels,
            );
            if let Some(candidate) = candidates.get(polarity_column) {
                refined = better(*candidate, refined);
            }
        }
        best = Some(best.map_or(refined, |current| better(refined, current)));
    }

    Ok(best.map(|candidate| AlignmentResult {
        delay_ms: candidate.step as f64 * FINE_STEP_MS,
        polarity: candidate.polarity,
        mean_level_db: candidate.mean_level_db,
        level_spread_db: candidate.level_spread_db,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::DisplayGrid;

    fn delayed(
        response: &[Option<Complex64>],
        frequencies: &[f64],
        delay_ms: f64,
    ) -> Vec<Option<Complex64>> {
        response
            .iter()
            .zip(frequencies)
            .map(|(value, frequency)| {
                value.map(|value| {
                    value
                        * Complex64::from_polar(
                            1.0,
                            -TAU * frequency * delay_ms / MILLISECONDS_PER_SECOND,
                        )
                })
            })
            .collect()
    }

    #[test]
    fn polarity_toggles() {
        assert_eq!(Polarity::Normal.toggled(), Polarity::Inverted);
        assert_eq!(Polarity::Inverted.sign(), -1.0);
    }

    #[test]
    fn traces_missing_in_band_give_no_result() {
        let grid = DisplayGrid::default();
        let invalid = vec![None; grid.len()];
        let result = find_optimal_alignment(
            &invalid,
            &invalid,
            grid.frequencies_hz(),
            &AlignmentSearch::default(),
        )
        .unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn huge_search_range_is_bounded_and_band_order_does_not_matter() {
        let grid = DisplayGrid::default();
        let frequencies = grid.frequencies_hz();
        let a: Vec<Option<Complex64>> = vec![Some(Complex64::new(1.0, 0.0)); grid.len()];
        let b = delayed(&a, frequencies, 0.5);
        let search = AlignmentSearch {
            band_low_hz: 150.0,
            band_high_hz: 60.0,
            range_ms: 1.0e12,
            ..AlignmentSearch::default()
        };
        let result = find_optimal_alignment(&a, &b, frequencies, &search)
            .unwrap()
            .unwrap();
        assert!((result.delay_ms + 0.5).abs() < 0.015);
    }

    #[test]
    fn delays_at_the_edge_of_the_range_are_found() {
        let grid = DisplayGrid::default();
        let frequencies = grid.frequencies_hz();
        let a: Vec<Option<Complex64>> = vec![Some(Complex64::new(1.0, 0.0)); grid.len()];
        for edge_ms in [-50.0, 50.0] {
            let b = delayed(&a, frequencies, -edge_ms);
            let result = find_optimal_alignment(&a, &b, frequencies, &AlignmentSearch::default())
                .unwrap()
                .unwrap();
            assert!(
                (result.delay_ms - edge_ms).abs() < 0.015,
                "{edge_ms}: {}",
                result.delay_ms
            );
            assert_eq!(result.polarity, Polarity::Normal);
        }
    }

    #[test]
    fn inverted_polarity_is_detected() {
        let grid = DisplayGrid::default();
        let frequencies = grid.frequencies_hz();
        let a: Vec<Option<Complex64>> = vec![Some(Complex64::new(1.0, 0.0)); grid.len()];
        let b: Vec<Option<Complex64>> = delayed(&a, frequencies, 2.0)
            .into_iter()
            .map(|value| value.map(|value| -value))
            .collect();
        let result = find_optimal_alignment(&a, &b, frequencies, &AlignmentSearch::default())
            .unwrap()
            .unwrap();
        assert_eq!(result.polarity, Polarity::Inverted);
        assert!((result.delay_ms + 2.0).abs() < 0.015);
    }
}
