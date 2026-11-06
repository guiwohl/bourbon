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
    name TEXT NOT NULL UNIQUE,
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

const MIGRATIONS: &[&str] = &[NOTES_TABLE, ORGANIZATION];

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
