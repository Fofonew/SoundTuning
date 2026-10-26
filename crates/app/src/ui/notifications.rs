use std::time::{Duration, Instant};

use eframe::egui::{self, Align2, CornerRadius, Frame, Id, Margin, Order, Stroke};

use crate::actions::Action;
use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{motion, radius, size, spacing, stroke, text_size};
use crate::design::widgets::{Emphasis, Tone, action_button};

const NOTIFICATION_LIFETIME: Duration = Duration::from_secs(6);
const MAX_VISIBLE_NOTIFICATIONS: usize = 3;
const NOTIFICATION_WIDTH: f32 = 360.0;
const ACCENT_BAR_WIDTH: f32 = 3.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Notification {
    id: u64,
    text: String,
    tone: Tone,
    action: Option<Action>,
    created: Instant,
}

#[derive(Debug, Default)]
pub struct Notifications {
    items: Vec<Notification>,
    next_id: u64,
}

impl Notifications {
    pub fn push(&mut self, text: impl Into<String>, tone: Tone, action: Option<Action>) {
        let text = text.into();
        self.items.retain(|item| item.text != text);
        self.next_id += 1;
        self.items.push(Notification {
            id: self.next_id,
            text,
            tone,
            action,
            created: Instant::now(),
        });
        if self.items.len() > MAX_VISIBLE_NOTIFICATIONS {
            self.items.remove(0);
        }
    }

    pub fn texts(&self) -> impl Iterator<Item = &str> {
        self.items.iter().map(|item| item.text.as_str())
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn show(
        &mut self,
        context: &egui::Context,
        palette: &Palette,
        is_mac: bool,
        actions: &mut Vec<Action>,
    ) {
        let now = Instant::now();
        self.items
            .retain(|item| now.duration_since(item.created) < NOTIFICATION_LIFETIME);
        let mut dismissed = Vec::new();
        let mut hovered = Vec::new();
        let bottom_offset = size::BOTTOM_BAR_HEIGHT + spacing::LG;
        egui::Area::new(Id::unique("notifications"))
            .order(Order::Foreground)
            .anchor(Align2::RIGHT_BOTTOM, [-spacing::LG, -bottom_offset])
            .show(context, |ui| {
                ui.set_width(NOTIFICATION_WIDTH);
                for item in self.items.iter().rev() {
                    let opacity = context.animate_bool_with_time(
                        Id::unique(("notification", item.id)),
                        true,
                        motion::STATE_CHANGE_SECONDS,
                    );
                    ui.set_opacity(opacity);
                    let frame = Frame::new()
                        .fill(palette.surface_raised)
                        .stroke(Stroke::new(stroke::BORDER, palette.border))
                        .corner_radius(CornerRadius::same(radius::LARGE))
                        .inner_margin(Margin::symmetric(spacing::MD as i8, spacing::SM as i8));
                    let response = frame.show(ui, |ui| {
                        ui.set_width(NOTIFICATION_WIDTH - 2.0 * spacing::MD);
                        ui.horizontal(|ui| {
                            let (bar, _) = ui.allocate_exact_size(
                                egui::vec2(ACCENT_BAR_WIDTH, size::BUTTON_HEIGHT),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(
                                bar,
                                CornerRadius::same(1),
                                item.tone.color(palette),
                            );
                            ui.label(
                                egui::RichText::new(&item.text)
                                    .font(fonts::ui_medium(text_size::BODY))
                                    .color(palette.text_primary),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if let Some(action) = item.action {
                                        let shortcut = action.shortcut_text(is_mac);
                                        if action_button(
                                            ui,
                                            palette,
                                            action_label(action),
                                            shortcut.as_deref(),
                                            Emphasis::Primary,
                                            true,
                                        )
                                        .clicked()
                                        {
                                            actions.push(action);
                                            dismissed.push(item.id);
                                        }
                                    }
                                },
                            );
                        });
                    });
                    let hit = ui.interact(
                        response.response.rect,
                        Id::unique(("notification-hit", item.id)),
                        egui::Sense::click(),
                    );
                    if hit.clicked() {
                        dismissed.push(item.id);
                    }
                    if hit.hovered() {
                        hovered.push(item.id);
                    }
                    ui.add_space(spacing::SM);
                }
            });
        self.items.retain(|item| !dismissed.contains(&item.id));
        for item in &mut self.items {
            if hovered.contains(&item.id) {
                item.created = now;
            }
        }
    }
}

fn action_label(action: Action) -> &'static str {
    match action {
        Action::ApplyDelay => crate::text::BUTTON_APPLY_DELAY,
        Action::Undo => crate::text::ACTION_UNDO,
        Action::CalibrateLatency => crate::text::BUTTON_CALIBRATE,
        other => other.label(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicates_are_replaced_and_the_stack_is_bounded() {
        let mut notifications = Notifications::default();
        for index in 0..5 {
            notifications.push(format!("message {index}"), Tone::Neutral, None);
        }
        notifications.push("message 4", Tone::Success, Some(Action::ApplyDelay));
        let texts: Vec<_> = notifications.texts().collect();
        assert_eq!(texts, vec!["message 2", "message 3", "message 4"]);
    }
}
