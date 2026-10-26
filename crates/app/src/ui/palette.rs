use eframe::egui::{
    self, Align2, CornerRadius, Frame, Id, Key, Margin, Order, Role, Sense, Stroke, WidgetInfo,
};

use crate::actions::Action;
use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{radius, size, spacing, stroke, text_size};
use crate::design::widgets::{keycap_width, paint_keycap};
use crate::text;

const PALETTE_WIDTH: f32 = 520.0;
const PALETTE_TOP_OFFSET: f32 = 96.0;
const ROW_HEIGHT: f32 = 36.0;

#[derive(Debug, Default)]
pub struct CommandPalette {
    open: bool,
    query: String,
    selected: usize,
    focus_requested: bool,
}

pub fn matching_actions(query: &str) -> Vec<Action> {
    let normalized = normalize(query);
    Action::ALL
        .iter()
        .copied()
        .filter(|action| normalize(action.label()).contains(&normalized))
        .collect()
}

fn normalize(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .map(|character| match character {
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'à' | 'â' => 'a',
            'î' | 'ï' => 'i',
            'ô' => 'o',
            'û' | 'ù' => 'u',
            'ç' => 'c',
            other => other,
        })
        .collect()
}

impl CommandPalette {
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.selected = 0;
        self.focus_requested = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn toggle(&mut self) {
        if self.open {
            self.close();
        } else {
            self.open();
        }
    }

    pub fn show(
        &mut self,
        context: &egui::Context,
        palette: &Palette,
        is_mac: bool,
        actions: &mut Vec<Action>,
    ) {
        if !self.open {
            return;
        }
        let matches = matching_actions(&self.query);
        let (escape, enter, up, down) = context.input_mut(|input| {
            (
                input.consume_key(egui::Modifiers::NONE, Key::Escape),
                input.consume_key(egui::Modifiers::NONE, Key::Enter),
                input.consume_key(egui::Modifiers::NONE, Key::ArrowUp),
                input.consume_key(egui::Modifiers::NONE, Key::ArrowDown),
            )
        });
        if escape {
            self.close();
            return;
        }
        if down && !matches.is_empty() {
            self.selected = (self.selected + 1) % matches.len();
        }
        if up && !matches.is_empty() {
            self.selected = (self.selected + matches.len() - 1) % matches.len();
        }
        self.selected = self.selected.min(matches.len().saturating_sub(1));
        if enter {
            if let Some(action) = matches.get(self.selected) {
                actions.push(*action);
            }
            self.close();
            return;
        }

        let mut chosen = None;
        egui::Area::new(Id::unique("command-palette"))
            .order(Order::Foreground)
            .anchor(Align2::CENTER_TOP, [0.0, PALETTE_TOP_OFFSET])
            .show(context, |ui| {
                Frame::new()
                    .fill(palette.surface)
                    .stroke(Stroke::new(stroke::BORDER, palette.border))
                    .corner_radius(CornerRadius::same(radius::LARGE))
                    .shadow(ui.visuals().popup_shadow)
                    .inner_margin(Margin::same(spacing::MD as i8))
                    .show(ui, |ui| {
                        ui.set_width(PALETTE_WIDTH);
                        let edit = egui::TextEdit::singleline(&mut self.query)
                            .hint_text(text::PALETTE_PLACEHOLDER)
                            .font(fonts::ui(text_size::TITLE))
                            .desired_width(f32::INFINITY)
                            .margin(Margin::symmetric(spacing::SM as i8, spacing::SM as i8));
                        let response = ui.add(edit);
                        if self.focus_requested {
                            response.request_focus();
                            self.focus_requested = false;
                        }
                        if response.changed() {
                            self.selected = 0;
                        }
                        ui.add_space(spacing::SM);
                        if matches.is_empty() {
                            ui.label(
                                egui::RichText::new(text::PALETTE_NO_MATCH)
                                    .color(palette.text_muted),
                            );
                        }
                        for (index, action) in matches.iter().enumerate() {
                            if palette_row(ui, palette, *action, index == self.selected, is_mac)
                                .clicked()
                            {
                                chosen = Some(*action);
                            }
                        }
                    });
            });
        if let Some(action) = chosen {
            actions.push(action);
            self.close();
        }
    }
}

fn palette_row(
    ui: &mut egui::Ui,
    palette: &Palette,
    action: Action,
    selected: bool,
    is_mac: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), ROW_HEIGHT.max(size::MIN_CLICK_HEIGHT)),
        Sense::click(),
    );
    response.widget_info(|| WidgetInfo::labeled(Role::MenuItem, true, action.label()));
    let highlighted = selected || response.hovered();
    if highlighted {
        ui.painter().rect_filled(
            rect,
            CornerRadius::same(radius::MEDIUM),
            palette.accent.gamma_multiply(0.18),
        );
    }
    ui.painter().text(
        egui::pos2(rect.left() + spacing::MD, rect.center().y),
        Align2::LEFT_CENTER,
        action.label(),
        fonts::ui(text_size::BODY),
        palette.text_primary,
    );
    if let Some(shortcut) = action.shortcut_text(is_mac) {
        let width = keycap_width(ui, &shortcut);
        let keycap = egui::Rect::from_center_size(
            egui::pos2(rect.right() - spacing::MD - width / 2.0, rect.center().y),
            egui::vec2(width, 18.0),
        );
        paint_keycap(ui, palette, keycap, &shortcut, false);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_ignores_case_and_accents() {
        assert!(matching_actions("theme").contains(&Action::ToggleTheme));
        assert!(matching_actions("DÉLAI").contains(&Action::FindDelay));
        assert_eq!(matching_actions("").len(), Action::ALL.len());
        assert!(matching_actions("zzz").is_empty());
    }

    #[test]
    fn palette_opens_and_toggles() {
        let mut palette = CommandPalette::default();
        palette.toggle();
        assert!(palette.is_open());
        palette.toggle();
        assert!(!palette.is_open());
    }
}
