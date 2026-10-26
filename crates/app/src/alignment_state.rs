use dsp_core::{AlignmentAnalysis, AlignmentResult, Polarity};
use serde::{Deserialize, Serialize};

pub const SEARCH_RANGE_MS: f64 = 50.0;
pub const BAND_LIMIT_LOW_HZ: f64 = 20.0;
pub const BAND_LIMIT_HIGH_HZ: f64 = 20_000.0;
pub const MIN_BANDWIDTH_OCTAVES: f64 = 1.0 / 3.0;
pub const MAX_BANDWIDTH_OCTAVES: f64 = 6.0;
const LOW_RANGE_HZ: (f64, f64) = (50.0, 200.0);
const MID_CENTER_HZ: f64 = 1000.0;
const MID_BANDWIDTH_OCTAVES: f64 = 3.0;
const DEFAULT_CROSSOVER_HZ: f64 = 100.0;
const DEFAULT_BANDWIDTH_OCTAVES: f64 = 2.0;

pub fn normalized_band(low_hz: f64, high_hz: f64) -> (f64, f64) {
    let low = low_hz
        .min(high_hz)
        .clamp(BAND_LIMIT_LOW_HZ, BAND_LIMIT_HIGH_HZ);
    let high = low_hz
        .max(high_hz)
        .clamp(BAND_LIMIT_LOW_HZ, BAND_LIMIT_HIGH_HZ);
    let min_ratio = 2_f64.powf(MIN_BANDWIDTH_OCTAVES);
    if high / low >= min_ratio {
        return (low, high);
    }
    let center = (low * high).sqrt();
    let half = min_ratio.sqrt();
    let low = (center / half).max(BAND_LIMIT_LOW_HZ);
    (low, (low * min_ratio).min(BAND_LIMIT_HIGH_HZ))
}

fn band_around(center_hz: f64, bandwidth_octaves: f64) -> (f64, f64) {
    let half_ratio = 2_f64.powf(bandwidth_octaves / 2.0);
    normalized_band(center_hz / half_ratio, center_hz * half_ratio)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlignmentRange {
    #[default]
    Low,
    Mid,
    Full,
    Custom,
}

impl AlignmentRange {
    pub const ALL: [Self; 4] = [Self::Low, Self::Mid, Self::Full, Self::Custom];
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlignmentChoice {
    #[default]
    Positive,
    Negative,
    Flat,
    Custom,
}

impl AlignmentChoice {
    pub const ALL: [Self; 4] = [Self::Positive, Self::Negative, Self::Flat, Self::Custom];
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AlignmentSolution {
    pub delay_ms: f64,
    pub inverted: bool,
    pub mean_level_db: f64,
}

impl From<AlignmentResult> for AlignmentSolution {
    fn from(result: AlignmentResult) -> Self {
        Self {
            delay_ms: result.delay_ms,
            inverted: result.polarity == Polarity::Inverted,
            mean_level_db: result.mean_level_db,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AlignmentSolutions {
    pub positive: AlignmentSolution,
    pub negative: AlignmentSolution,
    pub flat: AlignmentSolution,
}

impl From<AlignmentAnalysis> for AlignmentSolutions {
    fn from(analysis: AlignmentAnalysis) -> Self {
        Self {
            positive: analysis.positive.into(),
            negative: analysis.negative.into(),
            flat: analysis.flattest.into(),
        }
    }
}

impl AlignmentSolutions {
    pub fn louder(&self) -> AlignmentChoice {
        if self.negative.mean_level_db > self.positive.mean_level_db {
            AlignmentChoice::Negative
        } else {
            AlignmentChoice::Positive
        }
    }

    pub fn relative_power(&self, choice: AlignmentChoice) -> Option<f64> {
        let level_db = match choice {
            AlignmentChoice::Positive => self.positive.mean_level_db,
            AlignmentChoice::Negative => self.negative.mean_level_db,
            AlignmentChoice::Flat | AlignmentChoice::Custom => return None,
        };
        let best_db = self.positive.mean_level_db.max(self.negative.mean_level_db);
        Some(10_f64.powf((level_db - best_db) / 10.0).clamp(0.0, 1.0))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AlignmentState {
    pub reference_trace: Option<u64>,
    pub delayed_trace: Option<u64>,
    pub range: AlignmentRange,
    pub crossover_hz: f64,
    pub bandwidth_octaves: f64,
    pub choice: AlignmentChoice,
    pub custom_delay_ms: f64,
    pub custom_inverted: bool,
    pub sum_hidden: bool,
    pub solutions: Option<AlignmentSolutions>,
}

impl Default for AlignmentState {
    fn default() -> Self {
        Self {
            reference_trace: None,
            delayed_trace: None,
            range: AlignmentRange::default(),
            crossover_hz: DEFAULT_CROSSOVER_HZ,
            bandwidth_octaves: DEFAULT_BANDWIDTH_OCTAVES,
            choice: AlignmentChoice::default(),
            custom_delay_ms: 0.0,
            custom_inverted: false,
            sum_hidden: false,
            solutions: None,
        }
    }
}

impl AlignmentState {
    pub fn forget_trace(&mut self, id: u64) {
        if self.reference_trace == Some(id) {
            self.reference_trace = None;
        }
        if self.delayed_trace == Some(id) {
            self.delayed_trace = None;
        }
        if !self.is_ready() {
            self.solutions = None;
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!(
            (self.reference_trace, self.delayed_trace),
            (Some(reference), Some(delayed)) if reference != delayed
        )
    }

    pub fn band_hz(&self) -> (f64, f64) {
        match self.range {
            AlignmentRange::Low => LOW_RANGE_HZ,
            AlignmentRange::Mid => band_around(MID_CENTER_HZ, MID_BANDWIDTH_OCTAVES),
            AlignmentRange::Full => (BAND_LIMIT_LOW_HZ, BAND_LIMIT_HIGH_HZ),
            AlignmentRange::Custom => band_around(self.crossover_hz, self.bandwidth_octaves),
        }
    }

    pub fn set_custom_band(&mut self, low_hz: f64, high_hz: f64) {
        let (low_hz, high_hz) = normalized_band(low_hz, high_hz);
        self.range = AlignmentRange::Custom;
        self.crossover_hz = (low_hz * high_hz).sqrt();
        self.bandwidth_octaves = (high_hz / low_hz)
            .log2()
            .clamp(MIN_BANDWIDTH_OCTAVES, MAX_BANDWIDTH_OCTAVES);
    }

    pub fn setting(&self, choice: AlignmentChoice) -> Option<(f64, bool)> {
        let solution = |solution: AlignmentSolution| (solution.delay_ms, solution.inverted);
        match choice {
            AlignmentChoice::Custom => Some((self.custom_delay_ms, self.custom_inverted)),
            AlignmentChoice::Positive => self.solutions.map(|found| solution(found.positive)),
            AlignmentChoice::Negative => self.solutions.map(|found| solution(found.negative)),
            AlignmentChoice::Flat => self.solutions.map(|found| solution(found.flat)),
        }
    }

    pub fn select(&mut self, choice: AlignmentChoice) {
        if self.choice == choice {
            self.sum_hidden = !self.sum_hidden;
        } else {
            self.choice = choice;
            self.sum_hidden = false;
        }
    }

    pub fn shows_sum(&self) -> bool {
        self.is_ready() && !self.sum_hidden
    }

    pub fn selected_setting(&self) -> Option<(f64, bool)> {
        self.setting(self.choice)
    }

    pub fn adopt_solutions(&mut self, solutions: Option<AlignmentSolutions>) {
        self.solutions = solutions;
        if let Some(found) = solutions
            && matches!(
                self.choice,
                AlignmentChoice::Positive | AlignmentChoice::Negative
            )
        {
            self.choice = found.louder();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solution(delay_ms: f64, inverted: bool, mean_level_db: f64) -> AlignmentSolution {
        AlignmentSolution {
            delay_ms,
            inverted,
            mean_level_db,
        }
    }

    fn solutions() -> AlignmentSolutions {
        AlignmentSolutions {
            positive: solution(1.5, false, 6.0),
            negative: solution(4.0, true, 3.0),
            flat: solution(2.0, false, 5.0),
        }
    }

    #[test]
    fn forgetting_a_trace_clears_the_solutions() {
        let mut state = AlignmentState {
            reference_trace: Some(1),
            delayed_trace: Some(2),
            solutions: Some(solutions()),
            ..AlignmentState::default()
        };
        assert!(state.is_ready());
        state.forget_trace(2);
        assert!(!state.is_ready());
        assert_eq!(state.solutions, None);
    }

    #[test]
    fn bands_are_ordered_clamped_and_at_least_a_third_of_an_octave() {
        assert_eq!(normalized_band(150.0, 60.0), (60.0, 150.0));
        assert_eq!(normalized_band(5.0, 40_000.0), (20.0, 20_000.0));
        let (low, high) = normalized_band(100.0, 101.0);
        assert!((high / low - 2_f64.powf(1.0 / 3.0)).abs() < 1.0e-9);
        assert!(low < 100.0 && high > 101.0);
    }

    #[test]
    fn ranges_follow_systune() {
        let mut state = AlignmentState::default();
        assert_eq!(state.band_hz(), (50.0, 200.0));
        state.range = AlignmentRange::Mid;
        let (low, high) = state.band_hz();
        assert!((low * high).sqrt() - 1000.0 < 1.0e-6);
        assert!(((high / low).log2() - 3.0).abs() < 1.0e-9);
        state.range = AlignmentRange::Custom;
        state.crossover_hz = 80.0;
        state.bandwidth_octaves = 2.0;
        let (low, high) = state.band_hz();
        assert!((low - 40.0).abs() < 1.0e-9 && (high - 160.0).abs() < 1.0e-9);
        state.set_custom_band(400.0, 100.0);
        assert!((state.crossover_hz - 200.0).abs() < 1.0e-9);
        assert!((state.bandwidth_octaves - 2.0).abs() < 1.0e-9);
    }

    #[test]
    fn the_louder_polarity_is_selected_and_custom_is_kept() {
        let mut state = AlignmentState::default();
        let mut found = solutions();
        found.negative.mean_level_db = 7.0;
        state.adopt_solutions(Some(found));
        assert_eq!(state.choice, AlignmentChoice::Negative);
        assert_eq!(state.selected_setting(), Some((4.0, true)));
        assert!((found.relative_power(AlignmentChoice::Positive).unwrap() - 0.7943).abs() < 1.0e-3);
        assert_eq!(found.relative_power(AlignmentChoice::Negative), Some(1.0));
        state.choice = AlignmentChoice::Custom;
        state.custom_delay_ms = -2.5;
        state.adopt_solutions(Some(solutions()));
        assert_eq!(state.choice, AlignmentChoice::Custom);
        assert_eq!(state.selected_setting(), Some((-2.5, false)));
    }

    #[test]
    fn clicking_the_selected_choice_again_hides_its_sum() {
        let mut state = AlignmentState {
            reference_trace: Some(1),
            delayed_trace: Some(2),
            ..AlignmentState::default()
        };
        assert!(state.shows_sum());
        state.select(AlignmentChoice::Positive);
        assert!(!state.shows_sum());
        state.select(AlignmentChoice::Positive);
        assert!(state.shows_sum());
        state.select(AlignmentChoice::Positive);
        state.select(AlignmentChoice::Flat);
        assert_eq!(state.choice, AlignmentChoice::Flat);
        assert!(state.shows_sum());
    }

    #[test]
    fn same_trace_twice_is_not_ready() {
        let state = AlignmentState {
            reference_trace: Some(1),
            delayed_trace: Some(1),
            ..AlignmentState::default()
        };
        assert!(!state.is_ready());
    }
}
