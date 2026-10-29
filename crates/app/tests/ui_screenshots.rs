#![cfg(target_os = "macos")]
#![allow(clippy::unwrap_used)]

mod common;

use common::{
    file_options, first_launch_options, rendering_harness, save_screenshot,
    wait_for_stable_measurement,
};
use eframe::egui::{Key, pos2};
use egui_kittest::kittest::Queryable;
use soundtuning::design::Theme;

fn hover_plot(
    harness: &mut egui_kittest::Harness<'static, soundtuning::app::SoundTuningApp>,
    x_fraction: f32,
    y_fraction: f32,
) {
    harness.step();
    let rect = harness.state().last_plot_rect().unwrap();
    harness.hover_at(pos2(
        rect.left() + rect.width() * x_fraction,
        rect.top() + rect.height() * y_fraction,
    ));
}

fn measurement_screenshot(theme: Theme, name: &str) {
    let mut harness = rendering_harness(file_options(theme, true));
    wait_for_stable_measurement(&mut harness);
    harness.key_press(Key::F);
    harness.step();
    hover_plot(&mut harness, 0.55, 0.45);
    harness.step();
    harness.step();
    let path = save_screenshot(&mut harness, name);
    assert!(path.exists());
}

#[test]
fn screenshot_first_launch() {
    let mut harness = rendering_harness(first_launch_options());
    harness.step();
    harness.get_by_label("UAX 2-2").click();
    harness.step();
    harness.step();
    assert!(save_screenshot(&mut harness, "first_launch").exists());
}

#[test]
fn screenshot_measurement_night() {
    measurement_screenshot(Theme::Night, "measurement_night");
}

#[test]
fn screenshot_measurement_daylight() {
    measurement_screenshot(Theme::Daylight, "measurement_daylight");
}

#[test]
fn screenshot_traces() {
    let mut harness = rendering_harness(file_options(Theme::Night, true));
    wait_for_stable_measurement(&mut harness);
    for _ in 0..3 {
        harness.key_press(Key::S);
        harness.step();
    }
    let ids: Vec<u64> = harness
        .state()
        .session()
        .traces
        .iter()
        .map(|trace| trace.id)
        .collect();
    harness.state_mut().apply_test_offset(ids[1], -4.0);
    harness.state_mut().apply_test_offset(ids[2], 3.0);
    harness.key_press(Key::F);
    harness.step();
    hover_plot(&mut harness, 0.6, 0.4);
    harness.step();
    harness.step();
    assert!(save_screenshot(&mut harness, "traces").exists());
}

#[test]
fn screenshot_alignment() {
    alignment_screenshot(Theme::Night, "alignment");
}

#[test]
fn screenshot_alignment_daylight() {
    alignment_screenshot(Theme::Daylight, "alignment_daylight");
}

fn alignment_screenshot(theme: Theme, name: &str) {
    use soundtuning::plot::ExternalModule;
    use soundtuning::ui::alignment_panel::AlignmentCommand;
    let mut harness = rendering_harness(file_options(theme, true));
    wait_for_stable_measurement(&mut harness);
    harness.key_press(Key::S);
    harness.step();
    let applied_ms = harness.state().measurement().snapshot().delay.applied_ms;
    harness
        .state()
        .measurement()
        .send(engine::EngineCommand::SetCompensationMs(applied_ms - 1.2));
    harness
        .state()
        .measurement()
        .send(engine::EngineCommand::ResetAverages);
    common::step_until(&mut harness, std::time::Duration::from_secs(20), |app| {
        let snapshot = app.measurement().snapshot();
        (snapshot.delay.applied_ms - (applied_ms - 1.2)).abs() < 1.0e-6
            && snapshot.averages > 30
            && !snapshot.warming_up
    });
    harness.key_press(Key::S);
    harness.step();
    harness
        .state_mut()
        .show_module(ExternalModule::DelayAnalysis);
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
    harness.key_press(Key::F);
    harness.step();
    harness.key_press(Key::Num2);
    harness.step();
    harness.step();
    let solutions = harness.state().session().alignment.solutions.unwrap();
    let proposal = solutions.positive.delay_ms;
    assert!((proposal.abs() - 1.2).abs() < 0.05, "{solutions:?}");
    assert!(save_screenshot(&mut harness, name).exists());
}

#[test]
fn screenshot_eq() {
    eq_screenshot(Theme::Night, "eq");
}

#[test]
fn screenshot_eq_daylight() {
    eq_screenshot(Theme::Daylight, "eq_daylight");
}

fn eq_screenshot(theme: Theme, name: &str) {
    use soundtuning::eq_state::{StoredFilter, StoredFilterKind, StoredSlope};
    use soundtuning::plot::ExternalModule;
    use soundtuning::ui::eq_panel::EqCommand;
    let mut harness = rendering_harness(file_options(theme, true));
    wait_for_stable_measurement(&mut harness);
    harness.key_press(Key::F);
    harness.step();
    harness.state_mut().show_module(ExternalModule::VirtualEq);
    let mut state = harness.state().session().eq.clone();
    let mut bell = StoredFilter::new(StoredFilterKind::Bell, 1000.0, -5.0);
    bell.q = 1.4;
    let mut shelf = StoredFilter::new(StoredFilterKind::LowShelf, 120.0, -4.0);
    shelf.q = 0.7;
    let cut = StoredFilter::new(
        StoredFilterKind::LowCut(StoredSlope::LinkwitzRiley24),
        30.0,
        0.0,
    );
    for filter in [bell, shelf, cut] {
        state.add_filter(filter);
    }
    state.selected_filter = Some(0);
    harness.state_mut().apply_test_eq(EqCommand::Update(state));
    harness.step();
    harness.step();
    assert!(save_screenshot(&mut harness, name).exists());
}

#[test]
fn screenshot_audio_settings() {
    let mut harness = rendering_harness(file_options(Theme::Night, true));
    wait_for_stable_measurement(&mut harness);
    harness.key_press(Key::F);
    harness.step();
    harness.key_press_modifiers(eframe::egui::Modifiers::COMMAND, Key::Comma);
    harness.step();
    harness
        .state_mut()
        .audio_settings_mut()
        .unwrap()
        .select_input("coreaudio:UAX22");
    harness.step();
    harness.step();
    assert!(save_screenshot(&mut harness, "audio_settings").exists());
}

#[test]
fn screenshot_auto_eq() {
    use soundtuning::plot::ExternalModule;
    use soundtuning::ui::eq_panel::EqCommand;
    let mut harness = rendering_harness(file_options(Theme::Night, true));
    wait_for_stable_measurement(&mut harness);
    harness.key_press(Key::F);
    harness.step();
    harness.state_mut().show_module(ExternalModule::VirtualEq);
    harness.step();
    harness
        .get_by_label(soundtuning::text::AUTO_EQ_TITLE)
        .click();
    harness.step();
    harness.state_mut().apply_test_eq(EqCommand::AutoEq);
    harness.step();
    harness.step();
    assert!(save_screenshot(&mut harness, "auto_eq").exists());
}

fn captured_traces_harness(
    theme: Theme,
) -> egui_kittest::Harness<'static, soundtuning::app::SoundTuningApp> {
    let mut harness = rendering_harness(file_options(theme, true));
    wait_for_stable_measurement(&mut harness);
    harness.key_press(Key::S);
    harness.step();
    let id = harness.state().session().traces[0].id;
    harness.state_mut().apply_test_offset(id, -3.0);
    harness
}

#[test]
fn screenshot_phase_stacked() {
    let mut harness = captured_traces_harness(Theme::Night);
    harness.key_press(Key::F);
    harness.step();
    hover_plot(&mut harness, 0.6, 0.5);
    harness.step();
    harness.step();
    assert!(save_screenshot(&mut harness, "phase_stacked").exists());
}

#[test]
fn screenshot_zoom_selection() {
    let mut harness = captured_traces_harness(Theme::Night);
    harness.key_press(Key::F);
    harness.step();
    let rect = harness.state().last_plot_rect().unwrap();
    let view = harness.state().plot_view();
    let from = view.to_screen(rect, 80.0, 10.0);
    let to = view.to_screen(rect, 600.0, -12.0);
    harness.hover_at(from);
    harness.step();
    harness.event(eframe::egui::Event::PointerButton {
        pos: from,
        button: eframe::egui::PointerButton::Primary,
        pressed: true,
        modifiers: eframe::egui::Modifiers::NONE,
    });
    harness.step();
    for step in 1..=3 {
        harness.hover_at(from + (to - from) * step as f32 / 3.0);
        harness.step();
    }
    harness.step();
    assert!(save_screenshot(&mut harness, "zoom_selection").exists());
}

#[test]
fn screenshot_impulse() {
    let mut harness = captured_traces_harness(Theme::Night);
    harness.key_press(Key::V);
    harness.step();
    harness.key_press(Key::I);
    harness.step();
    harness.key_press(Key::F);
    harness.step();
    hover_plot(&mut harness, 0.5, 0.5);
    harness.step();
    harness.step();
    assert!(save_screenshot(&mut harness, "impulse").exists());
}

#[test]
fn screenshot_impulse_stacked_fullscreen_daylight() {
    let mut harness = captured_traces_harness(Theme::Daylight);
    harness.key_press(Key::I);
    harness.step();
    harness.key_press(Key::F);
    harness.step();
    harness.key_press_modifiers(eframe::egui::Modifiers::SHIFT, Key::F);
    harness.step();
    harness.step();
    assert!(harness.state().is_plot_fullscreen());
    assert!(save_screenshot(&mut harness, "impulse_stacked_fullscreen_daylight").exists());
}
