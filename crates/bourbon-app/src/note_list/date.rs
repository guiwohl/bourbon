use bourbon_core::Timestamp;
use jiff::Zoned;
use jiff::tz::TimeZone;

pub(crate) fn short_date(updated: Timestamp, now: Timestamp, zone: &TimeZone) -> String {
    let updated = local(updated, zone);
    let now = local(now, zone);
    if updated.date() == now.date() {
        updated.strftime("%H:%M").to_string()
    } else if updated.year() == now.year() {
        updated.strftime("%b %-d").to_string()
    } else {
        updated.strftime("%b %-d, %Y").to_string()
    }
}

fn local(timestamp: Timestamp, zone: &TimeZone) -> Zoned {
    jiff::Timestamp::from_millisecond(timestamp.as_millis())
        .unwrap_or_default()
        .to_zoned(zone.clone())
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;

    use super::*;

    fn zone() -> TimeZone {
        TimeZone::get("America/Sao_Paulo").unwrap()
    }

    fn at(year: i16, month: i8, day: i8, hour: i8, minute: i8) -> Timestamp {
        let moment = date(year, month, day)
            .at(hour, minute, 0, 0)
            .to_zoned(zone())
            .unwrap();
        Timestamp::from_millis(moment.timestamp().as_millisecond())
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
