use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use crate::migrations::migrate;
use crate::note::{Note, NoteId, NoteSummary, Timestamp};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("note not found: {0}")]
    NoteNotFound(NoteId),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct NoteStore {
    connection: Connection,
}

impl NoteStore {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(path)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        Self::migrated(connection)
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        Self::migrated(Connection::open_in_memory()?)
    }

    fn migrated(mut connection: Connection) -> Result<Self, StoreError> {
        migrate(&mut connection)?;
        Ok(Self { connection })
    }

    pub fn create_note(&self, now: Timestamp) -> Result<Note, StoreError> {
        let note = Note {
            id: NoteId::generate(),
            title: String::new(),
            body: String::new(),
            created_at: now,
            updated_at: now,
        };
        self.connection.execute(
            "INSERT INTO notes (id, title, body, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                note.id.as_str(),
                note.title,
                note.body,
                note.created_at.as_millis(),
                note.updated_at.as_millis()
            ],
        )?;
        Ok(note)
    }

    pub fn save_note(
        &self,
        id: &NoteId,
        title: &str,
        body: &str,
        now: Timestamp,
    ) -> Result<(), StoreError> {
        let changed = self.connection.execute(
            "UPDATE notes SET title = ?2, body = ?3, updated_at = ?4
             WHERE id = ?1 AND (title <> ?2 OR body <> ?3)",
            params![id.as_str(), title, body, now.as_millis()],
        )?;
        if changed == 0 && self.find_note(id)?.is_none() {
            return Err(StoreError::NoteNotFound(id.clone()));
        }
        Ok(())
    }

    pub fn find_note(&self, id: &NoteId) -> Result<Option<Note>, StoreError> {
        let note = self
            .connection
            .query_row(
                "SELECT id, title, body, created_at, updated_at FROM notes WHERE id = ?1",
                params![id.as_str()],
                |row| {
                    Ok(Note {
                        id: id.clone(),
                        title: row.get(1)?,
                        body: row.get(2)?,
                        created_at: Timestamp::from_millis(row.get(3)?),
                        updated_at: Timestamp::from_millis(row.get(4)?),
                    })
                },
            )
            .optional()?;
        Ok(note)
    }

    pub fn list_notes(&self) -> Result<Vec<NoteSummary>, StoreError> {
        let mut statement = self.connection.prepare(
            "SELECT id, title, updated_at FROM notes ORDER BY updated_at DESC, created_at DESC",
        )?;
        let summaries = statement
            .query_map([], |row| {
                Ok(NoteSummary {
                    id: stored_id(row, 0)?,
                    title: row.get(1)?,
                    updated_at: Timestamp::from_millis(row.get(2)?),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(summaries)
    }
}

fn stored_id(row: &rusqlite::Row<'_>, column: usize) -> rusqlite::Result<NoteId> {
    let text: String = row.get(column)?;
    NoteId::parse(&text).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            column,
            rusqlite::types::Type::Text,
            Box::new(error),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(millis: i64) -> Timestamp {
        Timestamp::from_millis(millis)
    }

    fn store() -> NoteStore {
        NoteStore::open_in_memory().unwrap()
    }

    #[test]
    fn created_note_is_empty_and_stamped() {
        let note = store().create_note(at(10)).unwrap();
        assert_eq!(note.title, "");
        assert_eq!(note.body, "");
        assert_eq!(note.created_at, at(10));
        assert_eq!(note.updated_at, at(10));
    }

    #[test]
    fn created_note_can_be_found() {
        let store = store();
        let note = store.create_note(at(10)).unwrap();
        assert_eq!(store.find_note(&note.id).unwrap(), Some(note));
    }

    #[test]
    fn find_unknown_note_is_none() {
        assert_eq!(store().find_note(&NoteId::generate()).unwrap(), None);
    }

    #[test]
    fn save_stores_title_body_and_bumps_updated_at() {
        let store = store();
        let note = store.create_note(at(10)).unwrap();
        store.save_note(&note.id, "Title", "Body", at(20)).unwrap();
        let found = store.find_note(&note.id).unwrap().unwrap();
        assert_eq!(found.title, "Title");
        assert_eq!(found.body, "Body");
        assert_eq!(found.created_at, at(10));
        assert_eq!(found.updated_at, at(20));
    }

    #[test]
    fn saving_identical_content_keeps_updated_at() {
        let store = store();
        let note = store.create_note(at(10)).unwrap();
        store.save_note(&note.id, "Title", "Body", at(20)).unwrap();
        store.save_note(&note.id, "Title", "Body", at(30)).unwrap();
        let found = store.find_note(&note.id).unwrap().unwrap();
        assert_eq!(found.updated_at, at(20));
    }

    #[test]
    fn saving_empty_content_on_new_note_is_a_no_op() {
        let store = store();
        let note = store.create_note(at(10)).unwrap();
        store.save_note(&note.id, "", "", at(30)).unwrap();
        assert_eq!(
            store.find_note(&note.id).unwrap().unwrap().updated_at,
            at(10)
        );
    }

    #[test]
    fn changing_only_title_updates() {
        let store = store();
        let note = store.create_note(at(10)).unwrap();
        store.save_note(&note.id, "A", "Body", at(20)).unwrap();
        store.save_note(&note.id, "B", "Body", at(30)).unwrap();
        assert_eq!(
            store.find_note(&note.id).unwrap().unwrap().updated_at,
            at(30)
        );
    }

    #[test]
    fn changing_only_body_updates() {
        let store = store();
        let note = store.create_note(at(10)).unwrap();
        store.save_note(&note.id, "A", "x", at(20)).unwrap();
        store.save_note(&note.id, "A", "y", at(30)).unwrap();
        assert_eq!(
            store.find_note(&note.id).unwrap().unwrap().updated_at,
            at(30)
        );
    }

    #[test]
    fn saving_unknown_note_fails() {
        let id = NoteId::generate();
        let error = store().save_note(&id, "t", "b", at(1)).unwrap_err();
        assert!(matches!(error, StoreError::NoteNotFound(missing) if missing == id));
    }

    #[test]
    fn list_orders_by_updated_at_then_created_at_descending() {
        let store = store();
        let oldest = store.create_note(at(1)).unwrap();
        let middle = store.create_note(at(2)).unwrap();
        let newest = store.create_note(at(3)).unwrap();
        let tied_early = store.create_note(at(4)).unwrap();
        let tied_late = store.create_note(at(5)).unwrap();
        store.save_note(&oldest.id, "oldest", "", at(100)).unwrap();
        store.save_note(&tied_early.id, "te", "", at(50)).unwrap();
        store.save_note(&tied_late.id, "tl", "", at(50)).unwrap();
        let ids: Vec<_> = store
            .list_notes()
            .unwrap()
            .into_iter()
            .map(|s| s.id)
            .collect();
        assert_eq!(
            ids,
            vec![oldest.id, tied_late.id, tied_early.id, newest.id, middle.id]
        );
    }

    #[test]
    fn list_summary_carries_title_and_updated_at() {
        let store = store();
        let note = store.create_note(at(1)).unwrap();
        store
            .save_note(&note.id, "Hello", "long body", at(9))
            .unwrap();
        let summaries = store.list_notes().unwrap();
        assert_eq!(
            summaries,
            vec![NoteSummary {
                id: note.id,
                title: "Hello".into(),
                updated_at: at(9)
            }]
        );
    }

    #[test]
    fn list_of_empty_store_is_empty() {
        assert_eq!(store().list_notes().unwrap().len(), 0);
    }

    #[test]
    fn open_creates_missing_directories_and_persists_across_reopen() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("nested/deeper/bourbon.db");
        let id = {
            let store = NoteStore::open(&path).unwrap();
            let note = store.create_note(at(1)).unwrap();
            store.save_note(&note.id, "Kept", "body", at(2)).unwrap();
            note.id
        };
        let reopened = NoteStore::open(&path).unwrap();
        let found = reopened.find_note(&id).unwrap().unwrap();
        assert_eq!(found.title, "Kept");
        assert_eq!(found.updated_at, at(2));
    }

    #[test]
    fn open_uses_write_ahead_logging() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("bourbon.db");
        drop(NoteStore::open(&path).unwrap());
        assert_eq!(journal_mode(&path), "wal");
    }

    fn journal_mode(path: &std::path::Path) -> String {
        rusqlite::Connection::open(path)
            .unwrap()
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn open_fails_when_parent_is_a_file() {
        let directory = tempfile::tempdir().unwrap();
        let blocker = directory.path().join("blocker");
        std::fs::write(&blocker, "x").unwrap();
        let error = NoteStore::open(&blocker.join("bourbon.db")).err().unwrap();
        assert!(matches!(error, StoreError::Io(_)));
    }
}
