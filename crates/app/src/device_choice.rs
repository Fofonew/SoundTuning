use audio_io::{DeviceInfo, ReferenceMode, STANDARD_SAMPLE_RATES_HZ, StreamSettings};

pub fn find<'a>(devices: &'a [DeviceInfo], id: &str) -> Option<&'a DeviceInfo> {
    devices.iter().find(|device| device.id == id)
}

pub fn resolved_input<'a>(
    devices: &'a [DeviceInfo],
    stream: &StreamSettings,
) -> Option<&'a DeviceInfo> {
    match stream.input_device_id.as_deref() {
        Some(id) => find(devices, id),
        None => devices.iter().find(|device| device.is_default_input),
    }
}

pub fn resolved_output<'a>(
    devices: &'a [DeviceInfo],
    stream: &StreamSettings,
) -> Option<&'a DeviceInfo> {
    match stream.output_device_id.as_deref() {
        Some(id) => find(devices, id),
        None => resolved_input(devices, stream)
            .filter(|input| {
                input.output_channel_count >= stream.required_output_channels()
                    && !input.output_sample_rates_hz.is_empty()
            })
            .or_else(|| devices.iter().find(|device| device.is_default_output)),
    }
}

pub fn shared_sample_rates(devices: &[DeviceInfo], stream: &StreamSettings) -> Vec<u32> {
    let input = resolved_input(devices, stream).map(|device| &device.input_sample_rates_hz);
    let output = resolved_output(devices, stream)
        .filter(|_| matches!(stream.reference, ReferenceMode::Internal))
        .map(|device| &device.output_sample_rates_hz);
    STANDARD_SAMPLE_RATES_HZ
        .into_iter()
        .filter(|rate| input.is_none_or(|rates| rates.contains(rate)))
        .filter(|rate| output.is_none_or(|rates| rates.contains(rate)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device(id: &str, inputs: &[u32], outputs: &[u32]) -> DeviceInfo {
        DeviceInfo {
            id: id.into(),
            name: id.into(),
            driver: "CoreAudio".into(),
            is_default_driver: true,
            input_channel_count: usize::from(!inputs.is_empty()),
            output_channel_count: if outputs.is_empty() { 0 } else { 2 },
            input_sample_rates_hz: inputs.to_vec(),
            output_sample_rates_hz: outputs.to_vec(),
            is_default_input: false,
            is_default_output: false,
        }
    }

    fn laptop() -> Vec<DeviceInfo> {
        let mut microphone = device("mic", &[48_000], &[]);
        microphone.is_default_input = true;
        let mut speakers = device("speakers", &[], &[44_100]);
        speakers.is_default_output = true;
        vec![microphone, speakers, device("uax", &[48_000], &[48_000])]
    }

    #[test]
    fn an_input_only_device_falls_back_to_the_system_output() {
        let devices = laptop();
        let stream = StreamSettings {
            input_device_id: Some("mic".into()),
            ..StreamSettings::default()
        };
        assert_eq!(
            resolved_output(&devices, &stream).map(|device| device.id.as_str()),
            Some("speakers")
        );
    }

    #[test]
    fn a_duplex_input_is_also_the_automatic_output() {
        let devices = laptop();
        let stream = StreamSettings {
            input_device_id: Some("uax".into()),
            ..StreamSettings::default()
        };
        assert_eq!(
            resolved_output(&devices, &stream).map(|device| device.id.as_str()),
            Some("uax")
        );
    }

    #[test]
    fn a_duplex_input_without_enough_outputs_is_skipped() {
        let mut devices = laptop();
        if let Some(uax) = devices.last_mut() {
            uax.output_channel_count = 1;
        }
        let mut stream = StreamSettings {
            input_device_id: Some("uax".into()),
            ..StreamSettings::default()
        };
        stream.generator.output_channel_indices = vec![1];
        assert_eq!(
            resolved_output(&devices, &stream).map(|device| device.id.as_str()),
            Some("speakers")
        );
    }

    #[test]
    fn internal_reference_needs_rates_shared_by_both_devices() {
        let mut devices = laptop();
        if let Some(microphone) = devices.first_mut() {
            microphone.input_sample_rates_hz = vec![44_100, 48_000];
        }
        let mut stream = StreamSettings {
            input_device_id: Some("mic".into()),
            ..StreamSettings::default()
        };
        assert_eq!(shared_sample_rates(&devices, &stream), vec![44_100]);
        stream.reference = ReferenceMode::Loopback {
            reference_channel_index: 0,
        };
        assert_eq!(shared_sample_rates(&devices, &stream), vec![44_100, 48_000]);
    }
}
