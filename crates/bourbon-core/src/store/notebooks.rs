use std::collections::HashMap;

use rusqlite::params;

use crate::identifier::NotebookId;
use crate::notebook::{Notebook, NotebookNode, build_tree};
use crate::store::{NoteStore, StoreError};
use crate::timestamp::Timestamp;

pub(super) const NOTEBOOK_SUBTREE: &str = "WITH RECURSIVE subtree (id) AS (
    SELECT ?1 UNION ALL
    SELECT notebooks.id FROM notebooks JOIN subtree ON notebooks.parent_id = subtree.id
)";

impl NoteStore {
    pub fn default_notebook(&self) -> Result<NotebookId, StoreError> {
        let id = self.connection.query_row(
            "SELECT value FROM settings WHERE key = 'default_notebook_id'",
            [],
            |row| row.get(0),
        )?;
        Ok(id)
    }

    pub fn create_notebook(
        &self,
        name: &str,
        parent: Option<&NotebookId>,
        now: Timestamp,
    ) -> Result<Notebook, StoreError> {
        let name = non_empty_notebook_name(name)?;
        if let Some(parent) = parent {
            self.ensure_notebook_exists(parent)?;
        }
        let notebook = Notebook {
            id: NotebookId::generate(),
            name: name.to_owned(),
            parent: parent.cloned(),
            created_at: now,
        };
        self.connection.execute(
            "INSERT INTO notebooks (id, name, parent_id, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                notebook.id,
                notebook.name,
                notebook.parent,
                notebook.created_at
            ],
        )?;
        Ok(notebook)
    }

    pub fn rename_notebook(&self, id: &NotebookId, name: &str) -> Result<(), StoreError> {
        let name = non_empty_notebook_name(name)?;
        let changed = self.connection.execute(
            "UPDATE notebooks SET name = ?2 WHERE id = ?1",
            params![id, name],
        )?;
        if changed == 0 {
            return Err(StoreError::NotebookNotFound(id.clone()));
        }
        Ok(())
    }

    pub fn notebook_tree(&self) -> Result<Vec<NotebookNode>, StoreError> {
        Ok(build_tree(
            self.all_notebooks()?,
            &self.direct_note_counts()?,
        ))
    }

    pub(super) fn ensure_notebook_exists(&self, id: &NotebookId) -> Result<(), StoreError> {
        let exists: bool = self.connection.query_row(
            "SELECT EXISTS (SELECT 1 FROM notebooks WHERE id = ?1)",
            params![id],
            |row| row.get(0),
        )?;
        if exists {
            Ok(())
        } else {
            Err(StoreError::NotebookNotFound(id.clone()))
        }
    }

    fn all_notebooks(&self) -> Result<Vec<Notebook>, StoreError> {
        let mut statement = self
            .connection
            .prepare("SELECT id, name, parent_id, created_at FROM notebooks")?;
        let notebooks = statement
            .query_map([], |row| {
                Ok(Notebook {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    parent: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(notebooks)
    }

    fn direct_note_counts(&self) -> Result<HashMap<NotebookId, usize>, StoreError> {
        let mut statement = self.connection.prepare(
            "SELECT notebook_id, COUNT(*) FROM notes WHERE deleted_at IS NULL GROUP BY notebook_id",
        )?;
        let counts = statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<HashMap<_, _>, _>>()?;
        Ok(counts)
    }
}

fn non_empty_notebook_name(name: &str) -> Result<&str, StoreError> {
    let name = name.trim();
    if name.is_empty() {
        Err(StoreError::EmptyNotebookName)
    } else {
        Ok(name)
    }
}
