use std::collections::HashMap;

use rusqlite::params_from_iter;

use crate::identifier::NoteId;
use crate::note::{NoteSummary, excerpt_of};
use crate::query::NoteQuery;
use crate::store::note_filter::{NoteFilter, order_clause};
use crate::store::tag_rows::{TAG_ORDER, tag_from_row};
use crate::store::{NoteStore, StoreError};
use crate::tag::Tag;

impl NoteStore {
    pub fn list_notes(&self, query: &NoteQuery) -> Result<Vec<NoteSummary>, StoreError> {
        let filter = NoteFilter::for_scope(&query.scope);
        let mut tags_by_note = self.tags_of_filtered_notes(&filter)?;
        let mut statement = self.connection.prepare(&format!(
            "{} SELECT n.id, n.title, n.body, n.updated_at, n.created_at, n.pinned, n.status
             FROM notes n WHERE {}
             ORDER BY n.pinned DESC, {}",
            filter.common_table,
            filter.condition,
            order_clause(query.sort)
        ))?;
        let summaries = statement
            .query_map(params_from_iter(&filter.parameters), |row| {
                let id: NoteId = row.get(0)?;
                let body: String = row.get(2)?;
                Ok(NoteSummary {
                    tags: tags_by_note.remove(&id).unwrap_or_default(),
                    id,
                    title: row.get(1)?,
                    excerpt: excerpt_of(&body),
                    updated_at: row.get(3)?,
                    created_at: row.get(4)?,
                    pinned: row.get(5)?,
                    status: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(summaries)
    }

    fn tags_of_filtered_notes(
        &self,
        filter: &NoteFilter,
    ) -> Result<HashMap<NoteId, Vec<Tag>>, StoreError> {
        let mut statement = self.connection.prepare(&format!(
            "{} SELECT nt.note_id, t.id, t.name, t.color
             FROM note_tags nt JOIN tags t ON t.id = nt.tag_id
             WHERE nt.note_id IN (SELECT n.id FROM notes n WHERE {})
             ORDER BY {TAG_ORDER}",
            filter.common_table, filter.condition
        ))?;
        let mut tags_by_note: HashMap<NoteId, Vec<Tag>> = HashMap::new();
        let mut rows = statement.query(params_from_iter(&filter.parameters))?;
        while let Some(row) = rows.next()? {
            tags_by_note
                .entry(row.get(0)?)
                .or_default()
                .push(tag_from_row(row, 1)?);
        }
        Ok(tags_by_note)
    }
}
