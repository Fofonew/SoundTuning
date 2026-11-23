use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{SampleFormat, SupportedStreamConfigRange};

use crate::error::AudioIoError;
use crate::sample_rate::STANDARD_SAMPLE_RATES_HZ;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub is_default_driver: bool,
    pub input_channel_count: usize,
    pub output_channel_count: usize,
    pub input_sample_rates_hz: Vec<u32>,
    pub output_sample_rates_hz: Vec<u32>,
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

    pub fn sample_rates_hz(&self) -> Vec<u32> {
        STANDARD_SAMPLE_RATES_HZ
            .into_iter()
            .filter(|rate| {
                self.input_sample_rates_hz.contains(rate)
                    || self.output_sample_rates_hz.contains(rate)
            })
            .collect()
    }
}

pub fn list_devices() -> Result<Vec<DeviceInfo>, AudioIoError> {
    let default_host_id = cpal::default_host().id();
    let mut devices = Vec::new();
    for host_id in cpal::available_hosts() {
        let is_default_driver = host_id == default_host_id;
        let listed = cpal::host_from_id(host_id)
            .map_err(AudioIoError::from)
            .and_then(|host| list_host_devices(&host, is_default_driver));
        match listed {
            Ok(host_devices) => devices.extend(host_devices),
            Err(error) if is_default_driver => return Err(error),
            Err(error) => tracing::warn!(%error, host = host_id.name(), "audio driver skipped"),
        }
    }
    Ok(devices)
}

fn list_host_devices(
    host: &cpal::Host,
    is_default_driver: bool,
) -> Result<Vec<DeviceInfo>, AudioIoError> {
    let default_input_id = host
        .default_input_device()
        .and_then(|device| device.id().ok());
    let default_output_id = host
        .default_output_device()
        .and_then(|device| device.id().ok());
    let driver = host.id().name().to_owned();

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
        devices.push(DeviceInfo {
            is_default_input: is_default_driver && default_input_id.as_ref() == Some(&id),
            is_default_output: is_default_driver && default_output_id.as_ref() == Some(&id),
            id: id.to_string(),
            name,
            driver: driver.clone(),
            is_default_driver,
            input_channel_count: max_channels(&input_ranges),
            output_channel_count: max_channels(&output_ranges),
            input_sample_rates_hz: usable_sample_rates(&input_ranges, 1),
            output_sample_rates_hz: usable_sample_rates(&output_ranges, 1),
        });
    }
    Ok(devices)
}

const COREAUDIO_HOST: &str = "coreaudio";

pub fn shares_hardware(
    input_id: Option<&str>,
    input_name: &str,
    output_id: Option<&str>,
    output_name: &str,
) -> bool {
    if input_id.is_some() && input_id == output_id {
        return true;
    }
    if [input_id, output_id]
        .into_iter()
        .flatten()
        .any(|id| id_host(id) == Some(COREAUDIO_HOST))
    {
        return false;
    }
    if input_name.trim().eq_ignore_ascii_case(output_name.trim()) {
        return true;
    }
    match (hardware_label(input_name), hardware_label(output_name)) {
        (Some(input), Some(output)) => input.eq_ignore_ascii_case(output),
        _ => false,
    }
}

fn id_host(id: &str) -> Option<&str> {
    id.split_once(':').map(|(host, _)| host)
}

fn hardware_label(endpoint_name: &str) -> Option<&str> {
    let trimmed = endpoint_name.trim_end();
    let inner = trimmed.strip_suffix(')')?;
    let start = inner.rfind('(')?;
    let label = inner.get(start + 1..)?.trim();
    (!label.is_empty()).then_some(label)
}

fn max_channels(ranges: &[SupportedStreamConfigRange]) -> usize {
    ranges
        .iter()
        .filter(|range| sample_format_preference(range.sample_format()) > 0)
        .map(|range| usize::from(range.channels()))
        .max()
        .unwrap_or(0)
}

pub(crate) fn sample_format_preference(format: SampleFormat) -> u8 {
    match format {
        SampleFormat::F32 => 5,
        SampleFormat::F64 => 4,
        SampleFormat::I32 => 3,
        SampleFormat::I24 => 2,
        SampleFormat::I16 => 1,
        _ => 0,
    }
}

pub(crate) fn usable_sample_rates(
    ranges: &[SupportedStreamConfigRange],
    minimum_channels: usize,
) -> Vec<u32> {
    STANDARD_SAMPLE_RATES_HZ
        .into_iter()
        .filter(|rate| {
            ranges.iter().any(|range| {
                sample_format_preference(range.sample_format()) > 0
                    && usize::from(range.channels()) >= minimum_channels
                    && range.contains_rate(*rate)
            })
        })
        .collect()
}

pub(crate) fn find_device(
    device_id: Option<&str>,
    want_input: bool,
) -> Result<cpal::Device, AudioIoError> {
    match device_id {
        Some(id_text) => {
            let id: cpal::DeviceId = id_text.parse()?;
            let host = cpal::host_from_id(id.host())?;
            host.device_by_id(&id)
                .ok_or_else(|| AudioIoError::DeviceNotFound(id_text.to_owned()))
        }
        None => {
            let host = cpal::default_host();
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

#[cfg(test)]
mod tests {
    use super::*;
    use cpal::SupportedBufferSize;

    fn range(
        channels: u16,
        min_hz: u32,
        max_hz: u32,
        format: SampleFormat,
    ) -> SupportedStreamConfigRange {
        SupportedStreamConfigRange::new(
            channels,
            min_hz,
            max_hz,
            SupportedBufferSize::Unknown,
            format,
        )
    }

    #[test]
    fn usable_rates_need_enough_channels_and_a_known_format() {
        let ranges = [
            range(1, 44_100, 44_100, SampleFormat::F32),
            range(2, 48_000, 48_000, SampleFormat::I24),
            range(2, 96_000, 96_000, SampleFormat::U8),
        ];
        assert_eq!(usable_sample_rates(&ranges, 1), vec![44_100, 48_000]);
        assert_eq!(usable_sample_rates(&ranges, 2), vec![48_000]);
        assert!(usable_sample_rates(&ranges, 3).is_empty());
    }

    #[test]
    fn windows_endpoints_of_one_card_share_hardware() {
        assert!(shares_hardware(
            Some("wasapi:{0.0.1.00000000}.{in}"),
            "Entrée ligne (UX2-2)",
            Some("wasapi:{0.0.0.00000000}.{out}"),
            "Haut-parleurs (UX2-2)",
        ));
        assert!(shares_hardware(
            None,
            "Line (2- US-2x2 & US-4x4)",
            None,
            "Speakers (2- us-2x2 & us-4x4) ",
        ));
    }

    #[test]
    fn swissonic_endpoints_seen_on_windows_share_hardware() {
        assert!(shares_hardware(
            Some("wasapi:{0.0.1.00000000}.{8e6fca5e-02bb-4ef0-b450-0f682e90d41b}"),
            "Ligne (UA-2X2)",
            Some("wasapi:{0.0.0.00000000}.{19bcb41e-68f0-41f0-8ddd-aa03f1dba757}"),
            "Haut-parleurs (UA-2X2)",
        ));
        assert!(!shares_hardware(
            Some("wasapi:{0.0.1.00000000}.{8e6fca5e-02bb-4ef0-b450-0f682e90d41b}"),
            "Ligne (UA-2X2)",
            Some("wasapi:{0.0.0.00000000}.{4dccf91f-acde-461b-b177-f798962ccb7f}"),
            "Haut-parleurs (PRO X 2 LIGHTSPEED)",
        ));
    }

    #[test]
    fn the_same_device_id_shares_hardware() {
        assert!(shares_hardware(
            Some("coreaudio:uax"),
            "UA-25EX",
            Some("coreaudio:uax"),
            "UA-25EX",
        ));
    }

    #[test]
    fn two_cards_do_not_share_hardware() {
        assert!(!shares_hardware(
            Some("wasapi:in"),
            "Microphone (Realtek Audio)",
            Some("wasapi:out"),
            "Haut-parleurs (UX2-2)",
        ));
        assert!(!shares_hardware(
            Some("coreaudio:mic"),
            "Micro MacBook Pro",
            Some("coreaudio:speakers"),
            "Haut-parleurs MacBook Pro",
        ));
        assert!(!shares_hardware(Some("a"), "Line ()", Some("b"), "Out ()"));
        assert!(!shares_hardware(
            Some("coreaudio:scarlett-a"),
            "Scarlett 2i2 USB",
            Some("coreaudio:scarlett-b"),
            "Scarlett 2i2 USB",
        ));
    }

    #[test]
    fn device_rates_merge_both_directions() {
        let device = DeviceInfo {
            id: "coreaudio:test".into(),
            name: "Test".into(),
            driver: "CoreAudio".into(),
            is_default_driver: true,
            input_channel_count: 1,
            output_channel_count: 2,
            input_sample_rates_hz: vec![48_000],
            output_sample_rates_hz: vec![44_100, 48_000],
            is_default_input: false,
            is_default_output: false,
        };
        assert_eq!(device.sample_rates_hz(), vec![44_100, 48_000]);
    }
}
