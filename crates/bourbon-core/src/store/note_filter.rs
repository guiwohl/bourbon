use std::collections::HashSet;

use rusqlite::types::Value;

use crate::identifier::TagId;
use crate::query::{NoteScope, NoteSort, NotebookDepth};
use crate::status::NoteStatus;

const NOT_TRASHED: &str = "n.deleted_at IS NULL";
const SUBTREE: &str = "WITH RECURSIVE subtree (id) AS (
    SELECT ? UNION ALL
    SELECT notebooks.id FROM notebooks JOIN subtree ON notebooks.parent_id = subtree.id
)";

pub struct NoteFilter {
    pub common_table: &'static str,
    pub condition: String,
    pub parameters: Vec<Value>,
}

impl NoteFilter {
    pub fn for_scope(scope: &NoteScope) -> Self {
        match scope {
            NoteScope::AllNotes => Self::visible("", Vec::new()),
            NoteScope::Notebook(notebook, NotebookDepth::DirectOnly) => {
                Self::visible(" AND n.notebook_id = ?", vec![text(notebook.as_str())])
            }
            NoteScope::Notebook(notebook, NotebookDepth::WithDescendants) => {
                let mut filter =
                    Self::visible(" AND n.notebook_id IN (SELECT id FROM subtree)", Vec::new());
                filter.common_table = SUBTREE;
                filter.parameters.insert(0, text(notebook.as_str()));
                filter
            }
            NoteScope::Status(status) => Self {
                common_table: "",
                condition: format!("{NOT_TRASHED} AND n.status = ?"),
                parameters: vec![text(status.as_stored())],
            },
            NoteScope::Tags(tags) => {
                let wanted: HashSet<&str> = tags.iter().map(TagId::as_str).collect();
                let mut filter = Self::visible(
                    &format!(
                        " AND (SELECT COUNT(*) FROM note_tags nt WHERE nt.note_id = n.id AND nt.tag_id IN ({})) = ?",
                        placeholders(wanted.len())
                    ),
                    wanted.iter().copied().map(text).collect(),
                );
                filter.parameters.push(Value::Integer(
                    i64::try_from(wanted.len()).unwrap_or(i64::MAX),
                ));
                filter
            }
            NoteScope::Trash => Self {
                common_table: "",
                condition: "n.deleted_at IS NOT NULL".to_owned(),
                parameters: Vec::new(),
            },
        }
    }

    fn visible(extra_condition: &str, extra_parameters: Vec<Value>) -> Self {
        let shown: Vec<&str> = NoteStatus::ALL
            .into_iter()
            .filter(|status| !status.is_hidden_by_default())
            .map(NoteStatus::as_stored)
            .collect();
        let mut parameters: Vec<Value> = shown.iter().map(|status| text(status)).collect();
        parameters.extend(extra_parameters);
        Self {
            common_table: "",
            condition: format!(
                "{NOT_TRASHED} AND n.status IN ({}){extra_condition}",
                placeholders(shown.len())
            ),
            parameters,
        }
    }
}

pub fn order_clause(sort: NoteSort) -> &'static str {
    match sort {
        NoteSort::UpdatedNewestFirst => "n.updated_at DESC, n.created_at DESC",
        NoteSort::CreatedNewestFirst => "n.created_at DESC, n.updated_at DESC",
        NoteSort::TitleAlphabetical => "n.title COLLATE NOCASE ASC, n.updated_at DESC",
    }
}

fn placeholders(count: usize) -> String {
    vec!["?"; count].join(", ")
}

fn text(value: &str) -> Value {
    Value::Text(value.to_owned())
}
