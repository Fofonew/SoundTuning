pub mod actions;
pub mod alignment_state;
pub mod app;
pub mod cli;
pub mod commands;
pub mod config;
pub mod design;
pub mod device_choice;
pub mod eq_state;
pub mod measurement;
pub mod plot;
pub mod session;
pub mod text;
pub mod ui;
pub mod update;

use clap::Parser;

use crate::app::{AppOptions, SoundTuningApp};
use crate::cli::{Cli, Command};
use crate::measurement::{FilePacing, SourceSpec};

const INITIAL_WINDOW_SIZE: [f32; 2] = [1440.0, 900.0];
const MIN_WINDOW_SIZE: [f32; 2] = [1024.0, 600.0];
const WINDOW_ICON_PNG: &[u8] = include_bytes!("../assets/icon/soundtuning-256.png");

pub fn run() -> anyhow::Result<()> {
    attach_to_launching_terminal();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    let cli = Cli::parse();
    match cli.command {
        Some(Command::Devices) => commands::print_devices(),
        Some(Command::Record(arguments)) => commands::record(&arguments),
        None => run_window(cli.input_file),
    }
}

#[cfg(windows)]
fn attach_to_launching_terminal() {
    const ATTACH_PARENT_PROCESS: u32 = u32::MAX;
    let _launched_from_terminal =
        win32console::console::WinConsole::attach_console(ATTACH_PARENT_PROCESS).is_ok();
}

#[cfg(not(windows))]
fn attach_to_launching_terminal() {}

fn run_window(input_file: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    let mut options = AppOptions::live();
    if let Some(path) = input_file {
        options.source = SourceSpec::File {
            path,
            pacing: FilePacing::RealTime,
        };
        options.start_measurement = true;
    }
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_title(text::APP_NAME)
        .with_inner_size(INITIAL_WINDOW_SIZE)
        .with_min_inner_size(MIN_WINDOW_SIZE);
    match eframe::icon_data::from_png_bytes(WINDOW_ICON_PNG) {
        Ok(icon) => viewport = viewport.with_icon(icon),
        Err(error) => tracing::warn!("window icon not loaded: {error}"),
    }
    let native_options = eframe::NativeOptions {
        viewport,
        wgpu_options: wgpu_options(),
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        text::APP_NAME,
        native_options,
        Box::new(move |creation_context| {
            Ok(Box::new(SoundTuningApp::new(
                &creation_context.egui_ctx,
                options,
            )))
        }),
    )
    .map_err(|error| anyhow::anyhow!("window failed: {error}"))
}

fn wgpu_options() -> eframe::egui_wgpu::WgpuConfiguration {
    let mut options = eframe::egui_wgpu::WgpuConfiguration::default();
    let backend_forced_by_environment = eframe::wgpu::Backends::from_env().is_some();
    if cfg!(target_os = "windows")
        && !backend_forced_by_environment
        && let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = &mut options.wgpu_setup
    {
        setup.instance_descriptor.backends = eframe::wgpu::Backends::DX12;
    }
    options
}

#[cfg(test)]
mod tests {
    use super::WINDOW_ICON_PNG;

    const WINDOW_ICON_SIDE_PX: u32 = 256;

    #[test]
    fn window_icon_decodes_to_square_rgba() {
        let icon = eframe::icon_data::from_png_bytes(WINDOW_ICON_PNG).expect("icon decodes");
        assert_eq!(icon.width, WINDOW_ICON_SIDE_PX);
        assert_eq!(icon.height, WINDOW_ICON_SIDE_PX);
        assert_eq!(
            icon.rgba.len(),
            (WINDOW_ICON_SIDE_PX * WINDOW_ICON_SIDE_PX * 4) as usize
        );
    }
}
