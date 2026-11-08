use std::time::Duration;

use bourbon_core::{NoteId, NoteStore};
use gpui::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};

use super::view_modes::focus_body;
use super::{at, click, open_view, press, seeded_store};
use crate::workspace::Workspace;

const BODY: &str = "intro\n- [ ] first\n- [x] second\n- plain\nclosing";

fn store_with_body(body: &str) -> (NoteStore, NoteId) {
    let (store, seeded) = seeded_store();
    store
        .save_note(&seeded.newer, "Newer", body, at(5))
        .unwrap();
    (store, seeded.newer)
}

fn editor_body(cx: &mut TestAppContext, view: &Entity<Workspace>) -> String {
    cx.read(|cx| view.read(cx).editor.body(cx))
}

fn stored_body(cx: &mut TestAppContext, view: &Entity<Workspace>, id: &NoteId) -> String {
    cx.read(|cx| view.read(cx).store.find_note(id).unwrap().unwrap().body)
}

fn place_cursor_on_line(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    view: &Entity<Workspace>,
    line: usize,
) {
    focus_body(cx, handle, view);
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |view, cx| {
            view.editor.place_cursor_on_line(line, window, cx);
        });
    })
    .unwrap();
}

#[gpui::test]
fn clicking_an_open_task_in_the_preview_checks_that_exact_line(cx: &mut TestAppContext) {
    let (store, _) = store_with_body(BODY);
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "view-mode-preview");

    click(cx, handle, "preview-task-1");

    assert_eq!(
        editor_body(cx, &view),
        "intro\n- [x] first\n- [x] second\n- plain\nclosing"
    );
}

#[gpui::test]
fn clicking_a_done_task_in_the_preview_reopens_it(cx: &mut TestAppContext) {
    let (store, _) = store_with_body(BODY);
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "view-mode-preview");

    click(cx, handle, "preview-task-2");

    assert_eq!(
        editor_body(cx, &view),
        "intro\n- [ ] first\n- [ ] second\n- plain\nclosing"
    );
}

#[gpui::test]
fn a_task_toggled_in_the_preview_is_autosaved_like_typing(cx: &mut TestAppContext) {
    let (store, id) = store_with_body(BODY);
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "view-mode-preview");
    click(cx, handle, "preview-task-1");

    cx.executor().advance_clock(Duration::from_millis(499));
    cx.run_until_parked();
    assert_eq!(stored_body(cx, &view, &id), BODY);

    cx.executor().advance_clock(Duration::from_millis(1));
    cx.run_until_parked();
    assert_eq!(
        stored_body(cx, &view, &id),
        "intro\n- [x] first\n- [x] second\n- plain\nclosing"
    );
}

#[gpui::test]
fn tasks_inside_a_code_fence_get_no_checkbox(cx: &mut TestAppContext) {
    let (store, _) = store_with_body("```\n- [ ] code\n```\n- [ ] real");
    let (handle, _) = open_view(cx, store);
    click(cx, handle, "view-mode-preview");

    assert!(!super::view_modes::is_rendered(
        cx,
        handle,
        "preview-task-1"
    ));
    assert!(super::view_modes::is_rendered(cx, handle, "preview-task-3"));
}

#[gpui::test]
fn ctrl_enter_toggles_the_task_on_the_cursor_line(cx: &mut TestAppContext) {
    let (store, _) = store_with_body(BODY);
    let (handle, view) = open_view(cx, store);
    place_cursor_on_line(cx, handle, &view, 1);

    press(cx, handle, "ctrl-enter");

    assert_eq!(
        editor_body(cx, &view),
        "intro\n- [x] first\n- [x] second\n- plain\nclosing"
    );
}

#[gpui::test]
fn ctrl_enter_turns_a_plain_list_line_into_a_task(cx: &mut TestAppContext) {
    let (store, _) = store_with_body(BODY);
    let (handle, view) = open_view(cx, store);
    place_cursor_on_line(cx, handle, &view, 3);

    press(cx, handle, "ctrl-enter");

    assert_eq!(
        editor_body(cx, &view),
        "intro\n- [ ] first\n- [x] second\n- [ ] plain\nclosing"
    );
}

#[gpui::test]
fn ctrl_enter_on_any_other_line_changes_nothing(cx: &mut TestAppContext) {
    let (store, _) = store_with_body(BODY);
    let (handle, view) = open_view(cx, store);
    place_cursor_on_line(cx, handle, &view, 0);

    press(cx, handle, "ctrl-enter");

    assert_eq!(editor_body(cx, &view), BODY);
}

#[gpui::test]
fn ctrl_enter_is_autosaved_and_keeps_the_cursor_on_its_line(cx: &mut TestAppContext) {
    let (store, id) = store_with_body(BODY);
    let (handle, view) = open_view(cx, store);
    place_cursor_on_line(cx, handle, &view, 1);

    press(cx, handle, "ctrl-enter");
    cx.executor().advance_clock(Duration::from_millis(500));
    cx.run_until_parked();
    assert_eq!(
        stored_body(cx, &view, &id),
        "intro\n- [x] first\n- [x] second\n- plain\nclosing"
    );

    press(cx, handle, "ctrl-enter");
    assert_eq!(editor_body(cx, &view), BODY);
}

#[gpui::test]
fn the_cursor_stays_on_a_line_that_became_a_task(cx: &mut TestAppContext) {
    let (store, _) = store_with_body(BODY);
    let (handle, view) = open_view(cx, store);
    place_cursor_on_line(cx, handle, &view, 3);

    press(cx, handle, "ctrl-enter");
    press(cx, handle, "ctrl-enter");

    assert_eq!(
        editor_body(cx, &view),
        "intro\n- [ ] first\n- [x] second\n- [x] plain\nclosing"
    );
}

fn cursor(cx: &mut TestAppContext, view: &Entity<Workspace>) -> usize {
    cx.read(|cx| view.read(cx).editor.cursor(cx))
}

#[gpui::test]
fn a_cursor_after_the_inserted_checkbox_moves_with_the_text(cx: &mut TestAppContext) {
    let (store, _) = store_with_body(BODY);
    let (handle, view) = open_view(cx, store);
    focus_body(cx, handle, &view);
    let end_of_plain = BODY.find("\nclosing").unwrap();
    cx.update_window(handle, |_, _, cx| {
        view.update(cx, |view, cx| view.editor.place_cursor_at(end_of_plain, cx));
    })
    .unwrap();

    press(cx, handle, "ctrl-enter");

    assert_eq!(cursor(cx, &view), end_of_plain + "[ ] ".len());
}

#[gpui::test]
fn a_cursor_before_the_edit_does_not_move(cx: &mut TestAppContext) {
    let (store, _) = store_with_body(BODY);
    let (handle, view) = open_view(cx, store);
    focus_body(cx, handle, &view);
    let start_of_plain = BODY.find("- plain").unwrap();
    cx.update_window(handle, |_, _, cx| {
        view.update(cx, |view, cx| {
            view.editor.place_cursor_at(start_of_plain, cx);
        });
    })
    .unwrap();

    press(cx, handle, "ctrl-enter");

    assert_eq!(cursor(cx, &view), start_of_plain);
}

#[gpui::test]
fn a_cursor_after_a_toggled_mark_stays_put(cx: &mut TestAppContext) {
    let (store, _) = store_with_body(BODY);
    let (handle, view) = open_view(cx, store);
    focus_body(cx, handle, &view);
    let end_of_first = BODY.find("\n- [x]").unwrap();
    cx.update_window(handle, |_, _, cx| {
        view.update(cx, |view, cx| view.editor.place_cursor_at(end_of_first, cx));
    })
    .unwrap();

    press(cx, handle, "ctrl-enter");

    assert_eq!(cursor(cx, &view), end_of_first);
}
