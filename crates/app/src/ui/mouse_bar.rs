use eframe::egui::{self, Align, Align2, Layout, Rect, Stroke, UiBuilder, pos2, vec2};

use crate::actions::Action;
use crate::design::Palette;
use crate::design::fonts;
use crate::design::icons::{self, icon_button};
use crate::design::tokens::{size, spacing, stroke, text_size};
use crate::design::widgets::toggle_button;
use crate::plot::MouseMode;
use crate::text;
use crate::ui::graph_bar::ViewCommand;
use crate::ui::pane_splitter::{self, SplitArea};

const MODE_BUTTON_WIDTH: f32 = 52.0;
const GRIP_INSET: f32 = 22.0;

pub struct MouseBarModel<'a> {
    pub readout: Option<&'a str>,
    pub mode: MouseMode,
    pub stacked: bool,
    pub fullscreen: bool,
}

#[derive(Debug, Default, PartialEq)]
pub struct MouseBarEvents {
    pub commands: Vec<ViewCommand>,
    pub upper_fraction: Option<f32>,
}

fn mode_label(mode: MouseMode) -> (&'static str, &'static str) {
    match mode {
        MouseMode::Peek => (text::MOUSE_PEEK, text::MOUSE_PEEK_HINT),
        MouseMode::Drag => (text::MOUSE_DRAG, text::MOUSE_DRAG_HINT),
        MouseMode::Zoom => (text::MOUSE_ZOOM, text::MOUSE_ZOOM_HINT),
    }
}

pub fn show(
    ui: &mut egui::Ui,
    palette: &Palette,
    rect: Rect,
    model: &MouseBarModel<'_>,
    split: Option<SplitArea>,
    is_mac: bool,
) -> MouseBarEvents {
    let mut events = MouseBarEvents::default();
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::ZERO, palette.surface);
    for y in [rect.top(), rect.bottom()] {
        ui.painter().line_segment(
            [pos2(rect.left(), y), pos2(rect.right(), y)],
            Stroke::new(stroke::BORDER, palette.border),
        );
    }
    if let Some(area) = split {
        let grip = pos2(rect.left() + GRIP_INSET, rect.center().y);
        events.upper_fraction = pane_splitter::show(ui, palette, rect, grip, area);
    }
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        model.readout.unwrap_or(text::MOUSE_BAR_IDLE),
        fonts::value_semibold(text_size::VALUE_LARGE),
        if model.readout.is_some() {
            palette.text_primary
        } else {
            palette.text_muted
        },
    );
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(rect.shrink2(vec2(spacing::SM, 0.0)))
            .layout(Layout::right_to_left(Align::Center)),
    );
    child.spacing_mut().item_spacing.x = size::SEGMENT_GAP;
    let fullscreen_label = if model.fullscreen {
        text::VIEW_EXIT_FULLSCREEN
    } else {
        text::VIEW_FULLSCREEN
    };
    let fullscreen = icon_button(
        &mut child,
        palette,
        if model.fullscreen {
            icons::EXIT_FULLSCREEN
        } else {
            icons::FULLSCREEN
        },
        fullscreen_label,
        &text::with_shortcut(
            fullscreen_label,
            Action::ToggleFullscreen.shortcut_text(is_mac).as_deref(),
        ),
        model.fullscreen,
    );
    if fullscreen.clicked() {
        events.commands.push(ViewCommand::ToggleFullscreen);
    }
    let stacked = icon_button(
        &mut child,
        palette,
        icons::STACK,
        text::VIEW_STACKED,
        &text::with_shortcut(
            text::ACTION_TOGGLE_STACKED_VIEWS,
            Action::ToggleStackedViews.shortcut_text(is_mac).as_deref(),
        ),
        model.stacked,
    );
    if stacked.clicked() {
        events.commands.push(ViewCommand::ToggleStacked);
    }
    child.add_space(spacing::MD);
    for mode in MouseMode::ALL.into_iter().rev() {
        let (label, hint) = mode_label(mode);
        let response = toggle_button(
            &mut child,
            palette,
            label,
            model.mode == mode,
            Some(MODE_BUTTON_WIDTH),
        )
        .on_hover_text(hint);
        if response.clicked() && model.mode != mode {
            events.commands.push(ViewCommand::MouseMode(mode));
        }
    }
    events
}
