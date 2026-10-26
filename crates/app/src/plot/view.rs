use eframe::egui::{Pos2, Rect, pos2};
use serde::{Deserialize, Serialize};

pub const FULL_LOW_HZ: f64 = 20.0;
pub const FULL_HIGH_HZ: f64 = 20_000.0;
pub const DEFAULT_LOW_DB: f64 = -24.0;
pub const DEFAULT_HIGH_DB: f64 = 18.0;
const MIN_SPAN_OCTAVES: f64 = 0.5;
const MIN_SPAN_DB: f64 = 6.0;
const MAX_SPAN_DB: f64 = 120.0;
const DB_LIMIT: f64 = 200.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RangePreset {
    Full,
    Sub,
    Mid,
    High,
}

impl RangePreset {
    pub fn frequency_range_hz(self) -> (f64, f64) {
        match self {
            Self::Full => (FULL_LOW_HZ, FULL_HIGH_HZ),
            Self::Sub => (20.0, 200.0),
            Self::Mid => (150.0, 4000.0),
            Self::High => (2000.0, 20_000.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlotView {
    pub low_hz: f64,
    pub high_hz: f64,
    pub low_db: f64,
    pub high_db: f64,
}

impl Default for PlotView {
    fn default() -> Self {
        Self {
            low_hz: FULL_LOW_HZ,
            high_hz: FULL_HIGH_HZ,
            low_db: DEFAULT_LOW_DB,
            high_db: DEFAULT_HIGH_DB,
        }
    }
}

impl PlotView {
    pub fn with_preset(self, preset: RangePreset) -> Self {
        let (low_hz, high_hz) = preset.frequency_range_hz();
        Self {
            low_hz,
            high_hz,
            ..self
        }
    }

    pub fn frequency_to_x(&self, rect: Rect, frequency_hz: f64) -> f32 {
        let fraction = (frequency_hz / self.low_hz).log2() / (self.high_hz / self.low_hz).log2();
        rect.left() + (fraction as f32) * rect.width()
    }

    pub fn x_to_frequency(&self, rect: Rect, x: f32) -> f64 {
        let fraction = f64::from((x - rect.left()) / rect.width().max(1.0));
        self.low_hz * ((self.high_hz / self.low_hz).log2() * fraction).exp2()
    }

    pub fn db_to_y(&self, rect: Rect, level_db: f64) -> f32 {
        let fraction = (level_db - self.low_db) / (self.high_db - self.low_db);
        rect.bottom() - (fraction as f32) * rect.height()
    }

    pub fn y_to_db(&self, rect: Rect, y: f32) -> f64 {
        let fraction = f64::from((rect.bottom() - y) / rect.height().max(1.0));
        self.low_db + fraction * (self.high_db - self.low_db)
    }

    pub fn to_screen(&self, rect: Rect, frequency_hz: f64, level_db: f64) -> Pos2 {
        pos2(
            self.frequency_to_x(rect, frequency_hz),
            self.db_to_y(rect, level_db),
        )
    }

    pub fn zoom_frequency(&mut self, anchor_hz: f64, factor: f64) {
        if !(factor.is_finite() && factor > 0.0 && anchor_hz.is_finite() && anchor_hz > 0.0) {
            return;
        }
        let low_octaves = (self.low_hz / anchor_hz).log2() * factor;
        let high_octaves = (self.high_hz / anchor_hz).log2() * factor;
        if high_octaves - low_octaves < MIN_SPAN_OCTAVES {
            return;
        }
        self.low_hz = anchor_hz * low_octaves.exp2();
        self.high_hz = anchor_hz * high_octaves.exp2();
        self.clamp_frequency();
    }

    pub fn zoom_level(&mut self, anchor_db: f64, factor: f64) {
        if !(factor.is_finite() && factor > 0.0 && anchor_db.is_finite()) {
            return;
        }
        let low = anchor_db + (self.low_db - anchor_db) * factor;
        let high = anchor_db + (self.high_db - anchor_db) * factor;
        if (MIN_SPAN_DB..=MAX_SPAN_DB).contains(&(high - low)) {
            self.low_db = low.max(-DB_LIMIT);
            self.high_db = high.min(DB_LIMIT);
        }
    }

    pub fn pan(&mut self, rect: Rect, delta_x: f32, delta_y: f32) {
        let octaves_per_pixel =
            (self.high_hz / self.low_hz).log2() / f64::from(rect.width().max(1.0));
        let shift_octaves = -f64::from(delta_x) * octaves_per_pixel;
        let low_hz = self.low_hz * shift_octaves.exp2();
        let high_hz = self.high_hz * shift_octaves.exp2();
        if low_hz >= FULL_LOW_HZ && high_hz <= FULL_HIGH_HZ {
            self.low_hz = low_hz;
            self.high_hz = high_hz;
        }
        let db_per_pixel = (self.high_db - self.low_db) / f64::from(rect.height().max(1.0));
        let shift_db = f64::from(delta_y) * db_per_pixel;
        if (self.low_db + shift_db) > -DB_LIMIT && (self.high_db + shift_db) < DB_LIMIT {
            self.low_db += shift_db;
            self.high_db += shift_db;
        }
    }

    fn clamp_frequency(&mut self) {
        let span = (self.high_hz / self.low_hz).log2();
        let full_span = (FULL_HIGH_HZ / FULL_LOW_HZ).log2();
        if span >= full_span {
            self.low_hz = FULL_LOW_HZ;
            self.high_hz = FULL_HIGH_HZ;
            return;
        }
        if self.low_hz < FULL_LOW_HZ {
            self.low_hz = FULL_LOW_HZ;
            self.high_hz = FULL_LOW_HZ * span.exp2();
        }
        if self.high_hz > FULL_HIGH_HZ {
            self.high_hz = FULL_HIGH_HZ;
            self.low_hz = FULL_HIGH_HZ / span.exp2();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect() -> Rect {
        Rect::from_min_size(pos2(0.0, 0.0), eframe::egui::vec2(1000.0, 500.0))
    }

    #[test]
    fn log_axis_places_decades_evenly() {
        let view = PlotView::default();
        let x200 = view.frequency_to_x(rect(), 200.0);
        let x2000 = view.frequency_to_x(rect(), 2000.0);
        let x20 = view.frequency_to_x(rect(), 20.0);
        assert!((x200 - x20 - (x2000 - x200)).abs() < 1.0e-3);
        assert!(x20.abs() < 1.0e-3);
        assert!((view.frequency_to_x(rect(), 20_000.0) - 1000.0).abs() < 1.0e-3);
    }

    #[test]
    fn screen_and_value_conversions_round_trip() {
        let view = PlotView::default();
        let x = view.frequency_to_x(rect(), 1234.0);
        assert!((view.x_to_frequency(rect(), x) - 1234.0).abs() < 0.01);
        let y = view.db_to_y(rect(), -3.5);
        assert!((view.y_to_db(rect(), y) + 3.5).abs() < 1.0e-4);
    }

    #[test]
    fn zoom_keeps_anchor_in_place_and_respects_limits() {
        let mut view = PlotView::default();
        let anchor_x = view.frequency_to_x(rect(), 1000.0);
        view.zoom_frequency(1000.0, 0.5);
        assert!((view.frequency_to_x(rect(), 1000.0) - anchor_x).abs() < 1.0);
        view.zoom_frequency(1000.0, 1000.0);
        assert_eq!((view.low_hz, view.high_hz), (FULL_LOW_HZ, FULL_HIGH_HZ));
        view.zoom_frequency(1000.0, 1.0e-6);
        assert!((view.high_hz / view.low_hz).log2() >= MIN_SPAN_OCTAVES);
        view.zoom_frequency(f64::NAN, 0.5);
    }

    #[test]
    fn presets_select_expected_bands() {
        let view = PlotView::default().with_preset(RangePreset::Sub);
        assert_eq!((view.low_hz, view.high_hz), (20.0, 200.0));
    }

    #[test]
    fn panning_never_leaves_the_audio_band() {
        let mut view = PlotView::default().with_preset(RangePreset::Sub);
        view.pan(rect(), 10_000.0, 0.0);
        assert!(view.low_hz >= FULL_LOW_HZ);
    }
}
