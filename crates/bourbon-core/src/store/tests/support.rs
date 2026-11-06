use rusqlite::params;

use crate::{NoteId, NoteStatus, NoteStore, NotebookId, TagColor, TagId, Timestamp};

pub fn at(millis: i64) -> Timestamp {
    Timestamp::from_millis(millis)
}

pub fn store() -> NoteStore {
    NoteStore::open_in_memory().unwrap()
}

pub fn inbox(store: &NoteStore) -> NotebookId {
    store.default_notebook().unwrap()
}

pub fn titled_note(
    store: &NoteStore,
    notebook: &NotebookId,
    title: &str,
    created: i64,
    updated: i64,
) -> NoteId {
    let note = store.create_note(notebook, at(created)).unwrap();
    store.save_note(&note.id, title, "", at(updated)).unwrap();
    note.id
}

pub fn pin(store: &NoteStore, note: &NoteId) {
    store
        .connection
        .execute("UPDATE notes SET pinned = 1 WHERE id = ?1", params![note])
        .unwrap();
}

pub fn set_status(store: &NoteStore, note: &NoteId, status: NoteStatus) {
    store
        .connection
        .execute(
            "UPDATE notes SET status = ?2 WHERE id = ?1",
            params![note, status],
        )
        .unwrap();
}

pub fn trash(store: &NoteStore, note: &NoteId, when: i64) {
    store
        .connection
        .execute(
            "UPDATE notes SET deleted_at = ?2 WHERE id = ?1",
            params![note, at(when)],
        )
        .unwrap();
}

pub fn add_tag(store: &NoteStore, name: &str, color: TagColor) -> TagId {
    let id = TagId::generate();
    store
        .connection
        .execute(
            "INSERT INTO tags (id, name, color, created_at) VALUES (?1, ?2, ?3, 0)",
            params![id, name, color],
        )
        .unwrap();
    id
}

pub fn attach_tag(store: &NoteStore, note: &NoteId, tag: &TagId) {
    store
        .connection
        .execute(
            "INSERT INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
            params![note, tag],
        )
        .unwrap();
}
