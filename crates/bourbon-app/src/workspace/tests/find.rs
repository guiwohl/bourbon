use gpui_kit::TestAppContext;

use super::view_modes::{focus_body, is_rendered, mode, press};
use super::{click, open_view, seeded_store};
use crate::editor::ViewMode;

#[gpui_kit::test]
fn ctrl_f_in_the_editor_opens_find(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    focus_body(cx, handle, &view);

    press(cx, handle, "ctrl-f");

    cx.read(|cx| {
        let editor = &view.read(cx).editor;
        assert!(editor.search_is_open(cx));
        assert!(!editor.search_is_replacing(cx));
    });
}

#[gpui_kit::test]
fn ctrl_h_in_the_editor_opens_find_with_replace(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    focus_body(cx, handle, &view);

    press(cx, handle, "ctrl-h");

    cx.read(|cx| {
        let editor = &view.read(cx).editor;
        assert!(editor.search_is_open(cx));
        assert!(editor.search_is_replacing(cx));
    });
}

#[gpui_kit::test]
fn ctrl_f_from_the_title_field_still_opens_find_in_the_body(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "title");

    press(cx, handle, "ctrl-f");

    cx.read(|cx| assert!(view.read(cx).editor.search_is_open(cx)));
}

#[gpui_kit::test]
fn ctrl_h_from_the_title_field_opens_replace(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "title");

    press(cx, handle, "ctrl-h");

    cx.read(|cx| assert!(view.read(cx).editor.search_is_replacing(cx)));
}

#[gpui_kit::test]
fn find_from_the_preview_returns_to_the_editor(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "view-mode-preview");
    assert!(!is_rendered(cx, handle, "body-editor"));

    press(cx, handle, "ctrl-f");

    assert_eq!(mode(cx, &view), ViewMode::Edit);
    assert!(is_rendered(cx, handle, "body-editor"));
    cx.read(|cx| assert!(view.read(cx).editor.search_is_open(cx)));
}

#[gpui_kit::test]
fn find_from_side_by_side_keeps_the_mode(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "view-mode-side-by-side");

    press(cx, handle, "ctrl-f");

    assert_eq!(mode(cx, &view), ViewMode::SideBySide);
    cx.read(|cx| assert!(view.read(cx).editor.search_is_open(cx)));
}
