use super::support::{add_tag, at, inbox, store};
use crate::{NoteId, NoteSeed, NoteStatus, NoteStore, NotebookId, StoreError, TagId};

#[test]
fn created_note_is_empty_stamped_and_in_the_given_notebook() {
    let store = store();
    let notebook = inbox(&store);
    let note = store.create_note(&notebook, at(10)).unwrap();
    assert_eq!(note.title, "");
    assert_eq!(note.body, "");
    assert_eq!(note.created_at, at(10));
    assert_eq!(note.updated_at, at(10));
    assert_eq!(note.notebook, notebook);
    assert_eq!(note.status, NoteStatus::None);
    assert!(!note.pinned);
    assert_eq!(note.tags.len(), 0);
    assert_eq!(note.trashed_at, None);
}

#[test]
fn created_note_can_be_found() {
    let store = store();
    let note = store.create_note(&inbox(&store), at(10)).unwrap();
    assert_eq!(store.find_note(&note.id).unwrap(), Some(note));
}

#[test]
fn creating_a_note_in_an_unknown_notebook_fails() {
    let store = store();
    let missing = NotebookId::generate();
    let error = store.create_note(&missing, at(1)).unwrap_err();
    assert!(matches!(error, StoreError::NotebookNotFound(id) if id == missing));
}

#[test]
fn find_unknown_note_is_none() {
    assert_eq!(store().find_note(&NoteId::generate()).unwrap(), None);
}

#[test]
fn save_stores_title_body_and_bumps_updated_at() {
    let store = store();
    let note = store.create_note(&inbox(&store), at(10)).unwrap();
    store.save_note(&note.id, "Title", "Body", at(20)).unwrap();
    let found = store.find_note(&note.id).unwrap().unwrap();
    assert_eq!(found.title, "Title");
    assert_eq!(found.body, "Body");
    assert_eq!(found.created_at, at(10));
    assert_eq!(found.updated_at, at(20));
}

#[test]
fn saving_identical_content_keeps_updated_at() {
    let store = store();
    let note = store.create_note(&inbox(&store), at(10)).unwrap();
    store.save_note(&note.id, "Title", "Body", at(20)).unwrap();
    store.save_note(&note.id, "Title", "Body", at(30)).unwrap();
    let found = store.find_note(&note.id).unwrap().unwrap();
    assert_eq!(found.updated_at, at(20));
}

#[test]
fn saving_empty_content_on_new_note_is_a_no_op() {
    let store = store();
    let note = store.create_note(&inbox(&store), at(10)).unwrap();
    store.save_note(&note.id, "", "", at(30)).unwrap();
    assert_eq!(
        store.find_note(&note.id).unwrap().unwrap().updated_at,
        at(10)
    );
}

#[test]
fn changing_only_title_updates() {
    let store = store();
    let note = store.create_note(&inbox(&store), at(10)).unwrap();
    store.save_note(&note.id, "A", "Body", at(20)).unwrap();
    store.save_note(&note.id, "B", "Body", at(30)).unwrap();
    assert_eq!(
        store.find_note(&note.id).unwrap().unwrap().updated_at,
        at(30)
    );
}

#[test]
fn changing_only_body_updates() {
    let store = store();
    let note = store.create_note(&inbox(&store), at(10)).unwrap();
    store.save_note(&note.id, "A", "x", at(20)).unwrap();
    store.save_note(&note.id, "A", "y", at(30)).unwrap();
    assert_eq!(
        store.find_note(&note.id).unwrap().unwrap().updated_at,
        at(30)
    );
}

#[test]
fn saving_unknown_note_fails() {
    let id = NoteId::generate();
    let error = store().save_note(&id, "t", "b", at(1)).unwrap_err();
    assert!(matches!(error, StoreError::NoteNotFound(missing) if missing == id));
}

#[test]
fn moving_a_note_changes_its_notebook_and_bumps_updated_at() {
    let store = store();
    let target = store.create_notebook("Target", None, at(1)).unwrap();
    let note = store.create_note(&inbox(&store), at(10)).unwrap();
    store.move_note(&note.id, &target.id, at(50)).unwrap();
    let found = store.find_note(&note.id).unwrap().unwrap();
    assert_eq!(found.notebook, target.id);
    assert_eq!(found.updated_at, at(50));
    assert_eq!(found.created_at, at(10));
}

#[test]
fn moving_an_unknown_note_fails() {
    let store = store();
    let missing = NoteId::generate();
    let error = store
        .move_note(&missing, &inbox(&store), at(1))
        .unwrap_err();
    assert!(matches!(error, StoreError::NoteNotFound(id) if id == missing));
}

#[test]
fn moving_into_an_unknown_notebook_fails_and_changes_nothing() {
    let store = store();
    let note = store.create_note(&inbox(&store), at(10)).unwrap();
    let missing = NotebookId::generate();
    let error = store.move_note(&note.id, &missing, at(50)).unwrap_err();
    assert!(matches!(error, StoreError::NotebookNotFound(id) if id == missing));
    assert_eq!(store.find_note(&note.id).unwrap(), Some(note));
}

#[test]
fn open_creates_missing_directories_and_persists_across_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("nested/deeper/bourbon.db");
    let id = {
        let store = NoteStore::open(&path).unwrap();
        let note = store.create_note(&inbox(&store), at(1)).unwrap();
        store.save_note(&note.id, "Kept", "body", at(2)).unwrap();
        note.id
    };
    let reopened = NoteStore::open(&path).unwrap();
    let found = reopened.find_note(&id).unwrap().unwrap();
    assert_eq!(found.title, "Kept");
    assert_eq!(found.updated_at, at(2));
}

#[test]
fn open_uses_write_ahead_logging() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("bourbon.db");
    drop(NoteStore::open(&path).unwrap());
    let mode: String = rusqlite::Connection::open(&path)
        .unwrap()
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .unwrap();
    assert_eq!(mode, "wal");
}

#[test]
fn open_fails_when_parent_is_a_file() {
    let directory = tempfile::tempdir().unwrap();
    let blocker = directory.path().join("blocker");
    std::fs::write(&blocker, "x").unwrap();
    let error = NoteStore::open(&blocker.join("bourbon.db")).err().unwrap();
    assert!(matches!(error, StoreError::Io(_)));
}

#[test]
fn seeded_note_gets_the_tags_and_status() {
    let store = store();
    let first = add_tag(&store, "a", crate::TagColor::Red);
    let second = add_tag(&store, "b", crate::TagColor::Blue);
    let seed = NoteSeed {
        tags: vec![first.clone(), second.clone()],
        status: NoteStatus::Completed,
    };
    let note = store
        .create_note_with(&inbox(&store), &seed, at(10))
        .unwrap();
    assert_eq!(note.status, NoteStatus::Completed);
    let mut ids: Vec<_> = note.tags.iter().map(|tag| tag.id.clone()).collect();
    ids.sort_by_key(ToString::to_string);
    let mut expected = vec![first, second];
    expected.sort_by_key(ToString::to_string);
    assert_eq!(ids, expected);
    assert_eq!(store.find_note(&note.id).unwrap(), Some(note));
}

#[test]
fn default_seed_makes_a_plain_note() {
    let store = store();
    let note = store
        .create_note_with(&inbox(&store), &NoteSeed::default(), at(10))
        .unwrap();
    assert_eq!(note.status, NoteStatus::None);
    assert_eq!(note.tags.len(), 0);
}

#[test]
fn seeding_with_an_unknown_tag_writes_nothing() {
    let store = store();
    let missing = TagId::generate();
    let seed = NoteSeed {
        tags: vec![missing.clone()],
        status: NoteStatus::None,
    };
    let error = store
        .create_note_with(&inbox(&store), &seed, at(10))
        .unwrap_err();
    assert!(matches!(error, StoreError::TagNotFound(id) if id == missing));
    let listed = store.list_notes(&crate::NoteQuery::default()).unwrap();
    assert_eq!(listed.len(), 0);
}

#[test]
fn seeding_an_unknown_notebook_fails() {
    let store = store();
    let missing = NotebookId::generate();
    let error = store
        .create_note_with(&missing, &NoteSeed::default(), at(1))
        .unwrap_err();
    assert!(matches!(error, StoreError::NotebookNotFound(id) if id == missing));
}
