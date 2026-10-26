use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioIoError {
    #[error("audio backend error: {0}")]
    Backend(#[from] cpal::Error),
    #[error("audio device not found: {0}")]
    DeviceNotFound(String),
    #[error("device has no {direction} channel {channel_index} (it has {available})")]
    ChannelOutOfRange {
        direction: &'static str,
        channel_index: usize,
        available: usize,
    },
    #[error("no {direction} configuration supports {sample_rate_hz} Hz")]
    UnsupportedSampleRate {
        direction: &'static str,
        sample_rate_hz: u32,
    },
    #[error("unsupported sample format: {0}")]
    UnsupportedSampleFormat(String),
    #[error("the noise generator needs at least one output channel")]
    NoGeneratorChannel,
    #[error("internal reference mode needs an output device with at least one channel")]
    MissingGeneratorOutput,
    #[error("wav file {path}: {source}")]
    Wav {
        path: PathBuf,
        #[source]
        source: hound::Error,
    },
    #[error("wav file {path} has {channels} channel(s); reference and measurement need 2")]
    WavChannelCount { path: PathBuf, channels: u16 },
    #[error("wav file {path} is empty")]
    WavEmpty { path: PathBuf },
    #[error("could not start file playback thread: {0}")]
    PlaybackThread(#[source] std::io::Error),
}
