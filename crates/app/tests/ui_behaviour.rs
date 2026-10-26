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
