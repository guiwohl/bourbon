use super::support::{at, inbox, store, titled_note, trash};
use crate::{NoteStore, NotebookId, StoreError};

#[test]
fn a_fresh_store_has_an_inbox_as_its_only_notebook() {
    let store = store();
    let tree = store.notebook_tree().unwrap();
    assert_eq!(tree.len(), 1);
    assert_eq!(tree[0].notebook.name, "Inbox");
    assert_eq!(tree[0].notebook.id, inbox(&store));
    assert_eq!(tree[0].notebook.parent, None);
    assert_eq!(tree[0].note_count, 0);
    assert_eq!(tree[0].children.len(), 0);
}

#[test]
fn the_inbox_is_created_with_the_current_time() {
    let before = crate::Timestamp::now().as_millis();
    let store = store();
    let created = store.notebook_tree().unwrap()[0].notebook.created_at;
    assert!(created.as_millis() >= before - 1000);
    assert!(created.as_millis() <= crate::Timestamp::now().as_millis() + 1000);
}

#[test]
fn the_default_notebook_survives_reopening() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("bourbon.db");
    let first = inbox(&NoteStore::open(&path).unwrap());
    let second = inbox(&NoteStore::open(&path).unwrap());
    assert_eq!(first, second);
}

#[test]
fn created_notebook_is_returned_and_listed() {
    let store = store();
    let notebook = store.create_notebook("Work", None, at(5)).unwrap();
    assert_eq!(notebook.name, "Work");
    assert_eq!(notebook.parent, None);
    assert_eq!(notebook.created_at, at(5));
    let names: Vec<_> = store
        .notebook_tree()
        .unwrap()
        .into_iter()
        .map(|node| node.notebook)
        .collect();
    assert!(names.contains(&notebook));
}

#[test]
fn nested_notebooks_form_a_tree_ordered_by_name() {
    let store = store();
    let work = store.create_notebook("work", None, at(1)).unwrap();
    store
        .create_notebook("Zeta", Some(&work.id), at(2))
        .unwrap();
    let alpha = store
        .create_notebook("alpha", Some(&work.id), at(3))
        .unwrap();
    store
        .create_notebook("Deep", Some(&alpha.id), at(4))
        .unwrap();
    let tree = store.notebook_tree().unwrap();
    let roots: Vec<_> = tree.iter().map(|n| n.notebook.name.as_str()).collect();
    assert_eq!(roots, ["Inbox", "work"]);
    let children: Vec<_> = tree[1]
        .children
        .iter()
        .map(|n| n.notebook.name.as_str())
        .collect();
    assert_eq!(children, ["alpha", "Zeta"]);
    assert_eq!(tree[1].children[0].children[0].notebook.name, "Deep");
    assert_eq!(
        tree[1].children[0].children[0].notebook.parent,
        Some(alpha.id)
    );
}

#[test]
fn notebook_names_need_not_be_unique() {
    let store = store();
    let first = store.create_notebook("Same", None, at(1)).unwrap();
    let second = store.create_notebook("Same", None, at(2)).unwrap();
    assert_ne!(first.id, second.id);
}

#[test]
fn creating_under_an_unknown_parent_fails() {
    let store = store();
    let missing = NotebookId::generate();
    let error = store
        .create_notebook("Orphan", Some(&missing), at(1))
        .unwrap_err();
    assert!(matches!(error, StoreError::NotebookNotFound(id) if id == missing));
    assert_eq!(store.notebook_tree().unwrap().len(), 1);
}

#[test]
fn renaming_changes_only_the_name() {
    let store = store();
    let parent = store.create_notebook("Parent", None, at(1)).unwrap();
    let child = store
        .create_notebook("Old", Some(&parent.id), at(2))
        .unwrap();
    store.rename_notebook(&child.id, "New").unwrap();
    let tree = store.notebook_tree().unwrap();
    let renamed = &tree
        .iter()
        .find(|node| node.notebook.id == parent.id)
        .unwrap()
        .children[0]
        .notebook;
    assert_eq!(renamed.name, "New");
    assert_eq!(renamed.parent, Some(parent.id));
    assert_eq!(renamed.created_at, at(2));
}

#[test]
fn renaming_an_unknown_notebook_fails() {
    let missing = NotebookId::generate();
    let error = store().rename_notebook(&missing, "x").unwrap_err();
    assert!(matches!(error, StoreError::NotebookNotFound(id) if id == missing));
}

#[test]
fn note_counts_cover_descendants_and_ignore_the_trash() {
    let store = store();
    let root = store.create_notebook("Root", None, at(1)).unwrap();
    let child = store
        .create_notebook("Child", Some(&root.id), at(2))
        .unwrap();
    titled_note(&store, &root.id, "a", 1, 1);
    titled_note(&store, &child.id, "b", 1, 1);
    titled_note(&store, &child.id, "c", 1, 1);
    let gone = titled_note(&store, &child.id, "d", 1, 1);
    trash(&store, &gone, 9);
    let tree = store.notebook_tree().unwrap();
    let root_node = tree.iter().find(|n| n.notebook.id == root.id).unwrap();
    assert_eq!(root_node.note_count, 3);
    assert_eq!(root_node.children[0].note_count, 2);
}

#[test]
fn note_counts_include_hidden_statuses() {
    let store = store();
    let note = titled_note(&store, &inbox(&store), "done", 1, 1);
    super::support::set_status(&store, &note, crate::NoteStatus::Completed);
    assert_eq!(store.notebook_tree().unwrap()[0].note_count, 1);
}

#[test]
fn notebooks_persist_across_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("bourbon.db");
    let id = {
        let store = NoteStore::open(&path).unwrap();
        store.create_notebook("Kept", None, at(3)).unwrap().id
    };
    let tree = NoteStore::open(&path).unwrap().notebook_tree().unwrap();
    assert!(tree.iter().any(|node| node.notebook.id == id));
}
