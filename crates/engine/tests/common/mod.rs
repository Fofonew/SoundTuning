#![allow(dead_code, clippy::unwrap_used)]

use audio_io::StereoFrame;
use dsp_core::Complex64;
use realfft::RealFftPlanner;

pub const SAMPLE_RATE_HZ: u32 = 48_000;

pub struct NoiseSource {
    state: u64,
}

impl NoiseSource {
    pub fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    pub fn white(&mut self) -> f64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        (self.state >> 11) as f64 / (1_u64 << 53) as f64 * 2.0 - 1.0
    }

    pub fn white_signal(&mut self, length: usize) -> Vec<f64> {
        (0..length).map(|_| self.white()).collect()
    }

    pub fn pink_signal(&mut self, length: usize) -> Vec<f64> {
        let mut b = [0.0_f64; 7];
        (0..length)
            .map(|_| {
                let white = self.white();
                b[0] = 0.99886 * b[0] + white * 0.0555179;
                b[1] = 0.99332 * b[1] + white * 0.0750759;
                b[2] = 0.96900 * b[2] + white * 0.1538520;
                b[3] = 0.86650 * b[3] + white * 0.3104856;
                b[4] = 0.55000 * b[4] + white * 0.5329522;
                b[5] = -0.7616 * b[5] - white * 0.0168980;
                let pink = b.iter().sum::<f64>() + white * 0.5362;
                b[6] = white * 0.115926;
                pink * 0.11
            })
            .collect()
    }
}

pub fn rms(signal: &[f64]) -> f64 {
    (signal.iter().map(|value| value * value).sum::<f64>() / signal.len() as f64).sqrt()
}

pub fn circular_fractional_delay(signal: &[f64], delay_samples: f64) -> Vec<f64> {
    let length = signal.len();
    let mut planner = RealFftPlanner::<f64>::new();
    let forward = planner.plan_fft_forward(length);
    let inverse = planner.plan_fft_inverse(length);
    let mut input = signal.to_vec();
    let mut spectrum = forward.make_output_vec();
    forward.process(&mut input, &mut spectrum).unwrap();
    for (bin, value) in spectrum.iter_mut().enumerate() {
        let phase = -std::f64::consts::TAU * bin as f64 * delay_samples / length as f64;
        *value *= Complex64::from_polar(1.0 / length as f64, phase);
    }
    spectrum[0].im = 0.0;
    if let Some(last) = spectrum.last_mut() {
        last.im = 0.0;
    }
    let mut output = inverse.make_output_vec();
    inverse.process(&mut spectrum, &mut output).unwrap();
    output
}

pub fn stereo_frames(reference: &[f64], measurement: &[f64]) -> Vec<StereoFrame> {
    reference
        .iter()
        .zip(measurement)
        .map(|(reference, measurement)| StereoFrame {
            reference: *reference as f32,
            measurement: *measurement as f32,
        })
        .collect()
}
