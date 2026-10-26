use serde::{Deserialize, Serialize};

use crate::generator::NoiseKind;

pub const DEFAULT_SAMPLE_RATE_HZ: u32 = 48_000;
const DEFAULT_GENERATOR_LEVEL_DBFS: f32 = -20.0;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum ReferenceMode {
    #[default]
    Internal,
    Loopback {
        reference_channel_index: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeneratorSettings {
    pub enabled: bool,
    pub kind: NoiseKind,
    pub level_dbfs: f32,
    pub output_channel_indices: Vec<usize>,
}

impl Default for GeneratorSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            kind: NoiseKind::Pink,
            level_dbfs: DEFAULT_GENERATOR_LEVEL_DBFS,
            output_channel_indices: vec![0],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StreamSettings {
    pub input_device_id: Option<String>,
    pub output_device_id: Option<String>,
    pub sample_rate_hz: u32,
    pub buffer_frames: Option<u32>,
    pub measurement_channel_index: usize,
    pub reference: ReferenceMode,
    pub generator: GeneratorSettings,
}

impl Default for StreamSettings {
    fn default() -> Self {
        Self {
            input_device_id: None,
            output_device_id: None,
            sample_rate_hz: DEFAULT_SAMPLE_RATE_HZ,
            buffer_frames: None,
            measurement_channel_index: 0,
            reference: ReferenceMode::Internal,
            generator: GeneratorSettings::default(),
        }
    }
}

impl StreamSettings {
    pub fn required_output_channels(&self) -> usize {
        self.generator
            .output_channel_indices
            .iter()
            .copied()
            .max()
            .unwrap_or(0)
            + 1
    }

    pub fn generator_channel_mask(&self) -> u64 {
        self.generator
            .output_channel_indices
            .iter()
            .filter(|index| **index < u64::BITS as usize)
            .fold(0_u64, |mask, index| mask | (1_u64 << index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_user_setup() {
        let settings = StreamSettings::default();
        assert_eq!(settings.sample_rate_hz, 48_000);
        assert_eq!(settings.measurement_channel_index, 0);
        assert_eq!(settings.reference, ReferenceMode::Internal);
        assert_eq!(settings.generator.kind, NoiseKind::Pink);
    }

    #[test]
    fn required_outputs_follow_the_highest_generator_channel() {
        let mut settings = StreamSettings::default();
        assert_eq!(settings.required_output_channels(), 1);
        settings.generator.output_channel_indices = vec![0, 3];
        assert_eq!(settings.required_output_channels(), 4);
    }

    #[test]
    fn channel_mask_covers_selected_outputs() {
        let mut settings = StreamSettings::default();
        settings.generator.output_channel_indices = vec![0, 1, 70];
        assert_eq!(settings.generator_channel_mask(), 0b11);
    }
}
