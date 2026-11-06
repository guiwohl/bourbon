mod listing;
mod note_filter;
mod note_state;
mod notebooks;
mod notes;
mod tag_rows;
mod tags;
mod templates;
#[cfg(test)]
mod tests;
mod trash;

use std::path::Path;

use rusqlite::Connection;

use crate::identifier::{NoteId, NotebookId, TagId};
use crate::migrations::migrate;

pub use notes::NoteSeed;
pub use tags::TagSummary;
pub use trash::TRASH_RETENTION;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("note not found: {0}")]
    NoteNotFound(NoteId),
    #[error("notebook not found: {0}")]
    NotebookNotFound(NotebookId),
    #[error("note is not a template: {0}")]
    NotATemplate(NoteId),
    #[error("notebook name must not be empty")]
    EmptyNotebookName,
    #[error("tag name must not be empty")]
    EmptyTagName,
    #[error("tag name already taken: {0}")]
    TagNameTaken(String),
    #[error("tag not found: {0}")]
    TagNotFound(TagId),
    #[error("the default notebook cannot be deleted")]
    DefaultNotebookUndeletable,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("database schema version {found} is newer than supported version {supported}")]
    SchemaTooNew { found: usize, supported: usize },
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
}
