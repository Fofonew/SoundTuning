use eframe::egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    #[default]
    Night,
    Daylight,
}

impl Theme {
    pub fn toggled(self) -> Self {
        match self {
            Self::Night => Self::Daylight,
            Self::Daylight => Self::Night,
        }
    }

    pub fn palette(self) -> &'static Palette {
        match self {
            Self::Night => &NIGHT,
            Self::Daylight => &DAYLIGHT,
        }
    }

    pub fn is_dark(self) -> bool {
        matches!(self, Self::Night)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub background: Color32,
    pub surface: Color32,
    pub surface_raised: Color32,
    pub surface_hover: Color32,
    pub border: Color32,
    pub text_primary: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub warning: Color32,
    pub danger: Color32,
    pub success: Color32,
    pub plot_background: Color32,
    pub grid_major: Color32,
    pub grid_minor: Color32,
    pub trace_live: Color32,
    pub coherence_band: Color32,
    pub predicted: Color32,
    pub alignment_sums: [Color32; 4],
    pub stored_traces: [Color32; STORED_TRACE_COLOR_COUNT],
}

pub const STORED_TRACE_COLOR_COUNT: usize = 7;

const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

pub const NIGHT: Palette = Palette {
    background: hex(0x0B0E12),
    surface: hex(0x12161C),
    surface_raised: hex(0x1A2028),
    surface_hover: hex(0x232B35),
    border: hex(0x2A333E),
    text_primary: hex(0xE4E8EE),
    text_muted: hex(0x8E98A5),
    accent: hex(0x56B4E9),
    warning: hex(0xE69F00),
    danger: hex(0xD55E00),
    success: hex(0x009E73),
    plot_background: hex(0x090B0E),
    grid_major: hex(0x2B323C),
    grid_minor: hex(0x1A1F26),
    trace_live: hex(0xF2F4F7),
    coherence_band: hex(0x56B4E9),
    predicted: hex(0xFF9BE0),
    alignment_sums: [hex(0xFF9BE0), hex(0xB8E986), hex(0xB4A7FF), hex(0xFFB3A7)],
    stored_traces: [
        hex(0xE69F00),
        hex(0x56B4E9),
        hex(0x009E73),
        hex(0xF0E442),
        hex(0x0072B2),
        hex(0xD55E00),
        hex(0xCC79A7),
    ],
};

pub const DAYLIGHT: Palette = Palette {
    background: hex(0xFFFFFF),
    surface: hex(0xF3F5F8),
    surface_raised: hex(0xE7EBF0),
    surface_hover: hex(0xDCE1E8),
    border: hex(0x9AA3AE),
    text_primary: hex(0x0A0C0F),
    text_muted: hex(0x3E4752),
    accent: hex(0x0060A0),
    warning: hex(0x9A5F00),
    danger: hex(0xB23A00),
    success: hex(0x00704F),
    plot_background: hex(0xFFFFFF),
    grid_major: hex(0xB3BAC4),
    grid_minor: hex(0xE1E5EA),
    trace_live: hex(0x0A0C0F),
    coherence_band: hex(0x0060A0),
    predicted: hex(0xA3007D),
    alignment_sums: [hex(0xA3007D), hex(0x3F6E00), hex(0x5B3FBF), hex(0xB0281A)],
    stored_traces: [
        hex(0xE69F00),
        hex(0x56B4E9),
        hex(0x009E73),
        hex(0xB8A800),
        hex(0x0072B2),
        hex(0xD55E00),
        hex(0xCC79A7),
    ],
};

pub mod spacing {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 32.0;
}

pub mod radius {
    pub const SMALL: u8 = 4;
    pub const MEDIUM: u8 = 6;
    pub const LARGE: u8 = 10;
}

pub mod text_size {
    pub const CAPTION: f32 = 12.0;
    pub const LABEL: f32 = 13.0;
    pub const BODY: f32 = 14.0;
    pub const TITLE: f32 = 16.0;
    pub const HEADING: f32 = 20.0;
    pub const VALUE: f32 = 14.0;
    pub const VALUE_LARGE: f32 = 18.0;
    pub const DISPLAY: f32 = 30.0;
}

pub mod stroke {
    pub const GRID_MINOR: f32 = 0.5;
    pub const GRID_MAJOR: f32 = 1.0;
    pub const TRACE_STORED: f32 = 1.5;
    pub const TRACE_LIVE: f32 = 2.5;
    pub const TRACE_LIVE_HALO: f32 = 6.0;
    pub const TRACE_LIVE_HALO_ALPHA: f32 = 0.18;
    pub const CROSSHAIR: f32 = 1.0;
    pub const FOCUS: f32 = 2.0;
    pub const BORDER: f32 = 1.0;
}

pub mod size {
    pub const MIN_CLICK_HEIGHT: f32 = 28.0;
    pub const BUTTON_HEIGHT: f32 = 32.0;
    pub const TOP_BAR_HEIGHT: f32 = 52.0;
    pub const BOTTOM_BAR_HEIGHT: f32 = 48.0;
    pub const SIDE_PANEL_WIDTH: f32 = 272.0;
    pub const COHERENCE_BAND_HEIGHT: f32 = 64.0;
    pub const METER_WIDTH: f32 = 120.0;
    pub const METER_HEIGHT: f32 = 8.0;
    pub const MIN_ZOOM: f32 = 0.75;
    pub const MAX_ZOOM: f32 = 2.0;
    pub const ZOOM_STEP: f32 = 0.1;
}

pub mod motion {
    pub const STATE_CHANGE_SECONDS: f32 = 0.15;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn relative_luminance(color: Color32) -> f32 {
        let channel = |value: u8| {
            let normalized = f32::from(value) / 255.0;
            if normalized <= 0.039_28 {
                normalized / 12.92
            } else {
                ((normalized + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
    }

    fn contrast(first: Color32, second: Color32) -> f32 {
        let (light, dark) = {
            let a = relative_luminance(first);
            let b = relative_luminance(second);
            (a.max(b), a.min(b))
        };
        (light + 0.05) / (dark + 0.05)
    }

    #[test]
    fn text_meets_wcag_aa_on_every_surface() {
        for palette in [&NIGHT, &DAYLIGHT] {
            for surface in [palette.background, palette.surface, palette.surface_raised] {
                assert!(contrast(palette.text_primary, surface) >= 7.0);
                assert!(contrast(palette.text_muted, surface) >= 4.5);
            }
        }
    }

    #[test]
    fn daylight_is_high_contrast() {
        assert!(contrast(DAYLIGHT.text_primary, DAYLIGHT.background) >= 15.0);
        assert!(contrast(DAYLIGHT.trace_live, DAYLIGHT.plot_background) >= 15.0);
    }

    #[test]
    fn live_trace_stands_out_from_stored_traces() {
        for palette in [&NIGHT, &DAYLIGHT] {
            for stored in palette.stored_traces {
                assert_ne!(stored, palette.trace_live);
            }
            assert!(contrast(palette.trace_live, palette.plot_background) >= 12.0);
        }
    }

    #[test]
    fn theme_toggles() {
        assert_eq!(Theme::Night.toggled(), Theme::Daylight);
        assert!(Theme::default().is_dark());
    }
}
