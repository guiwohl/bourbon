use super::support::{add_tag, at, attach_tag, inbox, pin, set_status, store, titled_note, trash};
use crate::{NoteId, NoteStatus, NoteStore, StoreError, TagColor};

fn assert_not_found(result: Result<(), StoreError>, missing: &NoteId) {
    assert!(matches!(result, Err(StoreError::NoteNotFound(id)) if id == *missing));
}

fn stored(store: &NoteStore, note: &NoteId) -> crate::Note {
    store.find_note(note).unwrap().unwrap()
}

#[test]
fn changing_status_stores_it_and_touches_updated_at() {
    let store = store();
    let note = titled_note(&store, &inbox(&store), "a", 1, 2);
    store
        .set_status(&note, NoteStatus::Completed, at(50))
        .unwrap();
    let found = stored(&store, &note);
    assert_eq!(found.status, NoteStatus::Completed);
    assert_eq!(found.updated_at, at(50));
}

#[test]
fn setting_the_same_status_keeps_updated_at() {
    let store = store();
    let note = titled_note(&store, &inbox(&store), "a", 1, 2);
    set_status(&store, &note, NoteStatus::Dropped);
    store
        .set_status(&note, NoteStatus::Dropped, at(50))
        .unwrap();
    assert_eq!(stored(&store, &note).updated_at, at(2));
}

#[test]
fn setting_status_back_to_none_works() {
    let store = store();
    let note = titled_note(&store, &inbox(&store), "a", 1, 2);
    set_status(&store, &note, NoteStatus::Dropped);
    store.set_status(&note, NoteStatus::None, at(9)).unwrap();
    assert_eq!(stored(&store, &note).status, NoteStatus::None);
}

#[test]
fn setting_status_of_an_unknown_note_fails() {
    let store = store();
    let missing = NoteId::generate();
    assert_not_found(
        store.set_status(&missing, NoteStatus::Dropped, at(1)),
        &missing,
    );
}

#[test]
fn pinning_pins_without_touching_updated_at() {
    let store = store();
    let note = titled_note(&store, &inbox(&store), "a", 1, 2);
    store.pin_note(&note).unwrap();
    let found = stored(&store, &note);
    assert!(found.pinned);
    assert_eq!(found.updated_at, at(2));
}

#[test]
fn pinning_twice_is_harmless() {
    let store = store();
    let note = titled_note(&store, &inbox(&store), "a", 1, 2);
    store.pin_note(&note).unwrap();
    store.pin_note(&note).unwrap();
    assert!(stored(&store, &note).pinned);
}

#[test]
fn unpinning_unpins_without_touching_updated_at() {
    let store = store();
    let note = titled_note(&store, &inbox(&store), "a", 1, 2);
    pin(&store, &note);
    store.unpin_note(&note).unwrap();
    let found = stored(&store, &note);
    assert!(!found.pinned);
    assert_eq!(found.updated_at, at(2));
}

#[test]
fn pinning_or_unpinning_an_unknown_note_fails() {
    let store = store();
    let missing = NoteId::generate();
    assert_not_found(store.pin_note(&missing), &missing);
    assert_not_found(store.unpin_note(&missing), &missing);
}

#[test]
fn duplicate_copies_content_status_notebook_and_tags_into_a_fresh_note() {
    let store = store();
    let notebook = store.create_notebook("Work", None, at(1)).unwrap().id;
    let source = titled_note(&store, &notebook, "Plan", 1, 2);
    store
        .save_note(&source, "Plan", "body text", at(3))
        .unwrap();
    set_status(&store, &source, NoteStatus::Completed);
    pin(&store, &source);
    let red = add_tag(&store, "red", TagColor::Red);
    let blue = add_tag(&store, "blue", TagColor::Blue);
    attach_tag(&store, &source, &red);
    attach_tag(&store, &source, &blue);

    let copy = store.duplicate_note(&source, at(100)).unwrap();

    assert_ne!(copy.id, source);
    assert_eq!(copy.title, "Plan");
    assert_eq!(copy.body, "body text");
    assert_eq!(copy.notebook, notebook);
    assert_eq!(copy.status, NoteStatus::Completed);
    assert!(!copy.pinned);
    assert_eq!(copy.created_at, at(100));
    assert_eq!(copy.updated_at, at(100));
    assert_eq!(copy.trashed_at, None);
    assert_eq!(copy.tags, stored(&store, &source).tags);
    assert_eq!(copy.tags.len(), 2);
    assert_eq!(store.find_note(&copy.id).unwrap(), Some(copy));
}

#[test]
fn duplicate_leaves_the_source_untouched() {
    let store = store();
    let source = titled_note(&store, &inbox(&store), "Plan", 1, 2);
    pin(&store, &source);
    let tag = add_tag(&store, "red", TagColor::Red);
    attach_tag(&store, &source, &tag);
    let before = stored(&store, &source);
    store.duplicate_note(&source, at(100)).unwrap();
    assert_eq!(stored(&store, &source), before);
}

#[test]
fn duplicating_a_trashed_note_gives_a_live_copy() {
    let store = store();
    let source = titled_note(&store, &inbox(&store), "Gone", 1, 2);
    trash(&store, &source, 5);
    let copy = store.duplicate_note(&source, at(100)).unwrap();
    assert_eq!(copy.trashed_at, None);
    assert_eq!(stored(&store, &copy.id).trashed_at, None);
    assert_eq!(stored(&store, &source).trashed_at, Some(at(5)));
}

#[test]
fn duplicating_an_unknown_note_fails() {
    let store = store();
    let missing = NoteId::generate();
    let error = store.duplicate_note(&missing, at(1)).unwrap_err();
    assert!(matches!(error, StoreError::NoteNotFound(id) if id == missing));
}
