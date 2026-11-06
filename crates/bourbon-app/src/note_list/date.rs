use bourbon_core::Timestamp;
use chrono::{DateTime, Datelike as _, TimeZone};

pub(crate) fn short_date<Zone: TimeZone>(updated: Timestamp, now: Timestamp, zone: &Zone) -> String
where
    Zone::Offset: std::fmt::Display,
{
    let updated = local(updated, zone);
    let now = local(now, zone);
    if updated.date_naive() == now.date_naive() {
        updated.format("%H:%M").to_string()
    } else if updated.year() == now.year() {
        updated.format("%b %-d").to_string()
    } else {
        updated.format("%b %-d, %Y").to_string()
    }
}

fn local<Zone: TimeZone>(timestamp: Timestamp, zone: &Zone) -> DateTime<Zone> {
    DateTime::from_timestamp_millis(timestamp.as_millis())
        .unwrap_or_default()
        .with_timezone(zone)
}

#[cfg(test)]
mod tests {
    use chrono::FixedOffset;

    use super::*;

    fn zone() -> FixedOffset {
        FixedOffset::west_opt(3 * 3600).unwrap()
    }

    fn at(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> Timestamp {
        let moment = zone()
            .with_ymd_and_hms(year, month, day, hour, minute, 0)
            .unwrap();
        Timestamp::from_millis(moment.timestamp_millis())
    }

    #[test]
    fn today_shows_the_local_time() {
        let now = at(2026, 11, 6, 18, 0);
        assert_eq!(short_date(at(2026, 11, 6, 14, 30), now, &zone()), "14:30");
    }

    #[test]
    fn earlier_this_year_shows_month_and_day() {
        let now = at(2026, 11, 6, 18, 0);
        assert_eq!(short_date(at(2026, 11, 5, 23, 59), now, &zone()), "Nov 5");
        assert_eq!(short_date(at(2026, 1, 31, 8, 0), now, &zone()), "Jan 31");
    }

    #[test]
    fn another_year_shows_the_year_too() {
        let now = at(2026, 1, 2, 9, 0);
        assert_eq!(
            short_date(at(2025, 11, 6, 14, 30), now, &zone()),
            "Nov 6, 2025"
        );
    }

    #[test]
    fn the_day_boundary_follows_the_zone_not_utc() {
        let now = at(2026, 11, 6, 22, 0);
        assert_eq!(short_date(at(2026, 11, 6, 0, 30), now, &zone()), "00:30");
        assert_eq!(short_date(at(2026, 11, 5, 23, 30), now, &zone()), "Nov 5");
    }
}
