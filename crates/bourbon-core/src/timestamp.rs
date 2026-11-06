use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(i64);

impl Timestamp {
    #[must_use]
    pub fn now() -> Self {
        let since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Self(i64::try_from(since_epoch.as_millis()).unwrap_or(i64::MAX))
    }

    #[must_use]
    pub fn from_millis(millis: i64) -> Self {
        Self(millis)
    }

    #[must_use]
    pub fn as_millis(self) -> i64 {
        self.0
    }
}

impl ToSql for Timestamp {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl FromSql for Timestamp {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        i64::column_result(value).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_round_trips_and_orders() {
        assert_eq!(Timestamp::from_millis(42).as_millis(), 42);
        assert!(Timestamp::from_millis(1) < Timestamp::from_millis(2));
    }

    #[test]
    fn now_is_after_2020() {
        assert!(Timestamp::now().as_millis() > 1_577_836_800_000);
    }

    #[test]
    fn timestamp_round_trips_through_sqlite() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        let read: Timestamp = connection
            .query_row("SELECT ?1", [Timestamp::from_millis(77)], |row| row.get(0))
            .unwrap();
        assert_eq!(read, Timestamp::from_millis(77));
    }
}
