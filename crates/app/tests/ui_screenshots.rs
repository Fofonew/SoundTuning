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

fn measurement_screenshot(theme: Theme, name: &str) {
    let mut harness = rendering_harness(file_options(theme, true));
    wait_for_stable_measurement(&mut harness);
    harness.key_press(Key::F);
    harness.step();
    harness.hover_at(pos2(560.0, 300.0));
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
    harness.hover_at(pos2(700.0, 330.0));
    harness.step();
    harness.step();
    assert!(save_screenshot(&mut harness, "traces").exists());
}

#[test]
fn screenshot_alignment() {
    use soundtuning::ui::alignment_panel::AlignmentCommand;
    use soundtuning::ui::side_panel::SideTab;
    let mut harness = rendering_harness(file_options(Theme::Night, true));
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
    harness.state_mut().set_side_tab(SideTab::Alignment);
    let ids: Vec<u64> = harness
        .state()
        .session()
        .traces
        .iter()
        .map(|trace| trace.id)
        .collect();
    let mut state = harness.state().session().alignment.clone();
    state.trace_a = Some(ids[0]);
    state.trace_b = Some(ids[1]);
    harness
        .state_mut()
        .apply_test_alignment(AlignmentCommand::Update(state));
    harness.key_press(Key::F);
    harness.step();
    harness.key_press(Key::Num2);
    harness.step();
    harness.step();
    let proposal = harness
        .state()
        .session()
        .alignment
        .proposal_delay_ms
        .unwrap();
    assert!((proposal.abs() - 1.2).abs() < 0.05, "{proposal}");
    assert!(save_screenshot(&mut harness, "alignment").exists());
}

#[test]
fn screenshot_eq() {
    use soundtuning::eq_state::{StoredFilter, StoredFilterKind, StoredSlope};
    use soundtuning::ui::eq_panel::EqCommand;
    use soundtuning::ui::side_panel::SideTab;
    let mut harness = rendering_harness(file_options(Theme::Night, true));
    wait_for_stable_measurement(&mut harness);
    harness.key_press(Key::F);
    harness.step();
    harness.state_mut().set_side_tab(SideTab::Eq);
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
    assert!(save_screenshot(&mut harness, "eq").exists());
}
