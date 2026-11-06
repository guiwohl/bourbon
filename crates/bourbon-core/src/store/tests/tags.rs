use super::support::{add_tag, at, attach_tag, inbox, set_status, store, trash};
use crate::{NoteId, NoteStatus, StoreError, TagColor, TagId};

#[test]
fn created_tag_is_returned_and_listed_with_no_notes() {
    let store = store();
    let tag = store.create_tag("rust", TagColor::Red, at(1)).unwrap();
    assert_eq!(tag.name, "rust");
    assert_eq!(tag.color, TagColor::Red);
    let listed = store.list_tags().unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].tag, tag);
    assert_eq!(listed[0].note_count, 0);
}

#[test]
fn created_tag_name_is_trimmed() {
    let tag = store()
        .create_tag("  rust \n", TagColor::Default, at(1))
        .unwrap();
    assert_eq!(tag.name, "rust");
}

#[test]
fn blank_tag_name_is_rejected() {
    let store = store();
    let error = store
        .create_tag("  ", TagColor::Default, at(1))
        .unwrap_err();
    assert!(matches!(error, StoreError::EmptyTagName));
    assert_eq!(store.list_tags().unwrap().len(), 0);
}

#[test]
fn duplicate_tag_name_ignoring_case_is_rejected() {
    let store = store();
    store.create_tag("Rust", TagColor::Default, at(1)).unwrap();
    let error = store
        .create_tag(" rUST ", TagColor::Default, at(2))
        .unwrap_err();
    assert!(matches!(error, StoreError::TagNameTaken(name) if name == "rUST"));
    assert_eq!(store.list_tags().unwrap().len(), 1);
}

#[test]
fn tags_are_listed_by_name_ignoring_case() {
    let store = store();
    for name in ["beta", "Gamma", "alpha"] {
        store.create_tag(name, TagColor::Default, at(1)).unwrap();
    }
    let names: Vec<_> = store
        .list_tags()
        .unwrap()
        .into_iter()
        .map(|summary| summary.tag.name)
        .collect();
    assert_eq!(names, ["alpha", "beta", "Gamma"]);
}

#[test]
fn renamed_tag_keeps_its_id_and_color() {
    let store = store();
    let tag = store.create_tag("old", TagColor::Blue, at(1)).unwrap();
    store.rename_tag(&tag.id, " new ").unwrap();
    let listed = store.list_tags().unwrap();
    assert_eq!(listed[0].tag.id, tag.id);
    assert_eq!(listed[0].tag.name, "new");
    assert_eq!(listed[0].tag.color, TagColor::Blue);
}

#[test]
fn a_tag_can_be_renamed_to_its_own_name_in_another_case() {
    let store = store();
    let tag = store.create_tag("rust", TagColor::Default, at(1)).unwrap();
    store.rename_tag(&tag.id, "Rust").unwrap();
    assert_eq!(store.list_tags().unwrap()[0].tag.name, "Rust");
}

#[test]
fn renaming_to_another_tags_name_is_rejected() {
    let store = store();
    store.create_tag("a", TagColor::Default, at(1)).unwrap();
    let b = store.create_tag("b", TagColor::Default, at(1)).unwrap();
    let error = store.rename_tag(&b.id, "A").unwrap_err();
    assert!(matches!(error, StoreError::TagNameTaken(name) if name == "A"));
    assert_eq!(store.list_tags().unwrap()[1].tag.name, "b");
}

#[test]
fn renaming_to_a_blank_name_is_rejected() {
    let store = store();
    let tag = store.create_tag("a", TagColor::Default, at(1)).unwrap();
    let error = store.rename_tag(&tag.id, " ").unwrap_err();
    assert!(matches!(error, StoreError::EmptyTagName));
    assert_eq!(store.list_tags().unwrap()[0].tag.name, "a");
}

#[test]
fn renaming_an_unknown_tag_fails() {
    let missing = TagId::generate();
    let error = store().rename_tag(&missing, "x").unwrap_err();
    assert!(matches!(error, StoreError::TagNotFound(id) if id == missing));
}

#[test]
fn tag_color_can_be_changed() {
    let store = store();
    let tag = store.create_tag("a", TagColor::Default, at(1)).unwrap();
    let other = store.create_tag("b", TagColor::Default, at(1)).unwrap();
    store.set_tag_color(&tag.id, TagColor::Pink).unwrap();
    let listed = store.list_tags().unwrap();
    assert_eq!(listed[0].tag.color, TagColor::Pink);
    assert_eq!(listed[1].tag.id, other.id);
    assert_eq!(listed[1].tag.color, TagColor::Default);
}

#[test]
fn changing_the_color_of_an_unknown_tag_fails() {
    let missing = TagId::generate();
    let error = store().set_tag_color(&missing, TagColor::Red).unwrap_err();
    assert!(matches!(error, StoreError::TagNotFound(id) if id == missing));
}

#[test]
fn deleting_a_tag_removes_it_from_its_notes_only() {
    let store = store();
    let note = store.create_note(&inbox(&store), at(1)).unwrap().id;
    let doomed = store
        .create_tag("doomed", TagColor::Default, at(1))
        .unwrap();
    let kept = store.create_tag("kept", TagColor::Default, at(1)).unwrap();
    attach_tag(&store, &note, &doomed.id);
    attach_tag(&store, &note, &kept.id);
    store.delete_tag(&doomed.id).unwrap();
    assert_eq!(
        store.find_note(&note).unwrap().unwrap().tags,
        std::slice::from_ref(&kept)
    );
    let listed = store.list_tags().unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].tag, kept);
    assert_eq!(listed[0].note_count, 1);
}

#[test]
fn deleting_an_unknown_tag_fails() {
    let missing = TagId::generate();
    let error = store().delete_tag(&missing).unwrap_err();
    assert!(matches!(error, StoreError::TagNotFound(id) if id == missing));
}

#[test]
fn note_count_counts_untrashed_notes_of_any_status() {
    let store = store();
    let notebook = inbox(&store);
    let tag = add_tag(&store, "t", TagColor::Default);
    let other = add_tag(&store, "u", TagColor::Default);
    let plain = store.create_note(&notebook, at(1)).unwrap().id;
    let completed = store.create_note(&notebook, at(1)).unwrap().id;
    let trashed = store.create_note(&notebook, at(1)).unwrap().id;
    set_status(&store, &completed, NoteStatus::Completed);
    trash(&store, &trashed, 5);
    for note in [&plain, &completed, &trashed] {
        attach_tag(&store, note, &tag);
    }
    attach_tag(&store, &plain, &other);
    let counts: Vec<_> = store
        .list_tags()
        .unwrap()
        .into_iter()
        .map(|summary| summary.note_count)
        .collect();
    assert_eq!(counts, [2, 1]);
}

#[test]
fn adding_a_tag_attaches_it_and_stamps_the_note() {
    let store = store();
    let note = store.create_note(&inbox(&store), at(1)).unwrap().id;
    let tag = store.create_tag("t", TagColor::Teal, at(1)).unwrap();
    store.add_tag_to_note(&note, &tag.id, at(9)).unwrap();
    let found = store.find_note(&note).unwrap().unwrap();
    assert_eq!(found.tags, [tag]);
    assert_eq!(found.updated_at, at(9));
}

#[test]
fn adding_a_present_tag_changes_nothing() {
    let store = store();
    let note = store.create_note(&inbox(&store), at(1)).unwrap().id;
    let tag = store.create_tag("t", TagColor::Default, at(1)).unwrap();
    store.add_tag_to_note(&note, &tag.id, at(5)).unwrap();
    store.add_tag_to_note(&note, &tag.id, at(9)).unwrap();
    let found = store.find_note(&note).unwrap().unwrap();
    assert_eq!(found.tags.len(), 1);
    assert_eq!(found.updated_at, at(5));
}

#[test]
fn adding_to_an_unknown_note_or_with_an_unknown_tag_fails() {
    let store = store();
    let note = store.create_note(&inbox(&store), at(1)).unwrap().id;
    let tag = store.create_tag("t", TagColor::Default, at(1)).unwrap();
    let missing_note = NoteId::generate();
    let missing_tag = TagId::generate();
    let error = store
        .add_tag_to_note(&missing_note, &tag.id, at(2))
        .unwrap_err();
    assert!(matches!(error, StoreError::NoteNotFound(id) if id == missing_note));
    let error = store
        .add_tag_to_note(&note, &missing_tag, at(2))
        .unwrap_err();
    assert!(matches!(error, StoreError::TagNotFound(id) if id == missing_tag));
    assert_eq!(store.list_tags().unwrap()[0].note_count, 0);
    assert_eq!(store.find_note(&note).unwrap().unwrap().updated_at, at(1));
}

#[test]
fn removing_a_tag_detaches_it_and_stamps_the_note() {
    let store = store();
    let note = store.create_note(&inbox(&store), at(1)).unwrap().id;
    let gone = store.create_tag("gone", TagColor::Default, at(1)).unwrap();
    let kept = store.create_tag("kept", TagColor::Default, at(1)).unwrap();
    attach_tag(&store, &note, &gone.id);
    attach_tag(&store, &note, &kept.id);
    store.remove_tag_from_note(&note, &gone.id, at(9)).unwrap();
    let found = store.find_note(&note).unwrap().unwrap();
    assert_eq!(found.tags, [kept]);
    assert_eq!(found.updated_at, at(9));
}

#[test]
fn removing_an_absent_tag_changes_nothing() {
    let store = store();
    let note = store.create_note(&inbox(&store), at(1)).unwrap().id;
    let tag = store.create_tag("t", TagColor::Default, at(1)).unwrap();
    store.remove_tag_from_note(&note, &tag.id, at(9)).unwrap();
    assert_eq!(store.find_note(&note).unwrap().unwrap().updated_at, at(1));
}

#[test]
fn removing_with_unknown_note_or_tag_fails() {
    let store = store();
    let note = store.create_note(&inbox(&store), at(1)).unwrap().id;
    let tag = store.create_tag("t", TagColor::Default, at(1)).unwrap();
    let missing_note = NoteId::generate();
    let missing_tag = TagId::generate();
    let error = store
        .remove_tag_from_note(&missing_note, &tag.id, at(2))
        .unwrap_err();
    assert!(matches!(error, StoreError::NoteNotFound(id) if id == missing_note));
    let error = store
        .remove_tag_from_note(&note, &missing_tag, at(2))
        .unwrap_err();
    assert!(matches!(error, StoreError::TagNotFound(id) if id == missing_tag));
}
