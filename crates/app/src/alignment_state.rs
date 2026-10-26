use dsp_core::{AlignmentCriterion, AlignmentSearch, DEFAULT_ALIGNMENT_BAND_HZ, Polarity};
use serde::{Deserialize, Serialize};

const DEFAULT_SEARCH_RANGE_MS: f64 = 50.0;

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
    fn same_trace_twice_is_not_ready() {
        let state = AlignmentState {
            trace_a: Some(1),
            trace_b: Some(1),
            ..AlignmentState::default()
        };
        assert!(!state.is_ready());
    }
}
