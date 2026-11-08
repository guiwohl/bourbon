use bourbon_core::{Note, NoteId, NoteStore, NotebookId, TagColor};
use gpui::VisualTestContext;
use gpui::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};

use super::{at, choose_menu_entry, click, listed_titles, menu_labels, note_in, open_view};
use crate::navigation::Navigation;
use crate::workspace::Workspace;

struct Fixture {
    store: NoteStore,
    inbox: NotebookId,
    work: NotebookId,
}

fn fixture() -> Fixture {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    let work = store.create_notebook("Work", None, at(1)).unwrap().id;
    Fixture { store, inbox, work }
}

fn template_in(store: &NoteStore, notebook: &NotebookId, title: &str, at_millis: i64) -> NoteId {
    let template = store.create_template(notebook, at(at_millis)).unwrap();
    store
        .save_note(&template.id, title, "body", at(at_millis))
        .unwrap();
    template.id
}

fn stored(cx: &mut TestAppContext, view: &Entity<Workspace>, id: &NoteId) -> Note {
    cx.read(|cx| view.read(cx).store.find_note(id).unwrap().unwrap())
}

fn selected(cx: &mut TestAppContext, view: &Entity<Workspace>) -> NoteId {
    cx.read(|cx| view.read(cx).selected.clone().unwrap())
}

fn selected_note(cx: &mut TestAppContext, view: &Entity<Workspace>) -> Note {
    let id = selected(cx, view);
    stored(cx, view, &id)
}

fn navigation(cx: &mut TestAppContext, view: &Entity<Workspace>) -> Navigation {
    cx.read(|cx| view.read(cx).listing.navigation.clone())
}

fn is_present(cx: &mut TestAppContext, handle: AnyWindowHandle, id: &str) -> bool {
    super::is_listed(cx, handle, id)
}

fn title_is_focused(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    view: &Entity<Workspace>,
) -> bool {
    super::render(cx, handle);
    cx.update_window(handle, |_, window, cx| {
        view.read(cx).editor.title_is_focused(window, cx)
    })
    .unwrap()
}

const FIRST_TEMPLATE_ITEM: usize = 4;
const SORT_ENTRIES: usize = 3;

fn new_from_template(cx: &mut TestAppContext, handle: AnyWindowHandle, index: usize) {
    click(cx, handle, "note-list-title");
    choose_menu_entry(cx, handle, SORT_ENTRIES + index);
}

#[gpui::test]
fn the_templates_row_lists_only_templates(cx: &mut TestAppContext) {
    let fixture = fixture();
    note_in(&fixture.store, &fixture.inbox, "Plain", 2);
    template_in(&fixture.store, &fixture.inbox, "Meeting", 3);
    let (handle, view) = open_view(cx, fixture.store);
    assert_eq!(listed_titles(cx, &view), ["Plain"]);

    click(cx, handle, "nav-templates");

    assert_eq!(navigation(cx, &view), Navigation::Templates);
    assert_eq!(listed_titles(cx, &view), ["Meeting"]);
}

#[gpui::test]
fn new_template_creates_selects_and_focuses_a_template(cx: &mut TestAppContext) {
    let fixture = fixture();
    note_in(&fixture.store, &fixture.inbox, "Plain", 2);
    let (handle, view) = open_view(cx, fixture.store);

    click(cx, handle, "nav-templates");
    click(cx, handle, "new-note");

    let created = selected_note(cx, &view);
    assert!(created.is_template);
    assert_eq!(created.notebook, fixture.inbox);
    assert_eq!(navigation(cx, &view), Navigation::Templates);
    assert_eq!(listed_titles(cx, &view).len(), 1);
    assert!(title_is_focused(cx, handle, &view));
}

#[gpui::test]
fn ctrl_n_in_the_templates_view_creates_a_template(cx: &mut TestAppContext) {
    let fixture = fixture();
    note_in(&fixture.store, &fixture.inbox, "Plain", 2);
    let (handle, view) = open_view(cx, fixture.store);
    click(cx, handle, "nav-templates");

    VisualTestContext::from_window(handle, cx).simulate_keystrokes("ctrl-n");

    assert!(selected_note(cx, &view).is_template);
    assert_eq!(navigation(cx, &view), Navigation::Templates);
    assert_eq!(listed_titles(cx, &view).len(), 1);
}

#[gpui::test]
fn ctrl_n_outside_the_templates_view_still_creates_a_plain_note(cx: &mut TestAppContext) {
    let fixture = fixture();
    note_in(&fixture.store, &fixture.inbox, "Plain", 2);
    let (handle, view) = open_view(cx, fixture.store);

    VisualTestContext::from_window(handle, cx).simulate_keystrokes("ctrl-n");

    assert!(!selected_note(cx, &view).is_template);
}

#[gpui::test]
fn the_new_note_button_in_the_templates_view_creates_a_template(cx: &mut TestAppContext) {
    let fixture = fixture();
    let (handle, view) = open_view(cx, fixture.store);
    click(cx, handle, "nav-templates");

    click(cx, handle, "new-note");

    assert!(selected_note(cx, &view).is_template);
    assert_eq!(navigation(cx, &view), Navigation::Templates);
}

#[gpui::test]
fn new_from_template_is_hidden_without_templates(cx: &mut TestAppContext) {
    let fixture = fixture();
    note_in(&fixture.store, &fixture.inbox, "Plain", 2);
    let (handle, view) = open_view(cx, fixture.store);

    click(cx, handle, "note-list-title");

    assert_eq!(
        menu_labels(cx, &view),
        ["Sort by updated", "Sort by created", "Sort by title"]
    );
}

#[gpui::test]
fn the_menu_lists_templates_by_title_with_untitled_for_blank_ones(cx: &mut TestAppContext) {
    let fixture = fixture();
    template_in(&fixture.store, &fixture.inbox, "Meeting", 3);
    template_in(&fixture.store, &fixture.inbox, "", 4);
    let (handle, view) = open_view(cx, fixture.store);

    click(cx, handle, "note-list-title");

    let labels = menu_labels(cx, &view);
    assert_eq!(labels[FIRST_TEMPLATE_ITEM - 1], "New from template");
    assert_eq!(labels[FIRST_TEMPLATE_ITEM..], ["Untitled", "Meeting"]);
}

#[gpui::test]
fn picking_a_template_in_a_notebook_creates_the_note_there(cx: &mut TestAppContext) {
    let fixture = fixture();
    template_in(&fixture.store, &fixture.inbox, "Meeting", 3);
    note_in(&fixture.store, &fixture.work, "In work", 2);
    let (handle, view) = open_view(cx, fixture.store);
    click(cx, handle, &format!("nav-notebook-{}", fixture.work));

    new_from_template(cx, handle, 0);

    let created = selected_note(cx, &view);
    assert_eq!(created.title, "Meeting");
    assert_eq!(created.body, "body");
    assert!(!created.is_template);
    assert_eq!(created.notebook, fixture.work);
    assert!(listed_titles(cx, &view).contains(&"Meeting".to_owned()));
    assert!(title_is_focused(cx, handle, &view));
}

#[gpui::test]
fn picking_a_template_in_all_notes_creates_the_note_in_the_inbox(cx: &mut TestAppContext) {
    let fixture = fixture();
    template_in(&fixture.store, &fixture.work, "Meeting", 3);
    let (handle, view) = open_view(cx, fixture.store);

    new_from_template(cx, handle, 0);

    let created = selected_note(cx, &view);
    assert_eq!(created.notebook, fixture.inbox);
    assert_eq!(listed_titles(cx, &view), ["Meeting"]);
}

#[gpui::test]
fn picking_a_template_copies_its_tags(cx: &mut TestAppContext) {
    let fixture = fixture();
    let template = template_in(&fixture.store, &fixture.inbox, "Meeting", 3);
    fixture
        .store
        .add_new_tag_to_note(&template, "weekly", TagColor::Default, at(4))
        .unwrap();
    let (handle, view) = open_view(cx, fixture.store);

    new_from_template(cx, handle, 0);

    let created = selected_note(cx, &view);
    assert_eq!(created.tags.len(), 1);
    assert_eq!(created.tags[0].name, "weekly");
}

#[gpui::test]
fn picking_a_template_from_the_trash_switches_to_all_notes(cx: &mut TestAppContext) {
    let fixture = fixture();
    template_in(&fixture.store, &fixture.work, "Meeting", 3);
    let (handle, view) = open_view(cx, fixture.store);
    click(cx, handle, "nav-trash");

    new_from_template(cx, handle, 0);

    assert_eq!(navigation(cx, &view), Navigation::AllNotes);
    let created = selected_note(cx, &view);
    assert_eq!(created.notebook, fixture.inbox);
    assert_eq!(listed_titles(cx, &view), ["Meeting"]);
}

#[gpui::test]
fn picking_a_template_from_the_templates_view_switches_to_all_notes(cx: &mut TestAppContext) {
    let fixture = fixture();
    template_in(&fixture.store, &fixture.work, "Meeting", 3);
    let (handle, view) = open_view(cx, fixture.store);
    click(cx, handle, "nav-templates");

    new_from_template(cx, handle, 0);

    assert_eq!(navigation(cx, &view), Navigation::AllNotes);
    let created = selected_note(cx, &view);
    assert!(!created.is_template);
    assert_eq!(created.notebook, fixture.inbox);
    assert_eq!(listed_titles(cx, &view), ["Meeting"]);
}

#[gpui::test]
fn picking_a_template_saves_the_pending_edit_first(cx: &mut TestAppContext) {
    let fixture = fixture();
    let plain = note_in(&fixture.store, &fixture.inbox, "Plain", 2);
    template_in(&fixture.store, &fixture.inbox, "Meeting", 3);
    let (handle, view) = open_view(cx, fixture.store);
    click(cx, handle, "title");
    super::type_text(cx, handle, "!");

    new_from_template(cx, handle, 0);

    assert_eq!(stored(cx, &view, &plain).title, "Plain!");
}

fn open_note_actions(cx: &mut TestAppContext, handle: AnyWindowHandle) {
    click(cx, handle, "note-actions");
}

fn choose_note_action(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    view: &Entity<Workspace>,
    label: &str,
) {
    open_note_actions(cx, handle);
    let position = menu_labels(cx, view)
        .iter()
        .position(|candidate| candidate == label)
        .unwrap();
    choose_menu_entry(cx, handle, position);
}

#[gpui::test]
fn the_note_actions_menu_offers_mark_as_template_for_a_plain_note(cx: &mut TestAppContext) {
    let fixture = fixture();
    note_in(&fixture.store, &fixture.inbox, "Plain", 2);
    let (handle, view) = open_view(cx, fixture.store);

    open_note_actions(cx, handle);

    assert_eq!(
        menu_labels(cx, &view),
        [
            "Pin",
            "Duplicate",
            "Move to Trash",
            "Mark as template",
            "Copy CLI command"
        ]
    );
}

#[gpui::test]
fn the_note_actions_menu_offers_unmark_for_a_template(cx: &mut TestAppContext) {
    let fixture = fixture();
    template_in(&fixture.store, &fixture.inbox, "Meeting", 3);
    let (handle, view) = open_view(cx, fixture.store);
    click(cx, handle, "nav-templates");
    click(cx, handle, "new-note");

    open_note_actions(cx, handle);

    assert_eq!(
        menu_labels(cx, &view),
        [
            "Pin",
            "Duplicate",
            "Move to Trash",
            "Unmark template",
            "Copy CLI command"
        ]
    );
}

#[gpui::test]
fn marking_the_open_note_keeps_it_open_and_moves_it_to_templates(cx: &mut TestAppContext) {
    let fixture = fixture();
    let plain = note_in(&fixture.store, &fixture.inbox, "Plain", 2);
    let (handle, view) = open_view(cx, fixture.store);

    choose_note_action(cx, handle, &view, "Mark as template");

    assert!(stored(cx, &view, &plain).is_template);
    assert_eq!(selected(cx, &view), plain);
    assert_eq!(listed_titles(cx, &view), no_titles());
    assert!(is_present(cx, handle, "title"));
    click(cx, handle, "nav-templates");
    assert_eq!(listed_titles(cx, &view), ["Plain"]);
    open_note_actions(cx, handle);
    assert!(menu_labels(cx, &view).contains(&"Unmark template".to_owned()));
}

#[gpui::test]
fn unmarking_the_open_template_keeps_it_open_and_returns_it_to_all_notes(cx: &mut TestAppContext) {
    let fixture = fixture();
    let template = template_in(&fixture.store, &fixture.inbox, "Meeting", 3);
    let (handle, view) = open_view(cx, fixture.store);
    click(cx, handle, "nav-templates");
    click(cx, handle, &format!("note-{template}"));

    choose_note_action(cx, handle, &view, "Unmark template");

    assert!(!stored(cx, &view, &template).is_template);
    assert_eq!(selected(cx, &view), template);
    assert_eq!(listed_titles(cx, &view), no_titles());
    click(cx, handle, "nav-all-notes");
    assert_eq!(listed_titles(cx, &view), ["Meeting"]);
}

#[gpui::test]
fn marking_saves_the_pending_edit_first(cx: &mut TestAppContext) {
    let fixture = fixture();
    let plain = note_in(&fixture.store, &fixture.inbox, "Plain", 2);
    let (handle, view) = open_view(cx, fixture.store);
    click(cx, handle, "title");
    super::type_text(cx, handle, "!");

    choose_note_action(cx, handle, &view, "Mark as template");

    assert_eq!(stored(cx, &view, &plain).title, "Plain!");
}

fn no_titles() -> Vec<String> {
    Vec::new()
}

#[gpui::test]
fn the_note_actions_menu_copies_the_cli_command_of_the_open_note(cx: &mut TestAppContext) {
    let fixture = fixture();
    let plain = note_in(&fixture.store, &fixture.inbox, "Plain", 2);
    let (handle, view) = open_view(cx, fixture.store);

    choose_note_action(cx, handle, &view, "Copy CLI command");

    assert_eq!(
        cx.read_from_clipboard().and_then(|item| item.text()),
        Some(format!("bourbon get {plain}"))
    );
}

#[gpui::test]
fn the_note_actions_menu_pins_the_open_note(cx: &mut TestAppContext) {
    let fixture = fixture();
    let plain = note_in(&fixture.store, &fixture.inbox, "Plain", 2);
    let (handle, view) = open_view(cx, fixture.store);

    choose_note_action(cx, handle, &view, "Pin");

    assert!(stored(cx, &view, &plain).pinned);
    open_note_actions(cx, handle);
    assert_eq!(menu_labels(cx, &view)[0], "Unpin");
}
