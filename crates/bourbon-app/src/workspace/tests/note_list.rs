use bourbon_core::{NoteId, NoteSort, NoteStore, TagColor, Timestamp};
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};

use super::{
    at, choose_menu_entry as choose_menu_item, click, listed_titles, menu_labels, note_in,
    open_view,
};
use crate::navigation::Navigation;
use crate::workspace::Workspace;

fn right_click(cx: &mut TestAppContext, handle: AnyWindowHandle, id: &str) {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.right_click(id.to_owned(), cx);
        window.render_frame(cx);
    })
    .unwrap();
}

fn no_titles() -> Vec<String> {
    Vec::new()
}

fn is_present(cx: &mut TestAppContext, handle: AnyWindowHandle, id: &str) -> bool {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.try_find(id.to_owned()).is_some()
    })
    .unwrap()
}

fn selected_title(cx: &mut TestAppContext, view: &Entity<Workspace>) -> Option<String> {
    cx.read(|cx| {
        let view = view.read(cx);
        let id = view.selected.as_ref()?;
        Some(view.store.find_note(id).unwrap().unwrap().title)
    })
}

fn three_notes() -> (NoteStore, [NoteId; 3]) {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    let mango = note_in(&store, &inbox, "Mango", 1);
    let zebra = note_in(&store, &inbox, "Zebra", 2);
    let apple = note_in(&store, &inbox, "Apple", 3);
    store.save_note(&mango, "Mango", "edited", at(10)).unwrap();
    (store, [mango, zebra, apple])
}

#[gpui_kit::test]
fn the_sort_selector_reorders_by_updated_created_and_title(cx: &mut TestAppContext) {
    let (store, _) = three_notes();
    let (handle, view) = open_view(cx, store);
    assert_eq!(listed_titles(cx, &view), ["Mango", "Apple", "Zebra"]);

    click(cx, handle, "note-list-title");
    choose_menu_item(cx, handle, 1);
    assert_eq!(listed_titles(cx, &view), ["Apple", "Zebra", "Mango"]);

    click(cx, handle, "note-list-title");
    choose_menu_item(cx, handle, 2);
    assert_eq!(listed_titles(cx, &view), ["Apple", "Mango", "Zebra"]);

    click(cx, handle, "note-list-title");
    choose_menu_item(cx, handle, 0);
    assert_eq!(listed_titles(cx, &view), ["Mango", "Apple", "Zebra"]);
    cx.read(|cx| {
        assert_eq!(view.read(cx).listing.sort, NoteSort::UpdatedNewestFirst);
    });
}

#[gpui_kit::test]
fn pinning_moves_a_note_to_the_top_and_shows_the_pin_icon(cx: &mut TestAppContext) {
    let (store, [_, zebra, _]) = three_notes();
    let (handle, view) = open_view(cx, store);
    assert!(!is_present(cx, handle, &format!("pin-{zebra}")));

    right_click(cx, handle, &format!("note-{zebra}"));
    choose_menu_item(cx, handle, 0);

    assert_eq!(listed_titles(cx, &view), ["Zebra", "Mango", "Apple"]);
    assert!(is_present(cx, handle, &format!("pin-{zebra}")));
}

#[gpui_kit::test]
fn unpinning_returns_the_note_to_its_sorted_place_and_hides_the_icon(cx: &mut TestAppContext) {
    let (store, [_, zebra, _]) = three_notes();
    store.pin_note(&zebra).unwrap();
    let (handle, view) = open_view(cx, store);
    assert_eq!(listed_titles(cx, &view), ["Zebra", "Mango", "Apple"]);

    right_click(cx, handle, &format!("note-{zebra}"));
    choose_menu_item(cx, handle, 0);

    assert_eq!(listed_titles(cx, &view), ["Mango", "Apple", "Zebra"]);
    assert!(!is_present(cx, handle, &format!("pin-{zebra}")));
}

#[gpui_kit::test]
fn duplicating_creates_a_copy_in_the_list_and_selects_it(cx: &mut TestAppContext) {
    let (store, [mango, _, _]) = three_notes();
    let (handle, view) = open_view(cx, store);

    right_click(cx, handle, &format!("note-{mango}"));
    choose_menu_item(cx, handle, 1);

    let titles = listed_titles(cx, &view);
    assert_eq!(
        titles
            .iter()
            .filter(|title| title.contains("Mango"))
            .count(),
        2
    );
    let selected = cx.read(|cx| view.read(cx).selected.clone());
    assert_ne!(selected, Some(mango));
    assert!(selected_title(cx, &view).unwrap().contains("Mango"));
    cx.read(|cx| assert_eq!(view.read(cx).editor.body(cx), "edited"));
}

#[gpui_kit::test]
fn trashing_the_selected_note_selects_the_next_row(cx: &mut TestAppContext) {
    let (store, [mango, _, _]) = three_notes();
    let (handle, view) = open_view(cx, store);
    assert_eq!(selected_title(cx, &view).as_deref(), Some("Mango"));

    right_click(cx, handle, &format!("note-{mango}"));
    choose_menu_item(cx, handle, 2);

    assert_eq!(listed_titles(cx, &view), ["Apple", "Zebra"]);
    assert_eq!(selected_title(cx, &view).as_deref(), Some("Apple"));
}

#[gpui_kit::test]
fn trashing_the_last_selected_row_selects_the_previous_one(cx: &mut TestAppContext) {
    let (store, [_, zebra, _]) = three_notes();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, &format!("note-{zebra}"));
    assert_eq!(selected_title(cx, &view).as_deref(), Some("Zebra"));

    right_click(cx, handle, &format!("note-{zebra}"));
    choose_menu_item(cx, handle, 2);

    assert_eq!(listed_titles(cx, &view), ["Mango", "Apple"]);
    assert_eq!(selected_title(cx, &view).as_deref(), Some("Apple"));
}

#[gpui_kit::test]
fn trashing_the_only_note_leaves_the_editor_empty(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    let only = note_in(&store, &inbox, "Only", 1);
    let (handle, view) = open_view(cx, store);

    right_click(cx, handle, &format!("note-{only}"));
    choose_menu_item(cx, handle, 2);

    assert_eq!(listed_titles(cx, &view), no_titles());
    assert_eq!(selected_title(cx, &view), None);
    assert!(is_present(cx, handle, "empty-new-note"));
    assert!(is_present(cx, handle, "no-notes"));
}

#[gpui_kit::test]
fn trashing_another_note_keeps_the_selection(cx: &mut TestAppContext) {
    let (store, [_, zebra, _]) = three_notes();
    let (handle, view) = open_view(cx, store);

    right_click(cx, handle, &format!("note-{zebra}"));
    choose_menu_item(cx, handle, 2);

    assert_eq!(listed_titles(cx, &view), ["Mango", "Apple"]);
    assert_eq!(selected_title(cx, &view).as_deref(), Some("Mango"));
}

#[gpui_kit::test]
fn trashing_saves_the_pending_edit_of_the_selected_note_first(cx: &mut TestAppContext) {
    let (store, [mango, _, _]) = three_notes();
    let (handle, view) = open_view(cx, store);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("title", cx);
        window.input("!", cx);
    })
    .unwrap();

    right_click(cx, handle, &format!("note-{mango}"));
    choose_menu_item(cx, handle, 2);

    let stored = cx.read(|cx| view.read(cx).store.find_note(&mango).unwrap().unwrap());
    assert_eq!(stored.title, "Mango!");
    assert!(stored.trashed_at.is_some());
}

#[gpui_kit::test]
fn the_trash_context_menu_restores_a_note_out_of_the_trash_list(cx: &mut TestAppContext) {
    let (store, [_, zebra, _]) = three_notes();
    store.trash_note(&zebra, Timestamp::now()).unwrap();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "nav-trash");
    assert_eq!(listed_titles(cx, &view), ["Zebra"]);

    right_click(cx, handle, &format!("note-{zebra}"));
    choose_menu_item(cx, handle, 0);

    assert_eq!(listed_titles(cx, &view), no_titles());
    click(cx, handle, "nav-all-notes");
    assert_eq!(listed_titles(cx, &view).len(), 3);
}

#[gpui_kit::test]
fn the_trash_context_menu_offers_only_restore(cx: &mut TestAppContext) {
    let (store, [_, zebra, _]) = three_notes();
    store.trash_note(&zebra, Timestamp::now()).unwrap();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "nav-trash");

    right_click(cx, handle, &format!("note-{zebra}"));

    assert_eq!(menu_labels(cx, &view), ["Restore"]);
}

#[gpui_kit::test]
fn the_list_context_menu_offers_pin_duplicate_and_trash(cx: &mut TestAppContext) {
    let (store, [mango, _, _]) = three_notes();
    store.pin_note(&mango).unwrap();
    let (handle, view) = open_view(cx, store);

    right_click(cx, handle, &format!("note-{mango}"));

    assert_eq!(
        menu_labels(cx, &view),
        ["Unpin", "Duplicate", "Move to Trash"]
    );
}

#[gpui_kit::test]
fn the_trash_header_swaps_compose_for_empty_trash(cx: &mut TestAppContext) {
    let (store, _) = three_notes();
    let (handle, _) = open_view(cx, store);
    assert!(is_present(cx, handle, "new-note"));
    assert!(!is_present(cx, handle, "empty-trash"));

    click(cx, handle, "nav-trash");

    assert!(!is_present(cx, handle, "new-note"));
    assert!(is_present(cx, handle, "empty-trash"));
}

#[gpui_kit::test]
fn emptying_the_trash_asks_for_confirmation_before_deleting(cx: &mut TestAppContext) {
    let (store, [mango, zebra, _]) = three_notes();
    store.trash_note(&mango, Timestamp::now()).unwrap();
    store.trash_note(&zebra, Timestamp::now()).unwrap();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "nav-trash");

    click(cx, handle, "empty-trash");
    cx.run_until_parked();
    assert_eq!(listed_titles(cx, &view).len(), 2);

    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.press("enter", cx);
    })
    .unwrap();
    cx.run_until_parked();
    assert_eq!(listed_titles(cx, &view), no_titles());
}

#[gpui_kit::test]
fn cancelling_the_confirmation_keeps_the_trash(cx: &mut TestAppContext) {
    let (store, [mango, _, _]) = three_notes();
    store.trash_note(&mango, Timestamp::now()).unwrap();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "nav-trash");

    click(cx, handle, "empty-trash");
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.press("escape", cx);
    })
    .unwrap();
    cx.run_until_parked();

    assert_eq!(listed_titles(cx, &view), ["Mango"]);
}

#[gpui_kit::test]
fn emptying_the_trash_clears_a_selected_note_that_was_purged(cx: &mut TestAppContext) {
    let (store, [mango, _, _]) = three_notes();
    store.trash_note(&mango, Timestamp::now()).unwrap();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "nav-trash");
    click(cx, handle, &format!("note-{mango}"));
    assert_eq!(selected_title(cx, &view).as_deref(), Some("Mango"));

    cx.update(|cx| view.update(cx, Workspace::empty_trash));

    assert_eq!(selected_title(cx, &view), None);
}

#[gpui_kit::test]
fn emptying_the_trash_keeps_a_selected_note_that_is_not_trashed(cx: &mut TestAppContext) {
    let (store, _) = three_notes();
    let (_, view) = open_view(cx, store);

    cx.update(|cx| view.update(cx, Workspace::empty_trash));

    assert_eq!(selected_title(cx, &view).as_deref(), Some("Mango"));
}

#[gpui_kit::test]
fn clicking_a_tag_chip_lists_that_tag_without_changing_the_selection(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    let selected = note_in(&store, &inbox, "Selected", 3);
    let tagged = note_in(&store, &inbox, "Tagged", 2);
    note_in(&store, &inbox, "Plain", 1);
    let tag = store.create_tag("idea", TagColor::Red, at(4)).unwrap();
    store.add_tag_to_note(&tagged, &tag.id, at(5)).unwrap();
    let (handle, view) = open_view(cx, store);
    assert_eq!(selected_title(cx, &view).as_deref(), Some("Tagged"));
    click(cx, handle, &format!("note-{selected}"));

    click(cx, handle, &format!("chip-{tagged}-{}", tag.id));

    assert_eq!(listed_titles(cx, &view), ["Tagged"]);
    cx.read(|cx| {
        let view = view.read(cx);
        assert_eq!(view.listing.navigation, Navigation::Tags(vec![tag.id]));
        assert_eq!(view.selected, Some(selected));
    });
}

#[gpui_kit::test]
fn an_empty_list_says_there_are_no_notes(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
    let (handle, _) = open_view(cx, store);
    assert!(is_present(cx, handle, "no-notes"));
}

#[gpui_kit::test]
fn a_list_with_notes_does_not_show_the_empty_message(cx: &mut TestAppContext) {
    let (store, _) = three_notes();
    let (handle, _) = open_view(cx, store);
    assert!(!is_present(cx, handle, "no-notes"));
}

#[gpui_kit::test]
fn a_failing_change_shows_the_error_banner(cx: &mut TestAppContext) {
    let (store, _) = three_notes();
    let (handle, view) = open_view(cx, store);

    cx.update(|cx| {
        view.update(cx, |workspace, cx| {
            workspace.toggle_pin(&NoteId::generate(), cx);
        });
    });

    assert!(is_present(cx, handle, "error-banner"));
}

#[gpui_kit::test]
fn changing_notes_refreshes_the_sidebar_counts(cx: &mut TestAppContext) {
    let (store, [mango, _, _]) = three_notes();
    let (handle, view) = open_view(cx, store);
    let count = |cx: &mut TestAppContext| {
        cx.read(|cx| {
            view.read(cx)
                .notebooks
                .iter()
                .map(|node| node.note_count)
                .sum::<usize>()
        })
    };
    assert_eq!(count(cx), 3);

    right_click(cx, handle, &format!("note-{mango}"));
    choose_menu_item(cx, handle, 2);

    assert_eq!(count(cx), 2);
}

#[gpui_kit::test]
fn the_filter_keeps_only_notes_matching_titles_or_bodies_and_empty_restores(
    cx: &mut TestAppContext,
) {
    let (store, [mango, _, _]) = three_notes();
    store
        .save_note(&mango, "Mango", "ripe fruit", at(11))
        .unwrap();
    let (handle, view) = open_view(cx, store);
    assert_eq!(listed_titles(cx, &view).len(), 3);

    type_filter(cx, handle, "fruit");
    assert_eq!(listed_titles(cx, &view), ["Mango"]);

    press_escape_in_filter(cx, handle);
    type_filter(cx, handle, "zebra");
    assert_eq!(listed_titles(cx, &view), ["Zebra"]);

    press_escape_in_filter(cx, handle);
    assert_eq!(listed_titles(cx, &view).len(), 3);
}

#[gpui_kit::test]
fn escape_in_the_filter_clears_it(cx: &mut TestAppContext) {
    let (store, _) = three_notes();
    let (handle, view) = open_view(cx, store);
    type_filter(cx, handle, "apple");
    assert_eq!(listed_titles(cx, &view), ["Apple"]);

    press_escape_in_filter(cx, handle);

    assert_eq!(listed_titles(cx, &view).len(), 3);
}

fn type_filter(cx: &mut TestAppContext, handle: AnyWindowHandle, text: &str) {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("note-filter-input", cx);
        window.input(text, cx);
        window.render_frame(cx);
    })
    .unwrap();
}

fn press_escape_in_filter(cx: &mut TestAppContext, handle: AnyWindowHandle) {
    cx.update_window(handle, |_, window, cx| {
        window.click("note-filter-input", cx);
        window.press("escape", cx);
        window.render_frame(cx);
    })
    .unwrap();
}

fn list_has_focus(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    view: &Entity<Workspace>,
) -> bool {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        view.read(cx).note_list_focus.contains_focused(window, cx)
    })
    .unwrap()
}

fn sidebar_has_focus(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    view: &Entity<Workspace>,
) -> bool {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        view.read(cx).sidebar_focus.contains_focused(window, cx)
    })
    .unwrap()
}

fn press_key(cx: &mut TestAppContext, handle: AnyWindowHandle, key: &str) {
    cx.update_window(handle, |_, window, cx| window.press(key, cx))
        .unwrap();
    cx.run_until_parked();
}

#[gpui_kit::test]
fn clicking_a_sidebar_row_focuses_the_sidebar_and_clicking_a_note_focuses_the_list(
    cx: &mut TestAppContext,
) {
    let (store, [mango, _, _]) = three_notes();
    let (handle, view) = open_view(cx, store);
    assert!(!sidebar_has_focus(cx, handle, &view));
    assert!(!list_has_focus(cx, handle, &view));

    click(cx, handle, "nav-all-notes");
    assert!(sidebar_has_focus(cx, handle, &view));
    assert!(!list_has_focus(cx, handle, &view));

    click(cx, handle, &format!("note-{mango}"));
    assert!(list_has_focus(cx, handle, &view));
    assert!(!sidebar_has_focus(cx, handle, &view));
}

#[gpui_kit::test]
fn arrow_keys_open_the_neighbouring_notes_while_the_list_keeps_the_focus(cx: &mut TestAppContext) {
    let (store, [mango, zebra, apple]) = three_notes();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, &format!("note-{mango}"));
    assert_eq!(listed_titles(cx, &view), ["Mango", "Apple", "Zebra"]);

    press_key(cx, handle, "down");
    assert_eq!(selected_title(cx, &view).as_deref(), Some("Apple"));
    assert_eq!(cx.read(|cx| view.read(cx).selected.clone()), Some(apple));
    assert!(list_has_focus(cx, handle, &view));

    press_key(cx, handle, "down");
    assert_eq!(cx.read(|cx| view.read(cx).selected.clone()), Some(zebra));
    assert_eq!(cx.read(|cx| view.read(cx).editor.title(cx)), "Zebra");
    assert!(list_has_focus(cx, handle, &view));

    press_key(cx, handle, "down");
    assert_eq!(selected_title(cx, &view).as_deref(), Some("Zebra"));

    press_key(cx, handle, "up");
    press_key(cx, handle, "up");
    assert_eq!(cx.read(|cx| view.read(cx).selected.clone()), Some(mango));
    assert!(list_has_focus(cx, handle, &view));
}

#[gpui_kit::test]
fn arrow_keys_outside_the_list_do_not_change_the_open_note(cx: &mut TestAppContext) {
    let (store, [mango, _, _]) = three_notes();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "nav-all-notes");

    press_key(cx, handle, "down");

    assert_eq!(cx.read(|cx| view.read(cx).selected.clone()), Some(mango));
}

#[gpui_kit::test]
fn opening_a_note_below_the_fold_scrolls_it_into_view(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    let oldest = note_in(&store, &inbox, "Note 0", 0);
    for index in 1..60 {
        note_in(&store, &inbox, &format!("Note {index}"), index);
    }
    let (handle, view) = open_view(cx, store);
    cx.update_window(handle, |_, window, cx| window.render_frame(cx))
        .unwrap();
    let before = view.read_with(cx, |view, _| {
        view.note_list_state.logical_scroll_top().item_ix
    });
    assert_eq!(before, 0);

    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |view, cx| view.select(&oldest, window, cx));
        window.render_frame(cx);
        window.render_frame(cx);
    })
    .unwrap();
    cx.run_until_parked();

    let after = view.read_with(cx, |view, _| {
        view.note_list_state.logical_scroll_top().item_ix
    });
    assert!(after > 0);
}
