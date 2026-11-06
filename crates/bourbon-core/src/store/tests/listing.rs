use super::support::{add_tag, at, attach_tag, inbox, pin, set_status, store, titled_note, trash};
use crate::{
    NoteId, NoteQuery, NoteScope, NoteSort, NoteStatus, NoteStore, NoteSummary, NotebookDepth,
    NotebookId, TagColor, TagId,
};

fn query(scope: NoteScope, sort: NoteSort) -> NoteQuery {
    NoteQuery { scope, sort }
}

fn listed(store: &NoteStore, scope: NoteScope) -> Vec<NoteId> {
    listed_sorted(store, scope, NoteSort::default())
}

fn listed_sorted(store: &NoteStore, scope: NoteScope, sort: NoteSort) -> Vec<NoteId> {
    store
        .list_notes(&query(scope, sort))
        .unwrap()
        .into_iter()
        .map(|summary| summary.id)
        .collect()
}

#[test]
fn empty_store_lists_nothing() {
    assert_eq!(store().list_notes(&NoteQuery::default()).unwrap().len(), 0);
}

#[test]
fn default_order_is_updated_then_created_descending() {
    let store = store();
    let notebook = inbox(&store);
    let oldest = titled_note(&store, &notebook, "oldest", 1, 100);
    let middle = titled_note(&store, &notebook, "middle", 2, 2);
    let newest = titled_note(&store, &notebook, "newest", 3, 3);
    let tied_early = titled_note(&store, &notebook, "te", 4, 50);
    let tied_late = titled_note(&store, &notebook, "tl", 5, 50);
    assert_eq!(
        listed(&store, NoteScope::AllNotes),
        vec![oldest, tied_late, tied_early, newest, middle]
    );
}

#[test]
fn summary_carries_every_list_field() {
    let store = store();
    let note = store.create_note(&inbox(&store), at(1)).unwrap();
    store
        .save_note(&note.id, "Hello", "\n  first line  \nsecond", at(9))
        .unwrap();
    pin(&store, &note.id);
    set_status(&store, &note.id, NoteStatus::Active);
    let tag = add_tag(&store, "idea", TagColor::Teal);
    attach_tag(&store, &note.id, &tag);
    let summaries = store.list_notes(&NoteQuery::default()).unwrap();
    assert_eq!(
        summaries,
        vec![NoteSummary {
            id: note.id,
            title: "Hello".into(),
            excerpt: "first line".into(),
            updated_at: at(9),
            created_at: at(1),
            pinned: true,
            status: NoteStatus::Active,
            tags: vec![crate::Tag {
                id: tag,
                name: "idea".into(),
                color: TagColor::Teal
            }],
        }]
    );
}

#[test]
fn pinned_notes_come_first_in_every_sort() {
    let store = store();
    let notebook = inbox(&store);
    let newest = titled_note(&store, &notebook, "A newest", 9, 90);
    let pinned_old = titled_note(&store, &notebook, "Z pinned", 1, 10);
    pin(&store, &pinned_old);
    for sort in [
        NoteSort::UpdatedNewestFirst,
        NoteSort::CreatedNewestFirst,
        NoteSort::TitleAlphabetical,
    ] {
        assert_eq!(
            listed_sorted(&store, NoteScope::AllNotes, sort),
            vec![pinned_old.clone(), newest.clone()],
            "{sort:?}"
        );
    }
}

#[test]
fn pinned_notes_are_sorted_among_themselves() {
    let store = store();
    let notebook = inbox(&store);
    let older = titled_note(&store, &notebook, "older", 1, 10);
    let newer = titled_note(&store, &notebook, "newer", 2, 20);
    pin(&store, &older);
    pin(&store, &newer);
    assert_eq!(listed(&store, NoteScope::AllNotes), vec![newer, older]);
}

#[test]
fn created_sort_is_newest_creation_first() {
    let store = store();
    let notebook = inbox(&store);
    let early_but_edited = titled_note(&store, &notebook, "a", 1, 100);
    let late = titled_note(&store, &notebook, "b", 5, 6);
    let middle = titled_note(&store, &notebook, "c", 3, 4);
    assert_eq!(
        listed_sorted(&store, NoteScope::AllNotes, NoteSort::CreatedNewestFirst),
        vec![late, middle, early_but_edited]
    );
}

#[test]
fn title_sort_ignores_case_and_ties_go_to_most_recently_updated() {
    let store = store();
    let notebook = inbox(&store);
    let banana = titled_note(&store, &notebook, "banana", 1, 1);
    let apple = titled_note(&store, &notebook, "Apple", 2, 2);
    let cherry = titled_note(&store, &notebook, "cherry", 3, 3);
    let apple_twin_old = titled_note(&store, &notebook, "apple", 4, 4);
    let apple_twin_new = titled_note(&store, &notebook, "APPLE", 5, 50);
    let order = listed_sorted(&store, NoteScope::AllNotes, NoteSort::TitleAlphabetical);
    assert_eq!(order[0], apple_twin_new);
    assert_eq!(order[1], apple_twin_old);
    assert_eq!(order[2], apple);
    assert_eq!(order[3], banana);
    assert_eq!(order[4], cherry);
}

#[test]
fn all_notes_hide_trashed_and_finished_notes() {
    let store = store();
    let notebook = inbox(&store);
    let plain = titled_note(&store, &notebook, "plain", 1, 1);
    let active = titled_note(&store, &notebook, "active", 2, 2);
    let on_hold = titled_note(&store, &notebook, "hold", 3, 3);
    let completed = titled_note(&store, &notebook, "done", 4, 4);
    let dropped = titled_note(&store, &notebook, "dropped", 5, 5);
    let trashed = titled_note(&store, &notebook, "trashed", 6, 6);
    set_status(&store, &active, NoteStatus::Active);
    set_status(&store, &on_hold, NoteStatus::OnHold);
    set_status(&store, &completed, NoteStatus::Completed);
    set_status(&store, &dropped, NoteStatus::Dropped);
    trash(&store, &trashed, 7);
    assert_eq!(
        listed(&store, NoteScope::AllNotes),
        vec![on_hold, active, plain]
    );
}

#[test]
fn status_scope_shows_exactly_that_status_even_when_hidden_by_default() {
    let store = store();
    let notebook = inbox(&store);
    let plain = titled_note(&store, &notebook, "plain", 1, 1);
    let completed = titled_note(&store, &notebook, "done", 2, 2);
    let dropped = titled_note(&store, &notebook, "dropped", 3, 3);
    let trashed_completed = titled_note(&store, &notebook, "gone", 4, 4);
    set_status(&store, &completed, NoteStatus::Completed);
    set_status(&store, &dropped, NoteStatus::Dropped);
    set_status(&store, &trashed_completed, NoteStatus::Completed);
    trash(&store, &trashed_completed, 9);
    assert_eq!(
        listed(&store, NoteScope::Status(NoteStatus::Completed)),
        vec![completed]
    );
    assert_eq!(
        listed(&store, NoteScope::Status(NoteStatus::Dropped)),
        vec![dropped]
    );
    assert_eq!(
        listed(&store, NoteScope::Status(NoteStatus::None)),
        vec![plain]
    );
}

#[test]
fn trash_scope_shows_only_trashed_notes_including_hidden_statuses() {
    let store = store();
    let notebook = inbox(&store);
    titled_note(&store, &notebook, "alive", 1, 1);
    let trashed = titled_note(&store, &notebook, "trashed", 2, 2);
    let trashed_done = titled_note(&store, &notebook, "trashed done", 3, 3);
    set_status(&store, &trashed_done, NoteStatus::Completed);
    trash(&store, &trashed, 10);
    trash(&store, &trashed_done, 11);
    assert_eq!(
        listed(&store, NoteScope::Trash),
        vec![trashed_done, trashed]
    );
}

#[test]
fn direct_only_notebook_scope_excludes_sub_notebooks() {
    let store = store();
    let root = store.create_notebook("Root", None, at(1)).unwrap();
    let child = store
        .create_notebook("Child", Some(&root.id), at(2))
        .unwrap();
    let in_root = titled_note(&store, &root.id, "root note", 1, 1);
    titled_note(&store, &child.id, "child note", 2, 2);
    titled_note(&store, &inbox(&store), "inbox note", 3, 3);
    assert_eq!(
        listed(
            &store,
            NoteScope::Notebook(root.id, NotebookDepth::DirectOnly)
        ),
        vec![in_root]
    );
}

#[test]
fn with_descendants_covers_the_whole_subtree_and_nothing_else() {
    let store = store();
    let root = store.create_notebook("Root", None, at(1)).unwrap();
    let child = store
        .create_notebook("Child", Some(&root.id), at(2))
        .unwrap();
    let grandchild = store
        .create_notebook("Grandchild", Some(&child.id), at(3))
        .unwrap();
    let sibling = store.create_notebook("Sibling", None, at(4)).unwrap();
    let in_root = titled_note(&store, &root.id, "r", 1, 1);
    let in_child = titled_note(&store, &child.id, "c", 2, 2);
    let in_grandchild = titled_note(&store, &grandchild.id, "g", 3, 3);
    titled_note(&store, &sibling.id, "s", 4, 4);
    titled_note(&store, &inbox(&store), "i", 5, 5);
    assert_eq!(
        listed(
            &store,
            NoteScope::Notebook(root.id.clone(), NotebookDepth::WithDescendants)
        ),
        vec![in_grandchild.clone(), in_child.clone(), in_root]
    );
    assert_eq!(
        listed(
            &store,
            NoteScope::Notebook(child.id, NotebookDepth::WithDescendants)
        ),
        vec![in_grandchild, in_child]
    );
}

#[test]
fn notebook_scopes_hide_trashed_and_finished_notes() {
    let store = store();
    let notebook = store.create_notebook("Work", None, at(1)).unwrap();
    let visible = titled_note(&store, &notebook.id, "visible", 1, 1);
    let completed = titled_note(&store, &notebook.id, "done", 2, 2);
    let trashed = titled_note(&store, &notebook.id, "trashed", 3, 3);
    set_status(&store, &completed, NoteStatus::Completed);
    trash(&store, &trashed, 9);
    for depth in [NotebookDepth::DirectOnly, NotebookDepth::WithDescendants] {
        assert_eq!(
            listed(&store, NoteScope::Notebook(notebook.id.clone(), depth)),
            vec![visible.clone()]
        );
    }
}

#[test]
fn unknown_notebook_lists_nothing() {
    let store = store();
    titled_note(&store, &inbox(&store), "x", 1, 1);
    for depth in [NotebookDepth::DirectOnly, NotebookDepth::WithDescendants] {
        assert_eq!(
            listed(&store, NoteScope::Notebook(NotebookId::generate(), depth)).len(),
            0
        );
    }
}

struct Tagged {
    store: NoteStore,
    red: TagId,
    blue: TagId,
    both: NoteId,
    only_red: NoteId,
    only_blue: NoteId,
}

fn tagged_store() -> Tagged {
    let store = store();
    let notebook = inbox(&store);
    let red = add_tag(&store, "red", TagColor::Red);
    let blue = add_tag(&store, "blue", TagColor::Blue);
    let both = titled_note(&store, &notebook, "both", 1, 1);
    let only_red = titled_note(&store, &notebook, "red only", 2, 2);
    let only_blue = titled_note(&store, &notebook, "blue only", 3, 3);
    titled_note(&store, &notebook, "untagged", 4, 4);
    attach_tag(&store, &both, &red);
    attach_tag(&store, &both, &blue);
    attach_tag(&store, &only_red, &red);
    attach_tag(&store, &only_blue, &blue);
    Tagged {
        store,
        red,
        blue,
        both,
        only_red,
        only_blue,
    }
}

#[test]
fn one_tag_lists_every_note_having_it() {
    let fixture = tagged_store();
    assert_eq!(
        listed(&fixture.store, NoteScope::Tags(vec![fixture.red.clone()])),
        vec![fixture.only_red.clone(), fixture.both.clone()]
    );
    assert_eq!(
        listed(&fixture.store, NoteScope::Tags(vec![fixture.blue.clone()])),
        vec![fixture.only_blue, fixture.both]
    );
}

#[test]
fn several_tags_narrow_to_notes_having_all_of_them() {
    let fixture = tagged_store();
    assert_eq!(
        listed(
            &fixture.store,
            NoteScope::Tags(vec![fixture.red.clone(), fixture.blue.clone()])
        ),
        vec![fixture.both]
    );
}

#[test]
fn repeating_a_tag_does_not_change_the_result() {
    let fixture = tagged_store();
    assert_eq!(
        listed(
            &fixture.store,
            NoteScope::Tags(vec![fixture.red.clone(), fixture.red.clone()])
        ),
        vec![fixture.only_red, fixture.both]
    );
}

#[test]
fn unknown_tag_lists_nothing() {
    let fixture = tagged_store();
    assert_eq!(
        listed(
            &fixture.store,
            NoteScope::Tags(vec![fixture.red, TagId::generate()])
        )
        .len(),
        0
    );
}

#[test]
fn no_tags_selected_lists_every_visible_note() {
    let fixture = tagged_store();
    assert_eq!(listed(&fixture.store, NoteScope::Tags(Vec::new())).len(), 4);
}

#[test]
fn tag_scope_hides_trashed_and_finished_notes() {
    let fixture = tagged_store();
    set_status(&fixture.store, &fixture.both, NoteStatus::Completed);
    trash(&fixture.store, &fixture.only_red, 9);
    assert_eq!(
        listed(&fixture.store, NoteScope::Tags(vec![fixture.red.clone()])).len(),
        0
    );
    assert_eq!(
        listed(&fixture.store, NoteScope::Tags(vec![fixture.blue])),
        vec![fixture.only_blue]
    );
}

#[test]
fn tag_scope_respects_the_chosen_sort_and_pins() {
    let fixture = tagged_store();
    pin(&fixture.store, &fixture.both);
    assert_eq!(
        listed_sorted(
            &fixture.store,
            NoteScope::Tags(vec![fixture.red]),
            NoteSort::TitleAlphabetical
        ),
        vec![fixture.both, fixture.only_red]
    );
}

#[test]
fn summary_tags_are_ordered_by_name_ignoring_case_and_belong_to_their_note() {
    let store = store();
    let notebook = inbox(&store);
    let tagged = titled_note(&store, &notebook, "tagged", 1, 1);
    let other = titled_note(&store, &notebook, "other", 2, 2);
    let zulu = add_tag(&store, "zulu", TagColor::Default);
    let alpha = add_tag(&store, "Alpha", TagColor::Green);
    let mike = add_tag(&store, "mike", TagColor::Pink);
    attach_tag(&store, &tagged, &zulu);
    attach_tag(&store, &tagged, &alpha);
    attach_tag(&store, &other, &mike);
    let summaries = store.list_notes(&NoteQuery::default()).unwrap();
    let names = |summary: &NoteSummary| -> Vec<String> {
        summary.tags.iter().map(|tag| tag.name.clone()).collect()
    };
    assert_eq!(names(&summaries[0]), ["mike"]);
    assert_eq!(names(&summaries[1]), ["Alpha", "zulu"]);
}

#[test]
fn found_note_carries_its_tags_in_name_order_even_when_trashed() {
    let store = store();
    let note = store.create_note(&inbox(&store), at(1)).unwrap();
    let beta = add_tag(&store, "beta", TagColor::Brown);
    let alpha = add_tag(&store, "alpha", TagColor::Grey);
    attach_tag(&store, &note.id, &beta);
    attach_tag(&store, &note.id, &alpha);
    trash(&store, &note.id, 77);
    let found = store.find_note(&note.id).unwrap().unwrap();
    let names: Vec<_> = found.tags.iter().map(|tag| tag.name.as_str()).collect();
    assert_eq!(names, ["alpha", "beta"]);
    assert_eq!(found.tags[0].color, TagColor::Grey);
    assert_eq!(found.trashed_at, Some(at(77)));
}

#[test]
fn found_note_reports_status_and_pin() {
    let store = store();
    let note = store.create_note(&inbox(&store), at(1)).unwrap();
    pin(&store, &note.id);
    set_status(&store, &note.id, NoteStatus::OnHold);
    let found = store.find_note(&note.id).unwrap().unwrap();
    assert!(found.pinned);
    assert_eq!(found.status, NoteStatus::OnHold);
}

#[test]
fn listing_works_on_a_file_database() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("bourbon.db");
    let store = NoteStore::open(&path).unwrap();
    let note = titled_note(&store, &inbox(&store), "on disk", 1, 2);
    assert_eq!(listed(&store, NoteScope::AllNotes), vec![note]);
}
