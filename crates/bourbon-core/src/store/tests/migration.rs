use rusqlite::Connection;

use super::support::at;
use crate::{NoteStatus, NoteStore, StoreError};

fn build_version_one_database(path: &std::path::Path) {
    let connection = Connection::open(path).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE notes (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                body TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            INSERT INTO notes VALUES ('aaaaaaaaaaaa', 'First', 'first body', 10, 20);
            INSERT INTO notes VALUES ('bbbbbbbbbbbb', 'Second', 'second body', 30, 40);
            PRAGMA user_version = 1;",
        )
        .unwrap();
}

#[test]
fn version_one_notes_move_into_the_inbox_with_defaults() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("bourbon.db");
    build_version_one_database(&path);

    let store = NoteStore::open(&path).unwrap();
    let inbox = store.default_notebook().unwrap();
    let first = store
        .find_note(&crate::NoteId::parse("aaaaaaaaaaaa").unwrap())
        .unwrap()
        .unwrap();
    let second = store
        .find_note(&crate::NoteId::parse("bbbbbbbbbbbb").unwrap())
        .unwrap()
        .unwrap();

    assert_eq!(first.title, "First");
    assert_eq!(first.body, "first body");
    assert_eq!(first.created_at, at(10));
    assert_eq!(first.updated_at, at(20));
    assert_eq!(second.title, "Second");
    assert_eq!(second.body, "second body");
    assert_eq!(second.created_at, at(30));
    assert_eq!(second.updated_at, at(40));
    for note in [&first, &second] {
        assert_eq!(note.notebook, inbox);
        assert_eq!(note.status, NoteStatus::None);
        assert!(!note.pinned);
        assert_eq!(note.tags.len(), 0);
        assert_eq!(note.trashed_at, None);
    }
    let tree = store.notebook_tree().unwrap();
    assert_eq!(tree.len(), 1);
    assert_eq!(tree[0].notebook.name, "Inbox");
    assert_eq!(tree[0].note_count, 2);
}

#[test]
fn migrated_notes_are_listed() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("bourbon.db");
    build_version_one_database(&path);
    let store = NoteStore::open(&path).unwrap();
    let titles: Vec<_> = store
        .list_notes(&crate::NoteQuery::default())
        .unwrap()
        .into_iter()
        .map(|summary| summary.title)
        .collect();
    assert_eq!(titles, ["Second", "First"]);
}

#[test]
fn reopening_a_migrated_database_keeps_the_same_inbox() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("bourbon.db");
    build_version_one_database(&path);
    let first = NoteStore::open(&path).unwrap().default_notebook().unwrap();
    let second = NoteStore::open(&path).unwrap().default_notebook().unwrap();
    assert_eq!(first, second);
}

#[test]
fn open_rejects_schema_newer_than_supported() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("bourbon.db");
    drop(NoteStore::open(&path).unwrap());
    Connection::open(&path)
        .unwrap()
        .pragma_update(None, "user_version", 99)
        .unwrap();
    let error = NoteStore::open(&path).err().unwrap();
    assert!(matches!(
        error,
        StoreError::SchemaTooNew {
            found: 99,
            supported: 4
        }
    ));
}

fn build_version_two_database(path: &std::path::Path) {
    let connection = Connection::open(path).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE notebooks (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, parent_id TEXT, created_at INTEGER NOT NULL
            );
            CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO notebooks VALUES ('inboxinboxin', 'Inbox', NULL, 0);
            INSERT INTO settings VALUES ('default_notebook_id', 'inboxinboxin');
            CREATE TABLE notes (
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
            CREATE TABLE tags (
                id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE COLLATE NOCASE,
                color TEXT NOT NULL, created_at INTEGER NOT NULL
            );
            CREATE TABLE note_tags (
                note_id TEXT NOT NULL, tag_id TEXT NOT NULL, PRIMARY KEY (note_id, tag_id)
            );
            INSERT INTO notes (id, title, body, created_at, updated_at, notebook_id, status, pinned)
            VALUES ('cccccccccccc', 'Kept', 'kept body', 50, 60, 'inboxinboxin', 'active', 1);
            PRAGMA user_version = 2;",
        )
        .unwrap();
}

#[test]
fn version_two_notes_migrate_to_the_latest_version_as_ordinary_notes() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("bourbon.db");
    build_version_two_database(&path);

    let store = NoteStore::open(&path).unwrap();
    let note = store
        .find_note(&crate::NoteId::parse("cccccccccccc").unwrap())
        .unwrap()
        .unwrap();

    assert!(!note.is_template);
    assert_eq!(note.title, "Kept");
    assert_eq!(note.body, "kept body");
    assert_eq!(note.status, NoteStatus::Active);
    assert!(note.pinned);
    assert_eq!(note.updated_at, at(60));
    let version: usize = Connection::open(&path)
        .unwrap()
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 4);
    assert_eq!(store.search_notes("kept", 10).unwrap().len(), 1);
    let templates = store
        .list_notes(&crate::NoteQuery {
            scope: crate::NoteScope::Templates,
            sort: crate::NoteSort::default(),
        })
        .unwrap();
    assert_eq!(templates.len(), 0);
    assert_eq!(
        store
            .list_notes(&crate::NoteQuery::default())
            .unwrap()
            .len(),
        1
    );
}
