use rusqlite::params;

use crate::identifier::{NoteId, NotebookId};
use crate::note::Note;
use crate::store::{NoteStore, StoreError};
use crate::template_variables::{TemplateTimeZone, expand};
use crate::timestamp::Timestamp;

#[derive(Clone, Copy)]
enum TemplateFlag {
    Off = 0,
    On = 1,
}

impl NoteStore {
    pub fn create_template(
        &self,
        notebook: &NotebookId,
        now: Timestamp,
    ) -> Result<Note, StoreError> {
        self.insert_empty_note(notebook, now, true)
    }

    pub fn mark_as_template(&self, note: &NoteId) -> Result<(), StoreError> {
        self.write_template_flag(note, TemplateFlag::On)
    }

    pub fn unmark_template(&self, note: &NoteId) -> Result<(), StoreError> {
        self.write_template_flag(note, TemplateFlag::Off)
    }

    pub fn create_note_from_template(
        &self,
        template: &NoteId,
        notebook: &NotebookId,
        now: Timestamp,
        zone: &TemplateTimeZone,
    ) -> Result<Note, StoreError> {
        let source = self
            .find_note(template)?
            .ok_or_else(|| StoreError::NoteNotFound(template.clone()))?;
        if !source.is_template {
            return Err(StoreError::NotATemplate(template.clone()));
        }
        self.ensure_notebook_exists(notebook)?;
        let note_id = NoteId::generate();
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute(
            "INSERT INTO notes (id, title, body, created_at, updated_at, notebook_id)
             VALUES (?1, ?2, ?3, ?4, ?4, ?5)",
            params![
                note_id,
                expand(&source.title, now, zone),
                expand(&source.body, now, zone),
                now,
                notebook
            ],
        )?;
        transaction.execute(
            "INSERT INTO note_tags (note_id, tag_id)
             SELECT ?2, tag_id FROM note_tags WHERE note_id = ?1",
            params![template, note_id],
        )?;
        transaction.commit()?;
        self.find_note(&note_id)?
            .ok_or(StoreError::NoteNotFound(note_id))
    }

    fn write_template_flag(&self, note: &NoteId, flag: TemplateFlag) -> Result<(), StoreError> {
        let matched = self.connection.execute(
            "UPDATE notes SET is_template = ?2 WHERE id = ?1",
            params![note, flag as i64],
        )?;
        if matched == 0 {
            return Err(StoreError::NoteNotFound(note.clone()));
        }
        Ok(())
    }
}
