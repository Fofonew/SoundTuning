pub const DEFAULT_AIR_TEMPERATURE_CELSIUS: f64 = 20.0;
const SPEED_OF_SOUND_AT_ZERO_CELSIUS_M_PER_S: f64 = 331.3;
const ZERO_CELSIUS_IN_KELVIN: f64 = 273.15;
const MILLISECONDS_PER_SECOND: f64 = 1000.0;

pub fn speed_of_sound_m_per_s(air_temperature_celsius: f64) -> f64 {
    let ratio = (1.0 + air_temperature_celsius / ZERO_CELSIUS_IN_KELVIN).max(0.0);
    SPEED_OF_SOUND_AT_ZERO_CELSIUS_M_PER_S * ratio.sqrt()
}

pub fn distance_m_for_delay_ms(delay_ms: f64, air_temperature_celsius: f64) -> f64 {
    delay_ms / MILLISECONDS_PER_SECOND * speed_of_sound_m_per_s(air_temperature_celsius)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speed_of_sound_at_twenty_celsius() {
        assert!((speed_of_sound_m_per_s(20.0) - 343.2).abs() < 0.05);
    }

    #[test]
    fn ten_milliseconds_is_about_3_43_meters() {
        assert!((distance_m_for_delay_ms(10.0, 20.0) - 3.432).abs() < 0.001);
    }

    #[test]
    fn absurd_temperature_stays_finite() {
        assert!(speed_of_sound_m_per_s(-400.0).is_finite());
    }
}
