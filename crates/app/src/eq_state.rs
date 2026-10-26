use dsp_core::{
    AutoEqSettings, Complex64, CutSlope, EqFilter, FilterKind, MAX_EQ_FILTERS, amplitude_to_db,
    bandwidth_octaves_from_q, eq_response,
};
use serde::{Deserialize, Serialize};

use crate::text;

const DEFAULT_Q: f64 = 1.4;
const DEFAULT_CUT_Q: f64 = 0.707;
const BANK_COUNT: usize = 2;
const INVERSION_REFERENCE_BAND_HZ: (f64, f64) = (200.0, 5_000.0);
pub const MIN_Q: f64 = 0.1;
pub const MAX_Q: f64 = 20.0;
pub const MIN_FREQUENCY_HZ: f64 = 20.0;
pub const MAX_FREQUENCY_HZ: f64 = 20_000.0;
pub const GAIN_LIMIT_DB: f64 = 24.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EqHandle {
    pub index: usize,
    pub frequency_hz: f64,
    pub gain_db: f64,
    pub selected: bool,
    pub bypassed: bool,
}

pub fn predicted_db(target_db: &[Option<f64>], eq_db: &[Option<f64>]) -> Vec<Option<f64>> {
    target_db
        .iter()
        .zip(eq_db)
        .map(|(target, eq)| Some((*target)? + (*eq)?))
        .collect()
}

pub fn inverted_db(target_db: &[Option<f64>], frequencies_hz: &[f64]) -> Vec<Option<f64>> {
    let (low_hz, high_hz) = INVERSION_REFERENCE_BAND_HZ;
    let (sum, count) = target_db
        .iter()
        .zip(frequencies_hz)
        .filter(|(_, frequency_hz)| (low_hz..=high_hz).contains(*frequency_hz))
        .filter_map(|(value, _)| *value)
        .fold((0.0, 0_usize), |(sum, count), value| {
            (sum + value, count + 1)
        });
    let reference_db = if count == 0 {
        let (sum, count) = target_db
            .iter()
            .flatten()
            .fold((0.0, 0_usize), |(sum, count), value| {
                (sum + value, count + 1)
            });
        if count == 0 { 0.0 } else { sum / count as f64 }
    } else {
        sum / count as f64
    };
    target_db
        .iter()
        .map(|value| value.map(|level| reference_db - level))
        .collect()
}

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

pub const AUTO_EQ_MAX_BOOST_LIMIT_DB: f64 = 12.0;
pub const AUTO_EQ_MAX_CUT_LIMIT_DB: f64 = 24.0;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AutoEqParams {
    pub low_hz: f64,
    pub high_hz: f64,
    pub max_boost_db: f64,
    pub max_cut_db: f64,
    pub max_filters: usize,
}

impl Default for AutoEqParams {
    fn default() -> Self {
        Self {
            low_hz: 40.0,
            high_hz: 12_000.0,
            max_boost_db: 3.0,
            max_cut_db: 12.0,
            max_filters: 8,
        }
    }
}

impl AutoEqParams {
    pub fn to_dsp(self) -> AutoEqSettings {
        AutoEqSettings {
            low_hz: self.low_hz,
            high_hz: self.high_hz,
            max_boost_db: self.max_boost_db,
            max_cut_db: self.max_cut_db,
            max_filters: self.max_filters.min(MAX_EQ_FILTERS),
            target_db: None,
        }
    }
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
    pub auto: AutoEqParams,
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
            auto: AutoEqParams::default(),
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

    pub fn response_db(&self, frequencies_hz: &[f64], sample_rate_hz: f64) -> Vec<Option<f64>> {
        let mut response = vec![Complex64::new(1.0, 0.0); frequencies_hz.len()];
        if eq_response(
            &self.active_dsp_filters(),
            frequencies_hz,
            sample_rate_hz,
            &mut response,
        )
        .is_err()
        {
            return vec![None; frequencies_hz.len()];
        }
        response
            .iter()
            .map(|value| amplitude_to_db(value.norm()))
            .collect()
    }

    pub fn handles(&self) -> Vec<EqHandle> {
        self.filters()
            .iter()
            .enumerate()
            .map(|(index, filter)| EqHandle {
                index,
                frequency_hz: filter.frequency_hz,
                gain_db: filter.gain_db,
                selected: self.selected_filter == Some(index),
                bypassed: filter.bypassed || self.global_bypass,
            })
            .collect()
    }

    pub fn replace_with_auto(&mut self, filters: &[EqFilter]) {
        if let Some(bank) = self.filters_mut() {
            *bank = filters
                .iter()
                .take(MAX_EQ_FILTERS)
                .map(|filter| StoredFilter {
                    kind: StoredFilterKind::Bell,
                    frequency_hz: filter.frequency_hz,
                    gain_db: filter.gain_db,
                    q: filter.q,
                    bypassed: false,
                })
                .collect();
        }
        self.selected_filter = None;
        self.global_bypass = false;
    }

    pub fn move_filter(&mut self, index: usize, frequency_hz: f64, gain_db: f64) {
        if let Some(filter) = self
            .filters_mut()
            .and_then(|filters| filters.get_mut(index))
        {
            filter.frequency_hz = frequency_hz.clamp(MIN_FREQUENCY_HZ, MAX_FREQUENCY_HZ);
            if filter.kind.to_dsp().uses_gain() {
                filter.gain_db = gain_db.clamp(-GAIN_LIMIT_DB, GAIN_LIMIT_DB);
            }
            self.selected_filter = Some(index);
        }
    }

    pub fn keep_view_after_history(&mut self, view: EqView) {
        self.view = view;
        if self
            .selected_filter
            .is_some_and(|index| index >= self.filters().len())
        {
            self.selected_filter = None;
        }
    }

    pub fn scale_q(&mut self, index: usize, factor: f64) {
        if let Some(filter) = self
            .filters_mut()
            .and_then(|filters| filters.get_mut(index))
            && filter.kind.to_dsp().uses_q()
        {
            filter.q = (filter.q * factor).clamp(MIN_Q, MAX_Q);
            self.selected_filter = Some(index);
        }
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

    #[test]
    fn eq_cancels_a_matching_bell_within_a_tenth_of_a_db() {
        let sample_rate_hz = 48_000.0;
        let frequencies: Vec<f64> = dsp_core::DisplayGrid::new(20.0, 20_000.0, 96.0)
            .frequencies_hz()
            .to_vec();
        let room = dsp_core::BiquadCoefficients::bell(1000.0, 2.0, 6.0, sample_rate_hz);
        let target: Vec<Option<f64>> = frequencies
            .iter()
            .map(|frequency_hz| {
                amplitude_to_db(room.response_at(*frequency_hz, sample_rate_hz).norm())
            })
            .collect();
        let mut state = EqState::default();
        let mut correction = StoredFilter::new(StoredFilterKind::Bell, 1000.0, -6.0);
        correction.q = 2.0;
        state.add_filter(correction);
        let eq = state.response_db(&frequencies, sample_rate_hz);
        for value in predicted_db(&target, &eq) {
            assert!(value.unwrap().abs() <= 0.1);
        }
    }

    #[test]
    fn inverted_trace_mirrors_around_its_mid_band_level() {
        let frequencies = [100.0, 1000.0, 2000.0];
        let inverted = inverted_db(&[Some(-10.0), Some(-3.0), Some(-5.0)], &frequencies);
        assert_eq!(inverted, vec![Some(6.0), Some(-1.0), Some(1.0)]);
    }

    #[test]
    fn dragging_clamps_and_ignores_gain_for_cuts() {
        let mut state = EqState::default();
        state.add_filter(StoredFilter::new(
            StoredFilterKind::HighCut(StoredSlope::Butterworth12),
            8000.0,
            0.0,
        ));
        state.move_filter(0, 50_000.0, 12.0);
        assert_eq!(state.filters()[0].frequency_hz, MAX_FREQUENCY_HZ);
        assert_eq!(state.filters()[0].gain_db, 0.0);
        state.add_filter(StoredFilter::new(StoredFilterKind::Bell, 1000.0, 0.0));
        state.move_filter(1, 500.0, 40.0);
        assert_eq!(state.filters()[1].gain_db, GAIN_LIMIT_DB);
        state.scale_q(1, 100.0);
        assert_eq!(state.filters()[1].q, MAX_Q);
        state.selected_filter = None;
        state.move_filter(7, 100.0, 0.0);
        state.scale_q(7, 2.0);
        assert_eq!(state.selected_filter, None);
        state.selected_filter = Some(1);
        state.remove_filter(1);
        state.selected_filter = Some(5);
        state.keep_view_after_history(EqView::EqOnly);
        assert_eq!(state.selected_filter, None);
        assert_eq!(state.view, EqView::EqOnly);
        state.move_filter(0, 100.0, 0.0);
        assert!(state.handles()[0].selected);
    }
}
