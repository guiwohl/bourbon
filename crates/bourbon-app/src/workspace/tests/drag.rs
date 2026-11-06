use bourbon_core::{NoteId, NoteStore, NotebookId, Timestamp};
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};

use super::{at, click, listed_titles, note_in, open_view, type_into_title};
use crate::workspace::Workspace;

fn drag_note_onto_notebook(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    note: &NoteId,
    notebook: &NotebookId,
) {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.drag_to(
            format!("note-{note}"),
            format!("nav-notebook-{notebook}"),
            cx,
        );
        window.render_frame(cx);
    })
    .unwrap();
    cx.run_until_parked();
}

fn notebook_of(cx: &mut TestAppContext, view: &Entity<Workspace>, note: &NoteId) -> NotebookId {
    cx.read(|cx| {
        view.read(cx)
            .store
            .find_note(note)
            .unwrap()
            .unwrap()
            .notebook
    })
}

fn two_notes_and_work() -> (NoteStore, NoteId, NoteId, NotebookId) {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    let work = store.create_notebook("Work", None, at(1)).unwrap().id;
    let first = note_in(&store, &inbox, "First", 2);
    let second = note_in(&store, &inbox, "Second", 3);
    (store, first, second, work)
}

#[gpui_kit::test]
fn dropping_a_note_on_a_notebook_moves_it_and_keeps_the_open_note_open(cx: &mut TestAppContext) {
    let (store, first, second, work) = two_notes_and_work();
    let (handle, view) = open_view(cx, store);
    cx.read(|cx| assert_eq!(view.read(cx).selected, Some(second.clone())));

    drag_note_onto_notebook(cx, handle, &first, &work);

    assert_eq!(notebook_of(cx, &view, &first), work);
    cx.read(|cx| assert_eq!(view.read(cx).selected, Some(second.clone())));
    assert_eq!(listed_titles(cx, &view), ["First", "Second"]);
}

#[gpui_kit::test]
fn dropping_a_note_leaves_the_notebook_list_without_it(cx: &mut TestAppContext) {
    let (store, first, _, work) = two_notes_and_work();
    let inbox = store.default_notebook().unwrap();
    let (handle, view) = open_view(cx, store);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click(format!("nav-notebook-{inbox}"), cx);
    })
    .unwrap();
    assert_eq!(listed_titles(cx, &view), ["Second", "First"]);

    drag_note_onto_notebook(cx, handle, &first, &work);

    assert_eq!(listed_titles(cx, &view), ["Second"]);
}

#[gpui_kit::test]
fn dropping_the_open_note_refreshes_its_notebook_in_the_header(cx: &mut TestAppContext) {
    let (store, _, second, work) = two_notes_and_work();
    let (handle, view) = open_view(cx, store);

    drag_note_onto_notebook(cx, handle, &second, &work);

    assert_eq!(notebook_of(cx, &view, &second), work);
    cx.read(|cx| assert_eq!(view.read(cx).selected, Some(second.clone())));
    click(cx, handle, "notebook-selector");
    let marked = cx
        .update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.find(format!("notebook-option-{work}")).selected()
        })
        .unwrap();
    assert_eq!(marked, Some(true));
}

#[gpui_kit::test]
fn dropping_a_note_saves_the_pending_edit_first(cx: &mut TestAppContext) {
    let (store, first, second, work) = two_notes_and_work();
    let (handle, view) = open_view(cx, store);
    type_into_title(cx, handle, "!");

    drag_note_onto_notebook(cx, handle, &first, &work);

    cx.read(|cx| {
        let stored = view.read(cx).store.find_note(&second).unwrap().unwrap();
        assert_eq!(stored.title, "Second!");
    });
}

#[gpui_kit::test]
fn notes_in_the_trash_cannot_be_dragged(cx: &mut TestAppContext) {
    let (store, first, _, work) = two_notes_and_work();
    let inbox = store.default_notebook().unwrap();
    store.trash_note(&first, Timestamp::now()).unwrap();
    let (handle, view) = open_view(cx, store);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-trash", cx);
    })
    .unwrap();
    assert_eq!(listed_titles(cx, &view), ["First"]);

    drag_note_onto_notebook(cx, handle, &first, &work);

    assert_eq!(notebook_of(cx, &view, &first), inbox);
}
