use rusqlite::{OptionalExtension, params};

use crate::identifier::{NoteId, NotebookId};
use crate::note::Note;
use crate::status::NoteStatus;
use crate::store::{NoteStore, StoreError};
use crate::timestamp::Timestamp;

impl NoteStore {
    pub fn create_note(&self, notebook: &NotebookId, now: Timestamp) -> Result<Note, StoreError> {
        self.ensure_notebook_exists(notebook)?;
        let note = Note {
            id: NoteId::generate(),
            title: String::new(),
            body: String::new(),
            created_at: now,
            updated_at: now,
            notebook: notebook.clone(),
            status: NoteStatus::None,
            pinned: false,
            tags: Vec::new(),
            trashed_at: None,
        };
        self.connection.execute(
            "INSERT INTO notes (id, title, body, created_at, updated_at, notebook_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                note.id,
                note.title,
                note.body,
                note.created_at,
                note.updated_at,
                note.notebook
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
            params![id, title, body, now],
        )?;
        if changed == 0 {
            self.ensure_note_exists(id)?;
        }
        Ok(())
    }

    pub fn move_note(
        &self,
        note: &NoteId,
        notebook: &NotebookId,
        now: Timestamp,
    ) -> Result<(), StoreError> {
        self.ensure_note_exists(note)?;
        self.ensure_notebook_exists(notebook)?;
        self.connection.execute(
            "UPDATE notes SET notebook_id = ?2, updated_at = ?3 WHERE id = ?1",
            params![note, notebook, now],
        )?;
        Ok(())
    }

    pub fn find_note(&self, id: &NoteId) -> Result<Option<Note>, StoreError> {
        let stored = self
            .connection
            .query_row(
                "SELECT title, body, created_at, updated_at, notebook_id, status, pinned, deleted_at
                 FROM notes WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Note {
                        id: id.clone(),
                        title: row.get(0)?,
                        body: row.get(1)?,
                        created_at: row.get(2)?,
                        updated_at: row.get(3)?,
                        notebook: row.get(4)?,
                        status: row.get(5)?,
                        pinned: row.get(6)?,
                        tags: Vec::new(),
                        trashed_at: row.get(7)?,
                    })
                },
            )
            .optional()?;
        stored
            .map(|note| {
                let tags = self.tags_of(&note.id)?;
                Ok(Note { tags, ..note })
            })
            .transpose()
    }

    fn ensure_note_exists(&self, id: &NoteId) -> Result<(), StoreError> {
        let exists: bool = self.connection.query_row(
            "SELECT EXISTS (SELECT 1 FROM notes WHERE id = ?1)",
            params![id],
            |row| row.get(0),
        )?;
        if exists {
            Ok(())
        } else {
            Err(StoreError::NoteNotFound(id.clone()))
        }
    }
}
