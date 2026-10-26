pub const MIN_POWER_FOR_DB: f64 = 1.0e-30;

pub fn power_to_db(power: f64) -> Option<f64> {
    (power.is_finite() && power >= MIN_POWER_FOR_DB).then(|| 10.0 * power.log10())
}

pub fn amplitude_to_db(amplitude: f64) -> Option<f64> {
    power_to_db(amplitude * amplitude)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unity_power_is_zero_db() {
        assert_eq!(power_to_db(1.0), Some(0.0));
        assert!((amplitude_to_db(2.0).unwrap() - 6.0206).abs() < 1.0e-4);
    }

    #[test]
    fn zero_negative_and_non_finite_powers_are_invalid() {
        assert_eq!(power_to_db(0.0), None);
        assert_eq!(power_to_db(-1.0), None);
        assert_eq!(power_to_db(f64::NAN), None);
        assert_eq!(power_to_db(f64::INFINITY), None);
    }
}
