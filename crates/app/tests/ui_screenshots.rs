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
