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

use clap::Parser;

use crate::app::{AppOptions, SoundTuningApp};
use crate::cli::{Cli, Command};
use crate::measurement::{FilePacing, SourceSpec};

const INITIAL_WINDOW_SIZE: [f32; 2] = [1440.0, 900.0];
const MIN_WINDOW_SIZE: [f32; 2] = [1280.0, 720.0];

pub fn run() -> anyhow::Result<()> {
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

fn run_window(input_file: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    let mut options = AppOptions::live();
    if let Some(path) = input_file {
        options.source = SourceSpec::File {
            path,
            pacing: FilePacing::RealTime,
        };
        options.start_measurement = true;
    }
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title(text::APP_NAME)
            .with_inner_size(INITIAL_WINDOW_SIZE)
            .with_min_inner_size(MIN_WINDOW_SIZE),
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
