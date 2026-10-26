use eframe::egui::{self, Key, KeyboardShortcut, Modifiers};

use crate::text;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    ToggleMeasurement,
    FindDelay,
    ApplyDelay,
    CaptureTrace,
    ToggleFreeze,
    RangeFull,
    RangeSub,
    RangeMid,
    RangeHigh,
    ResetView,
    ToggleTheme,
    OpenPalette,
    Undo,
    Redo,
    ToggleSidePanel,
    ToggleGenerator,
    CalibrateLatency,
    AddEqFilter,
    ExportSession,
    RestartMeasurement,
    ToggleDelayTracking,
    NudgeDelayDown,
    NudgeDelayUp,
    OpenAudioSettings,
    ViewMagnitude,
    ViewPhase,
    ViewImpulse,
    ToggleStackedViews,
    ToggleFullscreen,
}

impl Action {
    pub const ALL: [Self; 29] = [
        Self::ToggleMeasurement,
        Self::FindDelay,
        Self::ApplyDelay,
        Self::CaptureTrace,
        Self::ToggleFreeze,
        Self::RangeFull,
        Self::RangeSub,
        Self::RangeMid,
        Self::RangeHigh,
        Self::ResetView,
        Self::ToggleTheme,
        Self::OpenPalette,
        Self::Undo,
        Self::Redo,
        Self::ToggleSidePanel,
        Self::ToggleGenerator,
        Self::CalibrateLatency,
        Self::AddEqFilter,
        Self::ExportSession,
        Self::RestartMeasurement,
        Self::ToggleDelayTracking,
        Self::NudgeDelayDown,
        Self::NudgeDelayUp,
        Self::OpenAudioSettings,
        Self::ViewMagnitude,
        Self::ViewPhase,
        Self::ViewImpulse,
        Self::ToggleStackedViews,
        Self::ToggleFullscreen,
    ];

    const SHORTCUT_PRIORITY: [Self; 25] = [
        Self::Redo,
        Self::Undo,
        Self::OpenPalette,
        Self::ToggleMeasurement,
        Self::FindDelay,
        Self::ApplyDelay,
        Self::CaptureTrace,
        Self::ToggleFreeze,
        Self::RangeFull,
        Self::RangeSub,
        Self::RangeMid,
        Self::RangeHigh,
        Self::ToggleTheme,
        Self::ToggleGenerator,
        Self::AddEqFilter,
        Self::ToggleSidePanel,
        Self::ToggleDelayTracking,
        Self::NudgeDelayDown,
        Self::NudgeDelayUp,
        Self::OpenAudioSettings,
        Self::ToggleFullscreen,
        Self::ViewMagnitude,
        Self::ViewPhase,
        Self::ViewImpulse,
        Self::ToggleStackedViews,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::ToggleMeasurement => text::ACTION_TOGGLE_MEASUREMENT,
            Self::FindDelay => text::ACTION_FIND_DELAY,
            Self::ApplyDelay => text::ACTION_APPLY_DELAY,
            Self::CaptureTrace => text::ACTION_CAPTURE_TRACE,
            Self::ToggleFreeze => text::ACTION_TOGGLE_FREEZE,
            Self::RangeFull => text::ACTION_RANGE_FULL,
            Self::RangeSub => text::ACTION_RANGE_SUB,
            Self::RangeMid => text::ACTION_RANGE_MID,
            Self::RangeHigh => text::ACTION_RANGE_HIGH,
            Self::ResetView => text::ACTION_RESET_VIEW,
            Self::ToggleTheme => text::ACTION_TOGGLE_THEME,
            Self::OpenPalette => text::ACTION_OPEN_PALETTE,
            Self::Undo => text::ACTION_UNDO,
            Self::Redo => text::ACTION_REDO,
            Self::ToggleSidePanel => text::ACTION_TOGGLE_SIDE_PANEL,
            Self::ToggleGenerator => text::ACTION_TOGGLE_GENERATOR,
            Self::CalibrateLatency => text::ACTION_CALIBRATE_LATENCY,
            Self::AddEqFilter => text::ACTION_ADD_EQ_FILTER,
            Self::ExportSession => text::ACTION_EXPORT_SESSION,
            Self::RestartMeasurement => text::ACTION_RESTART_MEASUREMENT,
            Self::ToggleDelayTracking => text::ACTION_TOGGLE_DELAY_TRACKING,
            Self::NudgeDelayDown => text::ACTION_NUDGE_DELAY_DOWN,
            Self::NudgeDelayUp => text::ACTION_NUDGE_DELAY_UP,
            Self::OpenAudioSettings => text::ACTION_OPEN_AUDIO_SETTINGS,
            Self::ViewMagnitude => text::ACTION_VIEW_MAGNITUDE,
            Self::ViewPhase => text::ACTION_VIEW_PHASE,
            Self::ViewImpulse => text::ACTION_VIEW_IMPULSE,
            Self::ToggleStackedViews => text::ACTION_TOGGLE_STACKED_VIEWS,
            Self::ToggleFullscreen => text::ACTION_TOGGLE_FULLSCREEN,
        }
    }

    pub fn shortcut(self) -> Option<KeyboardShortcut> {
        let plain = |key| Some(KeyboardShortcut::new(Modifiers::NONE, key));
        match self {
            Self::ToggleMeasurement => plain(Key::Space),
            Self::FindDelay => plain(Key::D),
            Self::ApplyDelay => plain(Key::A),
            Self::CaptureTrace => plain(Key::S),
            Self::ToggleFreeze => plain(Key::F),
            Self::RangeFull => plain(Key::Num1),
            Self::RangeSub => plain(Key::Num2),
            Self::RangeMid => plain(Key::Num3),
            Self::RangeHigh => plain(Key::Num4),
            Self::ToggleTheme => plain(Key::L),
            Self::ToggleGenerator => plain(Key::G),
            Self::AddEqFilter => plain(Key::E),
            Self::ToggleSidePanel => plain(Key::P),
            Self::ToggleDelayTracking => plain(Key::T),
            Self::ViewMagnitude => plain(Key::M),
            Self::ViewPhase => plain(Key::H),
            Self::ViewImpulse => plain(Key::I),
            Self::ToggleStackedViews => plain(Key::V),
            Self::ToggleFullscreen => Some(KeyboardShortcut::new(Modifiers::SHIFT, Key::F)),
            Self::NudgeDelayDown => Some(KeyboardShortcut::new(Modifiers::SHIFT, Key::ArrowLeft)),
            Self::NudgeDelayUp => Some(KeyboardShortcut::new(Modifiers::SHIFT, Key::ArrowRight)),
            Self::OpenPalette => Some(KeyboardShortcut::new(Modifiers::COMMAND, Key::K)),
            Self::OpenAudioSettings => Some(KeyboardShortcut::new(Modifiers::COMMAND, Key::Comma)),
            Self::Undo => Some(KeyboardShortcut::new(Modifiers::COMMAND, Key::Z)),
            Self::Redo => Some(KeyboardShortcut::new(
                Modifiers::COMMAND | Modifiers::SHIFT,
                Key::Z,
            )),
            Self::ResetView
            | Self::CalibrateLatency
            | Self::ExportSession
            | Self::RestartMeasurement => None,
        }
    }

    pub fn shortcut_text(self, is_mac: bool) -> Option<String> {
        self.shortcut()
            .map(|shortcut| format_shortcut(&shortcut, is_mac))
    }
}

pub fn format_shortcut(shortcut: &KeyboardShortcut, is_mac: bool) -> String {
    let key = match shortcut.logical_key {
        Key::Space => text::KEY_SPACE.to_owned(),
        Key::ArrowLeft => text::KEY_ARROW_LEFT.to_owned(),
        Key::ArrowRight => text::KEY_ARROW_RIGHT.to_owned(),
        other => other.symbol_or_name().to_uppercase(),
    };
    let modifiers = shortcut.modifiers;
    if is_mac {
        let mut text = String::new();
        if modifiers.shift {
            text.push('⇧');
        }
        if modifiers.command || modifiers.mac_cmd {
            text.push('⌘');
        }
        text.push_str(&key);
        text
    } else {
        let mut parts = Vec::new();
        if modifiers.command || modifiers.ctrl {
            parts.push(text::KEY_CTRL.to_owned());
        }
        if modifiers.shift {
            parts.push(text::KEY_SHIFT.to_owned());
        }
        parts.push(key);
        parts.join("+")
    }
}

fn shortcut_matches(shortcut: &KeyboardShortcut, key: Key, modifiers: Modifiers) -> bool {
    let wanted = shortcut.modifiers;
    let command_pressed = modifiers.command || modifiers.mac_cmd || modifiers.ctrl;
    key == shortcut.logical_key
        && !modifiers.alt
        && wanted.shift == modifiers.shift
        && wanted.command == command_pressed
}

pub fn triggered_by_keyboard(context: &egui::Context) -> Vec<Action> {
    if context.egui_wants_keyboard_input() {
        return Vec::new();
    }
    let widget_has_focus = context.memory(|memory| memory.focused()).is_some();
    context.input_mut(|input| {
        let mut triggered = Vec::new();
        input.events.retain(|event| {
            let egui::Event::Key {
                key,
                pressed: true,
                repeat: false,
                modifiers,
                ..
            } = event
            else {
                return true;
            };
            let matched = Action::SHORTCUT_PRIORITY.iter().copied().find(|action| {
                action.shortcut().is_some_and(|shortcut| {
                    let blocked_by_focus = widget_has_focus && shortcut.modifiers.is_none();
                    !blocked_by_focus && shortcut_matches(&shortcut, *key, *modifiers)
                })
            });
            match matched {
                Some(action) => {
                    triggered.push(action);
                    false
                }
                None => true,
            }
        });
        triggered
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_action_has_a_label_and_the_spec_shortcuts_exist() {
        for action in Action::ALL {
            assert!(!action.label().is_empty());
        }
        for (action, expected) in [
            (Action::ToggleMeasurement, "Espace"),
            (Action::FindDelay, "D"),
            (Action::ApplyDelay, "A"),
            (Action::CaptureTrace, "S"),
            (Action::ToggleFreeze, "F"),
            (Action::RangeFull, "1"),
            (Action::RangeHigh, "4"),
            (Action::ToggleTheme, "L"),
            (Action::OpenPalette, "⌘K"),
            (Action::Undo, "⌘Z"),
            (Action::Redo, "⇧⌘Z"),
        ] {
            assert_eq!(action.shortcut_text(true).as_deref(), Some(expected));
        }
        assert_eq!(
            Action::OpenPalette.shortcut_text(false).as_deref(),
            Some("Ctrl+K")
        );
        assert_eq!(
            Action::Redo.shortcut_text(false).as_deref(),
            Some("Ctrl+Maj+Z")
        );
    }

    #[test]
    fn modifier_matching_distinguishes_undo_from_redo() {
        let undo = Action::Undo.shortcut().unwrap();
        let redo = Action::Redo.shortcut().unwrap();
        let command = Modifiers::COMMAND;
        let command_shift = Modifiers::COMMAND | Modifiers::SHIFT;
        assert!(shortcut_matches(&undo, Key::Z, command));
        assert!(!shortcut_matches(&undo, Key::Z, command_shift));
        assert!(shortcut_matches(&redo, Key::Z, command_shift));
        let space = Action::ToggleMeasurement.shortcut().unwrap();
        assert!(shortcut_matches(&space, Key::Space, Modifiers::NONE));
        assert!(!shortcut_matches(&space, Key::Space, command));
    }

    #[test]
    fn shortcuts_are_unique() {
        let shortcuts: Vec<_> = Action::ALL
            .iter()
            .filter_map(|action| action.shortcut())
            .collect();
        for (index, shortcut) in shortcuts.iter().enumerate() {
            assert!(!shortcuts[index + 1..].contains(shortcut));
        }
        assert_eq!(shortcuts.len(), Action::SHORTCUT_PRIORITY.len());
    }
}
