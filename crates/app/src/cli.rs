use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::text;

const DEFAULT_RECORD_SECONDS: f64 = 10.0;
const DEFAULT_SAMPLE_RATE_HZ: u32 = 48_000;
const DEFAULT_GENERATOR_LEVEL_DBFS: f32 = -20.0;

#[derive(Debug, Parser)]
#[command(name = "soundtuning", version, about = text::CLI_ABOUT)]
pub struct Cli {
    #[arg(long, help = text::CLI_INPUT_FILE_HELP)]
    pub input_file: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = text::CLI_DEVICES_ABOUT)]
    Devices,
    #[command(about = text::CLI_RECORD_ABOUT)]
    Record(RecordArguments),
}

#[derive(Debug, Args)]
pub struct RecordArguments {
    #[arg(long, default_value_t = DEFAULT_RECORD_SECONDS, help = text::CLI_SECONDS_HELP)]
    pub seconds: f64,
    #[arg(long, help = text::CLI_OUTPUT_HELP)]
    pub output: PathBuf,
    #[arg(long, help = text::CLI_DEVICE_HELP)]
    pub device: Option<String>,
    #[arg(long, help = text::CLI_OUTPUT_DEVICE_HELP)]
    pub output_device: Option<String>,
    #[arg(long, default_value_t = DEFAULT_SAMPLE_RATE_HZ, help = text::CLI_SAMPLE_RATE_HELP)]
    pub sample_rate: u32,
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u16).range(1..), help = text::CLI_MEASUREMENT_CHANNEL_HELP)]
    pub measurement_channel: u16,
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..), help = text::CLI_LOOPBACK_CHANNEL_HELP)]
    pub loopback_channel: Option<u16>,
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u16).range(1..), help = text::CLI_GENERATOR_CHANNEL_HELP)]
    pub generator_channel: u16,
    #[arg(long, default_value_t = DEFAULT_GENERATOR_LEVEL_DBFS, allow_negative_numbers = true, help = text::CLI_LEVEL_HELP)]
    pub level_dbfs: f32,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn cli_definition_is_consistent() {
        Cli::command().debug_assert();
    }

    #[test]
    fn record_defaults_to_ten_seconds_internal_reference() {
        let cli = Cli::parse_from(["soundtuning", "record", "--output", "out.wav"]);
        let Some(Command::Record(arguments)) = cli.command else {
            panic!("expected record command");
        };
        assert_eq!(arguments.seconds, 10.0);
        assert_eq!(arguments.measurement_channel, 1);
        assert_eq!(arguments.loopback_channel, None);
        assert_eq!(arguments.sample_rate, 48_000);
    }

    #[test]
    fn channel_zero_is_rejected() {
        let result = Cli::try_parse_from([
            "soundtuning",
            "record",
            "--output",
            "out.wav",
            "--measurement-channel",
            "0",
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn input_file_selects_file_mode() {
        let cli = Cli::parse_from(["soundtuning", "--input-file", "mesure.wav"]);
        assert_eq!(cli.input_file, Some(PathBuf::from("mesure.wav")));
        assert!(cli.command.is_none());
    }
}
