use std::collections::HashSet;

use rusqlite::types::Value;

use crate::identifier::TagId;
use crate::query::{NoteScope, NoteSort, NotebookDepth};
use crate::status::NoteStatus;
use crate::store::notebooks::NOTEBOOK_SUBTREE;

const NOT_TRASHED: &str = "n.deleted_at IS NULL AND n.is_template = 0";

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
                let (shown, mut parameters) = Self::visible_statuses();
                parameters.insert(0, text(notebook.as_str()));
                Self {
                    common_table: NOTEBOOK_SUBTREE,
                    condition: format!(
                        "{NOT_TRASHED} AND n.status IN ({shown}) AND n.notebook_id IN (SELECT id FROM subtree)"
                    ),
                    parameters,
                }
            }
            NoteScope::Status(status) => Self {
                common_table: "",
                condition: format!("{NOT_TRASHED} AND n.status = ?"),
                parameters: vec![text(status.as_stored())],
            },
            NoteScope::Tags(tags) => {
                let wanted: HashSet<&str> = tags.iter().map(TagId::as_str).collect();
                let (shown, mut parameters) = Self::visible_statuses();
                parameters.extend(wanted.iter().copied().map(text));
                parameters.push(Value::Integer(wanted.iter().map(|_| 1_i64).sum()));
                Self {
                    common_table: "",
                    condition: format!(
                        "{NOT_TRASHED} AND n.status IN ({shown}) AND (SELECT COUNT(*) FROM note_tags nt WHERE nt.note_id = n.id AND nt.tag_id IN ({})) = ?",
                        placeholders(wanted.len())
                    ),
                    parameters,
                }
            }
            NoteScope::Templates => Self {
                common_table: "",
                condition: "n.deleted_at IS NULL AND n.is_template = 1".to_owned(),
                parameters: Vec::new(),
            },
            NoteScope::Trash => Self {
                common_table: "",
                condition: "n.deleted_at IS NOT NULL".to_owned(),
                parameters: Vec::new(),
            },
        }
    }

    fn visible(extra_condition: &str, extra_parameters: Vec<Value>) -> Self {
        let (shown, mut parameters) = Self::visible_statuses();
        parameters.extend(extra_parameters);
        Self {
            common_table: "",
            condition: format!("{NOT_TRASHED} AND n.status IN ({shown}){extra_condition}"),
            parameters,
        }
    }

    fn visible_statuses() -> (String, Vec<Value>) {
        let parameters: Vec<Value> = NoteStatus::ALL
            .into_iter()
            .filter(|status| !status.is_hidden_by_default())
            .map(|status| text(status.as_stored()))
            .collect();
        (placeholders(parameters.len()), parameters)
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
