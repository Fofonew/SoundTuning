use cpal::traits::{DeviceTrait, HostTrait};

use crate::error::AudioIoError;

pub const STANDARD_SAMPLE_RATES_HZ: [u32; 4] = [44_100, 48_000, 88_200, 96_000];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub input_channel_count: usize,
    pub output_channel_count: usize,
    pub supported_sample_rates_hz: Vec<u32>,
    pub is_default_input: bool,
    pub is_default_output: bool,
}

impl DeviceInfo {
    pub fn supports_input(&self) -> bool {
        self.input_channel_count > 0
    }

    pub fn supports_output(&self) -> bool {
        self.output_channel_count > 0
    }
}

pub fn list_devices() -> Result<Vec<DeviceInfo>, AudioIoError> {
    let host = cpal::default_host();
    let default_input_id = host
        .default_input_device()
        .and_then(|device| device.id().ok());
    let default_output_id = host
        .default_output_device()
        .and_then(|device| device.id().ok());

    let mut devices = Vec::new();
    for device in host.devices()? {
        let Ok(id) = device.id() else {
            continue;
        };
        let name = device
            .description()
            .map(|description| description.name().to_owned())
            .unwrap_or_else(|_| id.to_string());
        let input_ranges: Vec<_> = device
            .supported_input_configs()
            .map(Iterator::collect)
            .unwrap_or_default();
        let output_ranges: Vec<_> = device
            .supported_output_configs()
            .map(Iterator::collect)
            .unwrap_or_default();
        let input_channel_count = max_channels(&input_ranges);
        let output_channel_count = max_channels(&output_ranges);
        let supported_sample_rates_hz = STANDARD_SAMPLE_RATES_HZ
            .into_iter()
            .filter(|rate| {
                input_ranges
                    .iter()
                    .chain(output_ranges.iter())
                    .any(|range| range.contains_rate(*rate))
            })
            .collect();
        devices.push(DeviceInfo {
            is_default_input: default_input_id.as_ref() == Some(&id),
            is_default_output: default_output_id.as_ref() == Some(&id),
            id: id.to_string(),
            name,
            input_channel_count,
            output_channel_count,
            supported_sample_rates_hz,
        });
    }
    Ok(devices)
}

fn max_channels(ranges: &[cpal::SupportedStreamConfigRange]) -> usize {
    ranges
        .iter()
        .map(|range| usize::from(range.channels()))
        .max()
        .unwrap_or(0)
}

pub(crate) fn find_device(
    host: &cpal::Host,
    device_id: Option<&str>,
    want_input: bool,
) -> Result<cpal::Device, AudioIoError> {
    match device_id {
        Some(id_text) => {
            let id: cpal::DeviceId = id_text.parse()?;
            host.device_by_id(&id)
                .ok_or_else(|| AudioIoError::DeviceNotFound(id_text.to_owned()))
        }
        None => {
            let device = if want_input {
                host.default_input_device()
            } else {
                host.default_output_device()
            };
            device.ok_or_else(|| {
                AudioIoError::DeviceNotFound(
                    if want_input {
                        "default input"
                    } else {
                        "default output"
                    }
                    .to_owned(),
                )
            })
        }
    }
}
