use std::path::Path;

use crate::error::AudioIoError;
use crate::frames::StereoFrame;

pub fn write_stereo_wav(
    path: &Path,
    sample_rate_hz: u32,
    frames: &[StereoFrame],
) -> Result<(), AudioIoError> {
    let wav_error = |source| AudioIoError::Wav {
        path: path.to_path_buf(),
        source,
    };
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: sample_rate_hz,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(path, spec).map_err(wav_error)?;
    for frame in frames {
        writer.write_sample(frame.reference).map_err(wav_error)?;
        writer.write_sample(frame.measurement).map_err(wav_error)?;
    }
    writer.finalize().map_err(wav_error)
}
