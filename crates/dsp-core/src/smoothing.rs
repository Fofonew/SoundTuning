use crate::Complex64;
use crate::error::{DspError, check_length};
use crate::grid::DisplayGrid;
use crate::transfer::BinTransfer;

const MIN_VALID_FRACTION: f64 = 0.5;
const MIN_BAND_WIDTH_BINS: f64 = 1.0e-9;
const MIN_PROJECTOR_FFT_LENGTH: usize = 4;
const FALLBACK_SAMPLE_RATE_HZ: f64 = 48_000.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Smoothing {
    #[default]
    None,
    OctaveFraction(u32),
}

impl Smoothing {
    pub const CHOICES: [Self; 6] = [
        Self::None,
        Self::OctaveFraction(48),
        Self::OctaveFraction(24),
        Self::OctaveFraction(12),
        Self::OctaveFraction(6),
        Self::OctaveFraction(3),
    ];

    fn half_band_ratio(self) -> f64 {
        match self {
            Self::None | Self::OctaveFraction(0) => 1.0,
            Self::OctaveFraction(fraction) => (0.5 / f64::from(fraction)).exp2(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct BandEdges {
    low_bin: f64,
    high_bin: f64,
}

#[derive(Clone, Debug)]
pub struct GridProjector {
    grid_frequencies_hz: Vec<f64>,
    bin_spacing_hz: f64,
    bin_count: usize,
    smoothing: Smoothing,
    bands: Vec<Option<BandEdges>>,
    weighted_values: Vec<f64>,
    weights: Vec<f64>,
    weighted_prefix: CompensatedPrefix,
    weight_prefix: CompensatedPrefix,
    imaginary_values: Vec<f64>,
    imaginary_prefix: CompensatedPrefix,
}

impl GridProjector {
    pub fn new(
        grid: &DisplayGrid,
        fft_length: usize,
        sample_rate_hz: f64,
        smoothing: Smoothing,
    ) -> Self {
        let fft_length = fft_length.max(MIN_PROJECTOR_FFT_LENGTH);
        let sample_rate_hz = if sample_rate_hz.is_finite() && sample_rate_hz > 0.0 {
            sample_rate_hz
        } else {
            FALLBACK_SAMPLE_RATE_HZ
        };
        let bin_count = fft_length / 2 + 1;
        let mut projector = Self {
            grid_frequencies_hz: grid.frequencies_hz().to_vec(),
            bin_spacing_hz: sample_rate_hz / fft_length as f64,
            bin_count,
            smoothing,
            bands: Vec::with_capacity(grid.len()),
            weighted_values: vec![0.0; bin_count],
            weights: vec![0.0; bin_count],
            weighted_prefix: CompensatedPrefix::new(bin_count),
            weight_prefix: CompensatedPrefix::new(bin_count),
            imaginary_values: vec![0.0; bin_count],
            imaginary_prefix: CompensatedPrefix::new(bin_count),
        };
        projector.rebuild_bands();
        projector
    }

    pub fn smoothing(&self) -> Smoothing {
        self.smoothing
    }

    pub fn set_smoothing(&mut self, smoothing: Smoothing) {
        if smoothing != self.smoothing {
            self.smoothing = smoothing;
            self.rebuild_bands();
        }
    }

    pub fn grid_len(&self) -> usize {
        self.grid_frequencies_hz.len()
    }

    pub fn bin_count(&self) -> usize {
        self.bin_count
    }

    fn rebuild_bands(&mut self) {
        let ratio = self.smoothing.half_band_ratio();
        let last_bin = self.bin_count.saturating_sub(1) as f64;
        let to_bin = |frequency_hz: f64| (frequency_hz / self.bin_spacing_hz).clamp(0.0, last_bin);
        let nyquist_hz = last_bin * self.bin_spacing_hz;
        self.bands = self
            .grid_frequencies_hz
            .iter()
            .map(|frequency_hz| {
                (frequency_hz.is_finite() && *frequency_hz <= nyquist_hz).then(|| BandEdges {
                    low_bin: to_bin(frequency_hz / ratio),
                    high_bin: to_bin(frequency_hz * ratio),
                })
            })
            .collect();
    }

    pub fn project_power(
        &mut self,
        bins: &[BinTransfer],
        output: &mut [Option<f64>],
    ) -> Result<(), DspError> {
        check_length("bins", self.bin_count, bins.len())?;
        check_length("grid output", self.grid_len(), output.len())?;
        for ((value, weight), bin) in self
            .weighted_values
            .iter_mut()
            .zip(self.weights.iter_mut())
            .zip(bins)
        {
            match bin.response {
                Some(response) => {
                    *value = response.norm_sqr();
                    *weight = 1.0;
                }
                None => {
                    *value = 0.0;
                    *weight = 0.0;
                }
            }
        }
        self.project_real(output);
        Ok(())
    }

    pub fn project_coherence(
        &mut self,
        bins: &[BinTransfer],
        output: &mut [Option<f64>],
    ) -> Result<(), DspError> {
        check_length("bins", self.bin_count, bins.len())?;
        check_length("grid output", self.grid_len(), output.len())?;
        for ((value, weight), bin) in self
            .weighted_values
            .iter_mut()
            .zip(self.weights.iter_mut())
            .zip(bins)
        {
            match bin.coherence {
                Some(coherence) => {
                    *value = coherence;
                    *weight = 1.0;
                }
                None => {
                    *value = 0.0;
                    *weight = 0.0;
                }
            }
        }
        self.project_real(output);
        for value in output.iter_mut().flatten() {
            *value = value.clamp(0.0, 1.0);
        }
        Ok(())
    }

    pub fn project_complex_ratio(
        &mut self,
        numerator: &[Complex64],
        denominator: &[f64],
        min_denominator: f64,
        output: &mut [Option<Complex64>],
    ) -> Result<(), DspError> {
        check_length("numerator", self.bin_count, numerator.len())?;
        check_length("denominator", self.bin_count, denominator.len())?;
        check_length("grid output", self.grid_len(), output.len())?;
        for (((real, imaginary), weight), (value, divisor)) in self
            .weighted_values
            .iter_mut()
            .zip(self.imaginary_values.iter_mut())
            .zip(self.weights.iter_mut())
            .zip(numerator.iter().zip(denominator))
        {
            let usable = divisor.is_finite() && value.re.is_finite() && value.im.is_finite();
            *real = if usable { value.re } else { 0.0 };
            *imaginary = if usable { value.im } else { 0.0 };
            *weight = if usable { *divisor } else { 0.0 };
        }
        self.weighted_prefix.fill(&self.weighted_values);
        self.imaginary_prefix.fill(&self.imaginary_values);
        self.weight_prefix.fill(&self.weights);
        for (band, slot) in self.bands.iter().zip(output.iter_mut()) {
            let Some(band) = *band else {
                *slot = None;
                continue;
            };
            let real = band_mean(&self.weighted_values, &self.weighted_prefix, band);
            let imaginary = band_mean(&self.imaginary_values, &self.imaginary_prefix, band);
            let divisor = band_mean(&self.weights, &self.weight_prefix, band);
            *slot = (divisor > min_denominator)
                .then(|| Complex64::new(real / divisor, imaginary / divisor))
                .filter(|ratio| ratio.re.is_finite() && ratio.im.is_finite());
        }
        Ok(())
    }

    fn project_real(&mut self, output: &mut [Option<f64>]) {
        for (value, weight) in self.weighted_values.iter_mut().zip(self.weights.iter()) {
            *value *= weight;
        }
        self.weighted_prefix.fill(&self.weighted_values);
        self.weight_prefix.fill(&self.weights);
        for (band, slot) in self.bands.iter().zip(output.iter_mut()) {
            let Some(band) = *band else {
                *slot = None;
                continue;
            };
            let weight = band_mean(&self.weights, &self.weight_prefix, band);
            *slot = (weight >= MIN_VALID_FRACTION)
                .then(|| {
                    (band_mean(&self.weighted_values, &self.weighted_prefix, band) / weight)
                        .max(0.0)
                })
                .filter(|value| value.is_finite());
        }
    }
}

#[derive(Clone, Debug)]
struct CompensatedPrefix {
    high: Vec<f64>,
    low: Vec<f64>,
}

impl CompensatedPrefix {
    fn new(length: usize) -> Self {
        Self {
            high: vec![0.0; length],
            low: vec![0.0; length],
        }
    }

    fn fill(&mut self, values: &[f64]) {
        let mut sum = 0.0_f64;
        let mut compensation = 0.0_f64;
        if let (Some(high), Some(low)) = (self.high.first_mut(), self.low.first_mut()) {
            *high = 0.0;
            *low = 0.0;
        }
        for ((high, low), pair) in self
            .high
            .iter_mut()
            .skip(1)
            .zip(self.low.iter_mut().skip(1))
            .zip(values.windows(2))
        {
            let term = 0.5 * (pair[0] + pair[1]);
            let total = sum + term;
            compensation += if sum.abs() >= term.abs() {
                (sum - total) + term
            } else {
                (term - total) + sum
            };
            sum = total;
            *high = sum;
            *low = compensation;
        }
    }

    fn between(&self, later: usize, earlier: usize) -> f64 {
        match (
            self.high.get(later),
            self.high.get(earlier),
            self.low.get(later),
            self.low.get(earlier),
        ) {
            (Some(high_later), Some(high_earlier), Some(low_later), Some(low_earlier)) => {
                (high_later - high_earlier) + (low_later - low_earlier)
            }
            _ => 0.0,
        }
    }
}

fn segment_and_fraction(values: &[f64], position_bins: f64) -> (usize, f64) {
    let last_segment = values.len().saturating_sub(2);
    let segment = (position_bins.floor().max(0.0) as usize).min(last_segment);
    (segment, position_bins - segment as f64)
}

fn partial_segment_integral(values: &[f64], segment: usize, fraction: f64) -> f64 {
    match (values.get(segment), values.get(segment + 1)) {
        (Some(start), Some(end)) => start * fraction + (end - start) * fraction * fraction * 0.5,
        _ => 0.0,
    }
}

fn interpolate(values: &[f64], position_bins: f64) -> f64 {
    let (segment, fraction) = segment_and_fraction(values, position_bins);
    match (values.get(segment), values.get(segment + 1)) {
        (Some(start), Some(end)) => start + (end - start) * fraction,
        (Some(start), None) => *start,
        _ => 0.0,
    }
}

fn band_mean(values: &[f64], prefix: &CompensatedPrefix, band: BandEdges) -> f64 {
    let width = band.high_bin - band.low_bin;
    if width < MIN_BAND_WIDTH_BINS {
        return interpolate(values, band.low_bin);
    }
    let (low_segment, low_fraction) = segment_and_fraction(values, band.low_bin);
    let (high_segment, high_fraction) = segment_and_fraction(values, band.high_bin);
    let integral = prefix.between(high_segment, low_segment)
        + partial_segment_integral(values, high_segment, high_fraction)
        - partial_segment_integral(values, low_segment, low_fraction);
    integral / width
}

#[cfg(test)]
mod tests {
    use super::*;

    const FFT_LENGTH: usize = 32_768;
    const SAMPLE_RATE_HZ: f64 = 48_000.0;

    fn flat_bins(gain: f64) -> Vec<BinTransfer> {
        vec![
            BinTransfer {
                response: Some(Complex64::new(gain, 0.0)),
                coherence: Some(1.0),
            };
            FFT_LENGTH / 2 + 1
        ]
    }

    #[test]
    fn flat_response_stays_flat_for_every_smoothing() {
        let grid = DisplayGrid::default();
        let bins = flat_bins(2.0);
        let mut output = vec![None; grid.len()];
        for smoothing in Smoothing::CHOICES {
            let mut projector = GridProjector::new(&grid, FFT_LENGTH, SAMPLE_RATE_HZ, smoothing);
            projector.project_power(&bins, &mut output).unwrap();
            for value in &output {
                let db = 10.0 * value.unwrap().log10();
                assert!((db - 6.0206).abs() < 0.01, "{smoothing:?} gave {db} dB");
            }
        }
    }

    #[test]
    fn invalid_bins_make_grid_points_invalid_without_nan() {
        let grid = DisplayGrid::default();
        let bins = vec![BinTransfer::default(); FFT_LENGTH / 2 + 1];
        let mut projector = GridProjector::new(
            &grid,
            FFT_LENGTH,
            SAMPLE_RATE_HZ,
            Smoothing::OctaveFraction(3),
        );
        let mut output = vec![Some(1.0); grid.len()];
        projector.project_power(&bins, &mut output).unwrap();
        assert!(output.iter().all(Option::is_none));
        projector.project_coherence(&bins, &mut output).unwrap();
        assert!(output.iter().all(Option::is_none));
    }

    #[test]
    fn smoothing_averages_power_not_amplitude() {
        let grid = DisplayGrid::new(1000.0, 1000.0, 96.0);
        let mut bins = flat_bins(1.0);
        for (index, bin) in bins.iter_mut().enumerate() {
            if index % 2 == 0 {
                bin.response = Some(Complex64::new(0.0, 0.0));
            }
        }
        let mut projector = GridProjector::new(
            &grid,
            FFT_LENGTH,
            SAMPLE_RATE_HZ,
            Smoothing::OctaveFraction(3),
        );
        let mut output = vec![None; grid.len()];
        projector.project_power(&bins, &mut output).unwrap();
        assert!((output[0].unwrap() - 0.5).abs() < 0.01);
    }

    #[test]
    fn complex_ratio_of_constant_spectra_is_the_constant() {
        let grid = DisplayGrid::default();
        let bins = FFT_LENGTH / 2 + 1;
        let numerator = vec![Complex64::new(3.0, -6.0); bins];
        let denominator = vec![3.0; bins];
        let mut projector = GridProjector::new(
            &grid,
            FFT_LENGTH,
            SAMPLE_RATE_HZ,
            Smoothing::OctaveFraction(6),
        );
        let mut output = vec![None; grid.len()];
        projector
            .project_complex_ratio(&numerator, &denominator, 1.0e-30, &mut output)
            .unwrap();
        for value in output {
            let value = value.unwrap();
            assert!((value - Complex64::new(1.0, -2.0)).norm() < 1.0e-9);
        }
    }

    #[test]
    fn steep_dynamic_range_does_not_create_false_holes() {
        let grid = DisplayGrid::default();
        let bin_spacing_hz = SAMPLE_RATE_HZ / FFT_LENGTH as f64;
        let bins: Vec<BinTransfer> = (0..=FFT_LENGTH / 2)
            .map(|index| {
                let gain = if index as f64 * bin_spacing_hz < 1000.0 {
                    100.0
                } else {
                    1.0e-6
                };
                BinTransfer {
                    response: Some(Complex64::new(gain, 0.0)),
                    coherence: Some(1.0),
                }
            })
            .collect();
        let mut projector = GridProjector::new(
            &grid,
            FFT_LENGTH,
            SAMPLE_RATE_HZ,
            Smoothing::OctaveFraction(3),
        );
        let mut output = vec![None; grid.len()];
        projector.project_power(&bins, &mut output).unwrap();
        for (frequency_hz, value) in grid.frequencies_hz().iter().zip(&output) {
            if *frequency_hz > 2000.0 {
                let relative_error = (value.unwrap() - 1.0e-12).abs() / 1.0e-12;
                assert!(relative_error < 1.0e-3, "{frequency_hz} Hz: {value:?}");
            }
        }
    }

    #[test]
    fn smoothing_band_spans_the_requested_fraction_of_octave() {
        let grid = DisplayGrid::default();
        let bin_spacing_hz = SAMPLE_RATE_HZ / FFT_LENGTH as f64;
        let line_bin = (4000.0 / bin_spacing_hz).round() as usize;
        let line_hz = line_bin as f64 * bin_spacing_hz;
        let bins: Vec<BinTransfer> = (0..=FFT_LENGTH / 2)
            .map(|index| BinTransfer {
                response: Some(Complex64::new(
                    if index == line_bin { 1.0 } else { 0.0 },
                    0.0,
                )),
                coherence: Some(1.0),
            })
            .collect();
        for fraction in [3_u32, 6, 12, 24, 48] {
            let mut projector = GridProjector::new(
                &grid,
                FFT_LENGTH,
                SAMPLE_RATE_HZ,
                Smoothing::OctaveFraction(fraction),
            );
            let mut output = vec![None; grid.len()];
            projector.project_power(&bins, &mut output).unwrap();
            let half_band_octaves = 0.5 / f64::from(fraction);
            for (frequency_hz, value) in grid.frequencies_hz().iter().zip(&output) {
                let distance_octaves = (line_hz / frequency_hz).log2().abs();
                let value = value.unwrap();
                if distance_octaves < half_band_octaves * 0.9 {
                    assert!(
                        value > 0.0,
                        "1/{fraction}: {frequency_hz} Hz should see the line"
                    );
                }
                if distance_octaves > half_band_octaves * 1.1 {
                    assert!(
                        value == 0.0,
                        "1/{fraction}: {frequency_hz} Hz should not see the line"
                    );
                }
            }
        }
    }

    #[test]
    fn grid_points_above_nyquist_are_invalid() {
        let grid = DisplayGrid::default();
        let fft_length = 8192;
        let bins = vec![
            BinTransfer {
                response: Some(Complex64::new(1.0, 0.0)),
                coherence: Some(1.0),
            };
            fft_length / 2 + 1
        ];
        let mut projector = GridProjector::new(&grid, fft_length, 32_000.0, Smoothing::None);
        let mut output = vec![None; grid.len()];
        projector.project_power(&bins, &mut output).unwrap();
        for (frequency_hz, value) in grid.frequencies_hz().iter().zip(&output) {
            assert_eq!(
                value.is_some(),
                *frequency_hz <= 16_000.0,
                "{frequency_hz} Hz"
            );
        }
    }

    #[test]
    fn wrong_lengths_are_rejected() {
        let grid = DisplayGrid::default();
        let mut projector = GridProjector::new(&grid, FFT_LENGTH, SAMPLE_RATE_HZ, Smoothing::None);
        let mut output = vec![None; grid.len()];
        assert!(projector.project_power(&[], &mut output).is_err());
    }
}
