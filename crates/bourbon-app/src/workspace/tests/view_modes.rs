use std::time::Duration;

use bourbon_core::NoteStore;
use gpui::{AnyWindowHandle, AppContext as _, Entity, TestAppContext, VisualTestContext};
use gpui_kit::base::test_support::texts;
use gpui_kit::test::TestWindowExt as _;

use super::{click, is_listed, open_view, seeded_store};
use crate::workspace::Workspace;
use bourbon_core::ViewMode;

pub(super) fn press(cx: &mut TestAppContext, handle: AnyWindowHandle, keystrokes: &str) {
    VisualTestContext::from_window(handle, cx).simulate_keystrokes(keystrokes);
}

pub(super) fn focus_body(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    view: &Entity<Workspace>,
) {
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |view, cx| view.editor.focus_body(window, cx));
    })
    .unwrap();
}

pub(super) fn type_into_body(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    view: &Entity<Workspace>,
    text: &str,
) {
    focus_body(cx, handle, view);
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |view, cx| view.editor.place_cursor_at_end(cx));
        window.render_frame(cx);
        window.input(text, cx);
    })
    .unwrap();
}

pub(super) fn is_rendered(cx: &mut TestAppContext, handle: AnyWindowHandle, id: &str) -> bool {
    is_listed(cx, handle, id)
}

pub(super) fn painted_text(cx: &mut TestAppContext, handle: AnyWindowHandle) -> Vec<String> {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        texts(window)
            .iter()
            .map(|snapshot| snapshot.text().to_string())
            .collect()
    })
    .unwrap()
}

pub(super) fn mode(cx: &mut TestAppContext, view: &Entity<Workspace>) -> ViewMode {
    cx.read(|cx| view.read(cx).view_mode)
}

fn store_with_body(body: &str) -> (NoteStore, bourbon_core::NoteId) {
    let (store, seeded) = seeded_store();
    store
        .save_note(&seeded.newer, "Newer", body, super::at(5))
        .unwrap();
    (store, seeded.newer)
}

#[gpui::test]
fn the_editor_starts_in_edit_mode_without_a_preview(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    assert_eq!(mode(cx, &view), ViewMode::Edit);
    assert!(is_rendered(cx, handle, "body-editor"));
    assert!(!is_rendered(cx, handle, "preview"));
}

#[gpui::test]
fn the_switch_offers_the_three_modes(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    click(cx, handle, "view-mode-preview");
    assert_eq!(mode(cx, &view), ViewMode::Preview);
    click(cx, handle, "view-mode-side-by-side");
    assert_eq!(mode(cx, &view), ViewMode::SideBySide);
    click(cx, handle, "view-mode-edit");
    assert_eq!(mode(cx, &view), ViewMode::Edit);
}

#[gpui::test]
fn preview_mode_shows_only_the_preview(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, _) = open_view(cx, store);

    click(cx, handle, "view-mode-preview");

    assert!(is_rendered(cx, handle, "preview"));
    assert!(!is_rendered(cx, handle, "body-editor"));
}

#[gpui::test]
fn side_by_side_shows_both_panes(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, _) = open_view(cx, store);

    click(cx, handle, "view-mode-side-by-side");

    assert!(is_rendered(cx, handle, "preview"));
    assert!(is_rendered(cx, handle, "body-editor"));
}

#[gpui::test]
fn ctrl_e_cycles_edit_side_by_side_preview_and_back(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    focus_body(cx, handle, &view);

    press(cx, handle, "ctrl-e");
    assert_eq!(mode(cx, &view), ViewMode::SideBySide);
    press(cx, handle, "ctrl-e");
    assert_eq!(mode(cx, &view), ViewMode::Preview);
    press(cx, handle, "ctrl-e");
    assert_eq!(mode(cx, &view), ViewMode::Edit);
}

#[gpui::test]
fn ctrl_e_cycles_even_from_the_title_field(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "title");

    press(cx, handle, "ctrl-e");

    assert_eq!(mode(cx, &view), ViewMode::SideBySide);
}

#[gpui::test]
fn the_mode_is_app_wide_and_survives_switching_notes(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "view-mode-preview");

    click(cx, handle, &format!("note-{}", seeded.older));

    assert_eq!(mode(cx, &view), ViewMode::Preview);
}

#[gpui::test]
fn the_preview_renders_the_body(cx: &mut TestAppContext) {
    let (store, _) = store_with_body("# Big heading\n\nplain words");
    let (handle, _) = open_view(cx, store);

    click(cx, handle, "view-mode-preview");

    let painted = painted_text(cx, handle).join("|");
    assert!(painted.contains("Big heading"), "{painted}");
    assert!(painted.contains("plain words"), "{painted}");
    assert!(!painted.contains("# Big heading"), "{painted}");
}

#[gpui::test]
fn a_table_renders_its_cells_in_the_preview(cx: &mut TestAppContext) {
    let (store, _) = store_with_body("| Name | Qty |\n|---|---|\n| apple | 3 |");
    let (handle, _) = open_view(cx, store);

    click(cx, handle, "view-mode-preview");

    let painted = painted_text(cx, handle).join("|");
    for cell in ["Name", "Qty", "apple", "3"] {
        assert!(painted.contains(cell), "{cell} missing in {painted}");
    }
    assert!(!painted.contains("|---|"), "{painted}");
}

#[gpui::test]
fn the_side_by_side_preview_follows_typing(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "view-mode-side-by-side");

    type_into_body(cx, handle, &view, " zebra");
    cx.run_until_parked();

    let painted = painted_text(cx, handle);
    let in_preview = cx
        .update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            let preview = window.find("preview".to_owned()).bounds();
            texts(window)
                .iter()
                .filter(|snapshot| preview.contains(&snapshot.bounds().center()))
                .map(|snapshot| snapshot.text().to_string())
                .collect::<Vec<_>>()
                .join("|")
        })
        .unwrap();
    assert!(in_preview.contains("zebra"), "{in_preview} / {painted:?}");
}

#[gpui::test]
fn the_footer_counts_words_and_characters_of_the_body(cx: &mut TestAppContext) {
    let (store, _) = store_with_body("one two\nthree");
    let (handle, view) = open_view(cx, store);

    assert_eq!(footer(cx, handle), "3 words \u{b7} 13 characters");

    type_into_body(cx, handle, &view, "!");
    cx.executor().advance_clock(Duration::from_millis(1));
    assert_eq!(footer(cx, handle), "3 words \u{b7} 14 characters");
}

fn footer(cx: &mut TestAppContext, handle: AnyWindowHandle) -> String {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let footer = window.find("word-count".to_owned()).bounds();
        texts(window)
            .iter()
            .filter(|snapshot| footer.contains(&snapshot.bounds().center()))
            .map(|snapshot| snapshot.text().to_string())
            .collect::<String>()
    })
    .unwrap()
}

fn button_state(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    id: &str,
) -> (Option<String>, Option<bool>) {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let label = window.find(id.to_owned()).label().map(str::to_owned);
        (label, window.find(format!("{id}-state")).selected())
    })
    .unwrap()
}

#[gpui::test]
fn the_switch_labels_its_modes_and_marks_only_the_current_one(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, _) = open_view(cx, store);
    click(cx, handle, "view-mode-side-by-side");

    let states: Vec<_> = [
        "view-mode-edit",
        "view-mode-side-by-side",
        "view-mode-preview",
    ]
    .map(|id| button_state(cx, handle, id))
    .into();

    assert_eq!(
        states,
        [
            (Some("Edit".to_owned()), Some(false)),
            (Some("Side-by-side".to_owned()), Some(true)),
            (Some("Preview".to_owned()), Some(false)),
        ]
    );
}
