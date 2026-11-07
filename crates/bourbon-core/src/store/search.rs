use rusqlite::{Row, params};

use crate::identifier::NoteId;
use crate::note::{NoteSummary, excerpt_of};
use crate::store::{NoteStore, StoreError};
use crate::task_progress::TaskProgress;

const MATCH_START: char = '\u{1}';
const MATCH_END: char = '\u{2}';
const SNIPPET_WORDS: usize = 12;
const TITLE_WEIGHT: f64 = 10.0;
const BODY_WEIGHT: f64 = 1.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    pub summary: NoteSummary,
    pub snippet: String,
}

struct HitRow {
    summary: NoteSummary,
    body: String,
    marked_snippet: String,
}

impl NoteStore {
    pub fn search_notes(&self, text: &str, limit: usize) -> Result<Vec<SearchHit>, StoreError> {
        let rows = match match_expression(text) {
            Some(expression) => self.matching_rows(&expression, limit)?,
            None => self.recent_rows(limit)?,
        };
        rows.into_iter()
            .map(|row| {
                let tags = self.tags_of(&row.summary.id)?;
                let snippet = snippet_of(&row.marked_snippet, &row.body);
                Ok(SearchHit {
                    summary: NoteSummary {
                        tags,
                        ..row.summary
                    },
                    snippet,
                })
            })
            .collect()
    }

    fn matching_rows(&self, expression: &str, limit: usize) -> Result<Vec<HitRow>, StoreError> {
        let mut statement = self.connection.prepare(&format!(
            "SELECT n.id, n.title, n.body, n.updated_at, n.created_at, n.pinned, n.status,
                    snippet(notes_fts, 1, '{MATCH_START}', '{MATCH_END}', '', {SNIPPET_WORDS})
             FROM notes_fts JOIN notes n ON n.rowid = notes_fts.rowid
             WHERE notes_fts MATCH ?1 AND n.deleted_at IS NULL AND n.is_template = 0
             ORDER BY bm25(notes_fts, {TITLE_WEIGHT:?}, {BODY_WEIGHT:?}), n.updated_at DESC
             LIMIT ?2"
        ))?;
        let rows = statement
            .query_map(params![expression, limit_value(limit)], hit_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn recent_rows(&self, limit: usize) -> Result<Vec<HitRow>, StoreError> {
        let mut statement = self.connection.prepare(
            "SELECT n.id, n.title, n.body, n.updated_at, n.created_at, n.pinned, n.status, ''
             FROM notes n
             WHERE n.deleted_at IS NULL AND n.is_template = 0
             ORDER BY n.updated_at DESC, n.created_at DESC
             LIMIT ?1",
        )?;
        let rows = statement
            .query_map(params![limit_value(limit)], hit_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

fn hit_row(row: &Row<'_>) -> rusqlite::Result<HitRow> {
    let id: NoteId = row.get(0)?;
    let body: String = row.get(2)?;
    Ok(HitRow {
        summary: NoteSummary {
            id,
            title: row.get(1)?,
            excerpt: excerpt_of(&body),
            updated_at: row.get(3)?,
            created_at: row.get(4)?,
            pinned: row.get(5)?,
            status: row.get(6)?,
            tags: Vec::new(),
            task_progress: TaskProgress::of_body(&body),
        },
        body,
        marked_snippet: row.get(7)?,
    })
}

fn limit_value(limit: usize) -> i64 {
    i64::try_from(limit).unwrap_or(i64::MAX)
}

fn match_expression(text: &str) -> Option<String> {
    let tokens: Vec<String> = text
        .split_whitespace()
        .map(|token| format!("\"{}\"*", token.replace('"', "\"\"")))
        .collect();
    (!tokens.is_empty()).then(|| tokens.join(" AND "))
}

fn snippet_of(marked_snippet: &str, body: &str) -> String {
    if !marked_snippet.contains(MATCH_START) {
        return excerpt_of(body);
    }
    marked_snippet
        .chars()
        .filter(|character| *character != MATCH_START && *character != MATCH_END)
        .map(|character| {
            if character.is_whitespace() {
                ' '
            } else {
                character
            }
        })
        .collect::<String>()
        .trim()
        .to_owned()
}
