use eframe::egui::{
    self, Align, CornerRadius, Layout, RichText, Role, Sense, Stroke, StrokeKind, WidgetInfo, vec2,
};

use crate::design::Palette;
use crate::design::fonts;
use crate::design::tokens::{STORED_TRACE_COLOR_COUNT, radius, size, spacing, stroke, text_size};
use crate::design::widgets::{Emphasis, action_button, combo_label, spin_label};
use crate::session::{Session, Trace};
use crate::text;
use crate::ui::bottom_bar::SMOOTHING_CHOICES;

const SWATCH_SIZE: f32 = 14.0;
const ROW_HEIGHT: f32 = 92.0;
const LINE_HEIGHT: f32 = 28.0;
const OFFSET_SPEED_DB: f64 = 0.1;
const OFFSET_LIMIT_DB: f64 = 40.0;
const ICON_BUTTON_SIZE: f32 = 28.0;
const SMOOTHING_COMBO_WIDTH: f32 = 64.0;

#[derive(Clone, Debug, PartialEq)]
pub enum TraceCommand {
    ToggleVisible(u64),
    CycleColor(u64),
    SetOffset(u64, f64),
    SetSmoothing(u64, u32),
    ToggleSelected(u64),
    Rename(u64, String),
    Delete(u64),
    AverageSelected,
    ExportCsv,
}

#[derive(Debug, Default)]
pub struct RenameState {
    editing: Option<(u64, String)>,
    focused: bool,
}

impl RenameState {
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
    rename: &mut RenameState,
) -> Vec<TraceCommand> {
    let mut commands = Vec::new();
    if rename.editing.is_none()
        && ui.input(|input| input.key_pressed(egui::Key::F2))
        && let [only] = session
            .traces
            .iter()
            .filter(|trace| trace.selected)
            .collect::<Vec<_>>()
            .as_slice()
    {
        rename.start(only);
    }
    ui.add_space(spacing::MD);
    egui::ScrollArea::vertical()
        .max_height(ui.available_height() - 2.0 * size::BUTTON_HEIGHT - 3.0 * spacing::MD)
        .show(ui, |ui| {
            for trace in session.traces.iter().rev() {
                trace_row(ui, palette, trace, rename, &mut commands);
                ui.add_space(spacing::XS);
            }
        });
    ui.add_space(spacing::MD);
    let selected = session.traces.iter().filter(|trace| trace.selected).count();
    ui.horizontal(|ui| {
        if action_button(
            ui,
            palette,
            text::TRACES_AVERAGE,
            None,
            Emphasis::Secondary,
            selected >= 2,
        )
        .clicked()
        {
            commands.push(TraceCommand::AverageSelected);
        }
    });
    ui.add_space(spacing::XS);
    if action_button(
        ui,
        palette,
        text::TRACES_EXPORT_CSV,
        None,
        Emphasis::Quiet,
        true,
    )
    .clicked()
    {
        commands.push(TraceCommand::ExportCsv);
    }
    commands
}

fn trace_row(
    ui: &mut egui::Ui,
    palette: &Palette,
    trace: &Trace,
    rename: &mut RenameState,
    commands: &mut Vec<TraceCommand>,
) {
    let color = palette.stored_traces[trace.color_index % STORED_TRACE_COLOR_COUNT];
    let (row_rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), ROW_HEIGHT), Sense::hover());
    ui.painter().rect(
        row_rect,
        CornerRadius::same(radius::MEDIUM),
        if trace.selected {
            palette.accent.gamma_multiply(0.12)
        } else {
            palette.surface_raised
        },
        Stroke::new(
            stroke::BORDER,
            if trace.selected {
                palette.accent
            } else {
                palette.border
            },
        ),
        StrokeKind::Inside,
    );
    let inner = row_rect.shrink2(vec2(spacing::SM, spacing::XS));
    let first_line = egui::Rect::from_min_size(inner.min, vec2(inner.width(), LINE_HEIGHT));
    let second_line = egui::Rect::from_min_size(
        inner.min + vec2(0.0, LINE_HEIGHT),
        vec2(inner.width(), LINE_HEIGHT),
    );
    let third_line = egui::Rect::from_min_size(
        inner.min + vec2(0.0, 2.0 * LINE_HEIGHT),
        vec2(inner.width(), LINE_HEIGHT),
    );
    let mut top = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(first_line)
            .layout(Layout::left_to_right(Align::Center)),
    );
    top.spacing_mut().item_spacing.x = spacing::SM;

    let (swatch_rect, swatch) =
        top.allocate_exact_size(vec2(ICON_BUTTON_SIZE, ICON_BUTTON_SIZE), Sense::click());
    swatch.widget_info(|| {
        WidgetInfo::labeled(
            Role::Button,
            true,
            text::trace_control_label(text::TRACE_COLOR, &trace.name),
        )
    });
    top.painter().rect_filled(
        egui::Rect::from_center_size(swatch_rect.center(), vec2(SWATCH_SIZE, SWATCH_SIZE)),
        CornerRadius::same(radius::SMALL),
        if trace.visible {
            color
        } else {
            color.gamma_multiply(0.3)
        },
    );
    if swatch.clicked() {
        commands.push(TraceCommand::CycleColor(trace.id));
    }

    let editing_this = rename
        .editing
        .as_ref()
        .is_some_and(|(id, _)| *id == trace.id);
    let name_width = (top.available_width() - ICON_BUTTON_SIZE - spacing::SM).max(0.0);
    if editing_this {
        if let Some((_, buffer)) = rename.editing.as_mut() {
            let response = top.add_sized(
                vec2(name_width, size::MIN_CLICK_HEIGHT),
                egui::TextEdit::singleline(buffer).font(fonts::ui(text_size::LABEL)),
            );
            if !rename.focused {
                response.request_focus();
                rename.focused = true;
            }
            let (escape, enter) = top.input(|input| {
                (
                    input.key_pressed(egui::Key::Escape),
                    input.key_pressed(egui::Key::Enter),
                )
            });
            if escape {
                rename.stop();
            } else if enter || response.lost_focus() {
                let name = buffer.trim().to_owned();
                if !name.is_empty() && name != trace.name {
                    commands.push(TraceCommand::Rename(trace.id, name));
                }
                rename.stop();
            }
        }
    } else {
        let label = top
            .allocate_ui_with_layout(
                vec2(name_width, size::MIN_CLICK_HEIGHT),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(&trace.name)
                                .font(fonts::ui_medium(text_size::LABEL))
                                .color(palette.text_primary),
                        )
                        .truncate()
                        .sense(Sense::click()),
                    )
                },
            )
            .inner
            .on_hover_text(text::TRACE_RENAME_HINT);
        label.widget_info(|| {
            WidgetInfo::labeled(
                Role::Button,
                true,
                text::trace_control_label(text::TRACE_RENAME, &trace.name),
            )
        });
        if label.double_clicked() {
            rename.start(trace);
        }
    }

    let delete = top
        .with_layout(Layout::right_to_left(Align::Center), |top| {
            top.add_sized(
                vec2(ICON_BUTTON_SIZE, ICON_BUTTON_SIZE),
                egui::Button::new(
                    RichText::new(text::TRACE_DELETE_GLYPH).font(fonts::ui(text_size::TITLE)),
                )
                .frame(false),
            )
        })
        .inner;
    delete.widget_info(|| {
        WidgetInfo::labeled(
            Role::Button,
            true,
            text::trace_control_label(text::TRACE_DELETE, &trace.name),
        )
    });
    if delete.on_hover_text(text::TRACE_DELETE).clicked() {
        commands.push(TraceCommand::Delete(trace.id));
    }

    let mut row = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(second_line)
            .layout(Layout::left_to_right(Align::Center)),
    );
    row.spacing_mut().item_spacing.x = spacing::SM;
    row.add_space(spacing::XS);
    let mut selected = trace.selected;
    let select = row.add(egui::Checkbox::new(&mut selected, text::TRACE_SELECT_SHORT));
    select.widget_info(|| {
        WidgetInfo::selected(
            Role::CheckBox,
            true,
            trace.selected,
            text::trace_control_label(text::TRACE_SELECT, &trace.name),
        )
    });
    if select.on_hover_text(text::TRACE_SELECT).changed() {
        commands.push(TraceCommand::ToggleSelected(trace.id));
    }
    let mut visible = trace.visible;
    let visibility = row.add(egui::Checkbox::new(&mut visible, text::TRACE_VISIBLE_SHORT));
    visibility.widget_info(|| {
        WidgetInfo::selected(
            Role::CheckBox,
            true,
            trace.visible,
            text::trace_control_label(text::TRACE_TOGGLE_VISIBLE, &trace.name),
        )
    });
    if visibility
        .on_hover_text(text::TRACE_TOGGLE_VISIBLE)
        .changed()
    {
        commands.push(TraceCommand::ToggleVisible(trace.id));
    }

    let mut offset = trace.offset_db;
    let mut smoothing = trace.metadata.smoothing_fraction;
    let offset_changed = row
        .with_layout(Layout::right_to_left(Align::Center), |row| {
            row.add_space(spacing::XS);
            let response = row
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
            response.changed()
        })
        .inner;

    let mut definition = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(third_line)
            .layout(Layout::left_to_right(Align::Center)),
    );
    definition.spacing_mut().item_spacing.x = spacing::SM;
    definition.add_space(spacing::XS);
    definition.label(
        RichText::new(text::TRACE_SMOOTHING_SHORT)
            .font(fonts::ui_medium(text_size::CAPTION))
            .color(palette.text_muted),
    );
    let can_change = trace.can_change_smoothing();
    let combo = definition
        .add_enabled_ui(can_change, |row| {
            egui::ComboBox::from_id_salt(("trace-smoothing", trace.id))
                .width(SMOOTHING_COMBO_WIDTH)
                .selected_text(text::format_smoothing(smoothing))
                .show_ui(row, |row| {
                    for choice in SMOOTHING_CHOICES {
                        row.selectable_value(
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
    if offset_changed {
        commands.push(TraceCommand::SetOffset(trace.id, offset));
    }
    if smoothing != trace.metadata.smoothing_fraction {
        commands.push(TraceCommand::SetSmoothing(trace.id, smoothing));
    }
}
