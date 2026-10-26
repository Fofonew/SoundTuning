use crate::decibel::power_to_db;
use crate::error::{DspError, check_length};

pub fn power_average_db(
    traces_db: &[&[Option<f64>]],
    output: &mut [Option<f64>],
) -> Result<(), DspError> {
    for trace in traces_db {
        check_length("trace", output.len(), trace.len())?;
    }
    for (index, slot) in output.iter_mut().enumerate() {
        let (power_sum, count) = traces_db
            .iter()
            .filter_map(|trace| trace.get(index).copied().flatten())
            .filter(|level_db| level_db.is_finite())
            .fold((0.0_f64, 0_usize), |(sum, count), level_db| {
                (sum + 10.0_f64.powf(level_db / 10.0), count + 1)
            });
        *slot = if count == 0 {
            None
        } else {
            power_to_db(power_sum / count as f64)
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn average_of_identical_traces_is_the_trace() {
        let trace = [Some(-3.0), Some(0.0), Some(12.5), None];
        let mut output = [None; 4];
        power_average_db(&[&trace, &trace], &mut output).unwrap();
        for (average, original) in output.iter().zip(trace) {
            match (average, original) {
                (Some(average), Some(original)) => assert!((average - original).abs() < 0.01),
                (None, None) => {}
                mismatch => panic!("unexpected {mismatch:?}"),
            }
        }
    }

    #[test]
    fn average_is_in_power_domain() {
        let mut output = [None];
        power_average_db(&[&[Some(0.0)], &[Some(-200.0)]], &mut output).unwrap();
        assert!((output[0].unwrap() + 3.0103).abs() < 0.001);
    }

    #[test]
    fn invalid_points_are_skipped_not_zeroed() {
        let mut output = [None];
        power_average_db(&[&[Some(6.0)], &[None]], &mut output).unwrap();
        assert!((output[0].unwrap() - 6.0).abs() < 1.0e-9);
    }

    #[test]
    fn mismatched_lengths_are_rejected() {
        let mut output = [None; 2];
        assert!(power_average_db(&[&[Some(0.0)]], &mut output).is_err());
    }
}
