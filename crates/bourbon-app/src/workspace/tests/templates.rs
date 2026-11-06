use bourbon_core::{Note, NoteId, NoteStore, NotebookId, TagColor};
use gpui_kit::VisualTestContext;
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};

use super::{at, click, listed_titles, note_in, open_view};
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
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.try_find(id.to_owned()).is_some()
    })
    .unwrap()
}

fn title_is_focused(cx: &mut TestAppContext, handle: AnyWindowHandle) -> bool {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.find("title").focused() == Some(true)
    })
    .unwrap()
}

fn choose_popup_item(cx: &mut TestAppContext, handle: AnyWindowHandle, index: usize) {
    cx.update_window(handle, |_, window, cx| {
        window.within("popup-menu").click(index, cx);
    })
    .unwrap();
    cx.run_until_parked();
}

fn popup_labels(cx: &mut TestAppContext, handle: AnyWindowHandle) -> Vec<String> {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let menu = window.within("popup-menu");
        (0usize..)
            .map_while(|index| menu.try_find(index))
            .map(|item| item.label().unwrap_or_default().to_string())
            .collect()
    })
    .unwrap()
}

fn new_from_template(cx: &mut TestAppContext, handle: AnyWindowHandle, index: usize) {
    click(cx, handle, "new-from-template");
    choose_popup_item(cx, handle, index);
}

#[gpui_kit::test]
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

#[gpui_kit::test]
fn new_template_creates_selects_and_focuses_a_template(cx: &mut TestAppContext) {
    let fixture = fixture();
    note_in(&fixture.store, &fixture.inbox, "Plain", 2);
    let (handle, view) = open_view(cx, fixture.store);

    click(cx, handle, "new-template");

    let created = selected_note(cx, &view);
    assert!(created.is_template);
    assert_eq!(created.notebook, fixture.inbox);
    assert_eq!(navigation(cx, &view), Navigation::Templates);
    assert_eq!(listed_titles(cx, &view).len(), 1);
    assert!(title_is_focused(cx, handle));
}

#[gpui_kit::test]
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

#[gpui_kit::test]
fn ctrl_n_outside_the_templates_view_still_creates_a_plain_note(cx: &mut TestAppContext) {
    let fixture = fixture();
    note_in(&fixture.store, &fixture.inbox, "Plain", 2);
    let (handle, view) = open_view(cx, fixture.store);

    VisualTestContext::from_window(handle, cx).simulate_keystrokes("ctrl-n");

    assert!(!selected_note(cx, &view).is_template);
}

#[gpui_kit::test]
fn the_new_note_button_in_the_templates_view_creates_a_template(cx: &mut TestAppContext) {
    let fixture = fixture();
    let (handle, view) = open_view(cx, fixture.store);
    click(cx, handle, "nav-templates");

    click(cx, handle, "list-new-template");

    assert!(selected_note(cx, &view).is_template);
    assert!(!is_present(cx, handle, "new-note"));
}

#[gpui_kit::test]
fn new_from_template_is_hidden_without_templates(cx: &mut TestAppContext) {
    let fixture = fixture();
    note_in(&fixture.store, &fixture.inbox, "Plain", 2);
    let (handle, _) = open_view(cx, fixture.store);

    assert!(is_present(cx, handle, "new-note"));
    assert!(!is_present(cx, handle, "new-from-template"));
}

#[gpui_kit::test]
fn the_menu_lists_templates_by_title_with_untitled_for_blank_ones(cx: &mut TestAppContext) {
    let fixture = fixture();
    template_in(&fixture.store, &fixture.inbox, "Meeting", 3);
    template_in(&fixture.store, &fixture.inbox, "", 4);
    let (handle, _) = open_view(cx, fixture.store);

    click(cx, handle, "new-from-template");

    assert_eq!(popup_labels(cx, handle), ["Untitled", "Meeting"]);
}

#[gpui_kit::test]
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
    assert!(title_is_focused(cx, handle));
}

#[gpui_kit::test]
fn picking_a_template_in_all_notes_creates_the_note_in_the_inbox(cx: &mut TestAppContext) {
    let fixture = fixture();
    template_in(&fixture.store, &fixture.work, "Meeting", 3);
    let (handle, view) = open_view(cx, fixture.store);

    new_from_template(cx, handle, 0);

    let created = selected_note(cx, &view);
    assert_eq!(created.notebook, fixture.inbox);
    assert_eq!(listed_titles(cx, &view), ["Meeting"]);
}

#[gpui_kit::test]
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

#[gpui_kit::test]
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

#[gpui_kit::test]
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

#[gpui_kit::test]
fn picking_a_template_saves_the_pending_edit_first(cx: &mut TestAppContext) {
    let fixture = fixture();
    let plain = note_in(&fixture.store, &fixture.inbox, "Plain", 2);
    template_in(&fixture.store, &fixture.inbox, "Meeting", 3);
    let (handle, view) = open_view(cx, fixture.store);
    click(cx, handle, "title");
    cx.update_window(handle, |_, window, cx| window.input("!", cx))
        .unwrap();

    new_from_template(cx, handle, 0);

    assert_eq!(stored(cx, &view, &plain).title, "Plain!");
}

fn open_note_actions(cx: &mut TestAppContext, handle: AnyWindowHandle) {
    click(cx, handle, "note-actions");
}

#[gpui_kit::test]
fn the_note_actions_menu_offers_mark_as_template_for_a_plain_note(cx: &mut TestAppContext) {
    let fixture = fixture();
    note_in(&fixture.store, &fixture.inbox, "Plain", 2);
    let (handle, _) = open_view(cx, fixture.store);

    open_note_actions(cx, handle);

    assert_eq!(popup_labels(cx, handle), ["Mark as template"]);
}

#[gpui_kit::test]
fn the_note_actions_menu_offers_unmark_for_a_template(cx: &mut TestAppContext) {
    let fixture = fixture();
    template_in(&fixture.store, &fixture.inbox, "Meeting", 3);
    let (handle, _) = open_view(cx, fixture.store);
    click(cx, handle, "nav-templates");
    click(cx, handle, "new-template");

    open_note_actions(cx, handle);

    assert_eq!(popup_labels(cx, handle), ["Unmark template"]);
}

#[gpui_kit::test]
fn marking_the_open_note_keeps_it_open_and_moves_it_to_templates(cx: &mut TestAppContext) {
    let fixture = fixture();
    let plain = note_in(&fixture.store, &fixture.inbox, "Plain", 2);
    let (handle, view) = open_view(cx, fixture.store);

    open_note_actions(cx, handle);
    choose_popup_item(cx, handle, 0);

    assert!(stored(cx, &view, &plain).is_template);
    assert_eq!(selected(cx, &view), plain);
    assert_eq!(listed_titles(cx, &view), no_titles());
    assert!(is_present(cx, handle, "title"));
    click(cx, handle, "nav-templates");
    assert_eq!(listed_titles(cx, &view), ["Plain"]);
    open_note_actions(cx, handle);
    assert_eq!(popup_labels(cx, handle), ["Unmark template"]);
}

#[gpui_kit::test]
fn unmarking_the_open_template_keeps_it_open_and_returns_it_to_all_notes(cx: &mut TestAppContext) {
    let fixture = fixture();
    let template = template_in(&fixture.store, &fixture.inbox, "Meeting", 3);
    let (handle, view) = open_view(cx, fixture.store);
    click(cx, handle, "nav-templates");
    click(cx, handle, &format!("note-{template}"));

    open_note_actions(cx, handle);
    choose_popup_item(cx, handle, 0);

    assert!(!stored(cx, &view, &template).is_template);
    assert_eq!(selected(cx, &view), template);
    assert_eq!(listed_titles(cx, &view), no_titles());
    click(cx, handle, "nav-all-notes");
    assert_eq!(listed_titles(cx, &view), ["Meeting"]);
}

#[gpui_kit::test]
fn marking_saves_the_pending_edit_first(cx: &mut TestAppContext) {
    let fixture = fixture();
    let plain = note_in(&fixture.store, &fixture.inbox, "Plain", 2);
    let (handle, view) = open_view(cx, fixture.store);
    click(cx, handle, "title");
    cx.update_window(handle, |_, window, cx| window.input("!", cx))
        .unwrap();

    open_note_actions(cx, handle);
    choose_popup_item(cx, handle, 0);

    assert_eq!(stored(cx, &view, &plain).title, "Plain!");
}

fn no_titles() -> Vec<String> {
    Vec::new()
}
