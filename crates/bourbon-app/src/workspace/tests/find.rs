use gpui::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};

use super::view_modes::{focus_body, is_rendered, mode, press};
use super::{click, open_view, seeded_store};
use crate::workspace::Workspace;
use bourbon_core::ViewMode;

fn body_has_focus(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    view: &Entity<Workspace>,
) -> bool {
    cx.update_window(handle, |_, window, cx| {
        let body = view
            .read(cx)
            .editor
            .body_editor()
            .read(cx)
            .focus_handle()
            .clone();
        body.is_focused(window)
    })
    .unwrap()
}

#[gpui::test]
fn ctrl_f_from_the_title_field_moves_to_the_body(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "title");
    assert!(!body_has_focus(cx, handle, &view));

    press(cx, handle, "ctrl-f");

    assert!(body_has_focus(cx, handle, &view));
}

#[gpui::test]
fn ctrl_h_from_the_title_field_moves_to_the_body(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "title");

    press(cx, handle, "ctrl-h");

    assert!(body_has_focus(cx, handle, &view));
}

#[gpui::test]
fn find_from_the_preview_returns_to_the_editor(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    super::choose_view_mode(cx, handle, ViewMode::Preview);
    assert!(!is_rendered(cx, handle, "body-editor"));

    press(cx, handle, "ctrl-f");

    assert_eq!(mode(cx, &view), ViewMode::Edit);
    assert!(is_rendered(cx, handle, "body-editor"));
    assert!(body_has_focus(cx, handle, &view));
}

#[gpui::test]
fn find_from_side_by_side_keeps_the_mode(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    super::choose_view_mode(cx, handle, ViewMode::SideBySide);
    focus_body(cx, handle, &view);

    press(cx, handle, "ctrl-f");

    assert_eq!(mode(cx, &view), ViewMode::SideBySide);
    assert!(body_has_focus(cx, handle, &view));
}

#[gpui::test]
fn find_without_an_open_note_does_nothing(cx: &mut TestAppContext) {
    let (handle, view) = open_view(cx, bourbon_core::NoteStore::open_in_memory().unwrap());

    press(cx, handle, "ctrl-f");

    assert!(!body_has_focus(cx, handle, &view));
}
