use gpui_kit::{AnyWindowHandle, Entity, TestAppContext};

use super::view_modes::{focus_body, is_rendered, press};
use super::{click, open_view, seeded_store};
use crate::workspace::Workspace;

fn columns_are_shown(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    view: &Entity<Workspace>,
) -> bool {
    let newer = cx.read(|cx| view.read(cx).selected.clone().unwrap());
    let sidebar = is_rendered(cx, handle, "nav-all-notes");
    let list = is_rendered(cx, handle, &format!("note-{newer}"));
    assert_eq!(sidebar, list, "sidebar and list must hide together");
    sidebar
}

#[gpui_kit::test]
fn ctrl_shift_f_hides_the_sidebar_and_the_list_and_again_restores_them(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    focus_body(cx, handle, &view);
    assert!(columns_are_shown(cx, handle, &view));

    press(cx, handle, "ctrl-shift-f");
    assert!(!columns_are_shown(cx, handle, &view));
    assert!(is_rendered(cx, handle, "body-editor"));

    press(cx, handle, "ctrl-shift-f");
    assert!(columns_are_shown(cx, handle, &view));
}

#[gpui_kit::test]
fn the_expand_button_enters_and_leaves_focus_mode(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    click(cx, handle, "focus-toggle");
    assert!(!columns_are_shown(cx, handle, &view));

    click(cx, handle, "focus-toggle");
    assert!(columns_are_shown(cx, handle, &view));
}

#[gpui_kit::test]
fn escape_leaves_focus_mode(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    focus_body(cx, handle, &view);
    press(cx, handle, "ctrl-shift-f");

    press(cx, handle, "escape");

    assert!(columns_are_shown(cx, handle, &view));
}

#[gpui_kit::test]
fn escape_outside_focus_mode_changes_nothing(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    focus_body(cx, handle, &view);

    press(cx, handle, "escape");

    assert!(columns_are_shown(cx, handle, &view));
    assert!(!cx.read(|cx| view.read(cx).focus_mode));
}

#[gpui_kit::test]
fn a_sidebar_hidden_before_focus_mode_stays_hidden_after_it(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    focus_body(cx, handle, &view);
    press(cx, handle, "ctrl-/");
    assert!(!is_rendered(cx, handle, "nav-all-notes"));

    press(cx, handle, "ctrl-shift-f");
    press(cx, handle, "escape");

    assert!(!is_rendered(cx, handle, "nav-all-notes"));
    assert!(is_rendered(
        cx,
        handle,
        &format!(
            "note-{}",
            cx.read(|cx| view.read(cx).selected.clone().unwrap())
        )
    ));
}

#[gpui_kit::test]
fn the_sidebar_shortcut_is_ignored_while_in_focus_mode(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    focus_body(cx, handle, &view);
    press(cx, handle, "ctrl-shift-f");

    press(cx, handle, "ctrl-/");
    assert!(!is_rendered(cx, handle, "nav-all-notes"));

    press(cx, handle, "escape");
    assert!(columns_are_shown(cx, handle, &view));
}
