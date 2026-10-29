use serde::{Deserialize, Serialize};

pub const DEFAULT_UPPER_PANE_FRACTION: f32 = 0.42;
const MIN_PANE_FRACTION: f32 = 0.15;
const MIN_PANE_HEIGHT: f32 = 140.0;

pub fn sanitized_upper_fraction(fraction: f32) -> f32 {
    if fraction.is_finite() {
        fraction.clamp(MIN_PANE_FRACTION, 1.0 - MIN_PANE_FRACTION)
    } else {
        DEFAULT_UPPER_PANE_FRACTION
    }
}

pub fn upper_pane_height(available_height: f32, fraction: f32) -> f32 {
    let available_height = available_height.max(0.0);
    let height = available_height * sanitized_upper_fraction(fraction);
    let min_height = MIN_PANE_HEIGHT.min(available_height / 2.0);
    height.clamp(min_height, available_height - min_height)
}

pub fn upper_fraction_at(available_height: f32, upper_height: f32) -> f32 {
    let height = upper_pane_height(available_height, upper_height / available_height.max(1.0));
    sanitized_upper_fraction(height / available_height.max(1.0))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewKind {
    #[default]
    Magnitude,
    Phase,
    Impulse,
    VirtualEq,
}

impl ViewKind {
    pub const ALL: [Self; 4] = [Self::Magnitude, Self::Phase, Self::Impulse, Self::VirtualEq];

    fn companion(self) -> Self {
        match self {
            Self::Magnitude => Self::Phase,
            Self::Phase | Self::Impulse | Self::VirtualEq => Self::Magnitude,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalModule {
    #[default]
    DelayAnalysis,
    VirtualEq,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseMode {
    Peek,
    Drag,
    #[default]
    Zoom,
}

impl MouseMode {
    pub const ALL: [Self; 3] = [Self::Peek, Self::Drag, Self::Zoom];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlotLayout {
    pub main: ViewKind,
    pub upper: Option<ViewKind>,
    pub impulse_in_db: bool,
    pub module: ExternalModule,
    pub mouse_mode: MouseMode,
    pub show_coherence: bool,
}

impl Default for PlotLayout {
    fn default() -> Self {
        Self {
            main: ViewKind::Magnitude,
            upper: Some(ViewKind::Phase),
            impulse_in_db: false,
            module: ExternalModule::default(),
            mouse_mode: MouseMode::default(),
            show_coherence: true,
        }
    }
}

impl PlotLayout {
    pub fn panes(&self) -> impl Iterator<Item = ViewKind> {
        self.upper.into_iter().chain(std::iter::once(self.main))
    }

    pub fn shows(&self, kind: ViewKind) -> bool {
        self.panes().any(|pane| pane == kind)
    }

    pub fn set_main(&mut self, kind: ViewKind) {
        if self.upper == Some(kind) {
            self.upper = Some(self.main);
        }
        self.main = kind;
    }

    pub fn set_upper(&mut self, kind: ViewKind) {
        if kind == self.main {
            self.main = self.upper.unwrap_or_else(|| kind.companion());
        }
        self.upper = Some(kind);
    }

    pub fn show_module(&mut self, module: ExternalModule) {
        self.module = module;
        if !self.shows(ViewKind::VirtualEq) {
            if self.main == ViewKind::Magnitude || self.upper.is_none() {
                self.set_upper(ViewKind::VirtualEq);
            } else {
                self.set_main(ViewKind::Magnitude);
                self.set_upper(ViewKind::VirtualEq);
            }
        }
    }

    pub fn toggle_stacked(&mut self) {
        self.upper = match self.upper {
            Some(_) => None,
            None => Some(self.main.companion()),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_shows_phase_above_magnitude_like_two_graphs() {
        let layout = PlotLayout::default();
        assert_eq!(
            layout.panes().collect::<Vec<_>>(),
            vec![ViewKind::Phase, ViewKind::Magnitude]
        );
        assert_eq!(layout.mouse_mode, MouseMode::Zoom);
        assert!(layout.show_coherence);
    }

    #[test]
    fn a_single_graph_can_be_shown_and_stacking_brings_the_companion_back() {
        let mut layout = PlotLayout::default();
        layout.toggle_stacked();
        assert_eq!(layout.upper, None);
        layout.toggle_stacked();
        assert_eq!(
            layout.panes().collect::<Vec<_>>(),
            vec![ViewKind::Phase, ViewKind::Magnitude]
        );
    }

    #[test]
    fn showing_a_module_puts_virtual_eq_above_the_magnitude() {
        let mut layout = PlotLayout {
            main: ViewKind::Phase,
            upper: Some(ViewKind::Impulse),
            ..PlotLayout::default()
        };
        layout.show_module(ExternalModule::VirtualEq);
        assert_eq!(layout.module, ExternalModule::VirtualEq);
        assert_eq!(layout.main, ViewKind::Magnitude);
        assert_eq!(layout.upper, Some(ViewKind::VirtualEq));
        layout.show_module(ExternalModule::DelayAnalysis);
        assert_eq!(layout.upper, Some(ViewKind::VirtualEq));
        let mut single = PlotLayout {
            upper: None,
            ..PlotLayout::default()
        };
        single.show_module(ExternalModule::DelayAnalysis);
        assert_eq!(single.upper, Some(ViewKind::VirtualEq));
    }

    #[test]
    fn choosing_the_view_already_shown_above_swaps_the_panes() {
        let mut layout = PlotLayout::default();
        layout.set_main(ViewKind::Phase);
        assert_eq!(layout.main, ViewKind::Phase);
        assert_eq!(layout.upper, Some(ViewKind::Magnitude));
        layout.set_upper(ViewKind::Phase);
        assert_eq!(layout.main, ViewKind::Magnitude);
        assert_eq!(layout.upper, Some(ViewKind::Phase));
    }

    #[test]
    fn panes_never_repeat_a_view() {
        let mut layout = PlotLayout::default();
        for main in ViewKind::ALL {
            for upper in ViewKind::ALL {
                layout.set_main(main);
                layout.set_upper(upper);
                assert_ne!(Some(layout.main), layout.upper);
                layout.toggle_stacked();
                layout.toggle_stacked();
                assert_ne!(Some(layout.main), layout.upper);
            }
        }
    }

    #[test]
    fn split_heights_keep_both_panes_usable() {
        assert!((upper_pane_height(1000.0, DEFAULT_UPPER_PANE_FRACTION) - 420.0).abs() < 1.0e-3);
        assert_eq!(upper_pane_height(1000.0, 0.01), 150.0);
        assert_eq!(upper_pane_height(1000.0, 0.99), 850.0);
        assert_eq!(upper_pane_height(400.0, 0.2), 140.0);
        assert_eq!(upper_pane_height(200.0, 0.9), 100.0);
        assert_eq!(upper_pane_height(1000.0, f32::NAN), 420.0);
        assert!((upper_fraction_at(1000.0, 600.0) - 0.6).abs() < 1.0e-6);
        assert!((upper_fraction_at(1000.0, 20.0) - 0.15).abs() < 1.0e-6);
        assert_eq!(
            sanitized_upper_fraction(f32::INFINITY),
            DEFAULT_UPPER_PANE_FRACTION
        );
    }

    #[test]
    fn layouts_without_new_fields_still_load() {
        let layout: PlotLayout = serde_json::from_str(r#"{"main":"Phase","upper":null}"#).unwrap();
        assert_eq!(layout.main, ViewKind::Phase);
        assert_eq!(layout.upper, None);
        assert!(layout.show_coherence);
        assert_eq!(layout.module, ExternalModule::DelayAnalysis);
    }
}
