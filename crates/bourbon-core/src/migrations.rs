use rusqlite::Connection;

use crate::store::StoreError;

const NOTES_TABLE: &str = "CREATE TABLE notes (
        id TEXT PRIMARY KEY,
        title TEXT NOT NULL,
        body TEXT NOT NULL,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
    )";

const ORGANIZATION: &str = "
CREATE TABLE notebooks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    parent_id TEXT,
    created_at INTEGER NOT NULL
);
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
INSERT INTO notebooks (id, name, parent_id, created_at)
VALUES (lower(hex(randomblob(6))), 'Inbox', NULL, CAST(unixepoch('subsec') * 1000 AS INTEGER));
INSERT INTO settings (key, value)
SELECT 'default_notebook_id', id FROM notebooks WHERE name = 'Inbox';
CREATE TABLE notes_organized (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    notebook_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'none',
    pinned INTEGER NOT NULL DEFAULT 0,
    deleted_at INTEGER
);
INSERT INTO notes_organized (id, title, body, created_at, updated_at, notebook_id)
SELECT id, title, body, created_at, updated_at,
       (SELECT value FROM settings WHERE key = 'default_notebook_id')
FROM notes;
DROP TABLE notes;
ALTER TABLE notes_organized RENAME TO notes;
CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,
    color TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE TABLE note_tags (
    note_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (note_id, tag_id)
);
CREATE INDEX notes_by_notebook ON notes (notebook_id);
CREATE INDEX notes_by_status ON notes (status);
CREATE INDEX notes_by_deleted_at ON notes (deleted_at);
CREATE INDEX note_tags_by_tag ON note_tags (tag_id);
";

const TEMPLATES: &str = "ALTER TABLE notes ADD COLUMN is_template INTEGER NOT NULL DEFAULT 0";

const FULL_TEXT_SEARCH: &str = "
CREATE VIRTUAL TABLE notes_fts USING fts5(title, body, content='notes');
INSERT INTO notes_fts (notes_fts) VALUES ('rebuild');
CREATE TRIGGER notes_fts_insert AFTER INSERT ON notes BEGIN
    INSERT INTO notes_fts (rowid, title, body) VALUES (new.rowid, new.title, new.body);
END;
CREATE TRIGGER notes_fts_delete AFTER DELETE ON notes BEGIN
    INSERT INTO notes_fts (notes_fts, rowid, title, body)
    VALUES ('delete', old.rowid, old.title, old.body);
END;
CREATE TRIGGER notes_fts_update AFTER UPDATE OF title, body ON notes BEGIN
    INSERT INTO notes_fts (notes_fts, rowid, title, body)
    VALUES ('delete', old.rowid, old.title, old.body);
    INSERT INTO notes_fts (rowid, title, body) VALUES (new.rowid, new.title, new.body);
END;
";

const MIGRATIONS: &[&str] = &[NOTES_TABLE, ORGANIZATION, TEMPLATES, FULL_TEXT_SEARCH];

pub fn migrate(connection: &mut Connection) -> Result<(), StoreError> {
    let applied: usize = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if applied > MIGRATIONS.len() {
        return Err(StoreError::SchemaTooNew {
            found: applied,
            supported: MIGRATIONS.len(),
        });
    }
    for (index, migration) in MIGRATIONS.iter().enumerate().skip(applied) {
        let transaction = connection.transaction()?;
        transaction.execute_batch(migration)?;
        transaction.pragma_update(None, "user_version", index + 1)?;
        transaction.commit()?;
    }
    Ok(())
}
