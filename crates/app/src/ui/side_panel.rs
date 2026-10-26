use eframe::egui::{
    self, Align2, CornerRadius, Rect, Role, Sense, Stroke, StrokeKind, WidgetInfo, vec2,
};
use serde::{Deserialize, Serialize};

use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{radius, size, spacing, stroke, text_size};
use crate::design::widgets::{muted, section_title};
use crate::text;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SideTab {
    #[default]
    Traces,
    Alignment,
    Eq,
}

impl SideTab {
    pub const ALL: [Self; 3] = [Self::Traces, Self::Alignment, Self::Eq];

    pub fn label(self) -> &'static str {
        match self {
            Self::Traces => text::TAB_TRACES,
            Self::Alignment => text::TAB_ALIGNMENT,
            Self::Eq => text::TAB_EQ,
        }
    }
}

pub fn tab_bar(ui: &mut egui::Ui, palette: &Palette, selected: &mut SideTab) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, size::BUTTON_HEIGHT), Sense::hover());
    ui.painter().rect(
        rect,
        CornerRadius::same(radius::MEDIUM),
        palette.surface_raised,
        Stroke::new(stroke::BORDER, palette.border),
        StrokeKind::Inside,
    );
    let tab_width = width / SideTab::ALL.len() as f32;
    for (index, tab) in SideTab::ALL.iter().enumerate() {
        let tab_rect = Rect::from_min_size(
            rect.left_top() + vec2(index as f32 * tab_width, 0.0),
            vec2(tab_width, rect.height()),
        )
        .shrink(2.0);
        let response = ui.interact(
            tab_rect,
            ui.scope_id().with(("side-tab", index)),
            Sense::click(),
        );
        response
            .widget_info(|| WidgetInfo::selected(Role::Tab, true, *selected == *tab, tab.label()));
        if response.clicked() {
            *selected = *tab;
        }
        let is_selected = *selected == *tab;
        if is_selected {
            ui.painter().rect_filled(
                tab_rect,
                CornerRadius::same(radius::SMALL),
                palette.surface_hover,
            );
        } else if response.hovered() {
            ui.painter().rect_filled(
                tab_rect,
                CornerRadius::same(radius::SMALL),
                palette.surface_hover.gamma_multiply(0.5),
            );
        }
        ui.painter().text(
            tab_rect.center(),
            Align2::CENTER_CENTER,
            tab.label(),
            if is_selected {
                fonts::ui_semibold(text_size::LABEL)
            } else {
                fonts::ui_medium(text_size::LABEL)
            },
            if is_selected {
                palette.text_primary
            } else {
                palette.text_muted
            },
        );
    }
}

pub fn empty_state(ui: &mut egui::Ui, palette: &Palette, title: &str, body: &str) {
    ui.add_space(spacing::XL);
    section_title(ui, palette, title);
    ui.add_space(spacing::XS);
    muted(ui, palette, body);
}
