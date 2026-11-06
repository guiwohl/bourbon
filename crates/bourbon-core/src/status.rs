use crate::stored_text::stored_text_enum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NoteStatus {
    #[default]
    None,
    Active,
    OnHold,
    Completed,
    Dropped,
}

impl NoteStatus {
    pub const ALL: [Self; 5] = [
        Self::None,
        Self::Active,
        Self::OnHold,
        Self::Completed,
        Self::Dropped,
    ];

    #[must_use]
    pub fn as_stored(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Active => "active",
            Self::OnHold => "on_hold",
            Self::Completed => "completed",
            Self::Dropped => "dropped",
        }
    }

    #[must_use]
    pub fn from_stored(text: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|status| status.as_stored() == text)
    }

    #[must_use]
    pub fn is_hidden_by_default(self) -> bool {
        matches!(self, Self::Completed | Self::Dropped)
    }
}

stored_text_enum!(NoteStatus);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_names_are_snake_case() {
        let stored: Vec<_> = NoteStatus::ALL
            .into_iter()
            .map(NoteStatus::as_stored)
            .collect();
        assert_eq!(
            stored,
            ["none", "active", "on_hold", "completed", "dropped"]
        );
    }

    #[test]
    fn stored_names_parse_back() {
        for status in NoteStatus::ALL {
            assert_eq!(NoteStatus::from_stored(status.as_stored()), Some(status));
        }
        assert_eq!(NoteStatus::from_stored("finished"), None);
    }

    #[test]
    fn only_completed_and_dropped_are_hidden_by_default() {
        let hidden: Vec<_> = NoteStatus::ALL
            .into_iter()
            .filter(|status| status.is_hidden_by_default())
            .collect();
        assert_eq!(hidden, [NoteStatus::Completed, NoteStatus::Dropped]);
    }

    #[test]
    fn default_status_is_none() {
        assert_eq!(NoteStatus::default(), NoteStatus::None);
    }

    #[test]
    fn status_round_trips_through_sqlite() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        let read: NoteStatus = connection
            .query_row("SELECT ?1", [NoteStatus::OnHold], |row| row.get(0))
            .unwrap();
        assert_eq!(read, NoteStatus::OnHold);
    }

    #[test]
    fn unknown_stored_status_fails_to_read() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        let result: rusqlite::Result<NoteStatus> =
            connection.query_row("SELECT 'finished'", [], |row| row.get(0));
        assert!(result.is_err());
    }
}
