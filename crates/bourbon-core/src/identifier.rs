use std::fmt;

use rand::Rng;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

const LENGTH: usize = 12;
const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid identifier: {0:?}")]
pub struct InvalidIdentifier(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Identifier(String);

impl Identifier {
    fn generate() -> Self {
        let mut rng = rand::rng();
        let text = (0..LENGTH)
            .map(|_| char::from(ALPHABET[rng.random_range(0..ALPHABET.len())]))
            .collect();
        Self(text)
    }

    fn parse(text: &str) -> Result<Self, InvalidIdentifier> {
        let well_formed = text.len() == LENGTH && text.bytes().all(|byte| ALPHABET.contains(&byte));
        if well_formed {
            Ok(Self(text.to_owned()))
        } else {
            Err(InvalidIdentifier(text.to_owned()))
        }
    }
}

macro_rules! identifier_type {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(Identifier);

        impl $name {
            #[must_use]
            pub fn generate() -> Self {
                Self(Identifier::generate())
            }

            pub fn parse(text: &str) -> Result<Self, InvalidIdentifier> {
                Identifier::parse(text).map(Self)
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl ToSql for $name {
            fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
                self.as_str().to_sql()
            }
        }

        impl FromSql for $name {
            fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
                Self::parse(value.as_str()?).map_err(|error| FromSqlError::Other(Box::new(error)))
            }
        }
    };
}

identifier_type!(NoteId);
identifier_type!(NotebookId);
identifier_type!(TagId);

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
        assert_ne!(NotebookId::generate(), NotebookId::generate());
        assert_ne!(TagId::generate(), TagId::generate());
    }

    #[test]
    fn parse_accepts_valid_id_for_every_kind() {
        let note = NoteId::parse("k7f3q2m9x4ta").unwrap();
        assert_eq!(note.as_str(), "k7f3q2m9x4ta");
        assert_eq!(note.to_string(), "k7f3q2m9x4ta");
        assert_eq!(
            NotebookId::parse("k7f3q2m9x4ta").unwrap().as_str(),
            "k7f3q2m9x4ta"
        );
        assert_eq!(
            TagId::parse("k7f3q2m9x4ta").unwrap().to_string(),
            "k7f3q2m9x4ta"
        );
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
            assert!(NotebookId::parse(bad).is_err(), "{bad}");
            assert!(TagId::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn invalid_identifier_error_names_the_offending_text() {
        let error = TagId::parse("nope").unwrap_err();
        assert_eq!(error.to_string(), "invalid identifier: \"nope\"");
    }

    #[test]
    fn identifier_round_trips_through_sqlite() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        let id = NotebookId::generate();
        let read: NotebookId = connection
            .query_row("SELECT ?1", [&id], |row| row.get(0))
            .unwrap();
        assert_eq!(read, id);
    }

    #[test]
    fn malformed_stored_identifier_fails_to_read() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        let result: rusqlite::Result<NoteId> =
            connection.query_row("SELECT 'BAD'", [], |row| row.get(0));
        assert!(result.is_err());
    }
}
