use gpui::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};

use super::view_modes::{focus_body, is_rendered, mode, press};
use super::{click, open_view, seeded_store, type_text};
use crate::control_pane::PaneAction;
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

fn find_field_has_focus(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    view: &Entity<Workspace>,
) -> bool {
    cx.update_window(handle, |_, window, cx| {
        let panel = view.read(cx).editor.search_panel().read(cx);
        let field = panel.find_field().read(cx);
        gpui::Focusable::focus_handle(field, cx).is_focused(window)
    })
    .unwrap()
}

fn search_state(cx: &mut TestAppContext, view: &Entity<Workspace>) -> (bool, bool) {
    view.read_with(cx, |view, cx| {
        let panel = view.editor.search_panel().read(cx);
        (panel.is_open(), panel.is_replacing())
    })
}

fn body_selection(cx: &mut TestAppContext, view: &Entity<Workspace>) -> std::ops::Range<usize> {
    view.read_with(cx, |view, cx| {
        view.editor.body_editor().read(cx).selected_range()
    })
}

#[gpui::test]
fn ctrl_f_opens_the_find_panel_with_focus_in_its_field(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "title");

    press(cx, handle, "ctrl-f");

    assert_eq!(search_state(cx, &view), (true, false));
    assert!(find_field_has_focus(cx, handle, &view));
}

#[gpui::test]
fn ctrl_h_and_ctrl_alt_f_open_it_with_replace(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    press(cx, handle, "ctrl-h");
    assert_eq!(search_state(cx, &view), (true, true));

    press(cx, handle, "escape");
    assert_eq!(search_state(cx, &view), (false, false));
    assert!(body_has_focus(cx, handle, &view));

    press(cx, handle, "ctrl-alt-f");
    assert_eq!(search_state(cx, &view), (true, true));
}

#[gpui::test]
fn typing_selects_the_match_and_ctrl_g_moves_on_from_the_body(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    press(cx, handle, "ctrl-f");
    type_text(cx, handle, "e");
    assert_eq!(body_selection(cx, &view), 1..2);

    focus_body(cx, handle, &view);
    press(cx, handle, "ctrl-g");
    assert_eq!(body_selection(cx, &view), 3..4);
    press(cx, handle, "ctrl-shift-g");
    assert_eq!(body_selection(cx, &view), 1..2);
}

#[gpui::test]
fn the_pane_actions_open_find_and_replace(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |view, cx| {
            view.run_pane_action(PaneAction::Replace, window, cx);
        });
    })
    .unwrap();
    assert_eq!(search_state(cx, &view), (true, true));

    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |view, cx| {
            view.run_pane_action(PaneAction::Find, window, cx);
        });
    })
    .unwrap();
    assert_eq!(search_state(cx, &view), (true, false));
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
    assert!(find_field_has_focus(cx, handle, &view));
}

#[gpui::test]
fn find_from_side_by_side_keeps_the_mode(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    super::choose_view_mode(cx, handle, ViewMode::SideBySide);
    focus_body(cx, handle, &view);

    press(cx, handle, "ctrl-f");

    assert_eq!(mode(cx, &view), ViewMode::SideBySide);
    assert!(find_field_has_focus(cx, handle, &view));
}

#[gpui::test]
fn find_without_an_open_note_does_nothing(cx: &mut TestAppContext) {
    let (handle, view) = open_view(cx, bourbon_core::NoteStore::open_in_memory().unwrap());

    press(cx, handle, "ctrl-f");

    assert_eq!(search_state(cx, &view), (false, false));
}

#[gpui::test]
fn switching_notes_refreshes_the_matches(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    press(cx, handle, "ctrl-f");
    type_text(cx, handle, "older");
    let matches = |cx: &mut TestAppContext| {
        view.read_with(cx, |view, cx| {
            view.editor.search_panel().read(cx).matches().to_vec()
        })
    };
    assert_eq!(matches(cx), Vec::<std::ops::Range<usize>>::new());

    view.update(cx, |view, cx| view.select(&seeded.older, cx));
    cx.run_until_parked();

    assert_eq!(matches(cx), vec![0..5]);
}
