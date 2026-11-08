use bourbon_core::Timestamp;
use jiff::SignedDuration;
use jiff::tz::TimeZone;

const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;
const WEEK: i64 = 7 * DAY;
const MONTH: i64 = 30 * DAY;
const YEAR: i64 = 365 * DAY;

const ABSOLUTE_AFTER: i64 = 37 * DAY;

pub(crate) fn date_label(stamp: Timestamp, now: Timestamp, zone: &TimeZone) -> String {
    let elapsed = SignedDuration::from_millis(now.as_millis() - stamp.as_millis()).as_secs();
    if elapsed > ABSOLUTE_AFTER {
        return absolute_date(stamp, zone);
    }
    relative_age(stamp, now)
}

fn absolute_date(stamp: Timestamp, zone: &TimeZone) -> String {
    jiff::Timestamp::from_millisecond(stamp.as_millis())
        .map(|instant| {
            instant
                .to_zoned(zone.clone())
                .strftime("%Y-%m-%d")
                .to_string()
        })
        .unwrap_or_default()
}

pub(crate) fn relative_age(updated: Timestamp, now: Timestamp) -> String {
    let elapsed = SignedDuration::from_millis(now.as_millis() - updated.as_millis()).as_secs();
    let (count, unit) = if elapsed < MINUTE {
        return "just now".to_owned();
    } else if elapsed < HOUR {
        (elapsed / MINUTE, "minute")
    } else if elapsed < DAY {
        (elapsed / HOUR, "hour")
    } else if elapsed < WEEK {
        (elapsed / DAY, "day")
    } else if elapsed < MONTH {
        (elapsed / WEEK, "week")
    } else if elapsed < YEAR {
        (elapsed / MONTH, "month")
    } else {
        (elapsed / YEAR, "year")
    };
    ago(count, unit)
}

fn ago(count: i64, unit: &str) -> String {
    let plural = if count == 1 { "" } else { "s" };
    format!("{count} {unit}{plural} ago")
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 2_000_000_000_000;

    fn age(seconds: i64) -> String {
        relative_age(
            Timestamp::from_millis(NOW - seconds * 1000),
            Timestamp::from_millis(NOW),
        )
    }

    #[test]
    fn under_a_minute_is_just_now() {
        assert_eq!(age(0), "just now");
        assert_eq!(age(59), "just now");
    }

    #[test]
    fn a_future_timestamp_is_just_now() {
        assert_eq!(age(-30), "just now");
    }

    #[test]
    fn minutes_are_counted_and_pluralised() {
        assert_eq!(age(60), "1 minute ago");
        assert_eq!(age(5 * 60 + 59), "5 minutes ago");
        assert_eq!(age(59 * 60 + 59), "59 minutes ago");
    }

    #[test]
    fn hours_are_counted_and_pluralised() {
        assert_eq!(age(3600), "1 hour ago");
        assert_eq!(age(2 * 3600), "2 hours ago");
        assert_eq!(age(23 * 3600 + 3599), "23 hours ago");
    }

    #[test]
    fn days_are_counted_up_to_six() {
        assert_eq!(age(86_400), "1 day ago");
        assert_eq!(age(4 * 86_400), "4 days ago");
        assert_eq!(age(6 * 86_400 + 86_399), "6 days ago");
    }

    #[test]
    fn weeks_run_until_thirty_days() {
        assert_eq!(age(7 * 86_400), "1 week ago");
        assert_eq!(age(14 * 86_400), "2 weeks ago");
        assert_eq!(age(29 * 86_400), "4 weeks ago");
    }

    #[test]
    fn months_and_years_follow() {
        assert_eq!(age(30 * 86_400), "1 month ago");
        assert_eq!(age(90 * 86_400), "3 months ago");
        assert_eq!(age(365 * 86_400), "1 year ago");
        assert_eq!(age(800 * 86_400), "2 years ago");
    }

    fn label(days: i64, extra_seconds: i64) -> String {
        date_label(
            Timestamp::from_millis(NOW - (days * 86_400 + extra_seconds) * 1000),
            Timestamp::from_millis(NOW),
            &TimeZone::UTC,
        )
    }

    #[test]
    fn a_note_up_to_thirty_seven_days_old_keeps_the_relative_date() {
        assert_eq!(label(0, 5 * 60), "5 minutes ago");
        assert_eq!(label(37, 0), "1 month ago");
    }

    #[test]
    fn a_note_older_than_thirty_seven_days_shows_the_calendar_date() {
        assert_eq!(label(37, 1), "2033-04-11");
        assert_eq!(label(400, 0), "2032-04-13");
    }

    #[test]
    fn the_calendar_date_is_read_in_the_given_time_zone() {
        let stamp = Timestamp::from_millis(NOW - 100 * 86_400 * 1000);
        let now = Timestamp::from_millis(NOW);
        let utc = date_label(stamp, now, &TimeZone::UTC);
        let behind = date_label(stamp, now, &TimeZone::fixed(jiff::tz::offset(-5)));
        assert_eq!(utc, "2033-02-07");
        assert_eq!(behind, "2033-02-06");
    }
}
