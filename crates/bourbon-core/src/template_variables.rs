use jiff::tz::TimeZone;

use crate::timestamp::Timestamp;

#[derive(Debug, Clone)]
pub struct TemplateTimeZone(pub(crate) TimeZone);

impl TemplateTimeZone {
    #[must_use]
    pub fn system() -> Self {
        Self(TimeZone::system())
    }

    #[must_use]
    pub fn utc() -> Self {
        Self(TimeZone::UTC)
    }
}

pub fn expand(text: &str, now: Timestamp, zone: &TemplateTimeZone) -> String {
    let Ok(instant) = jiff::Timestamp::from_millisecond(now.as_millis()) else {
        return text.to_owned();
    };
    let local = instant.to_zoned(zone.0.clone());
    text.replace("{{date}}", &local.strftime("%Y-%m-%d").to_string())
        .replace("{{time}}", &local.strftime("%H:%M").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAY_FIRST_2026_AT_0230_UTC: i64 = 1_777_602_600_000;

    fn sao_paulo() -> TemplateTimeZone {
        TemplateTimeZone(TimeZone::get("America/Sao_Paulo").unwrap())
    }

    fn utc_expand(text: &str) -> String {
        expand(
            text,
            Timestamp::from_millis(MAY_FIRST_2026_AT_0230_UTC),
            &TemplateTimeZone::utc(),
        )
    }

    #[test]
    fn date_becomes_year_month_day() {
        assert_eq!(utc_expand("on {{date}}"), "on 2026-05-01");
    }

    #[test]
    fn time_becomes_twenty_four_hour_clock() {
        assert_eq!(utc_expand("at {{time}}"), "at 02:30");
    }

    #[test]
    fn afternoon_time_is_zero_padded_twenty_four_hour() {
        let afternoon =
            Timestamp::from_millis(MAY_FIRST_2026_AT_0230_UTC + 12 * 3_600_000 + 300_000);
        assert_eq!(
            expand("{{time}}", afternoon, &TemplateTimeZone::utc()),
            "14:35"
        );
    }

    #[test]
    fn every_occurrence_is_replaced() {
        assert_eq!(
            utc_expand("{{date}} {{date}} {{time}} {{time}}"),
            "2026-05-01 2026-05-01 02:30 02:30"
        );
    }

    #[test]
    fn other_variables_and_wrong_case_are_untouched() {
        assert_eq!(
            utc_expand("{{Date}} {{TIME}} {{name}} {{ date }} {date} {{date"),
            "{{Date}} {{TIME}} {{name}} {{ date }} {date} {{date"
        );
    }

    #[test]
    fn text_without_variables_is_unchanged() {
        assert_eq!(utc_expand("plain ç text"), "plain ç text");
    }

    #[test]
    fn local_zone_crosses_the_date_boundary() {
        let now = Timestamp::from_millis(MAY_FIRST_2026_AT_0230_UTC);
        assert_eq!(
            expand("{{date}} {{time}}", now, &sao_paulo()),
            "2026-04-30 23:30"
        );
    }

    #[test]
    fn unrepresentable_timestamp_leaves_text_unchanged() {
        assert_eq!(
            expand(
                "{{date}}",
                Timestamp::from_millis(i64::MAX),
                &TemplateTimeZone::utc()
            ),
            "{{date}}"
        );
    }

    #[test]
    fn system_zone_expands_to_a_date_shape() {
        let expanded = expand(
            "{{date}}",
            Timestamp::from_millis(MAY_FIRST_2026_AT_0230_UTC),
            &TemplateTimeZone::system(),
        );
        assert_eq!(expanded.len(), 10);
        assert!(expanded.starts_with("2026-"));
    }
}
