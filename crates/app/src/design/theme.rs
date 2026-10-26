use eframe::egui::{
    self, Color32, CornerRadius, Margin, Shadow, Stroke, TextStyle, Visuals, style::WidgetVisuals,
};

use crate::design::fonts;
use crate::design::tokens::{Palette, Theme, radius, size, spacing, stroke, text_size};

const SELECTION_ALPHA: u8 = 64;
const SHADOW_ALPHA_NIGHT: u8 = 160;
const SHADOW_ALPHA_DAY: u8 = 48;
const SHADOW_BLUR: u8 = 16;
const SHADOW_SPREAD: u8 = 0;
const SHADOW_OFFSET_Y: i8 = 4;

pub fn apply(context: &egui::Context, theme: Theme) {
    let palette = theme.palette();
    context.set_theme(if theme.is_dark() {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    });
    context.all_styles_mut(|style| {
        style.visuals = visuals(palette, theme);
        style.text_styles = [
            (TextStyle::Small, fonts::caption()),
            (TextStyle::Body, fonts::body()),
            (TextStyle::Button, fonts::label()),
            (TextStyle::Heading, fonts::ui_semibold(text_size::HEADING)),
            (TextStyle::Monospace, fonts::value(text_size::VALUE)),
        ]
        .into();
        style.spacing.item_spacing = egui::vec2(spacing::SM, spacing::SM);
        style.spacing.button_padding = egui::vec2(spacing::MD, spacing::XS + 2.0);
        style.spacing.interact_size = egui::vec2(size::MIN_CLICK_HEIGHT, size::MIN_CLICK_HEIGHT);
        style.spacing.window_margin = Margin::same(spacing::LG as i8);
        style.spacing.menu_margin = Margin::same(spacing::SM as i8);
        style.animation_time = crate::design::tokens::motion::STATE_CHANGE_SECONDS;
    });
}

fn widget(fill: Color32, border: Color32, foreground: Color32) -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: Stroke::new(stroke::BORDER, border),
        corner_radius: CornerRadius::same(radius::MEDIUM),
        fg_stroke: Stroke::new(stroke::BORDER, foreground),
        expansion: 0.0,
    }
}

fn visuals(palette: &Palette, theme: Theme) -> Visuals {
    let mut visuals = if theme.is_dark() {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    let shadow_alpha = if theme.is_dark() {
        SHADOW_ALPHA_NIGHT
    } else {
        SHADOW_ALPHA_DAY
    };
    let shadow = Shadow {
        offset: [0, SHADOW_OFFSET_Y],
        blur: SHADOW_BLUR,
        spread: SHADOW_SPREAD,
        color: Color32::from_black_alpha(shadow_alpha),
    };
    visuals.override_text_color = Some(palette.text_primary);
    visuals.weak_text_color = Some(palette.text_muted);
    visuals.hyperlink_color = palette.accent;
    visuals.faint_bg_color = palette.surface;
    visuals.extreme_bg_color = palette.plot_background;
    visuals.code_bg_color = palette.surface_raised;
    visuals.warn_fg_color = palette.warning;
    visuals.error_fg_color = palette.danger;
    visuals.window_fill = palette.surface;
    visuals.window_stroke = Stroke::new(stroke::BORDER, palette.border);
    visuals.window_corner_radius = CornerRadius::same(radius::LARGE);
    visuals.window_shadow = shadow;
    visuals.popup_shadow = shadow;
    visuals.menu_corner_radius = CornerRadius::same(radius::MEDIUM);
    visuals.panel_fill = palette.surface;
    visuals.selection.bg_fill = palette.accent.gamma_multiply_u8(SELECTION_ALPHA);
    visuals.selection.stroke = Stroke::new(stroke::FOCUS, palette.accent);
    visuals.widgets.noninteractive = widget(palette.surface, palette.border, palette.text_primary);
    visuals.widgets.inactive = widget(palette.surface_raised, palette.border, palette.text_primary);
    visuals.widgets.hovered = widget(palette.surface_hover, palette.accent, palette.text_primary);
    visuals.widgets.active = widget(
        palette.accent.gamma_multiply_u8(SELECTION_ALPHA),
        palette.accent,
        palette.text_primary,
    );
    visuals.widgets.open = widget(palette.surface_hover, palette.accent, palette.text_primary);
    visuals.indent_has_left_vline = false;
    visuals.striped = false;
    visuals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_switch_changes_panel_fill_and_text_colors() {
        let context = egui::Context::default();
        apply(&context, Theme::Night);
        let night = context.global_style().visuals.clone();
        apply(&context, Theme::Daylight);
        let day = context.global_style().visuals.clone();
        assert_ne!(night.panel_fill, day.panel_fill);
        assert_eq!(
            night.override_text_color,
            Some(Theme::Night.palette().text_primary)
        );
        assert_eq!(
            day.override_text_color,
            Some(Theme::Daylight.palette().text_primary)
        );
    }
}
