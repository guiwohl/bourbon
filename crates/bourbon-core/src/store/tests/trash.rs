use super::support::{add_tag, at, attach_tag, inbox, store, titled_note, trash};
use crate::{
    NoteId, NoteQuery, NoteScope, NoteStore, NotebookDepth, NotebookId, StoreError, TRASH_RETENTION,
};

const DAY: i64 = 86_400_000;

fn retention_millis() -> i64 {
    i64::try_from(TRASH_RETENTION.as_millis()).unwrap()
}

fn trashed_ids(store: &NoteStore) -> Vec<NoteId> {
    ids(store, NoteScope::Trash)
}

fn ids(store: &NoteStore, scope: NoteScope) -> Vec<NoteId> {
    let query = NoteQuery {
        scope,
        ..NoteQuery::default()
    };
    store
        .list_notes(&query)
        .unwrap()
        .into_iter()
        .map(|summary| summary.id)
        .collect()
}

fn tag_rows(store: &NoteStore) -> usize {
    store
        .connection
        .query_row("SELECT COUNT(*) FROM note_tags", [], |row| row.get(0))
        .unwrap()
}

fn notebook_exists(store: &NoteStore, id: &NotebookId) -> bool {
    store.ensure_notebook_exists(id).is_ok()
}

#[test]
fn retention_is_seven_days() {
    assert_eq!(retention_millis(), 7 * DAY);
}

#[test]
fn trashing_a_note_moves_it_to_the_trash_without_touching_updated_at() {
    let store = store();
    let note = titled_note(&store, &inbox(&store), "a", 1, 10);
    store.trash_note(&note, at(500)).unwrap();
    let stored = store.find_note(&note).unwrap().unwrap();
    assert_eq!(stored.trashed_at, Some(at(500)));
    assert_eq!(stored.updated_at, at(10));
    assert_eq!(trashed_ids(&store), vec![note]);
    assert_eq!(ids(&store, NoteScope::AllNotes), vec![]);
}

#[test]
fn trashing_an_already_trashed_note_keeps_the_original_time() {
    let store = store();
    let note = titled_note(&store, &inbox(&store), "a", 1, 1);
    store.trash_note(&note, at(500)).unwrap();
    store.trash_note(&note, at(900)).unwrap();
    let stored = store.find_note(&note).unwrap().unwrap();
    assert_eq!(stored.trashed_at, Some(at(500)));
}

#[test]
fn trashing_an_unknown_note_fails() {
    let store = store();
    let unknown = NoteId::generate();
    let error = store.trash_note(&unknown, at(1)).unwrap_err();
    assert!(matches!(error, StoreError::NoteNotFound(id) if id == unknown));
}

#[test]
fn restoring_returns_the_note_to_its_notebook() {
    let store = store();
    let work = store.create_notebook("Work", None, at(1)).unwrap();
    let note = titled_note(&store, &work.id, "a", 1, 10);
    store.trash_note(&note, at(500)).unwrap();
    store.restore_note(&note).unwrap();
    let stored = store.find_note(&note).unwrap().unwrap();
    assert_eq!(stored.trashed_at, None);
    assert_eq!(stored.notebook, work.id);
    assert_eq!(stored.updated_at, at(10));
    assert_eq!(trashed_ids(&store), vec![]);
}

#[test]
fn restoring_a_note_whose_notebook_was_deleted_moves_it_to_the_default_notebook() {
    let store = store();
    let work = store.create_notebook("Work", None, at(1)).unwrap();
    let note = titled_note(&store, &work.id, "a", 1, 1);
    store.delete_notebook(&work.id, at(5)).unwrap();
    store.restore_note(&note).unwrap();
    let stored = store.find_note(&note).unwrap().unwrap();
    assert_eq!(stored.trashed_at, None);
    assert_eq!(stored.notebook, inbox(&store));
}

#[test]
fn restoring_a_note_that_is_not_trashed_changes_nothing() {
    let store = store();
    let work = store.create_notebook("Work", None, at(1)).unwrap();
    let note = titled_note(&store, &work.id, "a", 1, 1);
    let before = store.find_note(&note).unwrap();
    store.restore_note(&note).unwrap();
    assert_eq!(store.find_note(&note).unwrap(), before);
}

#[test]
fn restoring_an_unknown_note_fails() {
    let store = store();
    let unknown = NoteId::generate();
    let error = store.restore_note(&unknown).unwrap_err();
    assert!(matches!(error, StoreError::NoteNotFound(id) if id == unknown));
}

#[test]
fn emptying_the_trash_deletes_trashed_notes_and_their_tag_links_only() {
    let store = store();
    let notebook = inbox(&store);
    let tag = add_tag(&store, "t", crate::TagColor::Red);
    let doomed = titled_note(&store, &notebook, "doomed", 1, 1);
    let kept = titled_note(&store, &notebook, "kept", 2, 2);
    attach_tag(&store, &doomed, &tag);
    attach_tag(&store, &kept, &tag);
    trash(&store, &doomed, 5);
    assert_eq!(store.empty_trash().unwrap(), 1);
    assert_eq!(store.find_note(&doomed).unwrap(), None);
    assert!(store.find_note(&kept).unwrap().is_some());
    assert_eq!(tag_rows(&store), 1);
}

#[test]
fn emptying_an_empty_trash_deletes_nothing() {
    let store = store();
    titled_note(&store, &inbox(&store), "a", 1, 1);
    assert_eq!(store.empty_trash().unwrap(), 0);
    assert_eq!(ids(&store, NoteScope::AllNotes).len(), 1);
}

#[test]
fn purging_removes_a_note_trashed_exactly_the_retention_ago() {
    let store = store();
    let tag = add_tag(&store, "t", crate::TagColor::Red);
    let note = titled_note(&store, &inbox(&store), "a", 1, 1);
    attach_tag(&store, &note, &tag);
    trash(&store, &note, 1000);
    let purged = store
        .purge_expired_trash(at(1000 + retention_millis()))
        .unwrap();
    assert_eq!(purged, 1);
    assert_eq!(store.find_note(&note).unwrap(), None);
    assert_eq!(tag_rows(&store), 0);
}

#[test]
fn purging_keeps_a_note_trashed_one_millisecond_less_than_the_retention_ago() {
    let store = store();
    let tag = add_tag(&store, "t", crate::TagColor::Red);
    let note = titled_note(&store, &inbox(&store), "a", 1, 1);
    attach_tag(&store, &note, &tag);
    trash(&store, &note, 1000);
    let purged = store
        .purge_expired_trash(at(1000 + retention_millis() - 1))
        .unwrap();
    assert_eq!(purged, 0);
    assert!(store.find_note(&note).unwrap().is_some());
    assert_eq!(tag_rows(&store), 1);
}

#[test]
fn purging_never_touches_notes_that_are_not_trashed() {
    let store = store();
    let note = titled_note(&store, &inbox(&store), "a", 1, 1);
    let purged = store.purge_expired_trash(at(100 * DAY)).unwrap();
    assert_eq!(purged, 0);
    assert!(store.find_note(&note).unwrap().is_some());
}

#[test]
fn purging_counts_only_the_expired_notes() {
    let store = store();
    let notebook = inbox(&store);
    let old = titled_note(&store, &notebook, "old", 1, 1);
    let older = titled_note(&store, &notebook, "older", 2, 2);
    let recent = titled_note(&store, &notebook, "recent", 3, 3);
    trash(&store, &old, 0);
    trash(&store, &older, 10);
    trash(&store, &recent, 8 * DAY);
    assert_eq!(store.purge_expired_trash(at(8 * DAY)).unwrap(), 2);
    assert_eq!(trashed_ids(&store), vec![recent]);
}

#[test]
fn deleting_a_notebook_removes_its_whole_subtree_and_trashes_every_note_in_it() {
    let store = store();
    let parent = store.create_notebook("parent", None, at(1)).unwrap();
    let child = store
        .create_notebook("child", Some(&parent.id), at(2))
        .unwrap();
    let grandchild = store
        .create_notebook("grandchild", Some(&child.id), at(3))
        .unwrap();
    let sibling = store.create_notebook("sibling", None, at(4)).unwrap();
    let in_parent = titled_note(&store, &parent.id, "p", 1, 1);
    let in_grandchild = titled_note(&store, &grandchild.id, "g", 2, 2);
    let in_sibling = titled_note(&store, &sibling.id, "s", 3, 3);
    store.delete_notebook(&parent.id, at(900)).unwrap();
    assert!(!notebook_exists(&store, &parent.id));
    assert!(!notebook_exists(&store, &child.id));
    assert!(!notebook_exists(&store, &grandchild.id));
    assert!(notebook_exists(&store, &sibling.id));
    for note in [&in_parent, &in_grandchild] {
        let stored = store.find_note(note).unwrap().unwrap();
        assert_eq!(stored.trashed_at, Some(at(900)));
    }
    assert_eq!(
        store.find_note(&in_sibling).unwrap().unwrap().trashed_at,
        None
    );
    assert_eq!(
        ids(
            &store,
            NoteScope::Notebook(sibling.id, NotebookDepth::DirectOnly)
        ),
        vec![in_sibling]
    );
}

#[test]
fn deleting_a_notebook_keeps_the_time_of_notes_already_in_the_trash() {
    let store = store();
    let work = store.create_notebook("Work", None, at(1)).unwrap();
    let earlier = titled_note(&store, &work.id, "earlier", 1, 1);
    let live = titled_note(&store, &work.id, "live", 2, 2);
    trash(&store, &earlier, 100);
    store.delete_notebook(&work.id, at(900)).unwrap();
    assert_eq!(
        store.find_note(&earlier).unwrap().unwrap().trashed_at,
        Some(at(100))
    );
    assert_eq!(
        store.find_note(&live).unwrap().unwrap().trashed_at,
        Some(at(900))
    );
}

#[test]
fn deleting_a_notebook_leaves_updated_at_alone() {
    let store = store();
    let work = store.create_notebook("Work", None, at(1)).unwrap();
    let note = titled_note(&store, &work.id, "a", 1, 10);
    store.delete_notebook(&work.id, at(900)).unwrap();
    assert_eq!(store.find_note(&note).unwrap().unwrap().updated_at, at(10));
}

#[test]
fn the_default_notebook_cannot_be_deleted() {
    let store = store();
    let default = inbox(&store);
    let note = titled_note(&store, &default, "a", 1, 1);
    let error = store.delete_notebook(&default, at(5)).unwrap_err();
    assert!(matches!(error, StoreError::DefaultNotebookUndeletable));
    assert!(notebook_exists(&store, &default));
    assert_eq!(store.find_note(&note).unwrap().unwrap().trashed_at, None);
}

#[test]
fn deleting_an_unknown_notebook_fails() {
    let store = store();
    let unknown = NotebookId::generate();
    let error = store.delete_notebook(&unknown, at(1)).unwrap_err();
    assert!(matches!(error, StoreError::NotebookNotFound(id) if id == unknown));
}
