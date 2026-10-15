use std::time::Duration;

/// Both backends use the same fallback for an absent or zero frame limit.
pub(super) fn frame_interval(fps: Option<u32>, power_paused: bool) -> Duration {
    if power_paused {
        Duration::from_millis(100)
    } else {
        fps.filter(|fps| *fps > 0)
            .map(|fps| Duration::from_nanos(1_000_000_000 / u64::from(fps)))
            .unwrap_or(Duration::from_millis(16))
            .max(Duration::from_nanos(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_and_extreme_fps_are_safe_and_battery_has_a_common_interval() {
        assert_eq!(frame_interval(Some(0), false), frame_interval(None, false));
        assert_eq!(frame_interval(Some(25), false), Duration::from_millis(40));
        assert_eq!(frame_interval(Some(1), false), Duration::from_secs(1));
        assert!(!frame_interval(Some(u32::MAX), false).is_zero());
        assert_eq!(frame_interval(Some(60), true), Duration::from_millis(100));
    }
}
