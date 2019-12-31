use std::time::Duration;

const MILLISECONDS_PER_SECOND: u64 = 1_000;
const SECONDS_PER_MINUTE: u64 = 60;
const MINUTES_PER_HOUR: u64 = 60;
const HOURS_PER_DAY: u64 = 24;
const SECONDS_PER_HOUR: u64 = SECONDS_PER_MINUTE * MINUTES_PER_HOUR;
const SECONDS_PER_DAY: u64 = SECONDS_PER_HOUR * HOURS_PER_DAY;

pub fn duration_format(duration: &Duration) -> String {
    // TODO(#17): Simplify this logic when `std::time::Duration::as_millis`,
    // `std::time::Duration::as_float_secs`, etc. are stabilized.
    let seconds = duration.as_secs();
    let milliseconds = seconds * 1_000 + u64::from(duration.subsec_millis());
    let minutes = seconds / SECONDS_PER_MINUTE;
    let hours = minutes / MINUTES_PER_HOUR;
    let days = seconds / SECONDS_PER_DAY;

    if milliseconds < MILLISECONDS_PER_SECOND {
        return format!("{}ms", milliseconds);
    }
    if seconds < SECONDS_PER_MINUTE {
        return format!("{}s", seconds);
    }
    if seconds < SECONDS_PER_HOUR {
        return format!("{}m {}s", minutes, (seconds % SECONDS_PER_MINUTE));
    }
    if seconds < SECONDS_PER_DAY {
        return format!(
            "{}h {}m {}s",
            hours,
            (minutes % MINUTES_PER_HOUR),
            (seconds % SECONDS_PER_MINUTE)
        );
    }
    format!(
        "{}d {}h {}m",
        days,
        (hours % HOURS_PER_DAY),
        (minutes % MINUTES_PER_HOUR)
    )
}

#[cfg(test)]
mod test {
    use super::*;

    fn millis(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    fn secs(secs: u64) -> Duration {
        Duration::from_secs(secs)
    }

    #[test]
    fn formats_milliseconds() {
        assert_eq!(duration_format(&millis(0)), "0ms");
        assert_eq!(
            duration_format(&millis(MILLISECONDS_PER_SECOND - 1)),
            "999ms"
        );
    }

    #[test]
    fn formats_seconds() {
        assert_eq!(duration_format(&millis(MILLISECONDS_PER_SECOND)), "1s");
        assert_eq!(duration_format(&secs(SECONDS_PER_MINUTE - 1)), "59s");
    }

    #[test]
    fn formats_minutes() {
        assert_eq!(duration_format(&secs(SECONDS_PER_MINUTE)), "1m 0s");
        assert_eq!(duration_format(&secs(SECONDS_PER_HOUR - 1)), "59m 59s");
        assert_eq!(duration_format(&secs(SECONDS_PER_HOUR)), "1h 0m 0s");
        assert_eq!(duration_format(&secs(SECONDS_PER_DAY - 1)), "23h 59m 59s");
    }

    #[test]
    fn formats_days() {
        assert_eq!(duration_format(&secs(SECONDS_PER_DAY)), "1d 0h 0m");
        assert_eq!(
            duration_format(&secs(
                3 * SECONDS_PER_DAY +
                    14 * SECONDS_PER_HOUR +
                    16 * SECONDS_PER_MINUTE
            )),
            "3d 14h 16m"
        );
    }
}
