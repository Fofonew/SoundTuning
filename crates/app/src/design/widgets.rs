use eframe::egui::{
    self, Align2, Color32, CornerRadius, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui,
    WidgetInfo, vec2,
};

use crate::design::fonts;
use crate::design::tokens::{Palette, radius, size, spacing, stroke, text_size};

const KEYCAP_MIN_WIDTH: f32 = 20.0;
const KEYCAP_HEIGHT: f32 = 18.0;
const KEYCAP_PADDING_X: f32 = 5.0;
const PILL_HEIGHT: f32 = 24.0;
const PILL_DOT_RADIUS: f32 = 4.0;
const DISABLED_ALPHA: f32 = 0.4;
const ACCENT_TEXT_ON_NIGHT: Color32 = Color32::from_rgb(0x06, 0x10, 0x18);
const ACCENT_TEXT_ON_DAY: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
const CONFIDENCE_SEGMENTS: usize = 5;
const CONFIDENCE_DB_PER_SEGMENT: f64 = 6.0;
const CONFIDENCE_SEGMENT_WIDTH: f32 = 6.0;
const CONFIDENCE_SEGMENT_HEIGHT: f32 = 10.0;
const CONFIDENCE_SEGMENT_GAP: f32 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Neutral,
    Accent,
    Success,
    Warning,
    Danger,
}

impl Tone {
    pub fn color(self, palette: &Palette) -> Color32 {
        match self {
            Self::Neutral => palette.text_muted,
            Self::Accent => palette.accent,
            Self::Success => palette.success,
            Self::Warning => palette.warning,
            Self::Danger => palette.danger,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Emphasis {
    Primary,
    Secondary,
    Quiet,
}

pub fn keycap_width(ui: &Ui, text: &str) -> f32 {
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        fonts::value(text_size::CAPTION),
        Color32::WHITE,
    );
    (galley.size().x + 2.0 * KEYCAP_PADDING_X).max(KEYCAP_MIN_WIDTH)
}

pub fn paint_keycap(ui: &Ui, palette: &Palette, rect: Rect, text: &str, on_accent: bool) {
    let painter = ui.painter();
    let (fill, border, color) = if on_accent {
        (
            Color32::from_white_alpha(40),
            Color32::from_white_alpha(80),
            accent_text_color(palette),
        )
    } else {
        (palette.surface, palette.border, palette.text_muted)
    };
    painter.rect(
        rect,
        CornerRadius::same(radius::SMALL),
        fill,
        Stroke::new(stroke::BORDER, border),
        StrokeKind::Inside,
    );
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        fonts::value(text_size::CAPTION),
        color,
    );
}

fn accent_text_color(palette: &Palette) -> Color32 {
    if palette.background.r() < 128 {
        ACCENT_TEXT_ON_NIGHT
    } else {
        ACCENT_TEXT_ON_DAY
    }
}

pub fn action_button(
    ui: &mut Ui,
    palette: &Palette,
    label: &str,
    shortcut: Option<&str>,
    emphasis: Emphasis,
    enabled: bool,
) -> Response {
    let font = fonts::label();
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font.clone(), Color32::WHITE);
    let keycap = shortcut.map(|text| (text, keycap_width(ui, text)));
    let keycap_extent = keycap.map_or(0.0, |(_, width)| width + spacing::SM);
    let desired = vec2(
        galley.size().x + keycap_extent + 2.0 * spacing::MD,
        size::BUTTON_HEIGHT,
    );
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(desired, sense);
    response.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, label));

    if ui.is_rect_visible(rect) {
        let hovered = enabled && response.hovered();
        let pressed = enabled && response.is_pointer_button_down_on();
        let (fill, border, text_color) = match emphasis {
            Emphasis::Primary => (
                if pressed {
                    palette.accent.gamma_multiply(0.8)
                } else if hovered {
                    palette.accent.gamma_multiply(1.1)
                } else {
                    palette.accent
                },
                palette.accent,
                accent_text_color(palette),
            ),
            Emphasis::Secondary => (
                if hovered || pressed {
                    palette.surface_hover
                } else {
                    palette.surface_raised
                },
                if hovered {
                    palette.accent
                } else {
                    palette.border
                },
                palette.text_primary,
            ),
            Emphasis::Quiet => (
                if hovered || pressed {
                    palette.surface_hover
                } else {
                    Color32::TRANSPARENT
                },
                Color32::TRANSPARENT,
                palette.text_primary,
            ),
        };
        let alpha = if enabled { 1.0 } else { DISABLED_ALPHA };
        let painter = ui.painter();
        painter.rect(
            rect,
            CornerRadius::same(radius::MEDIUM),
            fill.gamma_multiply(alpha),
            Stroke::new(stroke::BORDER, border.gamma_multiply(alpha)),
            StrokeKind::Inside,
        );
        if response.has_focus() {
            painter.rect_stroke(
                rect.expand(2.0),
                CornerRadius::same(radius::MEDIUM + 2),
                Stroke::new(stroke::FOCUS, palette.accent),
                StrokeKind::Outside,
            );
        }
        let text_position = egui::pos2(rect.left() + spacing::MD, rect.center().y);
        painter.text(
            text_position,
            Align2::LEFT_CENTER,
            label,
            font,
            text_color.gamma_multiply(alpha),
        );
        if let Some((text, width)) = keycap {
            let keycap_rect = Rect::from_center_size(
                egui::pos2(rect.right() - spacing::MD - width / 2.0, rect.center().y),
                vec2(width, KEYCAP_HEIGHT),
            );
            paint_keycap(
                ui,
                palette,
                keycap_rect,
                text,
                emphasis == Emphasis::Primary,
            );
        }
    }
    if enabled {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}

pub fn chip(
    ui: &mut Ui,
    palette: &Palette,
    caption: &str,
    value: &str,
    interactive: bool,
) -> Response {
    let caption_galley =
        ui.painter()
            .layout_no_wrap(caption.to_owned(), fonts::caption(), palette.text_muted);
    let value_galley = ui.painter().layout_no_wrap(
        value.to_owned(),
        fonts::value_medium(text_size::VALUE),
        palette.text_primary,
    );
    let desired = vec2(
        caption_galley.size().x + value_galley.size().x + spacing::SM + 2.0 * spacing::MD,
        size::BUTTON_HEIGHT,
    );
    let sense = if interactive {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(desired, sense);
    let accessible_label = format!("{caption} {value}");
    let role = if interactive {
        Role::SpinButton
    } else {
        Role::Label
    };
    response.widget_info(|| WidgetInfo::labeled(role, true, &accessible_label));
    if ui.is_rect_visible(rect) {
        let hovered = interactive && response.hovered();
        let painter = ui.painter();
        painter.rect(
            rect,
            CornerRadius::same(radius::LARGE),
            if hovered {
                palette.surface_hover
            } else {
                palette.surface_raised
            },
            Stroke::new(
                stroke::BORDER,
                if hovered {
                    palette.accent
                } else {
                    palette.border
                },
            ),
            StrokeKind::Inside,
        );
        if response.has_focus() {
            painter.rect_stroke(
                rect.expand(2.0),
                CornerRadius::same(radius::LARGE + 2),
                Stroke::new(stroke::FOCUS, palette.accent),
                StrokeKind::Outside,
            );
        }
        let caption_position = egui::pos2(rect.left() + spacing::MD, rect.center().y);
        painter.galley(
            caption_position - vec2(0.0, caption_galley.size().y / 2.0),
            caption_galley.clone(),
            palette.text_muted,
        );
        let value_position = egui::pos2(
            caption_position.x + caption_galley.size().x + spacing::SM,
            rect.center().y - value_galley.size().y / 2.0,
        );
        painter.galley(value_position, value_galley, palette.text_primary);
    }
    if interactive {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}

pub fn status_pill(ui: &mut Ui, palette: &Palette, tone: Tone, text: &str) -> Response {
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), fonts::label(), palette.text_primary);
    let desired = vec2(
        galley.size().x + 2.0 * spacing::MD + PILL_DOT_RADIUS * 2.0 + spacing::SM,
        PILL_HEIGHT,
    );
    let (rect, response) = ui.allocate_exact_size(desired, Sense::hover());
    response.widget_info(|| WidgetInfo::labeled(Role::Status, true, text));
    if ui.is_rect_visible(rect) {
        let color = tone.color(palette);
        let painter = ui.painter();
        painter.rect(
            rect,
            CornerRadius::same(u8::MAX),
            color.gamma_multiply(0.14),
            Stroke::new(stroke::BORDER, color.gamma_multiply(0.6)),
            StrokeKind::Inside,
        );
        let dot_center = egui::pos2(rect.left() + spacing::MD + PILL_DOT_RADIUS, rect.center().y);
        painter.circle_filled(dot_center, PILL_DOT_RADIUS, color);
        painter.galley(
            egui::pos2(
                dot_center.x + PILL_DOT_RADIUS + spacing::SM,
                rect.center().y - galley.size().y / 2.0,
            ),
            galley,
            palette.text_primary,
        );
    }
    response
}

pub fn confidence_segments(
    ui: &mut Ui,
    palette: &Palette,
    confidence_db: Option<f64>,
    label: &str,
) {
    let width = CONFIDENCE_SEGMENTS as f32 * (CONFIDENCE_SEGMENT_WIDTH + CONFIDENCE_SEGMENT_GAP)
        - CONFIDENCE_SEGMENT_GAP;
    let (rect, response) =
        ui.allocate_exact_size(vec2(width, CONFIDENCE_SEGMENT_HEIGHT), Sense::hover());
    response.widget_info(|| WidgetInfo::labeled(Role::Meter, true, label));
    let lit = confidence_db.map_or(0, |value| {
        ((value / CONFIDENCE_DB_PER_SEGMENT).floor().max(0.0) as usize).min(CONFIDENCE_SEGMENTS)
    });
    let color = match lit {
        0..=2 => palette.warning,
        _ => palette.success,
    };
    for index in 0..CONFIDENCE_SEGMENTS {
        let left = rect.left() + index as f32 * (CONFIDENCE_SEGMENT_WIDTH + CONFIDENCE_SEGMENT_GAP);
        let segment = Rect::from_min_size(
            egui::pos2(left, rect.top()),
            vec2(CONFIDENCE_SEGMENT_WIDTH, CONFIDENCE_SEGMENT_HEIGHT),
        );
        ui.painter().rect_filled(
            segment,
            CornerRadius::same(1),
            if index < lit {
                color
            } else {
                palette.surface_hover
            },
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterReading {
    pub peak_dbfs: f32,
    pub rms_dbfs: f32,
    pub clipped: bool,
}

pub const METER_FLOOR_DBFS: f32 = -60.0;
const METER_WARNING_DBFS: f32 = -6.0;

pub fn level_meter(ui: &mut Ui, palette: &Palette, label: &str, reading: MeterReading) {
    let label_galley =
        ui.painter()
            .layout_no_wrap(label.to_owned(), fonts::caption(), palette.text_muted);
    let value_text = crate::text::format_meter_dbfs(reading.peak_dbfs);
    let value_galley = ui.painter().layout_no_wrap(
        value_text.clone(),
        fonts::value(text_size::CAPTION),
        palette.text_primary,
    );
    let total_width = size::METER_WIDTH;
    let desired = vec2(
        total_width,
        label_galley.size().y + spacing::XS + size::METER_HEIGHT,
    );
    let (rect, response) = ui.allocate_exact_size(desired, Sense::hover());
    let accessible = format!("{label} {value_text}");
    response.widget_info(|| WidgetInfo::labeled(Role::Meter, true, &accessible));
    if !ui.is_rect_visible(rect) {
        return;
    }
    let painter = ui.painter();
    painter.galley(rect.left_top(), label_galley.clone(), palette.text_muted);
    painter.galley(
        egui::pos2(rect.right() - value_galley.size().x, rect.top()),
        value_galley,
        if reading.clipped {
            palette.danger
        } else {
            palette.text_primary
        },
    );
    let bar = Rect::from_min_size(
        egui::pos2(rect.left(), rect.bottom() - size::METER_HEIGHT),
        vec2(total_width, size::METER_HEIGHT),
    );
    painter.rect_filled(bar, CornerRadius::same(2), palette.surface_hover);
    let fraction = |dbfs: f32| ((dbfs - METER_FLOOR_DBFS) / -METER_FLOOR_DBFS).clamp(0.0, 1.0);
    let rms_width = bar.width() * fraction(reading.rms_dbfs);
    let peak_x = bar.left() + bar.width() * fraction(reading.peak_dbfs);
    let fill_color = if reading.clipped {
        palette.danger
    } else if reading.peak_dbfs > METER_WARNING_DBFS {
        palette.warning
    } else {
        palette.success
    };
    painter.rect_filled(
        Rect::from_min_size(bar.left_top(), vec2(rms_width, bar.height())),
        CornerRadius::same(2),
        fill_color,
    );
    painter.line_segment(
        [
            egui::pos2(peak_x, bar.top()),
            egui::pos2(peak_x, bar.bottom()),
        ],
        Stroke::new(2.0, fill_color),
    );
}

pub fn section_title(ui: &mut Ui, palette: &Palette, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .font(fonts::ui_semibold(text_size::TITLE))
            .color(palette.text_primary),
    );
}

pub fn muted(ui: &mut Ui, palette: &Palette, text: &str) -> Response {
    ui.label(
        egui::RichText::new(text)
            .font(fonts::body())
            .color(palette.text_muted),
    )
}
