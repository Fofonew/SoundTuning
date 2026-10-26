use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use serde::{Deserialize, Serialize};

use crate::atomic_f32::AtomicF32;

const PINK_OUTPUT_SCALE: f32 = 0.11;
const DENORMAL_FLUSH_THRESHOLD: f32 = 1.0e-20;
const DEFAULT_SEED: u64 = 0x9E37_79B9_7F4A_7C15;
const XORSHIFT_MULTIPLIER: u64 = 0x2545_F491_4F6C_DD1D;
const UNIT_INTERVAL_BITS: u32 = 24;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoiseKind {
    #[default]
    Pink,
    White,
}

impl NoiseKind {
    fn to_code(self) -> u8 {
        match self {
            Self::Pink => 0,
            Self::White => 1,
        }
    }

    fn from_code(code: u8) -> Self {
        match code {
            1 => Self::White,
            _ => Self::Pink,
        }
    }
}

#[derive(Debug)]
pub struct GeneratorControls {
    enabled: AtomicBool,
    kind: AtomicU8,
    gain_linear: AtomicF32,
}

impl GeneratorControls {
    pub fn new(enabled: bool, kind: NoiseKind, level_dbfs: f32) -> Self {
        Self {
            enabled: AtomicBool::new(enabled),
            kind: AtomicU8::new(kind.to_code()),
            gain_linear: AtomicF32::new(dbfs_to_gain(level_dbfs)),
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn set_kind(&self, kind: NoiseKind) {
        self.kind.store(kind.to_code(), Ordering::Relaxed);
    }

    pub fn kind(&self) -> NoiseKind {
        NoiseKind::from_code(self.kind.load(Ordering::Relaxed))
    }

    pub fn set_level_dbfs(&self, level_dbfs: f32) {
        self.gain_linear.store(dbfs_to_gain(level_dbfs));
    }

    pub fn target_gain(&self) -> f32 {
        if self.is_enabled() {
            self.gain_linear.load()
        } else {
            0.0
        }
    }
}

fn dbfs_to_gain(level_dbfs: f32) -> f32 {
    10.0_f32.powf(level_dbfs / 20.0)
}

#[derive(Clone, Debug)]
pub struct NoiseGenerator {
    rng_state: u64,
    pink_state: [f32; 7],
    current_gain: f32,
}

impl Default for NoiseGenerator {
    fn default() -> Self {
        Self::with_seed(DEFAULT_SEED)
    }
}

impl NoiseGenerator {
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng_state: seed.max(1),
            pink_state: [0.0; 7],
            current_gain: 0.0,
        }
    }

    fn next_white(&mut self) -> f32 {
        self.rng_state ^= self.rng_state >> 12;
        self.rng_state ^= self.rng_state << 25;
        self.rng_state ^= self.rng_state >> 27;
        let random_bits =
            self.rng_state.wrapping_mul(XORSHIFT_MULTIPLIER) >> (64 - UNIT_INTERVAL_BITS);
        let unit = random_bits as f32 / (1_u32 << UNIT_INTERVAL_BITS) as f32;
        2.0 * unit - 1.0
    }

    fn next_pink(&mut self) -> f32 {
        let white = self.next_white();
        let [b0, b1, b2, b3, b4, b5, b6] = &mut self.pink_state;
        *b0 = 0.99886 * *b0 + white * 0.055_517_9;
        *b1 = 0.99332 * *b1 + white * 0.075_075_9;
        *b2 = 0.969 * *b2 + white * 0.153_852;
        *b3 = 0.8665 * *b3 + white * 0.310_485_6;
        *b4 = 0.55 * *b4 + white * 0.532_952_2;
        *b5 = -0.7616 * *b5 - white * 0.016_898;
        let pink = *b0 + *b1 + *b2 + *b3 + *b4 + *b5 + *b6 + white * 0.5362;
        *b6 = white * 0.115_926;
        pink * PINK_OUTPUT_SCALE
    }

    pub fn next_sample(&mut self, kind: NoiseKind) -> f32 {
        match kind {
            NoiseKind::Pink => self.next_pink(),
            NoiseKind::White => self.next_white(),
        }
    }

    pub fn fill_block(&mut self, kind: NoiseKind, target_gain: f32, block: &mut [f32]) {
        let frame_count = block.len();
        if frame_count == 0 {
            return;
        }
        let gain_step = (target_gain - self.current_gain) / frame_count as f32;
        for sample in block.iter_mut() {
            self.current_gain += gain_step;
            *sample = self.next_sample(kind) * self.current_gain;
        }
        self.current_gain = target_gain;
        self.flush_denormal_state();
    }

    fn flush_denormal_state(&mut self) {
        for state in &mut self.pink_state {
            if state.abs() < DENORMAL_FLUSH_THRESHOLD {
                *state = 0.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn band_power(samples: &[f32], low_bin: usize, high_bin: usize) -> f64 {
        let length = samples.len();
        (low_bin..high_bin)
            .map(|bin| {
                let (mut real, mut imaginary) = (0.0_f64, 0.0_f64);
                for (index, sample) in samples.iter().enumerate() {
                    let phase = -2.0 * std::f64::consts::PI * (bin * index) as f64 / length as f64;
                    real += f64::from(*sample) * phase.cos();
                    imaginary += f64::from(*sample) * phase.sin();
                }
                real * real + imaginary * imaginary
            })
            .sum()
    }

    #[test]
    fn white_noise_stays_in_unit_range_with_zero_mean() {
        let mut generator = NoiseGenerator::with_seed(7);
        let samples: Vec<f32> = (0..100_000)
            .map(|_| generator.next_sample(NoiseKind::White))
            .collect();
        assert!(samples.iter().all(|sample| (-1.0..1.0).contains(sample)));
        let mean: f64 =
            samples.iter().map(|sample| f64::from(*sample)).sum::<f64>() / samples.len() as f64;
        assert!(mean.abs() < 0.01);
    }

    #[test]
    fn pink_noise_loses_about_three_db_per_octave() {
        let mut generator = NoiseGenerator::with_seed(11);
        let samples: Vec<f32> = (0..4096)
            .map(|_| generator.next_sample(NoiseKind::Pink))
            .collect();
        let low_octave = band_power(&samples, 32, 64);
        let high_octave = band_power(&samples, 512, 1024);
        let octave_count = 4.0;
        let per_octave_db = 10.0 * (high_octave / low_octave).log10() / octave_count;
        assert!(
            (per_octave_db).abs() < 1.0,
            "pink noise band powers should be roughly equal per octave, got {per_octave_db} dB/oct"
        );
    }

    #[test]
    fn same_seed_gives_same_sequence() {
        let mut first = NoiseGenerator::with_seed(3);
        let mut second = NoiseGenerator::with_seed(3);
        for _ in 0..1000 {
            assert_eq!(
                first.next_sample(NoiseKind::Pink),
                second.next_sample(NoiseKind::Pink)
            );
        }
    }

    #[test]
    fn gain_ramps_without_jump() {
        let mut generator = NoiseGenerator::with_seed(5);
        let mut block = [0.0_f32; 256];
        generator.fill_block(NoiseKind::White, 1.0, &mut block);
        let first_quarter_peak = block[..64]
            .iter()
            .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
        assert!(first_quarter_peak <= 0.26);
    }

    #[test]
    fn disabled_controls_target_silence() {
        let controls = GeneratorControls::new(false, NoiseKind::Pink, -6.0);
        assert_eq!(controls.target_gain(), 0.0);
        controls.set_enabled(true);
        assert!((controls.target_gain() - 0.501).abs() < 0.001);
    }
}
