use eframe::egui::{
    self, Align, Align2, CornerRadius, Layout, Rect, Role, Sense, Stroke, StrokeKind, WidgetInfo,
    pos2, vec2,
};

use crate::config::SMOOTHING_CHOICES;
use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{STORED_TRACE_COLOR_COUNT, radius, size, spacing, stroke, text_size};
use crate::design::widgets::{
    Emphasis, combo_label, field_caption, muted, sized_action_button, spin_label,
};
use crate::session::{Session, Trace};
use crate::text;

const SWATCH_SIZE: f32 = 12.0;
const ROW_HEIGHT: f32 = 28.0;
const CELL_WIDTH: f32 = 24.0;
const OFFSET_SPEED_DB: f64 = 0.1;
const OFFSET_LIMIT_DB: f64 = 40.0;
const SMOOTHING_COMBO_WIDTH: f32 = 64.0;
const SELECTED_FILL_ALPHA: f32 = 0.14;
const HIDDEN_SWATCH_ALPHA: f32 = 0.3;

#[derive(Clone, Debug, PartialEq)]
pub enum TraceCommand {
    ToggleVisible(u64),
    CycleColor(u64),
    SetOffset(u64, f64),
    SetSmoothing(u64, u32),
    ToggleSelected(u64),
    Rename(u64, String),
    Delete(u64),
    RemoveSelected,
    AverageSelected,
    ExportCsv,
}

#[derive(Debug, Default)]
pub struct OverlayListState {
    editing: Option<(u64, String)>,
    focused: bool,
    expanded: Option<u64>,
}

impl OverlayListState {
    fn start(&mut self, trace: &Trace) {
        self.editing = Some((trace.id, trace.name.clone()));
        self.focused = false;
    }

    fn stop(&mut self) {
        self.editing = None;
        self.focused = false;
    }
}

pub fn show(
    ui: &mut egui::Ui,
    palette: &Palette,
    session: &Session,
    state: &mut OverlayListState,
) -> Vec<TraceCommand> {
    let mut commands = Vec::new();
    if state.editing.is_none()
        && ui.input(|input| input.key_pressed(egui::Key::F2))
        && let [only] = session
            .traces
            .iter()
            .filter(|trace| trace.selected)
            .collect::<Vec<_>>()
            .as_slice()
    {
        state.start(only);
    }
    let list_height = (ui.available_height() - size::BUTTON_HEIGHT - spacing::SM).max(0.0);
    ui.allocate_ui(vec2(ui.available_width(), list_height), |ui| {
        if session.traces.is_empty() {
            field_caption(ui, palette, text::EMPTY_TRACES_TITLE);
            muted(ui, palette, text::EMPTY_TRACES_BODY);
            return;
        }
        egui::ScrollArea::vertical()
            .id_salt("overlay-list")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = size::SEGMENT_GAP;
                for trace in session.traces.iter().rev() {
                    overlay_row(ui, palette, trace, state, &mut commands);
                }
            });
    });
    ui.add_space(spacing::SM);
    let selected = session.traces.iter().filter(|trace| trace.selected).count();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = spacing::XS;
        let width = (ui.available_width() - 2.0 * spacing::XS) / 3.0;
        for (label, hint, enabled, command) in [
            (
                text::TRACES_AVERAGE,
                text::AVERAGE_HINT,
                selected >= 2,
                TraceCommand::AverageSelected,
            ),
            (
                text::TRACES_EXPORT_CSV,
                text::SAVE_HINT,
                !session.traces.is_empty(),
                TraceCommand::ExportCsv,
            ),
            (
                text::BUTTON_REMOVE,
                text::REMOVE_HINT,
                selected >= 1,
                TraceCommand::RemoveSelected,
            ),
        ] {
            if sized_action_button(
                ui,
                palette,
                label,
                None,
                Emphasis::Secondary,
                enabled,
                width,
            )
            .on_hover_text(hint)
            .clicked()
            {
                commands.push(command);
            }
        }
    });
    commands
}

fn overlay_row(
    ui: &mut egui::Ui,
    palette: &Palette,
    trace: &Trace,
    state: &mut OverlayListState,
    commands: &mut Vec<TraceCommand>,
) {
    let color = palette.stored_traces[trace.color_index % STORED_TRACE_COLOR_COUNT];
    let (row_rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), ROW_HEIGHT), Sense::hover());
    let (fill, border) = if trace.selected {
        (
            palette.accent.gamma_multiply(SELECTED_FILL_ALPHA),
            palette.accent,
        )
    } else {
        (palette.surface_raised, palette.border)
    };
    ui.painter().rect(
        row_rect,
        CornerRadius::same(radius::SMALL),
        fill,
        Stroke::new(stroke::BORDER, border),
        StrokeKind::Inside,
    );

    let swatch_cell = Rect::from_min_size(row_rect.min, vec2(CELL_WIDTH, ROW_HEIGHT));
    let swatch = ui.interact(
        swatch_cell,
        ui.make_persistent_id(("overlay-color", trace.id)),
        Sense::click(),
    );
    swatch.widget_info(|| {
        WidgetInfo::labeled(
            Role::Button,
            true,
            text::trace_control_label(text::TRACE_COLOR, &trace.name),
        )
    });
    ui.painter().rect_filled(
        Rect::from_center_size(swatch_cell.center(), vec2(SWATCH_SIZE, SWATCH_SIZE)),
        CornerRadius::same(2),
        if trace.visible {
            color
        } else {
            color.gamma_multiply(HIDDEN_SWATCH_ALPHA)
        },
    );
    if swatch.on_hover_text(text::TRACE_COLOR).clicked() {
        commands.push(TraceCommand::CycleColor(trace.id));
    }

    let visible_cell = Rect::from_min_size(
        pos2(swatch_cell.right(), row_rect.top()),
        vec2(CELL_WIDTH, ROW_HEIGHT),
    );
    let visible = ui.interact(
        visible_cell.shrink(3.0),
        ui.make_persistent_id(("overlay-visible", trace.id)),
        Sense::click(),
    );
    visible.widget_info(|| {
        WidgetInfo::selected(
            Role::CheckBox,
            true,
            trace.visible,
            text::trace_control_label(text::TRACE_TOGGLE_VISIBLE, &trace.name),
        )
    });
    let visible_box = visible_cell.shrink(4.0);
    ui.painter().rect(
        visible_box,
        CornerRadius::same(2),
        if trace.visible {
            palette.surface_hover
        } else {
            egui::Color32::TRANSPARENT
        },
        Stroke::new(
            stroke::BORDER,
            if visible.hovered() {
                palette.accent
            } else {
                palette.border
            },
        ),
        StrokeKind::Inside,
    );
    ui.painter().text(
        visible_box.center(),
        Align2::CENTER_CENTER,
        text::TRACE_VISIBLE_SHORT,
        fonts::ui_semibold(text_size::CAPTION),
        if trace.visible {
            palette.text_primary
        } else {
            palette.text_muted
        },
    );
    if visible.on_hover_text(text::TRACE_TOGGLE_VISIBLE).clicked() {
        commands.push(TraceCommand::ToggleVisible(trace.id));
    }

    let properties_cell = Rect::from_min_size(
        pos2(row_rect.right() - CELL_WIDTH, row_rect.top()),
        vec2(CELL_WIDTH, ROW_HEIGHT),
    );
    let expanded = state.expanded == Some(trace.id);
    let properties = ui.interact(
        properties_cell,
        ui.make_persistent_id(("overlay-properties", trace.id)),
        Sense::click(),
    );
    properties.widget_info(|| {
        WidgetInfo::selected(
            Role::Button,
            true,
            expanded,
            text::trace_control_label(text::OVERLAY_PROPERTIES, &trace.name),
        )
    });
    ui.painter().text(
        properties_cell.center(),
        Align2::CENTER_CENTER,
        text::OVERLAY_PROPERTIES_GLYPH,
        fonts::ui_semibold(text_size::LABEL),
        if expanded || properties.hovered() {
            palette.accent
        } else {
            palette.text_muted
        },
    );
    if properties.on_hover_text(text::OVERLAY_PROPERTIES).clicked() {
        state.expanded = if expanded { None } else { Some(trace.id) };
    }

    let name_cell = Rect::from_min_max(
        pos2(visible_cell.right() + spacing::XS, row_rect.top()),
        pos2(properties_cell.left(), row_rect.bottom()),
    );
    let editing_this = state
        .editing
        .as_ref()
        .is_some_and(|(id, _)| *id == trace.id);
    if editing_this {
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(name_cell.shrink2(vec2(0.0, size::SEGMENT_GAP)))
                .layout(Layout::left_to_right(Align::Center)),
        );
        if let Some((_, buffer)) = state.editing.as_mut() {
            let response = child.add_sized(
                name_cell.shrink2(vec2(0.0, size::SEGMENT_GAP)).size(),
                egui::TextEdit::singleline(buffer).font(fonts::ui(text_size::LABEL)),
            );
            if !state.focused {
                response.request_focus();
                state.focused = true;
            }
            let (escape, enter) = child.input(|input| {
                (
                    input.key_pressed(egui::Key::Escape),
                    input.key_pressed(egui::Key::Enter),
                )
            });
            if escape {
                state.stop();
            } else if enter || response.lost_focus() {
                let name = buffer.trim().to_owned();
                if !name.is_empty() && name != trace.name {
                    commands.push(TraceCommand::Rename(trace.id, name));
                }
                state.stop();
            }
        }
    } else {
        let name = ui.interact(
            name_cell,
            ui.make_persistent_id(("overlay-name", trace.id)),
            Sense::click(),
        );
        name.widget_info(|| {
            WidgetInfo::selected(
                Role::CheckBox,
                true,
                trace.selected,
                text::trace_control_label(text::TRACE_SELECT, &trace.name),
            )
        });
        let galley = ui.painter().layout(
            trace.name.clone(),
            fonts::ui_medium(text_size::LABEL),
            palette.text_primary,
            name_cell.width(),
        );
        let painter = ui.painter_at(name_cell);
        painter.galley(
            pos2(
                name_cell.left(),
                name_cell.center().y - galley.size().y / 2.0,
            ),
            galley,
            palette.text_primary,
        );
        let name = name.on_hover_text(text::OVERLAY_SELECT_HINT);
        if name.double_clicked() {
            state.start(trace);
        } else if name.clicked() {
            commands.push(TraceCommand::ToggleSelected(trace.id));
        }
    }

    if expanded {
        properties_row(ui, palette, trace, commands);
    }
}

fn properties_row(
    ui: &mut egui::Ui,
    palette: &Palette,
    trace: &Trace,
    commands: &mut Vec<TraceCommand>,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = spacing::XS;
        ui.add_space(CELL_WIDTH);
        field_caption(ui, palette, text::TRACE_OFFSET);
        let mut offset = trace.offset_db;
        let response = ui
            .add(
                egui::DragValue::new(&mut offset)
                    .speed(OFFSET_SPEED_DB)
                    .range(-OFFSET_LIMIT_DB..=OFFSET_LIMIT_DB)
                    .max_decimals(1)
                    .custom_formatter(|value, _| text::format_db(value))
                    .custom_parser(text::parse_number),
            )
            .on_hover_text(text::TRACE_OFFSET);
        spin_label(
            &response,
            text::trace_control_label(text::TRACE_OFFSET, &trace.name),
        );
        if response.changed() {
            commands.push(TraceCommand::SetOffset(trace.id, offset));
        }
        ui.add_space(spacing::SM);
        let can_change = trace.can_change_smoothing();
        let mut smoothing = trace.metadata.smoothing_fraction;
        let combo = ui
            .add_enabled_ui(can_change, |ui| {
                egui::ComboBox::from_id_salt(("trace-smoothing", trace.id))
                    .width(SMOOTHING_COMBO_WIDTH)
                    .selected_text(text::format_smoothing(smoothing))
                    .show_ui(ui, |ui| {
                        for choice in SMOOTHING_CHOICES {
                            ui.selectable_value(
                                &mut smoothing,
                                choice,
                                text::format_smoothing(choice),
                            );
                        }
                    })
            })
            .inner;
        let combo_response = combo.response.on_hover_text(if can_change {
            text::TRACE_SMOOTHING
        } else {
            text::TRACE_SMOOTHING_FIXED
        });
        combo_label(
            &combo_response,
            text::trace_control_label(text::TRACE_SMOOTHING, &trace.name),
        );
        if smoothing != trace.metadata.smoothing_fraction {
            commands.push(TraceCommand::SetSmoothing(trace.id, smoothing));
        }
    });
}
