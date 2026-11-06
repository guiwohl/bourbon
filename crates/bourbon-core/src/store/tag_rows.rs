use rusqlite::{Row, params};

use crate::identifier::NoteId;
use crate::store::{NoteStore, StoreError};
use crate::tag::Tag;

pub const TAG_ORDER: &str = "t.name COLLATE NOCASE, t.id";

pub fn tag_from_row(row: &Row<'_>, first_column: usize) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(first_column)?,
        name: row.get(first_column + 1)?,
        color: row.get(first_column + 2)?,
    })
}

impl NoteStore {
    pub(super) fn tags_of(&self, note: &NoteId) -> Result<Vec<Tag>, StoreError> {
        let mut statement = self.connection.prepare(&format!(
            "SELECT t.id, t.name, t.color
             FROM note_tags nt JOIN tags t ON t.id = nt.tag_id
             WHERE nt.note_id = ?1
             ORDER BY {TAG_ORDER}"
        ))?;
        let tags = statement
            .query_map(params![note], |row| tag_from_row(row, 0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }
}
