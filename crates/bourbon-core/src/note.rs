use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;

const ID_LENGTH: usize = 12;
const ID_ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NoteId(String);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid note id: {0:?}")]
pub struct InvalidNoteId(String);

impl NoteId {
    #[must_use]
    pub fn generate() -> Self {
        let mut rng = rand::rng();
        let id = (0..ID_LENGTH)
            .map(|_| char::from(ID_ALPHABET[rng.random_range(0..ID_ALPHABET.len())]))
            .collect();
        Self(id)
    }

    pub fn parse(text: &str) -> Result<Self, InvalidNoteId> {
        let well_formed =
            text.len() == ID_LENGTH && text.bytes().all(|byte| ID_ALPHABET.contains(&byte));
        if well_formed {
            Ok(Self(text.to_owned()))
        } else {
            Err(InvalidNoteId(text.to_owned()))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NoteId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub id: NoteId,
    pub title: String,
    pub body: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteSummary {
    pub id: NoteId,
    pub title: String,
    pub updated_at: Timestamp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_id_has_twelve_lowercase_alphanumerics() {
        let id = NoteId::generate();
        assert_eq!(id.as_str().len(), 12);
        assert!(
            id.as_str()
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        );
    }

    #[test]
    fn generated_ids_differ() {
        assert_ne!(NoteId::generate(), NoteId::generate());
    }

    #[test]
    fn parse_accepts_valid_id() {
        let id = NoteId::parse("k7f3q2m9x4ta").unwrap();
        assert_eq!(id.as_str(), "k7f3q2m9x4ta");
        assert_eq!(id.to_string(), "k7f3q2m9x4ta");
    }

    #[test]
    fn parse_rejects_wrong_length_and_characters() {
        for bad in [
            "",
            "k7f3q2m9x4t",
            "k7f3q2m9x4tab",
            "K7f3q2m9x4ta",
            "k7f3q2m9x4t-",
            "k7f3q2m9x4tá",
        ] {
            assert!(NoteId::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn timestamp_round_trips_and_orders() {
        assert_eq!(Timestamp::from_millis(42).as_millis(), 42);
        assert!(Timestamp::from_millis(1) < Timestamp::from_millis(2));
    }

    #[test]
    fn now_is_after_2020() {
        assert!(Timestamp::now().as_millis() > 1_577_836_800_000);
    }
}
