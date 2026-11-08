use std::time::Duration;

use bourbon_core::NoteStore;
use bourbon_markdown::PreviewDoc;
use gpui::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};

use super::{click, open_view, seeded_store};
use crate::workspace::Workspace;
use bourbon_core::ViewMode;

pub(super) fn press(cx: &mut TestAppContext, handle: AnyWindowHandle, keystrokes: &str) {
    super::press(cx, handle, keystrokes);
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
    view.update(cx, |view, cx| view.editor.place_cursor_at_end(cx));
    super::type_text(cx, handle, text);
}

pub(super) fn is_rendered(cx: &mut TestAppContext, handle: AnyWindowHandle, id: &str) -> bool {
    super::is_listed(cx, handle, id)
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

    super::choose_view_mode(cx, handle, bourbon_core::ViewMode::Preview);
    assert_eq!(mode(cx, &view), ViewMode::Preview);
    super::choose_view_mode(cx, handle, bourbon_core::ViewMode::SideBySide);
    assert_eq!(mode(cx, &view), ViewMode::SideBySide);
    super::choose_view_mode(cx, handle, bourbon_core::ViewMode::Edit);
    assert_eq!(mode(cx, &view), ViewMode::Edit);
}

#[gpui::test]
fn preview_mode_shows_only_the_preview(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, _) = open_view(cx, store);

    super::choose_view_mode(cx, handle, bourbon_core::ViewMode::Preview);

    assert!(is_rendered(cx, handle, "preview"));
    assert!(!is_rendered(cx, handle, "body-editor"));
}

#[gpui::test]
fn side_by_side_shows_both_panes(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, _) = open_view(cx, store);

    super::choose_view_mode(cx, handle, bourbon_core::ViewMode::SideBySide);

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
    super::choose_view_mode(cx, handle, bourbon_core::ViewMode::Preview);

    click(cx, handle, &format!("note-{}", seeded.older));

    assert_eq!(mode(cx, &view), ViewMode::Preview);
}

fn preview_doc(cx: &mut TestAppContext, view: &Entity<Workspace>) -> PreviewDoc {
    cx.read(|cx| view.read(cx).editor.preview().read(cx).doc().clone())
}

#[gpui::test]
fn the_preview_shows_the_body_of_the_open_note(cx: &mut TestAppContext) {
    let (store, _) = store_with_body("# Big heading\n\nplain words");
    let (handle, view) = open_view(cx, store);

    super::choose_view_mode(cx, handle, ViewMode::Preview);

    assert_eq!(
        preview_doc(cx, &view),
        PreviewDoc::parse("# Big heading\n\nplain words")
    );
}

#[gpui::test]
fn switching_notes_replaces_the_preview(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    super::choose_view_mode(cx, handle, ViewMode::Preview);

    click(cx, handle, &format!("note-{}", seeded.older));

    assert_eq!(preview_doc(cx, &view), PreviewDoc::parse("older body"));
}

#[gpui::test]
fn the_side_by_side_preview_follows_typing(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    super::choose_view_mode(cx, handle, ViewMode::SideBySide);

    type_into_body(cx, handle, &view, " zebra");
    cx.run_until_parked();

    assert_eq!(
        preview_doc(cx, &view),
        PreviewDoc::parse("newer body zebra")
    );
}

#[gpui::test]
fn the_footer_counts_words_and_characters_of_the_body(cx: &mut TestAppContext) {
    let (store, _) = store_with_body("one two\nthree");
    let (handle, view) = open_view(cx, store);

    assert!(footer_reads(cx, handle, "3 words \u{b7} 13 characters"));

    type_into_body(cx, handle, &view, "!");
    cx.executor().advance_clock(Duration::from_millis(1));
    assert!(footer_reads(cx, handle, "3 words \u{b7} 14 characters"));
    assert!(!footer_reads(cx, handle, "3 words \u{b7} 13 characters"));
}

fn footer_reads(cx: &mut TestAppContext, handle: AnyWindowHandle, label: &str) -> bool {
    is_rendered(cx, handle, &format!("word-count: {label}"))
}

fn long_body() -> String {
    "Paragraph\n\n".repeat(80)
}

fn scroll_editor_to(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    view: &Entity<Workspace>,
    line: f32,
) {
    view.update(cx, |view, cx| {
        view.editor
            .body_editor()
            .update(cx, |body, cx| body.scroll_to_line(line, cx));
    });
    settle(cx, handle);
}

fn settle(cx: &mut TestAppContext, handle: AnyWindowHandle) {
    for _ in 0..3 {
        super::render(cx, handle);
        cx.run_until_parked();
    }
}

fn preview_top(cx: &mut TestAppContext, view: &Entity<Workspace>) -> f32 {
    cx.read(|cx| f32::from(view.read(cx).editor.preview().read(cx).scroll_top()))
}

fn editor_line(cx: &mut TestAppContext, view: &Entity<Workspace>) -> f32 {
    cx.read(|cx| view.read(cx).editor.body_editor().read(cx).top_line())
}

#[gpui::test]
fn scrolling_the_editor_side_by_side_scrolls_the_preview_to_the_same_line(cx: &mut TestAppContext) {
    let (store, _) = store_with_body(&long_body());
    let (handle, view) = open_view(cx, store);
    super::choose_view_mode(cx, handle, ViewMode::SideBySide);
    settle(cx, handle);

    scroll_editor_to(cx, handle, &view, 40.0);

    let anchors = cx.read(|cx| view.read(cx).editor.preview().read(cx).scroll_anchors());
    let expected = bourbon_markdown::preview_offset_for_line(&anchors, 40.0);
    assert!(expected > 0.0);
    assert!((preview_top(cx, &view) - expected).abs() < 1.0);
}

#[gpui::test]
fn scrolling_the_preview_side_by_side_scrolls_the_editor(cx: &mut TestAppContext) {
    let (store, _) = store_with_body(&long_body());
    let (handle, view) = open_view(cx, store);
    super::choose_view_mode(cx, handle, ViewMode::SideBySide);
    settle(cx, handle);

    view.update(cx, |view, cx| {
        view.editor.preview().update(cx, |preview, cx| {
            preview.set_scroll_top(gpui::px(600.), cx);
        });
    });
    settle(cx, handle);

    let anchors = cx.read(|cx| view.read(cx).editor.preview().read(cx).scroll_anchors());
    let expected = bourbon_markdown::line_for_preview_offset(&anchors, 600.0);
    assert!(expected > 1.0);
    assert!((editor_line(cx, &view) - expected).abs() < 0.05);
}

#[gpui::test]
fn the_editor_scrolls_alone_outside_side_by_side(cx: &mut TestAppContext) {
    let (store, _) = store_with_body(&long_body());
    let (handle, view) = open_view(cx, store);
    settle(cx, handle);

    scroll_editor_to(cx, handle, &view, 40.0);

    assert!(editor_line(cx, &view) > 1.0);
    assert!(preview_top(cx, &view) < 0.5);
}

#[gpui::test]
fn the_preview_scrolls_alone_in_preview_mode(cx: &mut TestAppContext) {
    let (store, _) = store_with_body(&long_body());
    let (handle, view) = open_view(cx, store);
    super::choose_view_mode(cx, handle, ViewMode::Preview);
    settle(cx, handle);

    view.update(cx, |view, cx| {
        view.editor.preview().update(cx, |preview, cx| {
            preview.set_scroll_top(gpui::px(600.), cx);
        });
    });
    settle(cx, handle);

    assert!(preview_top(cx, &view) > 500.0);
    assert!(editor_line(cx, &view) < 0.01);
}

#[gpui::test]
fn opening_another_note_resets_both_panes_to_the_top(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    store
        .save_note(&seeded.older, "Older", &long_body(), super::at(6))
        .unwrap();
    let (handle, view) = open_view(cx, store);
    super::choose_view_mode(cx, handle, ViewMode::SideBySide);
    click(cx, handle, &format!("note-{}", seeded.older));
    settle(cx, handle);
    scroll_editor_to(cx, handle, &view, 40.0);
    assert!(preview_top(cx, &view) > 0.0);

    click(cx, handle, &format!("note-{}", seeded.newer));
    click(cx, handle, &format!("note-{}", seeded.older));
    settle(cx, handle);

    assert!(preview_top(cx, &view) < 0.5);
    assert!(editor_line(cx, &view) < 0.01);
}

#[gpui::test]
fn the_panes_follow_the_installed_appearance(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    let light = bourbon_editor::EditorTheme::light().foreground;
    let dark = bourbon_editor::EditorTheme::dark().foreground;
    let foreground = |cx: &mut TestAppContext| {
        cx.read(|cx| {
            view.read(cx)
                .editor
                .body_editor()
                .read(cx)
                .theme()
                .foreground
        })
    };
    assert_eq!(foreground(cx), light);

    cx.update(|cx| bourbon_ui::Theme::install(bourbon_ui::Appearance::Dark, cx));
    settle(cx, handle);

    assert_eq!(foreground(cx), dark);
    let preview = cx.read(|cx| view.read(cx).editor.preview().read(cx).theme().text);
    assert_eq!(preview, bourbon_editor::PreviewTheme::dark().text);
}

#[gpui::test]
fn enter_in_the_title_moves_to_the_body(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "title");

    press(cx, handle, "enter");

    let focused = cx
        .update_window(handle, |_, window, cx| {
            let body = view
                .read(cx)
                .editor
                .body_editor()
                .read(cx)
                .focus_handle()
                .clone();
            body.is_focused(window)
        })
        .unwrap();
    assert!(focused);
}

#[gpui::test]
fn the_body_shows_line_numbers(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (_, view) = open_view(cx, store);

    let numbered = cx.read(|cx| view.read(cx).editor.body_editor().read(cx).line_numbers());
    assert!(numbered);
}
