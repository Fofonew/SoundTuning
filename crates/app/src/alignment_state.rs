use dsp_core::{AlignmentCriterion, AlignmentSearch, DEFAULT_ALIGNMENT_BAND_HZ, Polarity};
use serde::{Deserialize, Serialize};

const DEFAULT_SEARCH_RANGE_MS: f64 = 50.0;
pub const BAND_LIMIT_LOW_HZ: f64 = 20.0;
pub const BAND_LIMIT_HIGH_HZ: f64 = 20_000.0;
const MIN_BAND_OCTAVES: f64 = 1.0 / 3.0;

pub fn normalized_band(low_hz: f64, high_hz: f64) -> (f64, f64) {
    let low = low_hz
        .min(high_hz)
        .clamp(BAND_LIMIT_LOW_HZ, BAND_LIMIT_HIGH_HZ);
    let high = low_hz
        .max(high_hz)
        .clamp(BAND_LIMIT_LOW_HZ, BAND_LIMIT_HIGH_HZ);
    let min_ratio = 2_f64.powf(MIN_BAND_OCTAVES);
    if high / low >= min_ratio {
        return (low, high);
    }
    let center = (low * high).sqrt();
    let half = min_ratio.sqrt();
    let low = (center / half).max(BAND_LIMIT_LOW_HZ);
    (low, (low * min_ratio).min(BAND_LIMIT_HIGH_HZ))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoredCriterion {
    #[default]
    MaximumLevel,
    Flattest,
}

impl StoredCriterion {
    pub fn to_dsp(self) -> AlignmentCriterion {
        match self {
            Self::MaximumLevel => AlignmentCriterion::MaximumLevel,
            Self::Flattest => AlignmentCriterion::Flattest,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AlignmentState {
    pub trace_a: Option<u64>,
    pub trace_b: Option<u64>,
    pub criterion: StoredCriterion,
    pub band_low_hz: f64,
    pub band_high_hz: f64,
    pub delay_ms: f64,
    pub inverted: bool,
    pub proposal_delay_ms: Option<f64>,
    pub proposal_inverted: Option<bool>,
}

impl Default for AlignmentState {
    fn default() -> Self {
        Self {
            trace_a: None,
            trace_b: None,
            criterion: StoredCriterion::default(),
            band_low_hz: DEFAULT_ALIGNMENT_BAND_HZ.0,
            band_high_hz: DEFAULT_ALIGNMENT_BAND_HZ.1,
            delay_ms: 0.0,
            inverted: false,
            proposal_delay_ms: None,
            proposal_inverted: None,
        }
    }
}

impl AlignmentState {
    pub fn forget_trace(&mut self, id: u64) {
        if self.trace_a == Some(id) {
            self.trace_a = None;
        }
        if self.trace_b == Some(id) {
            self.trace_b = None;
        }
        if self.trace_a.is_none() || self.trace_b.is_none() {
            self.proposal_delay_ms = None;
            self.proposal_inverted = None;
        }
    }

    pub fn polarity(&self) -> Polarity {
        if self.inverted {
            Polarity::Inverted
        } else {
            Polarity::Normal
        }
    }

    pub fn search(&self) -> AlignmentSearch {
        AlignmentSearch {
            band_low_hz: self.band_low_hz,
            band_high_hz: self.band_high_hz,
            criterion: self.criterion.to_dsp(),
            range_ms: DEFAULT_SEARCH_RANGE_MS,
        }
    }

    pub fn use_proposal(&mut self) {
        if let (Some(delay_ms), Some(inverted)) = (self.proposal_delay_ms, self.proposal_inverted) {
            self.delay_ms = delay_ms;
            self.inverted = inverted;
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!((self.trace_a, self.trace_b), (Some(a), Some(b)) if a != b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forgetting_a_trace_clears_the_proposal() {
        let mut state = AlignmentState {
            trace_a: Some(1),
            trace_b: Some(2),
            proposal_delay_ms: Some(1.5),
            proposal_inverted: Some(false),
            ..AlignmentState::default()
        };
        assert!(state.is_ready());
        state.forget_trace(2);
        assert!(!state.is_ready());
        assert_eq!(state.proposal_delay_ms, None);
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
    fn same_trace_twice_is_not_ready() {
        let state = AlignmentState {
            trace_a: Some(1),
            trace_b: Some(1),
            ..AlignmentState::default()
        };
        assert!(!state.is_ready());
    }
}
