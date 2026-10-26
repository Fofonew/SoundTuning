use dsp_core::{CutSlope, EqFilter, FilterKind, MAX_EQ_FILTERS, bandwidth_octaves_from_q};
use serde::{Deserialize, Serialize};

use crate::text;

const DEFAULT_Q: f64 = 1.4;
const DEFAULT_CUT_Q: f64 = 0.707;
const BANK_COUNT: usize = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoredSlope {
    #[default]
    Butterworth12,
    Butterworth24,
    LinkwitzRiley24,
    LinkwitzRiley48,
}

impl StoredSlope {
    pub const ALL: [Self; 4] = [
        Self::Butterworth12,
        Self::Butterworth24,
        Self::LinkwitzRiley24,
        Self::LinkwitzRiley48,
    ];

    fn to_dsp(self) -> CutSlope {
        match self {
            Self::Butterworth12 => CutSlope::Butterworth12,
            Self::Butterworth24 => CutSlope::Butterworth24,
            Self::LinkwitzRiley24 => CutSlope::LinkwitzRiley24,
            Self::LinkwitzRiley48 => CutSlope::LinkwitzRiley48,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Butterworth12 => text::SLOPE_BW12,
            Self::Butterworth24 => text::SLOPE_BW24,
            Self::LinkwitzRiley24 => text::SLOPE_LR24,
            Self::LinkwitzRiley48 => text::SLOPE_LR48,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoredFilterKind {
    #[default]
    Bell,
    LowShelf,
    HighShelf,
    LowCut(StoredSlope),
    HighCut(StoredSlope),
    Notch,
}

impl StoredFilterKind {
    pub const MENU: [Self; 6] = [
        Self::Bell,
        Self::LowShelf,
        Self::HighShelf,
        Self::LowCut(StoredSlope::LinkwitzRiley24),
        Self::HighCut(StoredSlope::LinkwitzRiley24),
        Self::Notch,
    ];

    pub fn to_dsp(self) -> FilterKind {
        match self {
            Self::Bell => FilterKind::Bell,
            Self::LowShelf => FilterKind::LowShelf,
            Self::HighShelf => FilterKind::HighShelf,
            Self::LowCut(slope) => FilterKind::LowCut(slope.to_dsp()),
            Self::HighCut(slope) => FilterKind::HighCut(slope.to_dsp()),
            Self::Notch => FilterKind::Notch,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Bell => text::FILTER_BELL,
            Self::LowShelf => text::FILTER_LOW_SHELF,
            Self::HighShelf => text::FILTER_HIGH_SHELF,
            Self::LowCut(_) => text::FILTER_LOW_CUT,
            Self::HighCut(_) => text::FILTER_HIGH_CUT,
            Self::Notch => text::FILTER_NOTCH,
        }
    }

    pub fn same_family(self, other: Self) -> bool {
        std::mem::discriminant(&self) == std::mem::discriminant(&other)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StoredFilter {
    pub kind: StoredFilterKind,
    pub frequency_hz: f64,
    pub gain_db: f64,
    pub q: f64,
    pub bypassed: bool,
}

impl StoredFilter {
    pub fn new(kind: StoredFilterKind, frequency_hz: f64, gain_db: f64) -> Self {
        let q = match kind {
            StoredFilterKind::LowCut(_) | StoredFilterKind::HighCut(_) => DEFAULT_CUT_Q,
            _ => DEFAULT_Q,
        };
        Self {
            kind,
            frequency_hz,
            gain_db: if kind.to_dsp().uses_gain() {
                gain_db
            } else {
                0.0
            },
            q,
            bypassed: false,
        }
    }

    pub fn to_dsp(self) -> EqFilter {
        EqFilter {
            kind: self.kind.to_dsp(),
            frequency_hz: self.frequency_hz,
            gain_db: self.gain_db,
            q: self.q,
            bypassed: self.bypassed,
        }
    }

    pub fn bandwidth_octaves(self) -> Option<f64> {
        self.kind
            .to_dsp()
            .uses_q()
            .then(|| bandwidth_octaves_from_q(self.q))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EqTarget {
    #[default]
    Live,
    Trace(u64),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EqView {
    #[default]
    Predicted,
    EqOnly,
    InvertedTrace,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EqState {
    pub banks: [Vec<StoredFilter>; BANK_COUNT],
    pub active_bank: usize,
    pub global_bypass: bool,
    pub target: EqTarget,
    pub view: EqView,
    pub selected_filter: Option<usize>,
}

impl Default for EqState {
    fn default() -> Self {
        Self {
            banks: [Vec::new(), Vec::new()],
            active_bank: 0,
            global_bypass: false,
            target: EqTarget::Live,
            view: EqView::Predicted,
            selected_filter: None,
        }
    }
}

impl EqState {
    pub fn filters(&self) -> &[StoredFilter] {
        self.banks
            .get(self.active_bank)
            .map_or(&[][..], Vec::as_slice)
    }

    pub fn filters_mut(&mut self) -> Option<&mut Vec<StoredFilter>> {
        self.banks.get_mut(self.active_bank)
    }

    pub fn add_filter(&mut self, filter: StoredFilter) -> Option<usize> {
        let filters = self.filters_mut()?;
        if filters.len() >= MAX_EQ_FILTERS {
            return None;
        }
        filters.push(filter);
        let index = filters.len() - 1;
        self.selected_filter = Some(index);
        Some(index)
    }

    pub fn remove_filter(&mut self, index: usize) {
        if let Some(filters) = self.filters_mut()
            && index < filters.len()
        {
            filters.remove(index);
        }
        self.selected_filter = None;
    }

    pub fn active_dsp_filters(&self) -> Vec<EqFilter> {
        if self.global_bypass {
            return Vec::new();
        }
        self.filters()
            .iter()
            .map(|filter| filter.to_dsp())
            .collect()
    }

    pub fn switch_bank(&mut self) {
        self.active_bank = (self.active_bank + 1) % BANK_COUNT;
        self.selected_filter = None;
    }

    pub fn forget_trace(&mut self, id: u64) {
        if self.target == EqTarget::Trace(id) {
            self.target = EqTarget::Live;
        }
    }

    pub fn export_text(&self) -> String {
        let mut lines = vec![text::eq_export_header(self.active_bank)];
        for (index, filter) in self.filters().iter().enumerate() {
            lines.push(text::eq_export_line(index + 1, filter));
        }
        lines.join("\n")
    }

    pub fn export_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self.filters())?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bank_holds_at_most_sixteen_filters() {
        let mut state = EqState::default();
        for _ in 0..MAX_EQ_FILTERS {
            assert!(
                state
                    .add_filter(StoredFilter::new(StoredFilterKind::Bell, 1000.0, -3.0))
                    .is_some()
            );
        }
        assert!(
            state
                .add_filter(StoredFilter::new(StoredFilterKind::Bell, 1000.0, -3.0))
                .is_none()
        );
    }

    #[test]
    fn banks_are_independent_and_bypass_empties_the_chain() {
        let mut state = EqState::default();
        state.add_filter(StoredFilter::new(StoredFilterKind::Bell, 1000.0, -3.0));
        state.switch_bank();
        assert!(state.filters().is_empty());
        state.switch_bank();
        assert_eq!(state.active_dsp_filters().len(), 1);
        state.global_bypass = true;
        assert!(state.active_dsp_filters().is_empty());
    }

    #[test]
    fn cut_filters_ignore_gain() {
        let filter = StoredFilter::new(
            StoredFilterKind::LowCut(StoredSlope::LinkwitzRiley24),
            80.0,
            -6.0,
        );
        assert_eq!(filter.gain_db, 0.0);
        assert_eq!(filter.bandwidth_octaves(), None);
    }

    #[test]
    fn export_lists_every_filter() {
        let mut state = EqState::default();
        state.add_filter(StoredFilter::new(StoredFilterKind::Bell, 1000.0, -3.0));
        state.add_filter(StoredFilter::new(StoredFilterKind::HighShelf, 8000.0, 2.0));
        assert_eq!(state.export_text().lines().count(), 3);
        let json: Vec<StoredFilter> = serde_json::from_str(&state.export_json().unwrap()).unwrap();
        assert_eq!(json.len(), 2);
    }
}
