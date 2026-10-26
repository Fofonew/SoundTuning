use std::f64::consts::{LN_2, TAU};

use crate::Complex64;
use crate::error::{DspError, check_length};

pub const MAX_EQ_FILTERS: usize = 16;
const MIN_Q: f64 = 0.05;
const MAX_Q: f64 = 100.0;
const MIN_FREQUENCY_HZ: f64 = 1.0;
const MAX_FREQUENCY_FRACTION_OF_NYQUIST: f64 = 0.999;
const BUTTERWORTH_SECOND_ORDER_Q: f64 = std::f64::consts::FRAC_1_SQRT_2;
const BUTTERWORTH_FOURTH_ORDER_QS: [f64; 2] = [0.541_196_100_146_197, 1.306_562_964_876_376_5];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CutSlope {
    Butterworth12,
    Butterworth24,
    LinkwitzRiley24,
    LinkwitzRiley48,
}

impl CutSlope {
    pub const CHOICES: [Self; 4] = [
        Self::Butterworth12,
        Self::Butterworth24,
        Self::LinkwitzRiley24,
        Self::LinkwitzRiley48,
    ];

    fn section_qs(self) -> &'static [f64] {
        const BW12: [f64; 1] = [BUTTERWORTH_SECOND_ORDER_Q];
        const LR24: [f64; 2] = [BUTTERWORTH_SECOND_ORDER_Q, BUTTERWORTH_SECOND_ORDER_Q];
        const LR48: [f64; 4] = [
            BUTTERWORTH_FOURTH_ORDER_QS[0],
            BUTTERWORTH_FOURTH_ORDER_QS[1],
            BUTTERWORTH_FOURTH_ORDER_QS[0],
            BUTTERWORTH_FOURTH_ORDER_QS[1],
        ];
        match self {
            Self::Butterworth12 => &BW12,
            Self::Butterworth24 => &BUTTERWORTH_FOURTH_ORDER_QS,
            Self::LinkwitzRiley24 => &LR24,
            Self::LinkwitzRiley48 => &LR48,
        }
    }

    pub fn decibels_per_octave(self) -> u32 {
        match self {
            Self::Butterworth12 => 12,
            Self::Butterworth24 | Self::LinkwitzRiley24 => 24,
            Self::LinkwitzRiley48 => 48,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FilterKind {
    Bell,
    LowShelf,
    HighShelf,
    LowCut(CutSlope),
    HighCut(CutSlope),
    Notch,
}

impl FilterKind {
    pub fn uses_gain(self) -> bool {
        matches!(self, Self::Bell | Self::LowShelf | Self::HighShelf)
    }

    pub fn uses_q(self) -> bool {
        matches!(
            self,
            Self::Bell | Self::LowShelf | Self::HighShelf | Self::Notch
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EqFilter {
    pub kind: FilterKind,
    pub frequency_hz: f64,
    pub gain_db: f64,
    pub q: f64,
    pub bypassed: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BiquadCoefficients {
    pub b0: f64,
    pub b1: f64,
    pub b2: f64,
    pub a1: f64,
    pub a2: f64,
}

struct NormalizedParameters {
    cos_w0: f64,
    alpha: f64,
    amplitude: f64,
}

fn normalized_parameters(
    frequency_hz: f64,
    q: f64,
    gain_db: f64,
    sample_rate_hz: f64,
) -> NormalizedParameters {
    let nyquist_hz = sample_rate_hz / 2.0;
    let frequency_hz = if frequency_hz.is_finite() {
        frequency_hz.clamp(
            MIN_FREQUENCY_HZ,
            nyquist_hz * MAX_FREQUENCY_FRACTION_OF_NYQUIST,
        )
    } else {
        MIN_FREQUENCY_HZ
    };
    let q = if q.is_finite() {
        q.clamp(MIN_Q, MAX_Q)
    } else {
        BUTTERWORTH_SECOND_ORDER_Q
    };
    let gain_db = if gain_db.is_finite() { gain_db } else { 0.0 };
    let w0 = TAU * frequency_hz / sample_rate_hz;
    NormalizedParameters {
        cos_w0: w0.cos(),
        alpha: w0.sin() / (2.0 * q),
        amplitude: 10.0_f64.powf(gain_db / 40.0),
    }
}

impl BiquadCoefficients {
    fn from_unnormalized(b0: f64, b1: f64, b2: f64, a0: f64, a1: f64, a2: f64) -> Self {
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    pub fn bell(frequency_hz: f64, q: f64, gain_db: f64, sample_rate_hz: f64) -> Self {
        let NormalizedParameters {
            cos_w0,
            alpha,
            amplitude,
        } = normalized_parameters(frequency_hz, q, gain_db, sample_rate_hz);
        Self::from_unnormalized(
            1.0 + alpha * amplitude,
            -2.0 * cos_w0,
            1.0 - alpha * amplitude,
            1.0 + alpha / amplitude,
            -2.0 * cos_w0,
            1.0 - alpha / amplitude,
        )
    }

    pub fn low_shelf(frequency_hz: f64, q: f64, gain_db: f64, sample_rate_hz: f64) -> Self {
        let NormalizedParameters {
            cos_w0,
            alpha,
            amplitude: a,
        } = normalized_parameters(frequency_hz, q, gain_db, sample_rate_hz);
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;
        Self::from_unnormalized(
            a * ((a + 1.0) - (a - 1.0) * cos_w0 + two_sqrt_a_alpha),
            2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0),
            a * ((a + 1.0) - (a - 1.0) * cos_w0 - two_sqrt_a_alpha),
            (a + 1.0) + (a - 1.0) * cos_w0 + two_sqrt_a_alpha,
            -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0),
            (a + 1.0) + (a - 1.0) * cos_w0 - two_sqrt_a_alpha,
        )
    }

    pub fn high_shelf(frequency_hz: f64, q: f64, gain_db: f64, sample_rate_hz: f64) -> Self {
        let NormalizedParameters {
            cos_w0,
            alpha,
            amplitude: a,
        } = normalized_parameters(frequency_hz, q, gain_db, sample_rate_hz);
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;
        Self::from_unnormalized(
            a * ((a + 1.0) + (a - 1.0) * cos_w0 + two_sqrt_a_alpha),
            -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0),
            a * ((a + 1.0) + (a - 1.0) * cos_w0 - two_sqrt_a_alpha),
            (a + 1.0) - (a - 1.0) * cos_w0 + two_sqrt_a_alpha,
            2.0 * ((a - 1.0) - (a + 1.0) * cos_w0),
            (a + 1.0) - (a - 1.0) * cos_w0 - two_sqrt_a_alpha,
        )
    }

    pub fn low_pass(frequency_hz: f64, q: f64, sample_rate_hz: f64) -> Self {
        let NormalizedParameters { cos_w0, alpha, .. } =
            normalized_parameters(frequency_hz, q, 0.0, sample_rate_hz);
        Self::from_unnormalized(
            (1.0 - cos_w0) / 2.0,
            1.0 - cos_w0,
            (1.0 - cos_w0) / 2.0,
            1.0 + alpha,
            -2.0 * cos_w0,
            1.0 - alpha,
        )
    }

    pub fn high_pass(frequency_hz: f64, q: f64, sample_rate_hz: f64) -> Self {
        let NormalizedParameters { cos_w0, alpha, .. } =
            normalized_parameters(frequency_hz, q, 0.0, sample_rate_hz);
        Self::from_unnormalized(
            (1.0 + cos_w0) / 2.0,
            -(1.0 + cos_w0),
            (1.0 + cos_w0) / 2.0,
            1.0 + alpha,
            -2.0 * cos_w0,
            1.0 - alpha,
        )
    }

    pub fn notch(frequency_hz: f64, q: f64, sample_rate_hz: f64) -> Self {
        let NormalizedParameters { cos_w0, alpha, .. } =
            normalized_parameters(frequency_hz, q, 0.0, sample_rate_hz);
        Self::from_unnormalized(
            1.0,
            -2.0 * cos_w0,
            1.0,
            1.0 + alpha,
            -2.0 * cos_w0,
            1.0 - alpha,
        )
    }

    pub fn response_at(&self, frequency_hz: f64, sample_rate_hz: f64) -> Complex64 {
        self.response_with_unit_delay(unit_delay(frequency_hz, sample_rate_hz))
    }

    fn response_with_unit_delay(&self, z_inverse: Complex64) -> Complex64 {
        let z_inverse_squared = z_inverse * z_inverse;
        let numerator = self.b0 + z_inverse * self.b1 + z_inverse_squared * self.b2;
        let denominator =
            Complex64::new(1.0, 0.0) + z_inverse * self.a1 + z_inverse_squared * self.a2;
        numerator / denominator
    }

    pub fn is_stable(&self) -> bool {
        1.0 + self.a1 + self.a2 > 0.0 && 1.0 - self.a1 + self.a2 > 0.0 && 1.0 - self.a2 > 0.0
    }
}

impl EqFilter {
    pub fn sections(&self, sample_rate_hz: f64) -> SectionCascade {
        let single = |section| SectionCascade::from_slice(&[section]);
        match self.kind {
            FilterKind::Bell => single(BiquadCoefficients::bell(
                self.frequency_hz,
                self.q,
                self.gain_db,
                sample_rate_hz,
            )),
            FilterKind::LowShelf => single(BiquadCoefficients::low_shelf(
                self.frequency_hz,
                self.q,
                self.gain_db,
                sample_rate_hz,
            )),
            FilterKind::HighShelf => single(BiquadCoefficients::high_shelf(
                self.frequency_hz,
                self.q,
                self.gain_db,
                sample_rate_hz,
            )),
            FilterKind::Notch => single(BiquadCoefficients::notch(
                self.frequency_hz,
                self.q,
                sample_rate_hz,
            )),
            FilterKind::LowCut(slope) => SectionCascade::from_qs(slope.section_qs(), |q| {
                BiquadCoefficients::high_pass(self.frequency_hz, q, sample_rate_hz)
            }),
            FilterKind::HighCut(slope) => SectionCascade::from_qs(slope.section_qs(), |q| {
                BiquadCoefficients::low_pass(self.frequency_hz, q, sample_rate_hz)
            }),
        }
    }

    pub fn response_at(&self, frequency_hz: f64, sample_rate_hz: f64) -> Complex64 {
        self.sections(sample_rate_hz)
            .response_at(unit_delay(frequency_hz, sample_rate_hz))
    }

    pub fn bandwidth_octaves(&self) -> Option<f64> {
        self.kind.uses_q().then(|| bandwidth_octaves_from_q(self.q))
    }
}

pub fn bandwidth_octaves_from_q(q: f64) -> f64 {
    let q = q.clamp(MIN_Q, MAX_Q);
    2.0 / LN_2 * (1.0 / (2.0 * q)).asinh()
}

pub fn q_from_bandwidth_octaves(bandwidth_octaves: f64) -> f64 {
    let half_width = 0.5 * LN_2 * bandwidth_octaves.max(f64::EPSILON);
    (1.0 / (2.0 * half_width.sinh())).clamp(MIN_Q, MAX_Q)
}

pub fn eq_response(
    filters: &[EqFilter],
    frequencies_hz: &[f64],
    sample_rate_hz: f64,
    output: &mut [Complex64],
) -> Result<(), DspError> {
    check_length("eq output", frequencies_hz.len(), output.len())?;
    let mut cascades = [SectionCascade::default(); MAX_EQ_FILTERS];
    let mut cascade_count = 0;
    for (slot, filter) in cascades
        .iter_mut()
        .zip(filters.iter().filter(|filter| !filter.bypassed))
    {
        *slot = filter.sections(sample_rate_hz);
        cascade_count += 1;
    }
    let active = cascades.get(..cascade_count).unwrap_or_default();
    for (value, frequency_hz) in output.iter_mut().zip(frequencies_hz) {
        let z_inverse = unit_delay(*frequency_hz, sample_rate_hz);
        *value = active
            .iter()
            .fold(Complex64::new(1.0, 0.0), |product, cascade| {
                product * cascade.response_at(z_inverse)
            });
    }
    Ok(())
}

const MAX_SECTIONS_PER_FILTER: usize = 4;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SectionCascade {
    sections: [BiquadCoefficients; MAX_SECTIONS_PER_FILTER],
    count: usize,
}

impl SectionCascade {
    fn from_slice(sections: &[BiquadCoefficients]) -> Self {
        let mut cascade = Self::default();
        for (slot, section) in cascade.sections.iter_mut().zip(sections) {
            *slot = *section;
            cascade.count += 1;
        }
        cascade
    }

    fn from_qs(qs: &[f64], make: impl Fn(f64) -> BiquadCoefficients) -> Self {
        let mut cascade = Self::default();
        for (slot, q) in cascade.sections.iter_mut().zip(qs) {
            *slot = make(*q);
            cascade.count += 1;
        }
        cascade
    }

    pub fn as_slice(&self) -> &[BiquadCoefficients] {
        self.sections.get(..self.count).unwrap_or_default()
    }

    fn response_at(&self, z_inverse: Complex64) -> Complex64 {
        self.as_slice()
            .iter()
            .fold(Complex64::new(1.0, 0.0), |product, section| {
                product * section.response_with_unit_delay(z_inverse)
            })
    }
}

fn unit_delay(frequency_hz: f64, sample_rate_hz: f64) -> Complex64 {
    Complex64::from_polar(1.0, -TAU * frequency_hz / sample_rate_hz)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BiquadState {
    s1: f64,
    s2: f64,
}

impl BiquadState {
    pub fn process(&mut self, coefficients: &BiquadCoefficients, input: f64) -> f64 {
        let output = coefficients.b0 * input + self.s1;
        self.s1 = coefficients.b1 * input - coefficients.a1 * output + self.s2;
        self.s2 = coefficients.b2 * input - coefficients.a2 * output;
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE_HZ: f64 = 48_000.0;

    fn magnitude_db(value: Complex64) -> f64 {
        20.0 * value.norm().max(f64::MIN_POSITIVE).log10()
    }

    fn bell(frequency_hz: f64, gain_db: f64, q: f64) -> EqFilter {
        EqFilter {
            kind: FilterKind::Bell,
            frequency_hz,
            gain_db,
            q,
            bypassed: false,
        }
    }

    #[test]
    fn bell_center_gain_matches_request() {
        for gain_db in [-12.0, -6.0, 3.0, 6.0, 15.0] {
            for frequency_hz in [40.0, 1000.0, 12_000.0] {
                let response =
                    bell(frequency_hz, gain_db, 2.0).response_at(frequency_hz, SAMPLE_RATE_HZ);
                assert!((magnitude_db(response) - gain_db).abs() < 0.01);
            }
        }
    }

    #[test]
    fn opposite_bells_cancel() {
        let boost = bell(1000.0, 6.0, 2.0);
        let cut = bell(1000.0, -6.0, 2.0);
        for frequency_hz in [20.0, 200.0, 1000.0, 1500.0, 19_000.0] {
            let product = boost.response_at(frequency_hz, SAMPLE_RATE_HZ)
                * cut.response_at(frequency_hz, SAMPLE_RATE_HZ);
            assert!(magnitude_db(product).abs() < 1.0e-9);
        }
    }

    #[test]
    fn shelves_reach_their_gain_far_from_corner() {
        let low = EqFilter {
            kind: FilterKind::LowShelf,
            frequency_hz: 200.0,
            gain_db: 6.0,
            q: BUTTERWORTH_SECOND_ORDER_Q,
            bypassed: false,
        };
        let high = EqFilter {
            kind: FilterKind::HighShelf,
            ..low
        };
        assert!((magnitude_db(low.response_at(10.0, SAMPLE_RATE_HZ)) - 6.0).abs() < 0.05);
        assert!(magnitude_db(low.response_at(15_000.0, SAMPLE_RATE_HZ)).abs() < 0.05);
        assert!((magnitude_db(high.response_at(15_000.0, SAMPLE_RATE_HZ)) - 6.0).abs() < 0.05);
        assert!((magnitude_db(low.response_at(200.0, SAMPLE_RATE_HZ)) - 3.0).abs() < 0.05);
    }

    #[test]
    fn cut_filters_have_expected_corner_attenuation() {
        let cases = [
            (CutSlope::Butterworth12, -3.01),
            (CutSlope::Butterworth24, -3.01),
            (CutSlope::LinkwitzRiley24, -6.02),
            (CutSlope::LinkwitzRiley48, -6.02),
        ];
        for (slope, expected_db) in cases {
            let filter = EqFilter {
                kind: FilterKind::LowCut(slope),
                frequency_hz: 100.0,
                gain_db: 0.0,
                q: 1.0,
                bypassed: false,
            };
            let corner = magnitude_db(filter.response_at(100.0, SAMPLE_RATE_HZ));
            assert!((corner - expected_db).abs() < 0.05, "{slope:?}: {corner}");
            let octave_below = magnitude_db(filter.response_at(50.0, SAMPLE_RATE_HZ));
            let slope_db = corner - octave_below;
            assert!(
                slope_db > f64::from(slope.decibels_per_octave()) * 0.6,
                "{slope:?}: {slope_db}"
            );
        }
    }

    #[test]
    fn notch_removes_center_frequency() {
        let filter = EqFilter {
            kind: FilterKind::Notch,
            frequency_hz: 1000.0,
            gain_db: 0.0,
            q: 4.0,
            bypassed: false,
        };
        assert!(filter.response_at(1000.0, SAMPLE_RATE_HZ).norm() < 1.0e-6);
        assert!(magnitude_db(filter.response_at(100.0, SAMPLE_RATE_HZ)).abs() < 0.1);
    }

    #[test]
    fn extreme_parameters_stay_finite_and_stable() {
        for frequency_hz in [0.0, 1.0, 23_999.0, 30_000.0, f64::NAN] {
            for q in [0.0, 0.01, 50.0, f64::INFINITY] {
                for gain_db in [-24.0, 0.0, 24.0] {
                    for coefficients in [
                        BiquadCoefficients::bell(frequency_hz, q, gain_db, SAMPLE_RATE_HZ),
                        BiquadCoefficients::low_shelf(frequency_hz, q, gain_db, SAMPLE_RATE_HZ),
                        BiquadCoefficients::high_shelf(frequency_hz, q, gain_db, SAMPLE_RATE_HZ),
                        BiquadCoefficients::low_pass(frequency_hz, q, SAMPLE_RATE_HZ),
                        BiquadCoefficients::high_pass(frequency_hz, q, SAMPLE_RATE_HZ),
                        BiquadCoefficients::notch(frequency_hz, q, SAMPLE_RATE_HZ),
                    ] {
                        assert!(coefficients.is_stable(), "{coefficients:?}");
                        let response = coefficients.response_at(1000.0, SAMPLE_RATE_HZ);
                        assert!(response.re.is_finite() && response.im.is_finite());
                    }
                }
            }
        }
    }

    #[test]
    fn bypassed_filters_do_not_count() {
        let mut filter = bell(1000.0, 6.0, 2.0);
        filter.bypassed = true;
        let frequencies = [1000.0];
        let mut output = [Complex64::default()];
        eq_response(&[filter], &frequencies, SAMPLE_RATE_HZ, &mut output).unwrap();
        assert_eq!(output[0], Complex64::new(1.0, 0.0));
    }

    #[test]
    fn q_and_bandwidth_round_trip() {
        for q in [0.5, BUTTERWORTH_SECOND_ORDER_Q, 1.0, 2.0, 8.0] {
            let bandwidth = bandwidth_octaves_from_q(q);
            assert!((q_from_bandwidth_octaves(bandwidth) - q).abs() < 1.0e-9);
        }
        assert!((bandwidth_octaves_from_q(std::f64::consts::SQRT_2) - 1.0).abs() < 0.001);
    }

    #[test]
    fn time_domain_filter_matches_frequency_response() {
        let coefficients = BiquadCoefficients::bell(1000.0, 2.0, 6.0, SAMPLE_RATE_HZ);
        let mut state = BiquadState::default();
        let frequency_hz = 1000.0;
        let mut peak = 0.0_f64;
        for index in 0..48_000 {
            let input = (TAU * frequency_hz * index as f64 / SAMPLE_RATE_HZ).sin();
            let output = state.process(&coefficients, input);
            if index > 24_000 {
                peak = peak.max(output.abs());
            }
        }
        assert!((20.0 * peak.log10() - 6.0).abs() < 0.01);
    }
}
