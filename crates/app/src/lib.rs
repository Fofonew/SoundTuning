pub mod cli;
pub mod commands;
pub mod gui;
pub mod text;

use clap::Parser;

use crate::cli::{Cli, Command};

pub fn run() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    let cli = Cli::parse();
    match cli.command {
        Some(Command::Devices) => commands::print_devices(),
        Some(Command::Record(arguments)) => commands::record(&arguments),
        None => gui::run_window(cli.input_file),
    }
}
