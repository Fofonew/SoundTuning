#![allow(clippy::unwrap_used)]

mod common;

use common::{
    DEFAULT_FFT_LENGTH, DEFAULT_HOP, NoiseSource, SAMPLE_RATE_HZ, circular_fractional_delay,
    measure, power_db, rms, scaled_to_rms,
};
use dsp_core::{
    AlignmentCriterion, AlignmentSearch, Averaging, BiquadCoefficients, BiquadState, Complex64,
    CrossSpectrumAccumulator, DEFAULT_DELAY_FFT_LENGTH, DelayFinder, DisplayGrid, EqFilter,
    FilterKind, GridProjector, Polarity, Smoothing, eq_response, find_optimal_alignment,
    power_average_db, summed_response_db,
};

const MILLISECONDS_PER_SECOND: f64 = 1000.0;

#[test]
fn delay_of_3_2_ms_in_pink_noise_with_20_db_snr_is_found_within_0_02_ms() {
    let total_length = 1 << 18;
    let analysis_start = 20_000;
    let delay_ms = 3.2;
    let mut noise = NoiseSource::new(42);
    let reference = noise.pink_signal(total_length);
    let delayed = circular_fractional_delay(
        &reference,
        delay_ms / MILLISECONDS_PER_SECOND * SAMPLE_RATE_HZ,
    );
    let uncorrelated = scaled_to_rms(&noise.white_signal(total_length), rms(&delayed) / 10.0);
    let measurement: Vec<f64> = delayed
        .iter()
        .zip(&uncorrelated)
        .map(|(signal, noise)| signal + noise)
        .collect();

    let window = analysis_start..analysis_start + DEFAULT_DELAY_FFT_LENGTH;
    let mut finder = DelayFinder::new(DEFAULT_DELAY_FFT_LENGTH).unwrap();
    let estimate = finder
        .estimate(&reference[window.clone()], &measurement[window])
        .unwrap()
        .unwrap();
    let found_ms = estimate.delay_ms(SAMPLE_RATE_HZ);
    assert!(
        (found_ms - delay_ms).abs() <= 0.02,
        "found {found_ms} ms, expected {delay_ms} ms"
    );
    assert!(!estimate.inverted_polarity);
    assert!(estimate.confidence_db > 20.0);
}

#[test]
fn h1_of_a_bell_matches_the_filter_within_0_2_db_where_coherent() {
    let bell = BiquadCoefficients::bell(1000.0, 2.0, 6.0, SAMPLE_RATE_HZ);
    let mut noise = NoiseSource::new(7);
    let reference = noise.pink_signal(48_000 * 12);
    let mut state = BiquadState::default();
    let measurement: Vec<f64> = reference
        .iter()
        .map(|sample| state.process(&bell, *sample))
        .collect();

    let result = measure(
        &reference,
        &measurement,
        DEFAULT_FFT_LENGTH,
        DEFAULT_HOP,
        Averaging::Infinite,
    );
    let grid = DisplayGrid::default();
    let mut projector =
        GridProjector::new(&grid, DEFAULT_FFT_LENGTH, SAMPLE_RATE_HZ, Smoothing::None);
    let mut power = vec![None; grid.len()];
    let mut coherence = vec![None; grid.len()];
    projector.project_power(&result.bins, &mut power).unwrap();
    projector
        .project_coherence(&result.bins, &mut coherence)
        .unwrap();

    let mut checked_points = 0;
    for ((frequency_hz, power), coherence) in
        grid.frequencies_hz().iter().zip(&power).zip(&coherence)
    {
        if !(50.0..=18_000.0).contains(frequency_hz) || coherence.is_none_or(|value| value <= 0.95)
        {
            continue;
        }
        let measured_db = power_db(power.unwrap());
        let expected_db = power_db(bell.response_at(*frequency_hz, SAMPLE_RATE_HZ).norm_sqr());
        assert!(
            (measured_db - expected_db).abs() < 0.2,
            "{frequency_hz} Hz: measured {measured_db} dB, expected {expected_db} dB"
        );
        checked_points += 1;
    }
    assert!(
        checked_points > 800,
        "only {checked_points} coherent points"
    );
}

#[test]
fn uncorrelated_noise_at_0_db_snr_gives_coherence_of_one_half() {
    let length = 48_000 * 60;
    let mut noise = NoiseSource::new(11);
    let reference = noise.pink_signal(length);
    let mut other_noise = NoiseSource::new(99_991);
    let uncorrelated = scaled_to_rms(&other_noise.pink_signal(length), rms(&reference));
    let measurement: Vec<f64> = reference
        .iter()
        .zip(&uncorrelated)
        .map(|(signal, noise)| signal + noise)
        .collect();

    let result = measure(
        &reference,
        &measurement,
        DEFAULT_FFT_LENGTH,
        DEFAULT_HOP,
        Averaging::Infinite,
    );
    let grid = DisplayGrid::default();
    let mut projector = GridProjector::new(
        &grid,
        DEFAULT_FFT_LENGTH,
        SAMPLE_RATE_HZ,
        Smoothing::OctaveFraction(12),
    );
    let mut coherence = vec![None; grid.len()];
    projector
        .project_coherence(&result.bins, &mut coherence)
        .unwrap();

    let in_band: Vec<f64> = grid
        .frequencies_hz()
        .iter()
        .zip(&coherence)
        .filter(|(frequency_hz, _)| (500.0..=15_000.0).contains(*frequency_hz))
        .map(|(_, value)| value.unwrap())
        .collect();
    for value in &in_band {
        assert!((value - 0.5).abs() <= 0.05, "coherence {value}");
    }
    let mean = in_band.iter().sum::<f64>() / in_band.len() as f64;
    assert!((mean - 0.5).abs() <= 0.02, "mean coherence {mean}");
}

fn measured_time_constant_s(hop: usize, time_constant_s: f64) -> f64 {
    let averaging = Averaging::Exponential { time_constant_s };
    let mut accumulator = CrossSpectrumAccumulator::new(1, averaging, hop, SAMPLE_RATE_HZ);
    let silence = [Complex64::new(0.0, 0.0)];
    let step = [Complex64::new(1.0, 0.0)];
    accumulator.update(&silence, &silence).unwrap();
    let hop_s = hop as f64 / SAMPLE_RATE_HZ;
    let target_remaining = (-1.0_f64).exp();
    let mut previous_remaining = 1.0_f64;
    let mut elapsed_s = 0.0;
    loop {
        accumulator.update(&step, &step).unwrap();
        elapsed_s += hop_s;
        let remaining = 1.0 - accumulator.gxx()[0];
        if remaining <= target_remaining {
            let fraction = (previous_remaining.ln() - target_remaining.ln())
                / (previous_remaining.ln() - remaining.ln());
            return elapsed_s - hop_s + fraction * hop_s;
        }
        previous_remaining = remaining;
    }
}

#[test]
fn averaging_time_constant_does_not_depend_on_hop() {
    for time_constant_s in [0.5, 1.0, 2.0, 4.0] {
        let with_2048 = measured_time_constant_s(2048, time_constant_s);
        let with_4096 = measured_time_constant_s(4096, time_constant_s);
        assert!(
            (with_2048 - with_4096).abs() / with_2048 <= 0.05,
            "tau {time_constant_s}: {with_2048} vs {with_4096}"
        );
        assert!((with_2048 - time_constant_s).abs() / time_constant_s <= 0.05);
    }
}

#[test]
fn bell_center_gain_is_within_0_01_db() {
    for gain_db in [-15.0, -6.0, 1.5, 6.0, 12.0] {
        let filter = EqFilter {
            kind: FilterKind::Bell,
            frequency_hz: 2500.0,
            gain_db,
            q: 2.0,
            bypassed: false,
        };
        let center = power_db(filter.response_at(2500.0, SAMPLE_RATE_HZ).norm_sqr());
        assert!((center - gain_db).abs() <= 0.01);
    }
}

#[test]
fn flat_response_stays_flat_after_smoothing() {
    let mut noise = NoiseSource::new(5);
    let signal = noise.pink_signal(48_000 * 4);
    let result = measure(
        &signal,
        &signal,
        DEFAULT_FFT_LENGTH,
        DEFAULT_HOP,
        Averaging::Infinite,
    );
    let grid = DisplayGrid::default();
    let mut power = vec![None; grid.len()];
    for smoothing in Smoothing::CHOICES {
        let mut projector =
            GridProjector::new(&grid, DEFAULT_FFT_LENGTH, SAMPLE_RATE_HZ, smoothing);
        projector.project_power(&result.bins, &mut power).unwrap();
        for value in &power {
            assert!(power_db(value.unwrap()).abs() <= 0.01, "{smoothing:?}");
        }
    }
}

#[test]
fn opposite_virtual_eq_flattens_a_bell_trace() {
    let grid = DisplayGrid::default();
    let trace_filter = EqFilter {
        kind: FilterKind::Bell,
        frequency_hz: 1000.0,
        gain_db: 6.0,
        q: 2.0,
        bypassed: false,
    };
    let correction = EqFilter {
        gain_db: -6.0,
        ..trace_filter
    };
    let mut trace = vec![Complex64::default(); grid.len()];
    let mut eq = vec![Complex64::default(); grid.len()];
    eq_response(
        &[trace_filter],
        grid.frequencies_hz(),
        SAMPLE_RATE_HZ,
        &mut trace,
    )
    .unwrap();
    eq_response(
        &[correction],
        grid.frequencies_hz(),
        SAMPLE_RATE_HZ,
        &mut eq,
    )
    .unwrap();
    for (trace_value, eq_value) in trace.iter().zip(&eq) {
        assert!(power_db((trace_value * eq_value).norm_sqr()).abs() <= 0.1);
    }
}

#[test]
fn summation_of_identical_traces_adds_6_02_db_and_half_period_delay_cancels() {
    let grid = DisplayGrid::default();
    let frequencies = grid.frequencies_hz();
    let trace: Vec<Option<Complex64>> = frequencies
        .iter()
        .map(|frequency| Some(Complex64::from_polar(0.5, -frequency * 0.001)))
        .collect();
    let mut summed = vec![None; grid.len()];
    summed_response_db(
        &trace,
        &trace,
        frequencies,
        0.0,
        Polarity::Normal,
        &mut summed,
    )
    .unwrap();
    for (sum, single) in summed.iter().zip(&trace) {
        let single_db = power_db(single.unwrap().norm_sqr());
        assert!((sum.unwrap() - single_db - 6.0206).abs() < 0.001);
    }

    let null_index = grid.index_range_for_band(1000.0, 1100.0).start;
    let null_frequency_hz = frequencies[null_index];
    let half_period_ms = 0.5 / null_frequency_hz * MILLISECONDS_PER_SECOND;
    let mut cancelled = vec![None; grid.len()];
    summed_response_db(
        &trace,
        &trace,
        frequencies,
        half_period_ms,
        Polarity::Normal,
        &mut cancelled,
    )
    .unwrap();
    let dip_db = summed[null_index].unwrap() - cancelled[null_index].unwrap();
    assert!(dip_db > 30.0, "dip of {dip_db} dB");
}

#[test]
fn optimal_alignment_delays_a_by_1_5_ms_for_both_criteria() {
    let grid = DisplayGrid::default();
    let frequencies = grid.frequencies_hz();
    let propagation_ms = 4.0;
    let delay_of_b_ms = 1.5;
    let shifted = |delay_ms: f64| -> Vec<Option<Complex64>> {
        frequencies
            .iter()
            .map(|frequency| {
                Some(Complex64::from_polar(
                    1.0,
                    -std::f64::consts::TAU * frequency * delay_ms / MILLISECONDS_PER_SECOND,
                ))
            })
            .collect()
    };
    let trace_a = shifted(propagation_ms);
    let trace_b = shifted(propagation_ms + delay_of_b_ms);
    for criterion in [
        AlignmentCriterion::MaximumLevel,
        AlignmentCriterion::Flattest,
    ] {
        let search = AlignmentSearch {
            criterion,
            ..AlignmentSearch::default()
        };
        let result = find_optimal_alignment(&trace_a, &trace_b, frequencies, &search)
            .unwrap()
            .unwrap();
        assert!(
            (result.delay_ms + delay_of_b_ms).abs() <= 0.02,
            "{criterion:?}: delta {} ms",
            result.delay_ms
        );
        assert_eq!(result.polarity, Polarity::Normal, "{criterion:?}");
    }
}

#[test]
fn average_of_two_identical_traces_equals_each_trace() {
    let grid = DisplayGrid::default();
    let trace: Vec<Option<f64>> = grid
        .frequencies_hz()
        .iter()
        .map(|frequency| Some(-6.0 + 3.0 * (frequency / 1000.0).log2().sin()))
        .collect();
    let mut average = vec![None; grid.len()];
    power_average_db(&[&trace, &trace], &mut average).unwrap();
    for (average, original) in average.iter().zip(&trace) {
        assert!((average.unwrap() - original.unwrap()).abs() <= 0.01);
    }
}

fn assert_all_finite(values: &[Option<f64>]) {
    assert!(values.iter().flatten().all(|value| value.is_finite()));
}

#[test]
fn silent_bands_produce_no_nan_or_infinity_and_are_invalid() {
    let fft_length = DEFAULT_FFT_LENGTH;
    let bin_spacing_hz = SAMPLE_RATE_HZ / fft_length as f64;
    let length = fft_length * 8;
    let reference: Vec<f64> = (0..length)
        .map(|index| {
            (70..700)
                .step_by(7)
                .map(|bin| {
                    (std::f64::consts::TAU * bin as f64 * index as f64 / fft_length as f64
                        + bin as f64)
                        .sin()
                })
                .sum::<f64>()
                / 100.0
        })
        .collect();
    let measurement: Vec<f64> = reference.iter().map(|value| value * 2.0).collect();
    let grid = DisplayGrid::default();
    let mut power = vec![None; grid.len()];
    let mut coherence = vec![None; grid.len()];
    let mut complex = vec![None; grid.len()];

    for (reference, measurement) in [
        (reference.clone(), measurement.clone()),
        (vec![0.0; length], vec![0.0; length]),
        (vec![0.0; length], measurement.clone()),
    ] {
        let result = measure(
            &reference,
            &measurement,
            fft_length,
            DEFAULT_HOP,
            Averaging::Infinite,
        );
        for smoothing in Smoothing::CHOICES {
            let mut projector = GridProjector::new(&grid, fft_length, SAMPLE_RATE_HZ, smoothing);
            projector.project_power(&result.bins, &mut power).unwrap();
            projector
                .project_coherence(&result.bins, &mut coherence)
                .unwrap();
            projector
                .project_complex_ratio(
                    result.accumulator.gxy(),
                    result.accumulator.gxx(),
                    1.0e-30,
                    &mut complex,
                )
                .unwrap();
            assert_all_finite(&power);
            assert_all_finite(&coherence);
            assert!(
                complex
                    .iter()
                    .flatten()
                    .all(|value| value.re.is_finite() && value.im.is_finite())
            );
        }
        assert!(result.bins.iter().all(|bin| {
            bin.response
                .is_none_or(|value| value.re.is_finite() && value.im.is_finite())
                && bin.coherence.is_none_or(f64::is_finite)
        }));
    }

    let result = measure(
        &reference,
        &measurement,
        fft_length,
        DEFAULT_HOP,
        Averaging::Infinite,
    );
    let silent_bin = (5000.0 / bin_spacing_hz) as usize;
    assert!(result.bins[silent_bin].response.is_none());
    let mut projector = GridProjector::new(&grid, fft_length, SAMPLE_RATE_HZ, Smoothing::None);
    projector.project_power(&result.bins, &mut power).unwrap();
    for (frequency_hz, value) in grid.frequencies_hz().iter().zip(&power) {
        if *frequency_hz > 2000.0 {
            assert!(value.is_none(), "{frequency_hz} Hz should be invalid");
        }
    }
    let silent = measure(
        &vec![0.0; length],
        &vec![0.0; length],
        fft_length,
        DEFAULT_HOP,
        Averaging::Infinite,
    );
    assert!(silent.bins.iter().all(|bin| bin.response.is_none()));
    let mut finder = DelayFinder::new(DEFAULT_DELAY_FFT_LENGTH).unwrap();
    let zeros = vec![0.0; DEFAULT_DELAY_FFT_LENGTH];
    assert_eq!(finder.estimate(&zeros, &zeros).unwrap(), None);
}

#[test]
fn sub_sample_delay_is_resolved_within_a_tenth_of_a_sample() {
    let total_length = 1 << 18;
    let analysis_start = 30_000;
    let delay_samples = 153.6 + 0.37;
    let mut noise = NoiseSource::new(77);
    let reference = noise.white_signal(total_length);
    let measurement = circular_fractional_delay(&reference, delay_samples);
    let window = analysis_start..analysis_start + DEFAULT_DELAY_FFT_LENGTH;
    let mut finder = DelayFinder::new(DEFAULT_DELAY_FFT_LENGTH).unwrap();
    let estimate = finder
        .estimate(&reference[window.clone()], &measurement[window])
        .unwrap()
        .unwrap();
    assert!(
        (estimate.delay_samples - delay_samples).abs() <= 0.1,
        "found {} samples",
        estimate.delay_samples
    );
}

#[test]
fn compensating_the_found_delay_flattens_the_phase() {
    let total_length = 1 << 18;
    let delay_samples = 412.25;
    let mut noise = NoiseSource::new(123);
    let reference = noise.pink_signal(total_length);
    let measurement = circular_fractional_delay(&reference, delay_samples);
    let mut finder = DelayFinder::new(DEFAULT_DELAY_FFT_LENGTH).unwrap();
    let window = 10_000..10_000 + DEFAULT_DELAY_FFT_LENGTH;
    let estimate = finder
        .estimate(&reference[window.clone()], &measurement[window])
        .unwrap()
        .unwrap();

    let result = measure(
        &reference,
        &measurement,
        DEFAULT_FFT_LENGTH,
        DEFAULT_HOP,
        Averaging::Infinite,
    );
    let grid = DisplayGrid::default();
    let mut projector =
        GridProjector::new(&grid, DEFAULT_FFT_LENGTH, SAMPLE_RATE_HZ, Smoothing::None);
    let mut response = vec![None; grid.len()];
    projector
        .project_complex_ratio(
            result.accumulator.gxy(),
            result.accumulator.gxx(),
            dsp_core::min_valid_auto_spectrum_power(result.accumulator.gxx()),
            &mut response,
        )
        .unwrap();
    dsp_core::rotate_phase_for_delay(
        &mut response,
        grid.frequencies_hz(),
        estimate.delay_samples / SAMPLE_RATE_HZ,
    )
    .unwrap();
    for (frequency_hz, value) in grid.frequencies_hz().iter().zip(&response) {
        if (100.0..=2000.0).contains(frequency_hz) {
            let phase_degrees = value.unwrap().arg().to_degrees();
            assert!(
                phase_degrees.abs() < 5.0,
                "{frequency_hz} Hz: {phase_degrees} deg"
            );
        }
    }
}

#[test]
fn every_filter_kind_matches_its_time_domain_behaviour() {
    use dsp_core::CutSlope;
    let kinds = [
        FilterKind::Bell,
        FilterKind::LowShelf,
        FilterKind::HighShelf,
        FilterKind::Notch,
        FilterKind::LowCut(CutSlope::Butterworth24),
        FilterKind::HighCut(CutSlope::LinkwitzRiley48),
    ];
    for kind in kinds {
        let filter = EqFilter {
            kind,
            frequency_hz: 800.0,
            gain_db: 5.0,
            q: 1.2,
            bypassed: false,
        };
        let cascade = filter.sections(SAMPLE_RATE_HZ);
        for probe_hz in [150.0, 700.0, 3000.0] {
            let mut states = vec![BiquadState::default(); cascade.as_slice().len()];
            let mut energy = 0.0_f64;
            for index in 0..96_000 {
                let mut sample =
                    (std::f64::consts::TAU * probe_hz * index as f64 / SAMPLE_RATE_HZ).sin();
                for (state, section) in states.iter_mut().zip(cascade.as_slice()) {
                    sample = state.process(section, sample);
                }
                if index >= 48_000 {
                    energy += sample * sample;
                }
            }
            let amplitude = (2.0 * energy / 48_000.0).sqrt();
            let measured_db = 20.0 * amplitude.max(1.0e-12).log10();
            let expected_db = power_db(filter.response_at(probe_hz, SAMPLE_RATE_HZ).norm_sqr());
            if expected_db > -60.0 {
                assert!(
                    (measured_db - expected_db).abs() < 0.05,
                    "{kind:?} at {probe_hz} Hz: {measured_db} vs {expected_db}"
                );
            }
        }
    }
}
