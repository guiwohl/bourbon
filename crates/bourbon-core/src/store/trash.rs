use std::time::Duration;

use rusqlite::{params, params_from_iter};

use crate::identifier::{NoteId, NotebookId};
use crate::store::notebooks::NOTEBOOK_SUBTREE;
use crate::store::{NoteStore, StoreError};
use crate::timestamp::Timestamp;

pub const TRASH_RETENTION: Duration = Duration::from_hours(168);

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
        self.purge_trash(&TrashPurge::Everything)
    }

    pub fn purge_expired_trash(&self, now: Timestamp) -> Result<usize, StoreError> {
        let expiry_limit = Timestamp::from_millis(now.as_millis() - retention_millis());
        self.purge_trash(&TrashPurge::ExpiredBefore(expiry_limit))
    }

    pub fn delete_notebook(&self, notebook: &NotebookId, now: Timestamp) -> Result<(), StoreError> {
        if *notebook == self.default_notebook()? {
            return Err(StoreError::DefaultNotebookUndeletable);
        }
        self.ensure_notebook_exists(notebook)?;
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute(
            &format!(
                "{NOTEBOOK_SUBTREE}
                 UPDATE notes SET deleted_at = ?2
                 WHERE deleted_at IS NULL AND notebook_id IN (SELECT id FROM subtree)"
            ),
            params![notebook, now],
        )?;
        transaction.execute(
            &format!(
                "{NOTEBOOK_SUBTREE}
                 DELETE FROM notebooks WHERE id IN (SELECT id FROM subtree)"
            ),
            params![notebook],
        )?;
        transaction.commit()?;
        Ok(())
    }

    fn purge_trash(&self, purge: &TrashPurge) -> Result<usize, StoreError> {
        let condition = purge.condition();
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute(
            &format!(
                "DELETE FROM note_tags WHERE note_id IN (SELECT id FROM notes WHERE {condition})"
            ),
            params_from_iter(purge.parameters()),
        )?;
        let deleted = transaction.execute(
            &format!("DELETE FROM notes WHERE {condition}"),
            params_from_iter(purge.parameters()),
        )?;
        transaction.commit()?;
        Ok(deleted)
    }
}

enum TrashPurge {
    Everything,
    ExpiredBefore(Timestamp),
}

impl TrashPurge {
    fn condition(&self) -> &'static str {
        match self {
            Self::Everything => "deleted_at IS NOT NULL",
            Self::ExpiredBefore(_) => "deleted_at <= ?1",
        }
    }

    fn parameters(&self) -> Vec<Timestamp> {
        match self {
            Self::Everything => Vec::new(),
            Self::ExpiredBefore(limit) => vec![*limit],
        }
    }
}

fn retention_millis() -> i64 {
    i64::try_from(TRASH_RETENTION.as_millis()).unwrap_or(i64::MAX)
}
