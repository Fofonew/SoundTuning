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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlignmentAnalysis {
    pub positive: AlignmentResult,
    pub negative: AlignmentResult,
    pub flattest: AlignmentResult,
}

#[derive(Clone, Copy)]
struct Candidate {
    step: i64,
    polarity: Polarity,
    mean_level_db: f64,
    level_spread_db: f64,
}

impl Candidate {
    fn score(self, criterion: AlignmentCriterion, louder_trace_mean_db: f64) -> f64 {
        match criterion {
            AlignmentCriterion::MaximumLevel => self.mean_level_db,
            AlignmentCriterion::Flattest if self.mean_level_db >= louder_trace_mean_db => {
                -self.level_spread_db
            }
            AlignmentCriterion::Flattest => DESTRUCTIVE_SUM_SCORE_OFFSET + self.mean_level_db,
        }
    }

    fn result(self) -> AlignmentResult {
        AlignmentResult {
            delay_ms: self.step as f64 * FINE_STEP_MS,
            polarity: self.polarity,
            mean_level_db: self.mean_level_db,
            level_spread_db: self.level_spread_db,
        }
    }
}

const POLARITIES: [Polarity; 2] = [Polarity::Normal, Polarity::Inverted];
const CRITERIA: [AlignmentCriterion; 2] = [
    AlignmentCriterion::MaximumLevel,
    AlignmentCriterion::Flattest,
];

fn evaluate(points: &[BandPoint], step: i64, levels: &mut Vec<[f64; 2]>) -> [Candidate; 2] {
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
    POLARITIES.map(|polarity| {
        let column = usize::from(polarity == Polarity::Inverted);
        let mean = levels.iter().map(|level| level[column]).sum::<f64>() / count;
        let variance = levels
            .iter()
            .map(|level| (level[column] - mean).powi(2))
            .sum::<f64>()
            / count;
        Candidate {
            step,
            polarity,
            mean_level_db: mean,
            level_spread_db: variance.max(0.0).sqrt(),
        }
    })
}

fn better(
    left: Candidate,
    right: Candidate,
    criterion: AlignmentCriterion,
    louder_trace_mean_db: f64,
) -> Candidate {
    let left_score = left.score(criterion, louder_trace_mean_db);
    let right_score = right.score(criterion, louder_trace_mean_db);
    let left_is_better = left_score > right_score
        || (left_score == right_score && left.step.abs() < right.step.abs());
    if left_is_better { left } else { right }
}

struct SweepResult {
    best: [[Candidate; 2]; 2],
    louder_trace_mean_db: f64,
}

impl SweepResult {
    fn overall(&self, criterion: AlignmentCriterion) -> Candidate {
        let [normal, inverted] = self.best[criterion_index(criterion)];
        better(inverted, normal, criterion, self.louder_trace_mean_db)
    }
}

fn criterion_index(criterion: AlignmentCriterion) -> usize {
    usize::from(criterion == AlignmentCriterion::Flattest)
}

fn sweep(
    response_a: &[Option<Complex64>],
    response_b: &[Option<Complex64>],
    frequencies_hz: &[f64],
    band_hz: (f64, f64),
    range_ms: f64,
) -> Result<Option<SweepResult>, DspError> {
    check_length("trace B", response_a.len(), response_b.len())?;
    check_length("frequencies", response_a.len(), frequencies_hz.len())?;
    let band_low_hz = band_hz.0.min(band_hz.1);
    let band_high_hz = band_hz.0.max(band_hz.1);
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
    if points.len() < MIN_BAND_POINTS || !range_ms.is_finite() {
        return Ok(None);
    }

    let max_step = (range_ms.abs().min(MAX_SEARCH_RANGE_MS) / FINE_STEP_MS).round() as i64;
    let mean_db_of = |pick: fn(&BandPoint) -> Complex64| {
        points
            .iter()
            .map(|point| floored_db(pick(point).norm_sqr()))
            .sum::<f64>()
            / points.len() as f64
    };
    let louder_trace_mean_db = mean_db_of(|point| point.a).max(mean_db_of(|point| point.b));
    let mut levels = Vec::with_capacity(points.len());
    let mut coarse: [[Option<Candidate>; 2]; 2] = [[None; 2]; 2];
    let mut step = -max_step;
    loop {
        let candidates = evaluate(&points, step, &mut levels);
        for criterion in CRITERIA {
            for (slot, candidate) in coarse[criterion_index(criterion)]
                .iter_mut()
                .zip(candidates)
            {
                *slot = Some(slot.map_or(candidate, |current| {
                    better(candidate, current, criterion, louder_trace_mean_db)
                }));
            }
        }
        if step >= max_step {
            break;
        }
        step = (step + FINE_STEPS_PER_COARSE_STEP).min(max_step);
    }

    let mut best = [[None; 2]; 2];
    for criterion in CRITERIA {
        for (slot, start) in best[criterion_index(criterion)]
            .iter_mut()
            .zip(coarse[criterion_index(criterion)])
        {
            let Some(start) = start else {
                continue;
            };
            let column = usize::from(start.polarity == Polarity::Inverted);
            let low = (start.step - FINE_STEPS_PER_COARSE_STEP).max(-max_step);
            let high = (start.step + FINE_STEPS_PER_COARSE_STEP).min(max_step);
            let mut refined = start;
            for fine_step in low..=high {
                if let Some(candidate) = evaluate(&points, fine_step, &mut levels).get(column) {
                    refined = better(*candidate, refined, criterion, louder_trace_mean_db);
                }
            }
            *slot = Some(refined);
        }
    }
    let unwrap_slots = |[normal, inverted]: [Option<Candidate>; 2]| Some([normal?, inverted?]);
    let (Some(level), Some(flat)) = (unwrap_slots(best[0]), unwrap_slots(best[1])) else {
        return Ok(None);
    };
    Ok(Some(SweepResult {
        best: [level, flat],
        louder_trace_mean_db,
    }))
}

pub fn analyze_alignment(
    response_a: &[Option<Complex64>],
    response_b: &[Option<Complex64>],
    frequencies_hz: &[f64],
    band_hz: (f64, f64),
    range_ms: f64,
) -> Result<Option<AlignmentAnalysis>, DspError> {
    let sweep = sweep(response_a, response_b, frequencies_hz, band_hz, range_ms)?;
    Ok(sweep.map(|sweep| {
        let [positive, negative] = sweep.best[criterion_index(AlignmentCriterion::MaximumLevel)];
        AlignmentAnalysis {
            positive: positive.result(),
            negative: negative.result(),
            flattest: sweep.overall(AlignmentCriterion::Flattest).result(),
        }
    }))
}

pub fn find_optimal_alignment(
    response_a: &[Option<Complex64>],
    response_b: &[Option<Complex64>],
    frequencies_hz: &[f64],
    search: &AlignmentSearch,
) -> Result<Option<AlignmentResult>, DspError> {
    let sweep = sweep(
        response_a,
        response_b,
        frequencies_hz,
        (search.band_low_hz, search.band_high_hz),
        search.range_ms,
    )?;
    Ok(sweep.map(|sweep| sweep.overall(search.criterion).result()))
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
    fn analysis_offers_both_polarities_and_the_flattest_sum() {
        let grid = DisplayGrid::default();
        let frequencies = grid.frequencies_hz();
        let a: Vec<Option<Complex64>> = vec![Some(Complex64::new(1.0, 0.0)); grid.len()];
        let b = delayed(&a, frequencies, 1.5);
        let analysis = analyze_alignment(&a, &b, frequencies, DEFAULT_ALIGNMENT_BAND_HZ, 50.0)
            .unwrap()
            .unwrap();
        assert_eq!(analysis.positive.polarity, Polarity::Normal);
        assert!((analysis.positive.delay_ms + 1.5).abs() < 0.015);
        assert!((analysis.positive.mean_level_db - 6.0206).abs() < 0.01);
        assert_eq!(analysis.negative.polarity, Polarity::Inverted);
        assert!(analysis.negative.mean_level_db < analysis.positive.mean_level_db);
        assert!(analysis.negative.mean_level_db > 0.0);
        assert_eq!(analysis.flattest.polarity, Polarity::Normal);
        assert!((analysis.flattest.delay_ms + 1.5).abs() < 0.015);
        let single = find_optimal_alignment(
            &a,
            &b,
            frequencies,
            &AlignmentSearch {
                criterion: AlignmentCriterion::MaximumLevel,
                ..AlignmentSearch::default()
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(single, analysis.positive);
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
