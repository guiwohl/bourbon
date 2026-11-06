use super::support::{add_tag, at, attach_tag, inbox, pin, set_status, store, titled_note, trash};
use crate::{
    NoteQuery, NoteScope, NoteSort, NoteStatus, NoteStore, NotebookDepth, NotebookId, StoreError,
    TagColor, TemplateTimeZone,
};

fn titles(store: &NoteStore, scope: NoteScope) -> Vec<String> {
    store
        .list_notes(&NoteQuery {
            scope,
            sort: NoteSort::TitleAlphabetical,
        })
        .unwrap()
        .into_iter()
        .map(|summary| summary.title)
        .collect()
}

fn no_titles() -> Vec<String> {
    Vec::new()
}

fn template_titled(store: &NoteStore, notebook: &NotebookId, title: &str) -> crate::NoteId {
    let id = titled_note(store, notebook, title, 1, 1);
    store.mark_as_template(&id).unwrap();
    id
}

#[test]
fn ordinary_notes_are_not_templates() {
    let store = store();
    let note = store.create_note(&inbox(&store), at(1)).unwrap();
    assert!(!note.is_template);
    assert!(!store.find_note(&note.id).unwrap().unwrap().is_template);
}

#[test]
fn create_template_makes_an_empty_flagged_note() {
    let store = store();
    let notebook = inbox(&store);
    let template = store.create_template(&notebook, at(5)).unwrap();
    assert!(template.is_template);
    assert_eq!(template.title, "");
    assert_eq!(template.body, "");
    assert_eq!(template.notebook, notebook);
    assert_eq!(template.created_at, at(5));
    assert_eq!(template.updated_at, at(5));
    assert_eq!(template.status, NoteStatus::None);
    assert!(!template.pinned);
    assert_eq!(store.find_note(&template.id).unwrap(), Some(template));
}

#[test]
fn create_template_in_unknown_notebook_fails() {
    let store = store();
    let error = store
        .create_template(&NotebookId::generate(), at(1))
        .unwrap_err();
    assert!(matches!(error, StoreError::NotebookNotFound(_)));
}

#[test]
fn templates_are_listed_only_in_the_templates_scope() {
    let store = store();
    let notebook = inbox(&store);
    titled_note(&store, &notebook, "ordinary", 1, 1);
    template_titled(&store, &notebook, "template");
    assert_eq!(titles(&store, NoteScope::Templates), ["template"]);
    assert_eq!(titles(&store, NoteScope::AllNotes), ["ordinary"]);
}

#[test]
fn templates_are_hidden_from_notebook_status_and_tag_scopes() {
    let store = store();
    let parent = inbox(&store);
    let child = store
        .create_notebook("Child", Some(&parent), at(1))
        .unwrap()
        .id;
    let tag = add_tag(&store, "work", TagColor::Blue);
    let template = template_titled(&store, &parent, "direct");
    let nested = template_titled(&store, &child, "nested");
    for id in [&template, &nested] {
        attach_tag(&store, id, &tag);
        set_status(&store, id, NoteStatus::Active);
    }
    let hidden_everywhere = [
        NoteScope::Notebook(parent.clone(), NotebookDepth::DirectOnly),
        NoteScope::Notebook(parent, NotebookDepth::WithDescendants),
        NoteScope::Notebook(child, NotebookDepth::DirectOnly),
        NoteScope::Status(NoteStatus::Active),
        NoteScope::Tags(vec![tag]),
    ];
    for scope in hidden_everywhere {
        assert_eq!(titles(&store, scope.clone()), no_titles(), "{scope:?}");
    }
}

#[test]
fn completed_and_dropped_templates_are_hidden_from_status_scopes() {
    let store = store();
    let template = template_titled(&store, &inbox(&store), "done");
    set_status(&store, &template, NoteStatus::Completed);
    assert_eq!(
        titles(&store, NoteScope::Status(NoteStatus::Completed)),
        no_titles()
    );
}

#[test]
fn templates_scope_shows_every_status() {
    let store = store();
    let notebook = inbox(&store);
    let completed = template_titled(&store, &notebook, "completed");
    let active = template_titled(&store, &notebook, "active");
    set_status(&store, &completed, NoteStatus::Completed);
    set_status(&store, &active, NoteStatus::Active);
    assert_eq!(
        titles(&store, NoteScope::Templates),
        ["active", "completed"]
    );
}

#[test]
fn templates_scope_puts_pinned_first_and_honours_the_sort() {
    let store = store();
    let notebook = inbox(&store);
    template_titled(&store, &notebook, "a");
    template_titled(&store, &notebook, "b");
    let pinned = template_titled(&store, &notebook, "z");
    pin(&store, &pinned);
    assert_eq!(titles(&store, NoteScope::Templates), ["z", "a", "b"]);
}

#[test]
fn templates_scope_carries_tags() {
    let store = store();
    let template = template_titled(&store, &inbox(&store), "tagged");
    let tag = add_tag(&store, "work", TagColor::Blue);
    attach_tag(&store, &template, &tag);
    let listed = store
        .list_notes(&NoteQuery {
            scope: NoteScope::Templates,
            sort: NoteSort::default(),
        })
        .unwrap();
    assert_eq!(listed[0].tags.len(), 1);
    assert_eq!(listed[0].tags[0].id, tag);
}

#[test]
fn trashed_templates_leave_the_templates_scope_and_show_in_trash() {
    let store = store();
    let notebook = inbox(&store);
    let template = template_titled(&store, &notebook, "gone");
    let ordinary = titled_note(&store, &notebook, "also gone", 1, 1);
    trash(&store, &template, 9);
    trash(&store, &ordinary, 9);
    assert_eq!(titles(&store, NoteScope::Templates), no_titles());
    assert_eq!(titles(&store, NoteScope::Trash), ["also gone", "gone"]);
}

#[test]
fn restoring_a_template_returns_it_to_the_templates_scope() {
    let store = store();
    let template = template_titled(&store, &inbox(&store), "back");
    store.trash_note(&template, at(9)).unwrap();
    store.restore_note(&template).unwrap();
    assert_eq!(titles(&store, NoteScope::Templates), ["back"]);
}

#[test]
fn notebook_tree_and_tag_counts_ignore_templates() {
    let store = store();
    let notebook = inbox(&store);
    let ordinary = titled_note(&store, &notebook, "ordinary", 1, 1);
    let template = template_titled(&store, &notebook, "template");
    let tag = add_tag(&store, "work", TagColor::Blue);
    attach_tag(&store, &ordinary, &tag);
    attach_tag(&store, &template, &tag);
    assert_eq!(store.notebook_tree().unwrap()[0].note_count, 1);
    assert_eq!(store.list_tags().unwrap()[0].note_count, 1);
}

#[test]
fn marking_and_unmarking_move_a_note_between_scopes_without_editing_it() {
    let store = store();
    let notebook = inbox(&store);
    let id = titled_note(&store, &notebook, "switch", 1, 7);
    store.mark_as_template(&id).unwrap();
    assert_eq!(titles(&store, NoteScope::Templates), ["switch"]);
    assert_eq!(titles(&store, NoteScope::AllNotes), no_titles());
    let marked = store.find_note(&id).unwrap().unwrap();
    assert!(marked.is_template);
    assert_eq!(marked.updated_at, at(7));
    store.unmark_template(&id).unwrap();
    assert_eq!(titles(&store, NoteScope::AllNotes), ["switch"]);
    assert_eq!(titles(&store, NoteScope::Templates), no_titles());
    let unmarked = store.find_note(&id).unwrap().unwrap();
    assert!(!unmarked.is_template);
    assert_eq!(unmarked.updated_at, at(7));
}

#[test]
fn marking_and_unmarking_repeatedly_is_harmless() {
    let store = store();
    let id = titled_note(&store, &inbox(&store), "twice", 1, 1);
    store.mark_as_template(&id).unwrap();
    store.mark_as_template(&id).unwrap();
    assert!(store.find_note(&id).unwrap().unwrap().is_template);
    store.unmark_template(&id).unwrap();
    store.unmark_template(&id).unwrap();
    assert!(!store.find_note(&id).unwrap().unwrap().is_template);
}

#[test]
fn marking_an_unknown_note_fails() {
    let store = store();
    let unknown = crate::NoteId::generate();
    assert!(matches!(
        store.mark_as_template(&unknown),
        Err(StoreError::NoteNotFound(_))
    ));
    assert!(matches!(
        store.unmark_template(&unknown),
        Err(StoreError::NoteNotFound(_))
    ));
}

fn meeting_template(store: &NoteStore) -> crate::NoteId {
    let template = store.create_template(&inbox(store), at(1)).unwrap();
    store
        .save_note(
            &template.id,
            "Meeting {{date}}",
            "Started {{time}} on {{date}}\n{{unknown}}",
            at(2),
        )
        .unwrap();
    template.id
}

#[test]
fn note_from_template_copies_and_expands_title_and_body() {
    let store = store();
    let template = meeting_template(&store);
    let target = store.create_notebook("Work", None, at(3)).unwrap().id;
    let note = store
        .create_note_from_template(
            &template,
            &target,
            at(1_777_602_600_000),
            &TemplateTimeZone::utc(),
        )
        .unwrap();
    assert_eq!(note.title, "Meeting 2026-05-01");
    assert_eq!(note.body, "Started 02:30 on 2026-05-01\n{{unknown}}");
    assert_eq!(note.notebook, target);
    assert!(!note.is_template);
    assert_eq!(note.status, NoteStatus::None);
    assert!(!note.pinned);
    assert_eq!(note.created_at, at(1_777_602_600_000));
    assert_eq!(note.updated_at, at(1_777_602_600_000));
    assert_eq!(note.trashed_at, None);
    assert_eq!(store.find_note(&note.id).unwrap(), Some(note.clone()));
    assert_eq!(titles(&store, NoteScope::AllNotes), ["Meeting 2026-05-01"]);
}

#[test]
fn note_from_template_leaves_the_template_untouched() {
    let store = store();
    let template = meeting_template(&store);
    let before = store.find_note(&template).unwrap().unwrap();
    store
        .create_note_from_template(&template, &inbox(&store), at(9), &TemplateTimeZone::utc())
        .unwrap();
    assert_eq!(store.find_note(&template).unwrap().unwrap(), before);
    assert_eq!(titles(&store, NoteScope::Templates), ["Meeting {{date}}"]);
}

#[test]
fn note_from_template_expands_with_the_creation_time() {
    let store = store();
    let template = meeting_template(&store);
    let note = store
        .create_note_from_template(
            &template,
            &inbox(&store),
            at(86_400_000),
            &TemplateTimeZone::utc(),
        )
        .unwrap();
    assert_eq!(note.title, "Meeting 1970-01-02");
}

#[test]
fn note_from_template_copies_tags_but_not_status_or_pin() {
    let store = store();
    let template = meeting_template(&store);
    let first = add_tag(&store, "work", TagColor::Blue);
    let second = add_tag(&store, "weekly", TagColor::Green);
    attach_tag(&store, &template, &first);
    attach_tag(&store, &template, &second);
    set_status(&store, &template, NoteStatus::Active);
    pin(&store, &template);
    let note = store
        .create_note_from_template(&template, &inbox(&store), at(9), &TemplateTimeZone::utc())
        .unwrap();
    let mut copied: Vec<_> = note.tags.iter().map(|tag| tag.id.clone()).collect();
    copied.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    let mut expected = vec![first, second];
    expected.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    assert_eq!(copied, expected);
    assert_eq!(note.status, NoteStatus::None);
    assert!(!note.pinned);
}

#[test]
fn note_from_template_works_for_a_trashed_template() {
    let store = store();
    let template = meeting_template(&store);
    trash(&store, &template, 5);
    let note = store
        .create_note_from_template(&template, &inbox(&store), at(9), &TemplateTimeZone::utc())
        .unwrap();
    assert_eq!(note.trashed_at, None);
}

#[test]
fn note_from_unknown_template_fails_and_creates_nothing() {
    let store = store();
    let error = store
        .create_note_from_template(
            &crate::NoteId::generate(),
            &inbox(&store),
            at(9),
            &TemplateTimeZone::utc(),
        )
        .unwrap_err();
    assert!(matches!(error, StoreError::NoteNotFound(_)));
    assert_eq!(titles(&store, NoteScope::AllNotes), no_titles());
}

#[test]
fn note_in_unknown_notebook_fails_and_creates_nothing() {
    let store = store();
    let template = meeting_template(&store);
    let error = store
        .create_note_from_template(
            &template,
            &NotebookId::generate(),
            at(9),
            &TemplateTimeZone::utc(),
        )
        .unwrap_err();
    assert!(matches!(error, StoreError::NotebookNotFound(_)));
    assert_eq!(titles(&store, NoteScope::AllNotes), no_titles());
}

#[test]
fn note_from_a_note_that_is_not_a_template_fails() {
    let store = store();
    let notebook = inbox(&store);
    let ordinary = titled_note(&store, &notebook, "ordinary", 1, 1);
    let error = store
        .create_note_from_template(&ordinary, &notebook, at(9), &TemplateTimeZone::utc())
        .unwrap_err();
    assert!(matches!(&error, StoreError::NotATemplate(id) if *id == ordinary));
    assert_eq!(titles(&store, NoteScope::AllNotes), ["ordinary"]);
}

#[test]
fn duplicating_a_template_yields_a_template() {
    let store = store();
    let template = template_titled(&store, &inbox(&store), "Daily");
    let copy = store.duplicate_note(&template, at(5)).unwrap();
    assert!(copy.is_template);
    assert_eq!(titles(&store, NoteScope::Templates), vec!["Daily", "Daily"]);
}
