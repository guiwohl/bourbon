use bourbon_core::{NoteId, NoteStatus, NoteStore};
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Entity, Point, TestAppContext, VisualTestContext, px,
};

use super::{at, click, note_in, open_view, seeded_store};
use crate::control_pane::{Mode, PaneAction};
use crate::navigation::Navigation;
use crate::workspace::Workspace;

fn press(cx: &mut TestAppContext, handle: AnyWindowHandle, keystrokes: &str) {
    cx.update_window(handle, |_, window, cx| window.render_frame(cx))
        .unwrap();
    VisualTestContext::from_window(handle, cx).simulate_keystrokes(keystrokes);
}

fn type_text(cx: &mut TestAppContext, handle: AnyWindowHandle, text: &str) {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.input(text, cx);
    })
    .unwrap();
}

fn is_open(cx: &mut TestAppContext, view: &Entity<Workspace>) -> bool {
    view.read_with(cx, |view, _| view.control_pane.is_some())
}

fn mode(cx: &mut TestAppContext, view: &Entity<Workspace>) -> Option<Mode> {
    view.read_with(cx, |view, _| {
        view.control_pane
            .as_ref()
            .map(crate::control_pane::ControlPane::mode)
    })
}

fn found_titles(cx: &mut TestAppContext, view: &Entity<Workspace>) -> Vec<String> {
    view.read_with(cx, |view, _| {
        view.control_pane
            .as_ref()
            .map(crate::control_pane::ControlPane::note_titles)
            .unwrap_or_default()
    })
}

fn offered_actions(cx: &mut TestAppContext, view: &Entity<Workspace>) -> Vec<PaneAction> {
    view.read_with(cx, |view, _| {
        view.control_pane
            .as_ref()
            .map(crate::control_pane::ControlPane::actions)
            .unwrap_or_default()
    })
}

fn highlighted(cx: &mut TestAppContext, view: &Entity<Workspace>) -> usize {
    view.read_with(cx, |view, _| {
        view.control_pane
            .as_ref()
            .map_or(usize::MAX, crate::control_pane::ControlPane::highlighted)
    })
}

fn selected(cx: &mut TestAppContext, view: &Entity<Workspace>) -> Option<NoteId> {
    view.read_with(cx, |view, _| view.selected.clone())
}

fn navigation(cx: &mut TestAppContext, view: &Entity<Workspace>) -> Navigation {
    view.read_with(cx, |view, _| view.listing.navigation.clone())
}

fn stored_status(cx: &mut TestAppContext, view: &Entity<Workspace>, id: &NoteId) -> NoteStatus {
    view.read_with(cx, |view, _| {
        view.store.find_note(id).unwrap().unwrap().status
    })
}

fn run_action(cx: &mut TestAppContext, handle: AnyWindowHandle, query: &str) {
    press(cx, handle, "ctrl-p");
    type_text(cx, handle, query);
    press(cx, handle, "enter");
}

#[gpui_kit::test]
fn ctrl_o_opens_the_pane_in_notes_mode_listing_recent_notes(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    press(cx, handle, "ctrl-o");

    assert_eq!(mode(cx, &view), Some(Mode::Notes));
    assert_eq!(found_titles(cx, &view), ["Newer", "Older"]);
    assert_eq!(highlighted(cx, &view), 0);
}

#[gpui_kit::test]
fn ctrl_p_opens_the_pane_in_actions_mode_prefilled(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    press(cx, handle, "ctrl-p");

    assert_eq!(mode(cx, &view), Some(Mode::Actions));
    assert!(offered_actions(cx, &view).contains(&PaneAction::NewNote));
}

#[gpui_kit::test]
fn typing_searches_the_note_text(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    press(cx, handle, "ctrl-o");
    type_text(cx, handle, "older bo");

    assert_eq!(found_titles(cx, &view), ["Older"]);
}

#[gpui_kit::test]
fn deleting_the_leading_angle_bracket_switches_to_notes(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    press(cx, handle, "ctrl-p");
    press(cx, handle, "backspace");

    assert_eq!(mode(cx, &view), Some(Mode::Notes));
    assert_eq!(found_titles(cx, &view), ["Newer", "Older"]);
}

#[gpui_kit::test]
fn typing_an_angle_bracket_first_switches_to_actions(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    press(cx, handle, "ctrl-o");
    type_text(cx, handle, ">trash");

    assert_eq!(mode(cx, &view), Some(Mode::Actions));
    assert!(offered_actions(cx, &view).contains(&PaneAction::GoToTrash));
}

#[gpui_kit::test]
fn down_and_up_move_the_highlight_and_stop_at_the_ends(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    press(cx, handle, "ctrl-o");

    press(cx, handle, "up");
    assert_eq!(highlighted(cx, &view), 0);
    press(cx, handle, "down");
    assert_eq!(highlighted(cx, &view), 1);
    press(cx, handle, "down");
    assert_eq!(highlighted(cx, &view), 1);
    press(cx, handle, "up");
    assert_eq!(highlighted(cx, &view), 0);
}

#[gpui_kit::test]
fn changing_the_query_resets_the_highlight(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    press(cx, handle, "ctrl-o");
    press(cx, handle, "down");

    type_text(cx, handle, "body");

    assert_eq!(highlighted(cx, &view), 0);
}

#[gpui_kit::test]
fn enter_opens_the_highlighted_note_and_closes_the_pane(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    press(cx, handle, "ctrl-o");
    press(cx, handle, "down");
    press(cx, handle, "enter");

    assert!(!is_open(cx, &view));
    assert_eq!(selected(cx, &view), Some(seeded.older));
}

#[gpui_kit::test]
fn clicking_a_row_opens_that_note(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    press(cx, handle, "ctrl-o");

    click(cx, handle, "pane-row-1");

    assert!(!is_open(cx, &view));
    assert_eq!(selected(cx, &view), Some(seeded.older));
}

#[gpui_kit::test]
fn escape_closes_the_pane_without_opening_anything(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    press(cx, handle, "ctrl-o");
    press(cx, handle, "down");

    press(cx, handle, "escape");

    assert!(!is_open(cx, &view));
    assert_eq!(selected(cx, &view), Some(seeded.newer));
}

#[gpui_kit::test]
fn clicking_outside_the_panel_closes_it(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    press(cx, handle, "ctrl-o");

    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click_at("pane-backdrop", Point::new(px(5.), px(5.)), cx);
    })
    .unwrap();

    assert!(!is_open(cx, &view));
}

#[gpui_kit::test]
fn clicking_inside_the_panel_keeps_it_open(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    press(cx, handle, "ctrl-o");

    click(cx, handle, "pane-input");

    assert!(is_open(cx, &view));
}

#[gpui_kit::test]
fn opening_a_note_outside_the_listing_switches_to_all_notes(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    let work = store.create_notebook("Work", None, at(1)).unwrap();
    note_in(&store, &inbox, "Inbox note", 2);
    let hidden = note_in(&store, &work.id, "Work note", 3);
    let (handle, view) = open_view(cx, store);
    click(cx, handle, &format!("nav-notebook-{inbox}"));
    assert_eq!(found_count(cx, &view), 1);

    press(cx, handle, "ctrl-o");
    type_text(cx, handle, "work");
    press(cx, handle, "enter");

    assert_eq!(navigation(cx, &view), Navigation::AllNotes);
    assert_eq!(selected(cx, &view), Some(hidden));
}

fn found_count(cx: &mut TestAppContext, view: &Entity<Workspace>) -> usize {
    view.read_with(cx, |view, _| view.notes.len())
}

#[gpui_kit::test]
fn opening_a_note_inside_the_listing_keeps_the_navigation(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    note_in(&store, &inbox, "First", 2);
    let second = note_in(&store, &inbox, "Second", 3);
    let (handle, view) = open_view(cx, store);
    click(cx, handle, &format!("nav-notebook-{inbox}"));

    press(cx, handle, "ctrl-o");
    type_text(cx, handle, "first");
    press(cx, handle, "enter");

    assert_ne!(selected(cx, &view), Some(second));
    assert!(matches!(navigation(cx, &view), Navigation::Notebook(_, _)));
}

#[gpui_kit::test]
fn opening_a_note_saves_the_pending_edit_first(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("title", cx);
        window.input("!", cx);
    })
    .unwrap();

    press(cx, handle, "ctrl-o");
    type_text(cx, handle, "older");
    press(cx, handle, "enter");

    let title = view.read_with(cx, |view, _| {
        view.store.find_note(&seeded.newer).unwrap().unwrap().title
    });
    assert_eq!(title, "Newer!");
    assert_eq!(selected(cx, &view), Some(seeded.older));
}

#[gpui_kit::test]
fn an_action_filter_matches_a_subsequence_of_the_name(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    press(cx, handle, "ctrl-p");
    type_text(cx, handle, "gtt");

    assert_eq!(
        offered_actions(cx, &view),
        [
            PaneAction::GoToTemplates,
            PaneAction::GoToTrash,
            PaneAction::GoToAllNotes
        ]
    );
}

#[gpui_kit::test]
fn actions_needing_a_note_are_hidden_without_a_selection(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
    let (handle, view) = open_view(cx, store);

    press(cx, handle, "ctrl-p");

    let offered = offered_actions(cx, &view);
    assert!(offered.contains(&PaneAction::NewNote));
    assert!(!offered.contains(&PaneAction::DuplicateNote));
    assert!(!offered.contains(&PaneAction::MoveToTrash));
    assert!(!offered.contains(&PaneAction::SetStatus(NoteStatus::Active)));
}

#[gpui_kit::test]
fn the_new_note_action_creates_a_note(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    run_action(cx, handle, "new note");

    assert!(!is_open(cx, &view));
    let current = selected(cx, &view).unwrap();
    assert_ne!(current, seeded.newer);
    assert_ne!(current, seeded.older);
}

#[gpui_kit::test]
fn the_toggle_sidebar_action_hides_the_sidebar(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    run_action(cx, handle, "toggle sidebar");

    assert!(!view.read_with(cx, |view, _| view.sidebar.is_visible));
}

#[gpui_kit::test]
fn the_go_to_trash_action_navigates_to_trash(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    run_action(cx, handle, "go to trash");

    assert_eq!(navigation(cx, &view), Navigation::Trash);
}

#[gpui_kit::test]
fn the_go_to_templates_action_navigates_to_templates(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    run_action(cx, handle, "go to templates");

    assert_eq!(navigation(cx, &view), Navigation::Templates);
}

#[gpui_kit::test]
fn the_go_to_all_notes_action_leaves_the_trash(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "nav-trash");

    run_action(cx, handle, "go to all");

    assert_eq!(navigation(cx, &view), Navigation::AllNotes);
}

#[gpui_kit::test]
fn the_sort_action_changes_the_listing_order(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    run_action(cx, handle, "sort by title");

    let titles: Vec<String> = view.read_with(cx, |view, _| {
        view.notes.iter().map(|note| note.title.clone()).collect()
    });
    assert_eq!(titles, ["Newer", "Older"]);
    assert_eq!(
        view.read_with(cx, |view, _| view.listing.sort),
        bourbon_core::NoteSort::TitleAlphabetical
    );
}

#[gpui_kit::test]
fn the_set_status_action_changes_the_open_note(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    run_action(cx, handle, "set status on hold");

    assert_eq!(stored_status(cx, &view, &seeded.newer), NoteStatus::OnHold);
}

#[gpui_kit::test]
fn the_duplicate_action_copies_and_opens_the_note(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    run_action(cx, handle, "duplicate");

    let current = selected(cx, &view).unwrap();
    assert_ne!(current, seeded.newer);
    assert_eq!(found_count(cx, &view), 3);
}

#[gpui_kit::test]
fn the_pin_action_pins_the_open_note(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    run_action(cx, handle, "pin");

    let pinned = view.read_with(cx, |view, _| {
        view.store.find_note(&seeded.newer).unwrap().unwrap().pinned
    });
    assert!(pinned);
}

#[gpui_kit::test]
fn the_trash_action_trashes_the_open_note(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    run_action(cx, handle, "move to trash");

    let trashed = view.read_with(cx, |view, _| {
        view.store
            .find_note(&seeded.newer)
            .unwrap()
            .unwrap()
            .trashed_at
            .is_some()
    });
    assert!(trashed);
    assert_eq!(selected(cx, &view), Some(seeded.older));
}

#[gpui_kit::test]
fn the_move_to_notebook_action_opens_the_notebook_selector(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    run_action(cx, handle, "move to notebook");

    assert!(view.read_with(cx, |view, _| view.editor.notebook_selector_is_open()));
}

#[gpui_kit::test]
fn the_empty_trash_action_shows_the_trash_and_asks_for_confirmation(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    store.trash_note(&seeded.older, at(9)).unwrap();
    let (handle, view) = open_view(cx, store);

    run_action(cx, handle, "empty trash");
    press(cx, handle, "enter");

    assert_eq!(navigation(cx, &view), Navigation::Trash);
    assert_eq!(found_count(cx, &view), 0);
}

#[gpui_kit::test]
fn the_history_back_action_returns_to_the_previous_note(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, &format!("note-{}", seeded.older));

    run_action(cx, handle, "history back");

    assert_eq!(selected(cx, &view), Some(seeded.newer));
}

#[gpui_kit::test]
fn the_history_forward_action_returns_to_the_next_note(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, &format!("note-{}", seeded.older));
    press(cx, handle, "alt-left");

    run_action(cx, handle, "history forward");

    assert_eq!(selected(cx, &view), Some(seeded.older));
}

#[gpui_kit::test]
fn clicking_an_action_row_runs_it(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    press(cx, handle, "ctrl-p");
    type_text(cx, handle, "go to trash");

    click(cx, handle, "pane-row-0");

    assert_eq!(navigation(cx, &view), Navigation::Trash);
    assert!(!is_open(cx, &view));
}

#[gpui_kit::test]
fn a_search_that_matches_nothing_has_no_rows_and_enter_does_nothing(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    press(cx, handle, "ctrl-o");
    type_text(cx, handle, "zzzzqqq");

    press(cx, handle, "enter");

    assert_eq!(found_titles(cx, &view), Vec::<String>::new());
    assert_eq!(selected(cx, &view), Some(seeded.newer));
}

#[gpui_kit::test]
fn the_pane_works_while_the_editor_has_focus(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("title", cx);
    })
    .unwrap();

    press(cx, handle, "ctrl-o");

    assert!(is_open(cx, &view));
}
