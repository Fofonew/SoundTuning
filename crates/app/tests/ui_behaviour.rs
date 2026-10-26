#![allow(clippy::unwrap_used)]

mod common;

use std::time::Duration;

use common::{
    file_options, first_launch_options, harness, step_until, wait_for_stable_measurement,
    wait_until_running,
};
use eframe::egui::{Event, Key, Modifiers, Role};
use egui_kittest::kittest::Queryable;
use engine::EngineCommand;
use soundtuning::design::Theme;

const SHORT_TIMEOUT: Duration = Duration::from_secs(20);
const MAX_TRACE_JSON_BYTES: usize = 400_000;

#[test]
fn space_starts_and_stops_the_measurement() {
    let mut harness = harness(file_options(Theme::Night, false));
    harness.step();
    assert!(!harness.state().measurement().is_running());
    harness.key_press(Key::Space);
    harness.step();
    assert!(harness.state().measurement().is_running());
    harness.key_press(Key::Space);
    harness.step();
    assert!(!harness.state().measurement().is_running());
}

#[test]
fn d_finds_the_delay_and_a_applies_it() {
    let mut harness = harness(file_options(Theme::Night, true));
    wait_for_stable_measurement(&mut harness);
    harness
        .state()
        .measurement()
        .send(EngineCommand::SetCompensationMs(0.0));
    step_until(&mut harness, SHORT_TIMEOUT, |app| {
        app.measurement().snapshot().delay.applied_ms.abs() < 1.0e-9
    });
    harness.key_press(Key::D);
    harness.step();
    step_until(&mut harness, SHORT_TIMEOUT, |app| {
        app.notification_texts()
            .iter()
            .any(|text| text.starts_with(soundtuning::text::NOTIFY_DELAY_FOUND))
    });
    harness.key_press(Key::A);
    harness.step();
    step_until(&mut harness, SHORT_TIMEOUT, |app| {
        (app.measurement().snapshot().delay.applied_ms - 2.0).abs() < 0.05
    });
}

#[test]
fn command_k_opens_the_palette_and_escape_closes_it() {
    let mut harness = harness(file_options(Theme::Night, false));
    harness.step();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.step();
    assert!(harness.state().is_palette_open());
    harness.step();
    harness.key_press(Key::Escape);
    harness.step();
    assert!(!harness.state().is_palette_open());
}

#[test]
fn l_switches_between_night_and_daylight() {
    let mut harness = harness(file_options(Theme::Night, false));
    harness.step();
    harness.key_press(Key::L);
    harness.step();
    assert_eq!(harness.state().config().theme, Theme::Daylight);
    harness.key_press(Key::L);
    harness.step();
    assert_eq!(harness.state().config().theme, Theme::Night);
}

#[test]
fn range_keys_change_the_plot_range() {
    let mut harness = harness(file_options(Theme::Night, false));
    harness.step();
    harness.key_press(Key::Num2);
    harness.step();
    assert_eq!(harness.state().plot_view().high_hz, 200.0);
    harness.key_press(Key::Num1);
    harness.step();
    assert_eq!(harness.state().plot_view().high_hz, 20_000.0);
}

#[test]
fn f_freezes_the_display() {
    let mut harness = harness(file_options(Theme::Night, true));
    wait_for_stable_measurement(&mut harness);
    harness.key_press(Key::F);
    harness.step();
    assert!(harness.state().measurement().is_frozen());
    let frozen_sequence = harness.state().measurement().snapshot().sequence;
    for _ in 0..20 {
        harness.step();
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        harness.state().measurement().snapshot().sequence,
        frozen_sequence
    );
}

#[test]
fn every_main_action_is_a_visible_button() {
    let mut harness = harness(file_options(Theme::Night, true));
    harness.step();
    for label in [
        soundtuning::text::BUTTON_STOP,
        soundtuning::text::BUTTON_CAPTURE,
        soundtuning::text::BUTTON_FREEZE,
        soundtuning::text::BUTTON_FIND_DELAY,
        soundtuning::text::BUTTON_APPLY_DELAY,
        soundtuning::text::BUTTON_PALETTE,
    ] {
        assert!(
            harness
                .query_by_role_and_label(Role::Button, label)
                .is_some(),
            "missing button {label}"
        );
    }
}

#[test]
fn no_modal_window_opens_while_measuring() {
    let mut harness = harness(file_options(Theme::Night, true));
    wait_for_stable_measurement(&mut harness);
    for key in [Key::D, Key::A, Key::F, Key::F, Key::L, Key::Num3] {
        harness.key_press(key);
        harness.step();
        assert_eq!(harness.query_all_by_role(Role::Dialog).count(), 0);
        assert_eq!(harness.query_all_by_role(Role::AlertDialog).count(), 0);
    }
    assert!(harness.state().measurement().is_running());
}

#[test]
fn first_launch_shows_the_wizard_until_it_is_completed() {
    let mut harness = harness(first_launch_options());
    harness.step();
    assert!(harness.state().is_wizard_visible());
    harness.get_by_label("UAX 2-2").click();
    harness.step();
    assert_eq!(
        harness.state().config().stream.input_device_id.as_deref(),
        Some("coreaudio:UAX22")
    );
    harness
        .get_by_label(soundtuning::text::BUTTON_SKIP_WIZARD)
        .click();
    harness.step();
    assert!(!harness.state().is_wizard_visible());
    assert!(harness.state().config().wizard_completed);
}

#[test]
fn held_space_does_not_restart_the_measurement_on_key_repeat() {
    let mut harness = harness(file_options(Theme::Night, false));
    harness.step();
    harness.key_down(Key::Space);
    harness.step();
    assert!(harness.state().measurement().is_running());
    for _ in 0..5 {
        harness.event(Event::Key {
            key: Key::Space,
            physical_key: None,
            pressed: true,
            repeat: true,
            modifiers: Modifiers::NONE,
        });
        harness.step();
    }
    harness.key_up(Key::Space);
    harness.step();
    assert!(harness.state().measurement().is_running());
}

#[test]
fn space_on_a_focused_button_does_not_toggle_the_measurement() {
    let mut harness = harness(file_options(Theme::Night, true));
    wait_until_running(&mut harness);
    harness
        .get_by_role_and_label(Role::Button, soundtuning::text::BUTTON_FREEZE)
        .focus();
    harness.step();
    harness.key_press(Key::Space);
    harness.step();
    assert!(harness.state().measurement().is_running());
}

#[test]
fn palette_runs_the_selected_command_with_enter() {
    let mut harness = harness(file_options(Theme::Night, false));
    harness.step();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.step();
    harness.get_by_role(Role::TextInput).type_text("thème");
    harness.step();
    harness.key_press(Key::Enter);
    harness.step();
    assert!(!harness.state().is_palette_open());
    assert_eq!(harness.state().config().theme, Theme::Daylight);
}

#[test]
fn restart_keeps_the_measurement_running_and_clears_old_results() {
    let mut harness = harness(file_options(Theme::Night, true));
    wait_for_stable_measurement(&mut harness);
    harness
        .state_mut()
        .handle_action(soundtuning::actions::Action::RestartMeasurement);
    assert!(harness.state().measurement().is_running());
    assert_eq!(harness.state().measurement().snapshot().averages, 0);
    assert!(
        harness
            .state()
            .measurement()
            .snapshot()
            .delay
            .latest_confident
            .is_none()
    );
    wait_until_running(&mut harness);
}

#[test]
fn stopping_clears_the_displayed_measurement() {
    let mut harness = harness(file_options(Theme::Night, true));
    wait_for_stable_measurement(&mut harness);
    harness.key_press(Key::Space);
    harness.step();
    assert!(!harness.state().measurement().is_running());
    assert_eq!(harness.state().measurement().snapshot().sequence, 0);
}

#[test]
fn s_captures_a_trace_and_command_z_undoes_it() {
    let mut harness = harness(file_options(Theme::Night, true));
    wait_for_stable_measurement(&mut harness);
    harness.key_press(Key::S);
    harness.step();
    assert_eq!(harness.state().session().traces.len(), 1);
    assert!(
        harness.state().session().traces[0]
            .name
            .starts_with("Trace 1 — ")
    );
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.step();
    assert!(harness.state().session().traces.is_empty());
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z);
    harness.step();
    assert_eq!(harness.state().session().traces.len(), 1);
}

#[test]
fn session_is_saved_automatically_and_restored_on_next_launch() {
    let directory =
        std::env::temp_dir().join(format!("soundtuning-session-{}", std::process::id()));
    let path = directory.join("session.json");
    let mut options = file_options(Theme::Night, true);
    options.session_store = soundtuning::session::SessionStore::File(path.clone());
    let mut harness = harness(options.clone());
    wait_for_stable_measurement(&mut harness);
    harness.key_press(Key::S);
    harness.step();
    step_until(&mut harness, Duration::from_secs(10), |_| path.exists());
    drop(harness);
    options.start_measurement = false;
    let restored = common::harness(options);
    assert_eq!(restored.state().session().traces.len(), 1);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn csv_export_writes_every_trace() {
    let mut options = file_options(Theme::Night, true);
    let export_directory =
        std::env::temp_dir().join(format!("soundtuning-csv-{}", std::process::id()));
    options.export_directory = export_directory.clone();
    let mut harness = harness(options);
    wait_for_stable_measurement(&mut harness);
    harness.key_press(Key::S);
    harness.step();
    harness
        .state_mut()
        .handle_action(soundtuning::actions::Action::ExportSession);
    let files: Vec<_> = std::fs::read_dir(&export_directory)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(files.len(), 1);
    let csv = std::fs::read_to_string(files[0].path()).unwrap();
    assert!(csv.lines().next().unwrap().contains("Trace 1"));
    assert_eq!(csv.lines().count(), 958);
    std::fs::remove_dir_all(export_directory).unwrap();
}

fn capture_two_traces(
    harness: &mut egui_kittest::Harness<'static, soundtuning::app::SoundTuningApp>,
) {
    wait_for_stable_measurement(harness);
    harness.key_press(Key::S);
    harness.step();
    harness.key_press(Key::S);
    harness.step();
    assert_eq!(harness.state().session().traces.len(), 2);
}

#[test]
fn trace_rows_select_average_rename_and_delete() {
    let mut harness = harness(file_options(Theme::Night, true));
    capture_two_traces(&mut harness);
    harness.run_steps(4);

    for checkbox in harness.get_all_by_label_contains(soundtuning::text::TRACE_SELECT) {
        checkbox.click();
    }
    harness.run_steps(4);
    assert!(
        harness
            .state()
            .session()
            .traces
            .iter()
            .all(|trace| trace.selected)
    );

    harness
        .get_by_label(soundtuning::text::TRACES_AVERAGE)
        .click();
    harness.run_steps(4);
    assert_eq!(harness.state().session().traces.len(), 3);
    let average_id = harness.state().session().traces[2].id;
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run_steps(4);
    assert_eq!(harness.state().session().traces.len(), 2);

    let first_select = format!(
        "{} — {}",
        soundtuning::text::TRACE_SELECT,
        harness.state().session().traces[1].name
    );
    harness.get_by_label(&first_select).click();
    harness.run_steps(4);
    harness.key_press(Key::F2);
    harness.run_steps(4);
    harness.get_by_role(Role::TextInput).type_text(" façade");

    harness.key_press(Key::Enter);
    harness.run_steps(4);
    let renamed = &harness.state().session().traces[0];
    assert!(renamed.name.ends_with(" façade"), "{}", renamed.name);
    assert_ne!(renamed.id, average_id);

    harness.key_press(Key::F2);
    harness.run_steps(4);
    harness.get_by_role(Role::TextInput).type_text(" oublié");
    harness.key_press(Key::Escape);
    harness.run_steps(4);
    assert!(
        harness.state().session().traces[0]
            .name
            .ends_with(" façade")
    );

    let delete_label = format!(
        "{} — {}",
        soundtuning::text::TRACE_DELETE,
        harness.state().session().traces[1].name
    );
    harness.get_by_label(&delete_label).click();
    harness.run_steps(4);
    assert_eq!(harness.state().session().traces.len(), 1);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run_steps(4);
    assert_eq!(harness.state().session().traces.len(), 2);
}

#[test]
fn offset_edits_shift_the_trace_and_are_undoable() {
    let mut harness = harness(file_options(Theme::Night, true));
    capture_two_traces(&mut harness);
    let id = harness.state().session().traces[0].id;
    harness.state_mut().apply_test_offset(id, 4.5);
    assert_eq!(harness.state().session().trace(id).unwrap().offset_db, 4.5);
    let level = harness.state().session().trace(id).unwrap().magnitude_db[400].unwrap();
    let displayed = harness.state().session().trace(id).unwrap().displayed_db()[400].unwrap();
    assert!((displayed - level - 4.5).abs() < 1.0e-12);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run_steps(4);
    assert_eq!(harness.state().session().trace(id).unwrap().offset_db, 0.0);
}

#[test]
fn failed_autosave_is_reported_once_and_retried() {
    let directory =
        std::env::temp_dir().join(format!("soundtuning-save-failure-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let blocker = directory.join("blocker");
    std::fs::write(&blocker, b"").unwrap();
    let mut options = file_options(Theme::Night, true);
    options.session_store = soundtuning::session::SessionStore::File(blocker.join("session.json"));
    let mut harness = harness(options);
    wait_for_stable_measurement(&mut harness);
    harness.key_press(Key::S);
    harness.step();
    step_until(&mut harness, Duration::from_secs(10), |app| {
        app.notification_texts()
            .iter()
            .any(|text| text == soundtuning::text::NOTIFY_SESSION_SAVE_FAILED)
    });
    assert!(harness.state().is_session_save_pending());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn unreadable_session_is_reported_at_launch() {
    let directory =
        std::env::temp_dir().join(format!("soundtuning-corrupt-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("session.json");
    std::fs::write(&path, b"{ broken").unwrap();
    let mut options = file_options(Theme::Night, false);
    options.session_store = soundtuning::session::SessionStore::File(path.clone());
    let mut harness = harness(options);
    harness.step();
    assert!(
        harness
            .state()
            .notification_texts()
            .iter()
            .any(|text| text == soundtuning::text::NOTIFY_SESSION_CORRUPT_EMPTY)
    );
    assert!(!path.exists());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn delay_analysis_offers_pos_neg_flat_custom_and_shift_drag_picks_a_custom_range() {
    use soundtuning::alignment_state::{AlignmentChoice, AlignmentRange};
    use soundtuning::ui::alignment_panel::AlignmentCommand;
    use soundtuning::ui::side_panel::SideTab;
    let mut harness = harness(file_options(Theme::Night, true));
    capture_two_traces(&mut harness);
    harness.state_mut().set_side_tab(SideTab::Alignment);
    let ids: Vec<u64> = harness
        .state()
        .session()
        .traces
        .iter()
        .map(|trace| trace.id)
        .collect();
    let mut state = harness.state().session().alignment.clone();
    state.reference_trace = Some(ids[0]);
    state.delayed_trace = Some(ids[1]);
    harness
        .state_mut()
        .apply_test_alignment(AlignmentCommand::Update(state));
    harness.run_steps(2);
    let alignment = harness.state().session().alignment.clone();
    let solutions = alignment.solutions.expect("solutions");
    assert!(
        solutions.positive.delay_ms.abs() < 0.05,
        "same signal twice: {solutions:?}"
    );
    assert!(!solutions.positive.inverted && solutions.negative.inverted);
    assert_eq!(alignment.choice, AlignmentChoice::Positive);
    assert_eq!(alignment.range, AlignmentRange::Low);
    for label in [
        "Pos",
        "Neg",
        "Flat",
        "Custom",
        "Range Low",
        "Range Mid",
        "Range Full",
        "Range Custom",
    ] {
        harness.get_by_label(label);
    }

    let mut custom = alignment.clone();
    custom.custom_delay_ms = 3.0;
    custom.choice = AlignmentChoice::Custom;
    harness
        .state_mut()
        .apply_test_alignment(AlignmentCommand::Update(custom));
    harness.run_steps(2);
    assert_eq!(
        harness.state().session().alignment.selected_setting(),
        Some((3.0, false))
    );

    shift_drag(&mut harness, 200.0, 420.0);
    let band = harness.state().session().alignment.clone();
    let (low_hz, high_hz) = band.band_hz();
    assert_eq!(band.range, AlignmentRange::Custom);
    assert!(low_hz > 30.0 && low_hz < 80.0, "{band:?}");
    assert!(high_hz > 120.0 && high_hz < 300.0, "{band:?}");
    assert_eq!(
        harness.state().plot_view(),
        soundtuning::plot::PlotView::default()
    );
    let first_band = band.band_hz();

    shift_drag(&mut harness, 600.0, 900.0);
    let second = harness.state().session().alignment.clone();
    assert!(second.band_hz().0 > 150.0, "{second:?}");

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run_steps(2);
    assert_eq!(harness.state().session().alignment.band_hz(), first_band);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run_steps(2);
    assert_eq!(
        harness.state().session().alignment.range,
        AlignmentRange::Low
    );
}

fn shift_drag(
    harness: &mut egui_kittest::Harness<'static, soundtuning::app::SoundTuningApp>,
    from_x: f32,
    to_x: f32,
) {
    let y = 400.0;
    harness.event(Event::ModifiersChanged(Modifiers::SHIFT));
    harness.hover_at(eframe::egui::pos2(from_x, y));
    harness.step();
    harness.event(Event::PointerButton {
        pos: eframe::egui::pos2(from_x, y),
        button: eframe::egui::PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::SHIFT,
    });
    harness.step();
    for step in 1..=3 {
        let x = from_x + (to_x - from_x) * step as f32 / 3.0;
        harness.hover_at(eframe::egui::pos2(x, y));
        harness.step();
    }
    harness.event(Event::PointerButton {
        pos: eframe::egui::pos2(to_x, y),
        button: eframe::egui::PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::SHIFT,
    });
    harness.event(Event::ModifiersChanged(Modifiers::NONE));
    harness.run_steps(3);
}

#[test]
fn t_toggles_tracking_and_shift_arrows_nudge_the_delay() {
    let mut harness = harness(file_options(Theme::Night, true));
    wait_for_stable_measurement(&mut harness);
    assert!(!harness.state().config().delay_tracking);
    harness.key_press(Key::T);
    harness.step();
    assert!(harness.state().config().delay_tracking);
    harness.key_press(Key::T);
    harness.step();
    assert!(!harness.state().config().delay_tracking);

    let before = harness.state().measurement().snapshot().delay.applied_ms;
    harness.key_press_modifiers(Modifiers::SHIFT, Key::ArrowRight);
    harness.step();
    step_until(&mut harness, SHORT_TIMEOUT, |app| {
        (app.measurement().snapshot().delay.applied_ms - before - 0.01).abs() < 1.0e-6
    });
    harness
        .get_by_label(soundtuning::text::ACTION_NUDGE_DELAY_DOWN)
        .click();
    harness.step();
    step_until(&mut harness, SHORT_TIMEOUT, |app| {
        (app.measurement().snapshot().delay.applied_ms - before).abs() < 1.0e-6
    });
}

#[test]
fn a_changed_delay_raises_a_notification_that_applies_it() {
    let mut harness = harness(file_options(Theme::Night, true));
    wait_for_stable_measurement(&mut harness);
    harness
        .state()
        .measurement()
        .send(EngineCommand::SetCompensationMs(0.0));
    step_until(&mut harness, SHORT_TIMEOUT, |app| {
        app.notification_texts()
            .iter()
            .any(|text| text.starts_with(soundtuning::text::NOTIFY_DELAY_DRIFT))
    });
    harness.key_press(Key::A);
    harness.step();
    step_until(&mut harness, SHORT_TIMEOUT, |app| {
        (app.measurement().snapshot().delay.applied_ms - 2.0).abs() < 0.05
    });
}

#[test]
fn eq_filters_are_added_dragged_scrolled_exported_and_undone() {
    let mut options = file_options(Theme::Night, true);
    let export_directory =
        std::env::temp_dir().join(format!("soundtuning-eq-{}", std::process::id()));
    options.export_directory = export_directory.clone();
    let mut harness = harness(options);
    wait_for_stable_measurement(&mut harness);

    harness.key_press(Key::E);
    harness.run_steps(3);
    assert_eq!(
        harness.state().side_tab(),
        soundtuning::ui::side_panel::SideTab::Eq
    );
    assert_eq!(harness.state().session().eq.filters().len(), 1);
    let filter = harness.state().session().eq.filters()[0];
    assert_eq!(filter.frequency_hz, 1000.0);

    let rect = harness.state().last_plot_rect().unwrap();
    let view = harness.state().plot_view();
    let handle = view.to_screen(rect, filter.frequency_hz, filter.gain_db);
    let target = view.to_screen(rect, 2000.0, -6.0);
    harness.hover_at(handle);
    harness.step();
    harness.drag_at(handle);
    harness.step();
    for step in 1..=4 {
        harness.hover_at(handle + (target - handle) * (step as f32 / 4.0));
        harness.step();
    }
    harness.drop_at(target);
    harness.run_steps(3);
    let moved = harness.state().session().eq.filters()[0];
    assert!(
        (moved.frequency_hz / 2000.0 - 1.0).abs() < 0.05,
        "{moved:?}"
    );
    assert!((moved.gain_db + 6.0).abs() < 0.5, "{moved:?}");
    assert_eq!(harness.state().plot_view(), view);

    let moved_handle = view.to_screen(rect, moved.frequency_hz, moved.gain_db);
    harness.hover_at(moved_handle);
    harness.step();
    harness.event(Event::MouseWheel {
        unit: eframe::egui::MouseWheelUnit::Point,
        delta: eframe::egui::vec2(0.0, 120.0),
        phase: eframe::egui::TouchPhase::Move,
        source: eframe::egui::MouseWheelSource::Wheel,
        modifiers: Modifiers::NONE,
    });
    harness.run_steps(6);
    assert!(harness.state().session().eq.filters()[0].q > moved.q);
    assert_eq!(harness.state().plot_view(), view);

    harness
        .state_mut()
        .apply_test_eq(soundtuning::ui::eq_panel::EqCommand::ExportText);
    harness
        .state_mut()
        .apply_test_eq(soundtuning::ui::eq_panel::EqCommand::ExportJson);
    let mut names: Vec<String> = std::fs::read_dir(&export_directory)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert!(names.iter().any(|name| name.ends_with(".txt")), "{names:?}");
    assert!(
        names.iter().any(|name| name.ends_with(".json")),
        "{names:?}"
    );

    harness.remove_cursor();
    harness.run_steps(2);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run_steps(2);
    let after_undo = harness.state().session().eq.filters()[0];
    assert!((after_undo.q - moved.q).abs() < 1.0e-9, "{after_undo:?}");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run_steps(2);
    assert_eq!(
        harness.state().session().eq.filters()[0].frequency_hz,
        1000.0
    );
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run_steps(2);
    assert!(harness.state().session().eq.filters().is_empty());
    std::fs::remove_dir_all(export_directory).unwrap();
}

#[test]
fn double_click_on_the_plot_adds_an_eq_filter_in_the_eq_tab() {
    let mut harness = common::fast_clock_harness(file_options(Theme::Night, false));
    harness
        .state_mut()
        .set_side_tab(soundtuning::ui::side_panel::SideTab::Eq);
    harness.run_steps(2);
    let rect = harness.state().last_plot_rect().unwrap();
    let view = harness.state().plot_view();
    let position = view.to_screen(rect, 60.0, 8.0);
    harness.hover_at(position);
    harness.step();
    for _ in 0..2 {
        harness.drag_at(position);
        harness.event(Event::PointerButton {
            pos: position,
            button: eframe::egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        });
        harness.step();
    }
    harness.run_steps(2);
    let filters = harness.state().session().eq.filters().to_vec();
    assert_eq!(filters.len(), 1, "{filters:?}");
    assert!((filters[0].frequency_hz / 60.0 - 1.0).abs() < 0.02);
    assert!((filters[0].gain_db - 8.0).abs() < 0.2);

    let handle = view.to_screen(rect, filters[0].frequency_hz, filters[0].gain_db);
    harness.hover_at(handle);
    harness.step();
    for _ in 0..2 {
        harness.drag_at(handle);
        harness.event(Event::PointerButton {
            pos: handle,
            button: eframe::egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        });
        harness.step();
    }
    harness.run_steps(2);
    assert_eq!(harness.state().session().eq.filters().len(), 1);
    assert_eq!(harness.state().session().eq.selected_filter, Some(0));
}

#[test]
fn every_control_has_an_accessible_label_in_every_tab() {
    use egui_kittest::kittest::NodeT;
    use soundtuning::ui::side_panel::SideTab;
    let mut harness = harness(file_options(Theme::Night, true));
    capture_two_traces(&mut harness);
    harness.key_press(Key::E);
    harness.run_steps(3);
    for tab in [SideTab::Traces, SideTab::Alignment, SideTab::Eq] {
        harness.state_mut().set_side_tab(tab);
        harness.run_steps(3);
        let mut checked = 0;
        for role in [
            Role::Button,
            Role::CheckBox,
            Role::SpinButton,
            Role::ComboBox,
        ] {
            for node in harness.query_all_by_role(role) {
                let label = node.accesskit_node().label().unwrap_or_default();
                assert!(
                    !label.trim().is_empty(),
                    "{tab:?}: unlabeled {role:?} value={:?}",
                    node.accesskit_node().value()
                );
                checked += 1;
            }
        }
        assert!(checked > 20, "{tab:?}: only {checked} controls");
    }
}

#[test]
fn command_comma_opens_audio_settings_and_apply_saves_the_devices() {
    use soundtuning::ui::audio_panel::AudioCommand;
    let mut harness = harness(file_options(Theme::Night, false));
    harness.step();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run_steps(2);
    assert!(harness.state().is_audio_settings_open());
    harness.get_by_role_and_label(Role::ComboBox, soundtuning::text::AUDIO_INPUT_DEVICE);
    harness.get_by_role_and_label(Role::ComboBox, soundtuning::text::AUDIO_OUTPUT_DEVICE);
    harness.get_by_role_and_label(Role::ComboBox, soundtuning::text::AUDIO_SAMPLE_RATE);

    let draft = {
        let panel = harness.state_mut().audio_settings_mut().unwrap();
        panel.select_input("coreaudio:UAX22");
        panel.draft.measurement_channel_index = 0;
        panel.draft.generator.output_channel_indices = vec![1];
        panel.draft.clone()
    };
    harness.run_steps(2);
    harness.get_by_label(soundtuning::text::AUDIO_APPLY).click();
    harness.run_steps(2);
    let stream = &harness.state().config().stream;
    assert_eq!(stream.input_device_id.as_deref(), Some("coreaudio:UAX22"));
    assert_eq!(stream.output_device_id, None);
    assert_eq!(stream.generator.output_channel_indices, vec![1]);
    assert_eq!(*stream, draft);
    assert!(
        harness
            .state()
            .notification_texts()
            .iter()
            .any(|text| text == soundtuning::text::NOTIFY_AUDIO_APPLIED)
    );

    harness.state_mut().apply_test_audio(AudioCommand::Close);
    harness.run_steps(2);
    assert!(!harness.state().is_audio_settings_open());
    harness
        .get_by_label(soundtuning::text::BUTTON_AUDIO_SETTINGS)
        .click();
    harness.run_steps(2);
    assert!(harness.state().is_audio_settings_open());
}

#[test]
fn auto_eq_fills_the_bank_within_the_limits_and_is_undoable() {
    use soundtuning::ui::eq_panel::EqCommand;
    let mut harness = harness(file_options(Theme::Night, true));
    wait_for_stable_measurement(&mut harness);
    harness.key_press(Key::F);
    harness.step();
    harness
        .state_mut()
        .set_side_tab(soundtuning::ui::side_panel::SideTab::Eq);
    let mut state = harness.state().session().eq.clone();
    state.auto.max_filters = 4;
    state.auto.max_boost_db = 2.0;
    state.auto.max_cut_db = 6.0;
    harness.state_mut().apply_test_eq(EqCommand::Update(state));
    harness.state_mut().apply_test_eq(EqCommand::AutoEq);
    harness.run_steps(2);
    let filters = harness.state().session().eq.filters().to_vec();
    assert!(!filters.is_empty() && filters.len() <= 4, "{filters:?}");
    assert!(
        filters
            .iter()
            .all(|filter| filter.gain_db <= 2.0 + 1.0e-9 && filter.gain_db >= -6.0 - 1.0e-9)
    );
    let bell_near_1k = filters
        .iter()
        .any(|filter| (filter.frequency_hz / 1000.0 - 1.0).abs() < 0.2 && filter.gain_db < -2.0);
    assert!(bell_near_1k, "{filters:?}");
    assert!(
        harness
            .state()
            .notification_texts()
            .iter()
            .any(|text| text.starts_with("Auto EQ : "))
    );
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run_steps(2);
    assert!(harness.state().session().eq.filters().is_empty());
}

#[test]
fn view_shortcuts_switch_and_stack_the_plots() {
    use soundtuning::plot::ViewKind;
    let mut harness = harness(file_options(Theme::Night, false));
    harness.step();
    assert_eq!(harness.state().plot_layout().main, ViewKind::Magnitude);
    harness.key_press(Key::H);
    harness.step();
    assert_eq!(harness.state().plot_layout().main, ViewKind::Phase);
    harness.get_by_role_and_label(Role::Figure, soundtuning::text::PHASE_PLOT_ACCESSIBLE_LABEL);
    harness.key_press(Key::I);
    harness.step();
    assert_eq!(harness.state().plot_layout().main, ViewKind::Impulse);
    harness.get_by_role_and_label(
        Role::Figure,
        soundtuning::text::IMPULSE_PLOT_ACCESSIBLE_LABEL,
    );
    harness.key_press(Key::V);
    harness.step();
    let layout = harness.state().plot_layout();
    assert_eq!(layout.upper, Some(ViewKind::Magnitude));
    harness.get_by_role_and_label(Role::Figure, soundtuning::text::PLOT_ACCESSIBLE_LABEL);
    harness.get_by_role_and_label(
        Role::Figure,
        soundtuning::text::IMPULSE_PLOT_ACCESSIBLE_LABEL,
    );
    harness.key_press(Key::M);
    harness.step();
    let layout = harness.state().plot_layout();
    assert_eq!(layout.main, ViewKind::Magnitude);
    assert_eq!(layout.upper, Some(ViewKind::Impulse));
    harness
        .get_by_label(soundtuning::text::VIEW_STACKED)
        .click();
    harness.step();
    assert_eq!(harness.state().plot_layout().upper, None);
    harness.get_by_label(soundtuning::text::VIEW_PHASE).click();
    harness.step();
    assert_eq!(harness.state().plot_layout().main, ViewKind::Phase);
    assert_eq!(harness.state().config().plot_layout.main, ViewKind::Phase);
}

#[test]
fn fullscreen_hides_the_side_panel_and_the_bottom_bar() {
    let mut harness = harness(file_options(Theme::Night, false));
    harness.step();
    assert!(
        harness
            .query_by_label(soundtuning::text::BUTTON_AUDIO_SETTINGS)
            .is_some()
    );
    harness.key_press_modifiers(Modifiers::SHIFT, Key::F);
    harness.run_steps(2);
    assert!(harness.state().is_plot_fullscreen());
    assert!(
        harness
            .query_by_label(soundtuning::text::BUTTON_AUDIO_SETTINGS)
            .is_none()
    );
    harness
        .get_by_label(soundtuning::text::VIEW_EXIT_FULLSCREEN)
        .click();
    harness.run_steps(2);
    assert!(!harness.state().is_plot_fullscreen());
    assert!(
        harness
            .query_by_label(soundtuning::text::BUTTON_AUDIO_SETTINGS)
            .is_some()
    );
    harness.key_press_modifiers(Modifiers::SHIFT, Key::F);
    harness.run_steps(2);
    assert!(harness.state().is_plot_fullscreen());
    harness.key_press(Key::Escape);
    harness.run_steps(2);
    assert!(!harness.state().is_plot_fullscreen());
}

#[test]
fn a_captured_trace_changes_definition_and_the_change_is_undoable() {
    use soundtuning::ui::traces_panel::TraceCommand;
    let mut harness = harness(file_options(Theme::Night, true));
    wait_for_stable_measurement(&mut harness);
    harness.key_press(Key::S);
    harness.step();
    let trace = harness.state().session().traces[0].clone();
    assert!(trace.can_change_smoothing());
    assert!(trace.impulse.is_some());
    let stored_bytes = serde_json::to_string(&trace).unwrap().len();
    println!("trace JSON: {stored_bytes} bytes");
    assert!(stored_bytes < MAX_TRACE_JSON_BYTES, "{stored_bytes}");
    let captured_fraction = trace.metadata.smoothing_fraction;
    let target_fraction = if captured_fraction == 3 { 12 } else { 3 };
    harness
        .state_mut()
        .apply_test_trace_command(TraceCommand::SetSmoothing(trace.id, target_fraction));
    harness.step();
    let changed = harness.state().session().traces[0].clone();
    assert_eq!(changed.metadata.smoothing_fraction, target_fraction);
    assert_ne!(changed.magnitude_db, trace.magnitude_db);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.step();
    let restored = &harness.state().session().traces[0];
    assert_eq!(restored.metadata.smoothing_fraction, captured_fraction);
    assert_eq!(restored.magnitude_db, trace.magnitude_db);
    harness.get_by_role_and_label(
        Role::ComboBox,
        &soundtuning::text::trace_control_label(soundtuning::text::TRACE_SMOOTHING, &trace.name),
    );
}

fn update_options(source: soundtuning::update::UpdateSource) -> soundtuning::app::AppOptions {
    soundtuning::app::AppOptions {
        updates: source,
        ..file_options(Theme::Night, false)
    }
}

fn newer_release() -> soundtuning::update::Release {
    soundtuning::update::Release {
        version: soundtuning::update::Version::new(99, 0, 0),
        page_url: "https://github.com/Fofonew/SoundTuning/releases/tag/v99.0.0".to_owned(),
    }
}

fn opened_urls(
    harness: &egui_kittest::Harness<'static, soundtuning::app::SoundTuningApp>,
) -> Vec<String> {
    harness
        .output()
        .platform_output
        .commands
        .iter()
        .filter_map(|command| match command {
            eframe::egui::OutputCommand::OpenUrl(open) => Some(open.url.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_newer_release_shows_a_banner_that_opens_the_download_page() {
    use soundtuning::update::UpdateSource;
    let mut harness = harness(update_options(UpdateSource::Fixed(Some(newer_release()))));
    harness.step();
    assert_eq!(harness.state().available_update(), Some(&newer_release()));
    harness.step();
    assert!(
        harness
            .query_by_label_contains("SoundTuning 99.0.0 est disponible")
            .is_some()
    );
    harness.get_by_label("Télécharger").click();
    harness.step();
    assert_eq!(opened_urls(&harness), vec![newer_release().page_url]);
    assert_eq!(harness.state().available_update(), None);
    harness.step();
    assert!(harness.query_by_label_contains("99.0.0").is_none());
}

#[test]
fn later_hides_the_update_banner_without_opening_anything() {
    use soundtuning::update::UpdateSource;
    let mut harness = harness(update_options(UpdateSource::Fixed(Some(newer_release()))));
    harness.step();
    harness.step();
    harness.get_by_label("Plus tard").click();
    harness.step();
    assert!(opened_urls(&harness).is_empty());
    assert_eq!(harness.state().available_update(), None);
}

#[test]
fn offline_startup_is_silent_and_a_manual_check_explains_it() {
    use soundtuning::actions::Action;
    use soundtuning::update::UpdateSource;
    let mut harness = harness(update_options(UpdateSource::Offline));
    harness.step();
    harness.step();
    assert!(harness.state().notification_texts().is_empty());
    assert_eq!(harness.state().available_update(), None);
    harness.state_mut().handle_action(Action::CheckForUpdates);
    harness.step();
    let texts = harness.state().notification_texts();
    assert!(
        texts.iter().any(|text| text.contains("hors connexion")),
        "{texts:?}"
    );
}

#[test]
fn a_manual_check_reports_an_up_to_date_version() {
    use soundtuning::actions::Action;
    use soundtuning::update::UpdateSource;
    let mut harness = harness(update_options(UpdateSource::Fixed(None)));
    harness.step();
    assert!(harness.state().notification_texts().is_empty());
    harness.state_mut().handle_action(Action::CheckForUpdates);
    harness.step();
    let expected = format!("SoundTuning {} est à jour.", env!("CARGO_PKG_VERSION"));
    assert!(harness.state().notification_texts().contains(&expected));
}

#[test]
fn bars_and_audio_settings_do_not_overlap_in_small_windows() {
    use eframe::egui;
    use egui_kittest::kittest::NodeT;
    use soundtuning::design::tokens::size;
    for (width, height) in [(1024.0, 600.0), (1280.0, 680.0), (1440.0, 900.0)] {
        let options = file_options(Theme::Night, true);
        let mut harness = egui_kittest::Harness::builder()
            .with_size(egui::vec2(width, height))
            .with_max_steps(64)
            .build_eframe(move |creation_context| {
                soundtuning::app::SoundTuningApp::new(&creation_context.egui_ctx, options)
            });
        wait_until_running(&mut harness);
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
        harness.step();
        harness.step();
        let top_limit = soundtuning::app::top_bar_height();
        let bottom_limit = height - size::BOTTOM_BAR_HEIGHT;
        let mut rects: Vec<(String, egui::Rect)> = Vec::new();
        for role in [Role::Button, Role::SpinButton, Role::Label, Role::CheckBox] {
            for node in harness.query_all_by_role(role) {
                let rect = node.rect();
                let in_bars = rect.max.y <= top_limit || rect.min.y >= bottom_limit;
                let label = node.accesskit_node().label().unwrap_or_default();
                let in_audio_footer =
                    ["Appliquer les réglages", "Actualiser"].contains(&label.as_str());
                if (in_bars || in_audio_footer) && rect.area() > 0.0 {
                    rects.push((format!("{role:?} {label}"), rect));
                }
            }
        }
        for (index, (first, first_rect)) in rects.iter().enumerate() {
            assert!(
                first_rect.max.x <= width + 0.5,
                "{first} leaves the window at {width}"
            );
            for (second, second_rect) in &rects[index + 1..] {
                let overlap = first_rect.intersect(*second_rect);
                assert!(
                    overlap.width() <= 0.5 || overlap.height() <= 0.5,
                    "{first} overlaps {second} at {width}×{height}"
                );
            }
        }
    }
}
