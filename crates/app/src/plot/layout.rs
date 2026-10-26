use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewKind {
    #[default]
    Magnitude,
    Phase,
    Impulse,
}

impl ViewKind {
    pub const ALL: [Self; 3] = [Self::Magnitude, Self::Phase, Self::Impulse];

    fn companion(self) -> Self {
        match self {
            Self::Magnitude => Self::Phase,
            Self::Phase | Self::Impulse => Self::Magnitude,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlotLayout {
    pub main: ViewKind,
    pub upper: Option<ViewKind>,
    pub impulse_in_db: bool,
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
    fn the_default_is_a_single_magnitude_view() {
        let layout = PlotLayout::default();
        assert_eq!(
            layout.panes().collect::<Vec<_>>(),
            vec![ViewKind::Magnitude]
        );
    }

    #[test]
    fn stacking_adds_phase_above_magnitude() {
        let mut layout = PlotLayout::default();
        layout.toggle_stacked();
        assert_eq!(
            layout.panes().collect::<Vec<_>>(),
            vec![ViewKind::Phase, ViewKind::Magnitude]
        );
        layout.toggle_stacked();
        assert_eq!(layout.upper, None);
    }

    #[test]
    fn choosing_the_view_already_shown_above_swaps_the_panes() {
        let mut layout = PlotLayout::default();
        layout.toggle_stacked();
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
    fn layouts_without_new_fields_still_load() {
        let layout: PlotLayout = serde_json::from_str(r#"{"main":"Phase"}"#).unwrap();
        assert_eq!(layout.main, ViewKind::Phase);
        assert_eq!(layout.upper, None);
    }
}
