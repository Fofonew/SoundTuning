pub const STANDARD_SAMPLE_RATES_HZ: [u32; 4] = [44_100, 48_000, 88_200, 96_000];
pub const PREFERRED_SAMPLE_RATES_HZ: [u32; 4] = [48_000, 44_100, 96_000, 88_200];

pub fn negotiate_sample_rate(
    requested_hz: u32,
    input_rates_hz: &[u32],
    output_rates_hz: Option<&[u32]>,
) -> Option<u32> {
    std::iter::once(requested_hz)
        .chain(PREFERRED_SAMPLE_RATES_HZ)
        .find(|rate| {
            input_rates_hz.contains(rate)
                && output_rates_hz.is_none_or(|output| output.contains(rate))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requested_rate_wins_when_both_sides_support_it() {
        assert_eq!(
            negotiate_sample_rate(96_000, &[48_000, 96_000], Some(&[44_100, 96_000])),
            Some(96_000)
        );
    }

    #[test]
    fn falls_back_to_a_rate_shared_by_input_and_output() {
        assert_eq!(
            negotiate_sample_rate(48_000, &[44_100, 48_000], Some(&[44_100])),
            Some(44_100)
        );
    }

    #[test]
    fn prefers_48_khz_among_fallbacks() {
        assert_eq!(
            negotiate_sample_rate(88_200, &[44_100, 48_000], Some(&[44_100, 48_000])),
            Some(48_000)
        );
    }

    #[test]
    fn without_output_only_the_input_matters() {
        assert_eq!(negotiate_sample_rate(48_000, &[44_100], None), Some(44_100));
    }

    #[test]
    fn no_shared_rate_is_reported() {
        assert_eq!(
            negotiate_sample_rate(48_000, &[48_000], Some(&[44_100])),
            None
        );
        assert_eq!(negotiate_sample_rate(48_000, &[], None), None);
    }
}
