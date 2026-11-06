use std::time::Duration;

use rusqlite::params;

use crate::identifier::{NoteId, NotebookId};
use crate::store::{NoteStore, StoreError};
use crate::timestamp::Timestamp;

pub const TRASH_RETENTION: Duration = Duration::from_hours(168);

const SUBTREE_OF_NOTEBOOK: &str = "WITH RECURSIVE subtree(id) AS (
    SELECT ?1
    UNION ALL
    SELECT notebooks.id FROM notebooks JOIN subtree ON notebooks.parent_id = subtree.id
)";

impl NoteStore {
    pub fn trash_note(&self, note: &NoteId, now: Timestamp) -> Result<(), StoreError> {
        let changed = self.connection.execute(
            "UPDATE notes SET deleted_at = ?2 WHERE id = ?1 AND deleted_at IS NULL",
            params![note, now],
        )?;
        if changed == 0 {
            self.ensure_note_exists(note)?;
        }
        Ok(())
    }

    pub fn restore_note(&self, note: &NoteId) -> Result<(), StoreError> {
        let changed = self.connection.execute(
            "UPDATE notes SET deleted_at = NULL,
                 notebook_id = CASE
                     WHEN EXISTS (SELECT 1 FROM notebooks WHERE id = notes.notebook_id)
                     THEN notebook_id
                     ELSE (SELECT value FROM settings WHERE key = 'default_notebook_id')
                 END
             WHERE id = ?1 AND deleted_at IS NOT NULL",
            params![note],
        )?;
        if changed == 0 {
            self.ensure_note_exists(note)?;
        }
        Ok(())
    }

    pub fn empty_trash(&self) -> Result<usize, StoreError> {
        self.delete_trashed_notes_where("deleted_at IS NOT NULL", [])
    }

    pub fn purge_expired_trash(&self, now: Timestamp) -> Result<usize, StoreError> {
        let expiry_limit = now.as_millis() - retention_millis();
        self.delete_trashed_notes_where("deleted_at <= ?1", [expiry_limit])
    }

    pub fn delete_notebook(&self, notebook: &NotebookId, now: Timestamp) -> Result<(), StoreError> {
        if *notebook == self.default_notebook()? {
            return Err(StoreError::DefaultNotebookUndeletable);
        }
        self.ensure_notebook_exists(notebook)?;
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute(
            &format!(
                "{SUBTREE_OF_NOTEBOOK}
                 UPDATE notes SET deleted_at = ?2
                 WHERE deleted_at IS NULL AND notebook_id IN (SELECT id FROM subtree)"
            ),
            params![notebook, now],
        )?;
        transaction.execute(
            &format!(
                "{SUBTREE_OF_NOTEBOOK}
                 DELETE FROM notebooks WHERE id IN (SELECT id FROM subtree)"
            ),
            params![notebook],
        )?;
        transaction.commit()?;
        Ok(())
    }

    fn delete_trashed_notes_where<P: rusqlite::Params + Copy>(
        &self,
        condition: &str,
        parameters: P,
    ) -> Result<usize, StoreError> {
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute(
            &format!(
                "DELETE FROM note_tags WHERE note_id IN (SELECT id FROM notes WHERE {condition})"
            ),
            parameters,
        )?;
        let deleted =
            transaction.execute(&format!("DELETE FROM notes WHERE {condition}"), parameters)?;
        transaction.commit()?;
        Ok(deleted)
    }
}

fn retention_millis() -> i64 {
    i64::try_from(TRASH_RETENTION.as_millis()).unwrap_or(i64::MAX)
}
