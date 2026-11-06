use rusqlite::params;

use crate::identifier::{NoteId, TagId};
use crate::store::tag_rows::{TAG_ORDER, tag_from_row};
use crate::store::{NoteStore, StoreError};
use crate::tag::{Tag, TagColor};
use crate::timestamp::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagSummary {
    pub tag: Tag,
    pub note_count: usize,
}

impl NoteStore {
    pub fn create_tag(
        &self,
        name: &str,
        color: TagColor,
        now: Timestamp,
    ) -> Result<Tag, StoreError> {
        let name = name.trim();
        self.ensure_tag_name_available(name, None)?;
        let tag = Tag {
            id: TagId::generate(),
            name: name.to_owned(),
            color,
        };
        self.connection
            .execute(
                "INSERT INTO tags (id, name, color, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![tag.id, tag.name, tag.color, now],
            )
            .map_err(|error| tag_name_conflict(error, &tag.name))?;
        Ok(tag)
    }

    pub fn rename_tag(&self, id: &TagId, name: &str) -> Result<(), StoreError> {
        let name = name.trim();
        self.ensure_tag_exists(id)?;
        self.ensure_tag_name_available(name, Some(id))?;
        self.connection
            .execute("UPDATE tags SET name = ?2 WHERE id = ?1", params![id, name])
            .map_err(|error| tag_name_conflict(error, name))?;
        Ok(())
    }

    pub fn set_tag_color(&self, id: &TagId, color: TagColor) -> Result<(), StoreError> {
        let changed = self.connection.execute(
            "UPDATE tags SET color = ?2 WHERE id = ?1",
            params![id, color],
        )?;
        if changed == 0 {
            return Err(StoreError::TagNotFound(id.clone()));
        }
        Ok(())
    }

    pub fn delete_tag(&self, id: &TagId) -> Result<(), StoreError> {
        self.ensure_tag_exists(id)?;
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute("DELETE FROM note_tags WHERE tag_id = ?1", params![id])?;
        transaction.execute("DELETE FROM tags WHERE id = ?1", params![id])?;
        transaction.commit()?;
        Ok(())
    }

    pub fn list_tags(&self) -> Result<Vec<TagSummary>, StoreError> {
        let mut statement = self.connection.prepare(&format!(
            "SELECT t.id, t.name, t.color,
                (SELECT COUNT(*) FROM note_tags nt JOIN notes n ON n.id = nt.note_id
                 WHERE nt.tag_id = t.id AND n.deleted_at IS NULL AND n.is_template = 0)
             FROM tags t
             ORDER BY {TAG_ORDER}"
        ))?;
        let summaries = statement
            .query_map([], |row| {
                Ok(TagSummary {
                    tag: tag_from_row(row, 0)?,
                    note_count: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(summaries)
    }

    pub fn add_tag_to_note(
        &self,
        note: &NoteId,
        tag: &TagId,
        now: Timestamp,
    ) -> Result<(), StoreError> {
        self.ensure_taggable(note, tag)?;
        let transaction = self.connection.unchecked_transaction()?;
        let added = transaction.execute(
            "INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
            params![note, tag],
        )?;
        if added > 0 {
            stamp_note(&transaction, note, now)?;
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn remove_tag_from_note(
        &self,
        note: &NoteId,
        tag: &TagId,
        now: Timestamp,
    ) -> Result<(), StoreError> {
        self.ensure_taggable(note, tag)?;
        let transaction = self.connection.unchecked_transaction()?;
        let removed = transaction.execute(
            "DELETE FROM note_tags WHERE note_id = ?1 AND tag_id = ?2",
            params![note, tag],
        )?;
        if removed > 0 {
            stamp_note(&transaction, note, now)?;
        }
        transaction.commit()?;
        Ok(())
    }

    fn ensure_taggable(&self, note: &NoteId, tag: &TagId) -> Result<(), StoreError> {
        self.ensure_note_exists(note)?;
        self.ensure_tag_exists(tag)
    }

    fn ensure_tag_exists(&self, id: &TagId) -> Result<(), StoreError> {
        let exists: bool = self.connection.query_row(
            "SELECT EXISTS (SELECT 1 FROM tags WHERE id = ?1)",
            params![id],
            |row| row.get(0),
        )?;
        if exists {
            Ok(())
        } else {
            Err(StoreError::TagNotFound(id.clone()))
        }
    }

    fn ensure_tag_name_available(
        &self,
        name: &str,
        renamed: Option<&TagId>,
    ) -> Result<(), StoreError> {
        if name.is_empty() {
            return Err(StoreError::EmptyTagName);
        }
        let taken: bool = self.connection.query_row(
            "SELECT EXISTS (SELECT 1 FROM tags WHERE name = ?1 COLLATE NOCASE AND id IS NOT ?2)",
            params![name, renamed],
            |row| row.get(0),
        )?;
        if taken {
            Err(StoreError::TagNameTaken(name.to_owned()))
        } else {
            Ok(())
        }
    }
}

pub(super) fn tag_name_conflict(error: rusqlite::Error, name: &str) -> StoreError {
    match &error {
        rusqlite::Error::SqliteFailure(failure, _)
            if failure.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE =>
        {
            StoreError::TagNameTaken(name.to_owned())
        }
        _ => StoreError::Database(error),
    }
}

fn stamp_note(
    transaction: &rusqlite::Transaction<'_>,
    note: &NoteId,
    now: Timestamp,
) -> Result<(), StoreError> {
    transaction.execute(
        "UPDATE notes SET updated_at = ?2 WHERE id = ?1",
        params![note, now],
    )?;
    Ok(())
}
