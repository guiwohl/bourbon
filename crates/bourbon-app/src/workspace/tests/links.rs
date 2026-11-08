use std::time::Duration;

use bourbon_core::NoteId;
use gpui::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};
use gpui_kit::test::TestWindowExt as _;

use super::view_modes::{is_rendered, type_into_body};
use super::{at, banner_is_shown, click, open_view, press, seeded_store, type_text};
use crate::navigation::Navigation;
use crate::workspace::Workspace;

fn editor_body(cx: &mut TestAppContext, view: &Entity<Workspace>) -> String {
    cx.read(|cx| view.read(cx).editor.body(cx))
}

fn selected(cx: &mut TestAppContext, view: &Entity<Workspace>) -> Option<NoteId> {
    cx.read(|cx| view.read(cx).selected.clone())
}

fn suggestion(id: &NoteId) -> String {
    format!("link-suggestion-{id}")
}

fn open_link(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    view: &Entity<Workspace>,
    url: &str,
) {
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |view, cx| view.open_link(url, window, cx));
    })
    .unwrap();
}

fn note_url(id: &NoteId) -> String {
    format!("bourbon://note/{id}")
}

#[gpui::test]
fn typing_two_brackets_offers_the_notes(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    type_into_body(cx, handle, &view, "[[");

    assert!(is_rendered(cx, handle, "link-suggestions"));
    assert!(is_rendered(cx, handle, &suggestion(&seeded.older)));
    assert!(is_rendered(cx, handle, &suggestion(&seeded.newer)));
}

#[gpui::test]
fn typing_after_the_brackets_filters_the_titles_ignoring_case(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    type_into_body(cx, handle, &view, "[[oLD");

    assert!(is_rendered(cx, handle, &suggestion(&seeded.older)));
    assert!(!is_rendered(cx, handle, &suggestion(&seeded.newer)));
}

#[gpui::test]
fn suggestions_also_match_note_bodies(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let inbox = store.default_notebook().unwrap();
    let hidden = store.create_note(&inbox, at(5)).unwrap();
    store
        .save_note(&hidden.id, "Zed", "contains pineapple", at(6))
        .unwrap();
    let (handle, view) = open_view(cx, store);

    type_into_body(cx, handle, &view, "[[pineapple");

    assert!(is_rendered(cx, handle, &suggestion(&hidden.id)));
}

#[gpui::test]
fn no_matching_note_closes_the_popup(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    type_into_body(cx, handle, &view, "[[zzz");

    assert!(!is_rendered(cx, handle, "link-suggestions"));
}

#[gpui::test]
fn a_single_bracket_offers_nothing(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    type_into_body(cx, handle, &view, "[");

    assert!(!is_rendered(cx, handle, "link-suggestions"));
}

#[gpui::test]
fn enter_replaces_the_query_with_a_markdown_link(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    type_into_body(cx, handle, &view, " see [[Old");

    press(cx, handle, "enter");

    assert_eq!(
        editor_body(cx, &view),
        format!("newer body see [Older](bourbon://note/{})", seeded.older)
    );
    assert!(!is_rendered(cx, handle, "link-suggestions"));
}

#[gpui::test]
fn down_moves_the_highlight_and_enter_picks_that_note(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    type_into_body(cx, handle, &view, "[[");

    press(cx, handle, "down");
    press(cx, handle, "enter");
    let first = editor_body(cx, &view);

    assert!(
        first.ends_with(&format!("(bourbon://note/{})", seeded.older)),
        "{first}"
    );
}

#[gpui::test]
fn up_returns_the_highlight_to_the_first_note(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    type_into_body(cx, handle, &view, "[[");

    press(cx, handle, "down");
    press(cx, handle, "up");
    press(cx, handle, "enter");

    let body = editor_body(cx, &view);
    assert!(
        body.ends_with(&format!("(bourbon://note/{})", seeded.newer)),
        "{body}"
    );
}

#[gpui::test]
fn escape_cancels_and_leaves_the_typed_text(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    type_into_body(cx, handle, &view, "[[Old");

    press(cx, handle, "escape");

    assert_eq!(editor_body(cx, &view), "newer body[[Old]]");
    assert!(!is_rendered(cx, handle, "link-suggestions"));
}

#[gpui::test]
fn clicking_a_suggestion_inserts_its_link(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    type_into_body(cx, handle, &view, "[[");

    click(cx, handle, &suggestion(&seeded.older));

    assert_eq!(
        editor_body(cx, &view),
        format!("newer body[Older](bourbon://note/{})", seeded.older)
    );
}

#[gpui::test]
fn an_untitled_note_is_linked_as_untitled(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let blank = store
        .create_note(&store.default_notebook().unwrap(), at(9))
        .unwrap();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, &format!("note-{}", seeded.newer));
    type_into_body(cx, handle, &view, "[[");

    press(cx, handle, "enter");

    assert_eq!(
        editor_body(cx, &view),
        format!("newer body[Untitled](bourbon://note/{})", blank.id)
    );
}

#[gpui::test]
fn enter_without_a_popup_still_inserts_a_newline(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    type_into_body(cx, handle, &view, "x");

    press(cx, handle, "enter");

    assert_eq!(editor_body(cx, &view), "newer bodyx\n");
}

#[gpui::test]
fn a_picked_link_is_autosaved(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    type_into_body(cx, handle, &view, "[[Old");
    press(cx, handle, "enter");

    cx.executor().advance_clock(Duration::from_millis(500));
    cx.run_until_parked();

    let stored = cx.read(|cx| {
        view.read(cx)
            .store
            .find_note(&seeded.newer)
            .unwrap()
            .unwrap()
            .body
    });
    assert_eq!(
        stored,
        format!("newer body[Older](bourbon://note/{})", seeded.older)
    );
}

#[gpui::test]
fn opening_a_note_link_selects_that_note(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    open_link(cx, handle, &view, &note_url(&seeded.older));

    assert_eq!(selected(cx, &view), Some(seeded.older.clone()));
    assert_eq!(cx.read(|cx| view.read(cx).editor.body(cx)), "older body");
}

#[gpui::test]
fn opening_a_note_link_saves_the_pending_edit_first(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    type_into_body(cx, handle, &view, "!");

    open_link(cx, handle, &view, &note_url(&seeded.older));

    let stored = cx.read(|cx| {
        view.read(cx)
            .store
            .find_note(&seeded.newer)
            .unwrap()
            .unwrap()
            .body
    });
    assert_eq!(stored, "newer body!");
}

#[gpui::test]
fn a_link_to_a_note_outside_the_listing_switches_to_all_notes(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let work = store.create_notebook("Work", None, at(5)).unwrap();
    super::note_in(&store, &work.id, "Elsewhere", 6);
    let (handle, view) = open_view(cx, store);
    click(cx, handle, &format!("nav-notebook-{}", work.id));
    cx.read(|cx| {
        assert!(
            !view
                .read(cx)
                .notes
                .iter()
                .any(|note| note.id == seeded.older)
        );
    });

    open_link(cx, handle, &view, &note_url(&seeded.older));

    assert_eq!(selected(cx, &view), Some(seeded.older.clone()));
    cx.read(|cx| assert_eq!(view.read(cx).listing.navigation, Navigation::AllNotes));
}

#[gpui::test]
fn a_link_to_a_listed_note_keeps_the_navigation(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let work = store.create_notebook("Work", None, at(5)).unwrap();
    store.move_note(&seeded.older, &work.id, at(6)).unwrap();
    store.move_note(&seeded.newer, &work.id, at(6)).unwrap();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, &format!("nav-notebook-{}", work.id));

    open_link(cx, handle, &view, &note_url(&seeded.older));

    assert_eq!(selected(cx, &view), Some(seeded.older.clone()));
    cx.read(|cx| {
        assert!(matches!(
            view.read(cx).listing.navigation,
            Navigation::Notebook(..)
        ));
    });
}

#[gpui::test]
fn a_link_to_a_missing_note_shows_the_banner_and_keeps_the_open_note(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    open_link(cx, handle, &view, &note_url(&NoteId::generate()));

    assert!(banner_is_shown(cx, handle));
    assert_eq!(selected(cx, &view), Some(seeded.newer.clone()));
}

#[gpui::test]
fn a_link_to_a_trashed_note_shows_the_banner_and_keeps_the_open_note(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    store.trash_note(&seeded.older, at(9)).unwrap();
    let (handle, view) = open_view(cx, store);

    open_link(cx, handle, &view, &note_url(&seeded.older));

    assert!(banner_is_shown(cx, handle));
    assert_eq!(selected(cx, &view), Some(seeded.newer.clone()));
}

#[gpui::test]
fn other_links_go_to_the_system_opener(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    open_link(cx, handle, &view, "https://example.com/page");

    assert_eq!(cx.opened_url().as_deref(), Some("https://example.com/page"));
    assert_eq!(selected(cx, &view), Some(seeded.newer.clone()));
}

fn store_with_link_to_older() -> (bourbon_core::NoteStore, NoteId, NoteId) {
    let (store, seeded) = seeded_store();
    store
        .save_note(
            &seeded.newer,
            "Newer",
            &format!("[Older]({})", note_url(&seeded.older)),
            at(5),
        )
        .unwrap();
    (store, seeded.newer, seeded.older)
}

#[gpui::test]
fn clicking_a_note_link_in_the_preview_opens_that_note(cx: &mut TestAppContext) {
    let (store, _, older) = store_with_link_to_older();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "view-mode-preview");
    cx.run_until_parked();

    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click_at(
            "preview".to_owned(),
            gpui::point(gpui::px(14.), gpui::px(16.)),
            cx,
        );
    })
    .unwrap();

    assert_eq!(selected(cx, &view), Some(older));
}

fn highlighted_suggestions(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    ids: [&NoteId; 2],
) -> [Option<bool>; 2] {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        ids.map(|id| window.find(suggestion(id)).selected())
    })
    .unwrap()
}

#[gpui::test]
fn only_the_highlighted_suggestion_is_marked_selected(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    type_into_body(cx, handle, &view, "[[");
    let ids = [&seeded.newer, &seeded.older];

    assert_eq!(
        highlighted_suggestions(cx, handle, ids),
        [Some(true), Some(false)]
    );
    press(cx, handle, "down");
    assert_eq!(
        highlighted_suggestions(cx, handle, ids),
        [Some(false), Some(true)]
    );
}

#[gpui::test]
fn picking_a_link_keeps_the_text_that_follows_the_query(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    super::view_modes::focus_body(cx, handle, &view);
    type_text(cx, handle, "[[Old");

    press(cx, handle, "enter");

    assert_eq!(
        editor_body(cx, &view),
        format!("[Older](bourbon://note/{})newer body", seeded.older)
    );
}
