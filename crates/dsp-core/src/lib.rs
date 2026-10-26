mod acoustics;
mod alignment;
mod averaging;
mod biquad;
mod decibel;
mod delay;
mod error;
mod grid;
mod smoothing;
mod spectrum;
mod trace_average;
mod transfer;
mod window;

pub use acoustics::{
    DEFAULT_AIR_TEMPERATURE_CELSIUS, distance_m_for_delay_ms, speed_of_sound_m_per_s,
};
pub use alignment::{
    AlignmentCriterion, AlignmentResult, AlignmentSearch, DEFAULT_ALIGNMENT_BAND_HZ, Polarity,
    find_optimal_alignment, summed_response_db,
};
pub use averaging::{Averaging, CrossSpectrumAccumulator, exponential_smoothing_coefficient};
pub use biquad::{
    BiquadCoefficients, BiquadState, CutSlope, EqFilter, FilterKind, MAX_EQ_FILTERS,
    SectionCascade, bandwidth_octaves_from_q, eq_response, q_from_bandwidth_octaves,
};
pub use decibel::{MIN_POWER_FOR_DB, amplitude_to_db, power_to_db};
pub use delay::{DEFAULT_DELAY_FFT_LENGTH, DelayEstimate, DelayFinder, rotate_phase_for_delay};
pub use error::DspError;
pub use grid::{DISPLAY_GRID_POINTS_PER_OCTAVE, DisplayGrid};
pub use smoothing::{GridProjector, Smoothing};
pub use spectrum::SpectrumAnalyzer;
pub use trace_average::power_average_db;
pub use transfer::{
    BinTransfer, compute_bin_transfer, compute_transfer, min_valid_auto_spectrum_power,
};
pub use window::hann_window;

pub type Complex64 = num_complex::Complex<f64>;
