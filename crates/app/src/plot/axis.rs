const DECADE_MULTIPLIERS: [f64; 9] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
const LABELED_MULTIPLIERS: [f64; 3] = [1.0, 2.0, 5.0];
const FIRST_DECADE_HZ: f64 = 10.0;
const DECADE_COUNT: i32 = 4;
const DB_STEP_CHOICES: [f64; 4] = [3.0, 6.0, 12.0, 24.0];
const MAX_DB_LINES: f64 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrequencyTick {
    pub frequency_hz: f64,
    pub is_major: bool,
}

pub fn frequency_ticks(low_hz: f64, high_hz: f64) -> Vec<FrequencyTick> {
    (0..DECADE_COUNT)
        .flat_map(|decade| {
            let base = FIRST_DECADE_HZ * 10_f64.powi(decade);
            DECADE_MULTIPLIERS
                .iter()
                .map(move |multiplier| FrequencyTick {
                    frequency_hz: base * multiplier,
                    is_major: LABELED_MULTIPLIERS.contains(multiplier),
                })
        })
        .filter(|tick| (low_hz..=high_hz * 1.0001).contains(&tick.frequency_hz))
        .collect()
}

pub fn db_step(span_db: f64) -> f64 {
    DB_STEP_CHOICES
        .iter()
        .copied()
        .find(|step| span_db / step <= MAX_DB_LINES)
        .unwrap_or(DB_STEP_CHOICES[DB_STEP_CHOICES.len() - 1])
}

pub fn db_ticks(low_db: f64, high_db: f64) -> Vec<f64> {
    let step = db_step(high_db - low_db);
    let first = (low_db / step).ceil() as i64;
    let last = (high_db / step).floor() as i64;
    (first..=last).map(|index| index as f64 * step).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_range_has_standard_labeled_frequencies() {
        let labeled: Vec<f64> = frequency_ticks(20.0, 20_000.0)
            .into_iter()
            .filter(|tick| tick.is_major)
            .map(|tick| tick.frequency_hz)
            .collect();
        assert_eq!(
            labeled,
            vec![
                20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10_000.0, 20_000.0
            ]
        );
    }

    #[test]
    fn minor_ticks_fill_each_decade() {
        let ticks = frequency_ticks(100.0, 1000.0);
        assert_eq!(ticks.len(), 10);
    }

    #[test]
    fn db_ticks_use_a_readable_step() {
        assert_eq!(
            db_ticks(-24.0, 18.0),
            vec![-24.0, -18.0, -12.0, -6.0, 0.0, 6.0, 12.0, 18.0]
        );
        assert_eq!(db_step(30.0), 3.0);
        assert_eq!(db_step(200.0), 24.0);
    }
}
