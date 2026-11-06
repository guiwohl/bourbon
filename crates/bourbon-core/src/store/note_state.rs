use rusqlite::params;

use crate::identifier::NoteId;
use crate::note::Note;
use crate::status::NoteStatus;
use crate::store::{NoteStore, StoreError};
use crate::timestamp::Timestamp;

impl NoteStore {
    pub fn set_status(
        &self,
        note: &NoteId,
        status: NoteStatus,
        now: Timestamp,
    ) -> Result<(), StoreError> {
        let changed = self.connection.execute(
            "UPDATE notes SET status = ?2, updated_at = ?3 WHERE id = ?1 AND status <> ?2",
            params![note, status, now],
        )?;
        if changed == 0 {
            self.require_note(note)?;
        }
        Ok(())
    }

    pub fn pin_note(&self, note: &NoteId) -> Result<(), StoreError> {
        self.write_pinned(note, true)
    }

    pub fn unpin_note(&self, note: &NoteId) -> Result<(), StoreError> {
        self.write_pinned(note, false)
    }

    pub fn duplicate_note(&self, note: &NoteId, now: Timestamp) -> Result<Note, StoreError> {
        let copy_id = NoteId::generate();
        let transaction = self.connection.unchecked_transaction()?;
        let copied = transaction.execute(
            "INSERT INTO notes (id, title, body, created_at, updated_at, notebook_id, status)
             SELECT ?2, title, body, ?3, ?3, notebook_id, status FROM notes WHERE id = ?1",
            params![note, copy_id, now],
        )?;
        if copied == 0 {
            return Err(StoreError::NoteNotFound(note.clone()));
        }
        transaction.execute(
            "INSERT INTO note_tags (note_id, tag_id)
             SELECT ?2, tag_id FROM note_tags WHERE note_id = ?1",
            params![note, copy_id],
        )?;
        transaction.commit()?;
        self.find_note(&copy_id)?
            .ok_or(StoreError::NoteNotFound(copy_id))
    }

    fn write_pinned(&self, note: &NoteId, pinned: bool) -> Result<(), StoreError> {
        let matched = self.connection.execute(
            "UPDATE notes SET pinned = ?2 WHERE id = ?1",
            params![note, pinned],
        )?;
        if matched == 0 {
            return Err(StoreError::NoteNotFound(note.clone()));
        }
        Ok(())
    }

    fn require_note(&self, note: &NoteId) -> Result<(), StoreError> {
        let exists: bool = self.connection.query_row(
            "SELECT EXISTS (SELECT 1 FROM notes WHERE id = ?1)",
            params![note],
            |row| row.get(0),
        )?;
        if exists {
            Ok(())
        } else {
            Err(StoreError::NoteNotFound(note.clone()))
        }
    }
}
