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
