use bourbon_core::{NoteId, NoteStatus, NoteStore, NotebookId, TagColor};
use gpui::{AnyWindowHandle, Entity, TestAppContext};

use super::{at, click, note_in, open_view, seeded_store, type_text};
use crate::editor::History;
use crate::workspace::Workspace;

fn press(cx: &mut TestAppContext, handle: AnyWindowHandle, keystrokes: &str) {
    super::press(cx, handle, keystrokes);
}

fn type_into_tag_input(cx: &mut TestAppContext, handle: AnyWindowHandle, text: &str) {
    click(cx, handle, "tag-input");
    type_text(cx, handle, text);
}

fn press_in_window(cx: &mut TestAppContext, handle: AnyWindowHandle, key: &str) {
    press(cx, handle, key);
}

fn is_rendered(cx: &mut TestAppContext, handle: AnyWindowHandle, id: &str) -> bool {
    super::is_listed(cx, handle, id)
}

fn stored_note(
    cx: &mut TestAppContext,
    view: &Entity<Workspace>,
    id: &NoteId,
) -> bourbon_core::Note {
    cx.read(|cx| view.read(cx).store.find_note(id).unwrap().unwrap())
}

fn work_notebook(store: &NoteStore) -> NotebookId {
    store
        .create_notebook("Work", Some(&store.default_notebook().unwrap()), at(5))
        .unwrap()
        .id
}

fn note_ids_after_opening(cx: &mut TestAppContext, view: &Entity<Workspace>) -> Vec<NoteId> {
    cx.read(|cx| view.read(cx).notes.iter().map(|n| n.id.clone()).collect())
}

fn choose_notebook(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    view: &Entity<Workspace>,
    name: &str,
) {
    click(cx, handle, "notebook-selector");
    let position = super::menu_labels(cx, view)
        .iter()
        .position(|label| label.trim() == name)
        .unwrap();
    super::choose_menu_entry(cx, handle, position);
}

#[gpui::test]
fn choosing_a_notebook_in_the_selector_moves_the_note(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let work = work_notebook(&store);
    let (handle, view) = open_view(cx, store);

    choose_notebook(cx, handle, &view, "Work");

    assert_eq!(stored_note(cx, &view, &seeded.newer).notebook, work);
    cx.read(|cx| assert_eq!(view.read(cx).selected, Some(seeded.newer.clone())));
    assert_eq!(super::menu_labels(cx, &view), Vec::<String>::new());
}

#[gpui::test]
fn moving_a_note_flushes_the_pending_edit_first(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let work = work_notebook(&store);
    let (handle, view) = open_view(cx, store);
    super::type_into_title(cx, handle, "!");

    choose_notebook(cx, handle, &view, "Work");

    let note = stored_note(cx, &view, &seeded.newer);
    assert_eq!(note.title, "Newer!");
    assert_eq!(note.notebook, work);
}

#[gpui::test]
fn moving_a_note_refreshes_the_sidebar_counts(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    work_notebook(&store);
    let (handle, view) = open_view(cx, store);

    choose_notebook(cx, handle, &view, "Work");

    cx.read(|cx| {
        let inbox = &view.read(cx).notebooks[0];
        assert_eq!(inbox.note_count, 2);
        assert_eq!(inbox.children[0].note_count, 1);
    });
}

#[gpui::test]
fn the_selector_lists_every_notebook_indented_by_depth(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    work_notebook(&store);
    let (handle, view) = open_view(cx, store);

    assert_eq!(super::menu_labels(cx, &view), Vec::<String>::new());
    click(cx, handle, "notebook-selector");

    assert_eq!(super::menu_labels(cx, &view), ["Inbox", "    Work"]);
}

#[gpui::test]
fn the_selector_opens_just_below_its_button(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    click(cx, handle, "notebook-selector");

    let button = super::bounds(cx, handle, "notebook-selector").unwrap();
    let origin = cx.read(|cx| view.read(cx).last_menu_position());
    assert_eq!(origin.x, button.left());
    assert_eq!(origin.y, button.bottom() + gpui::px(4.));
}

#[gpui::test]
fn ctrl_alt_m_opens_the_notebook_selector(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    work_notebook(&store);
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "title");

    press(cx, handle, "ctrl-alt-m");

    assert_eq!(super::menu_labels(cx, &view), ["Inbox", "    Work"]);
}

#[gpui::test]
fn ctrl_alt_m_without_an_open_note_does_nothing(cx: &mut TestAppContext) {
    let (handle, view) = open_view(cx, NoteStore::open_in_memory().unwrap());

    press(cx, handle, "ctrl-alt-m");

    assert_eq!(super::menu_labels(cx, &view), Vec::<String>::new());
}

fn choose_status(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    view: &Entity<Workspace>,
    index: usize,
) {
    click(cx, handle, "status-button");
    assert_eq!(super::menu_labels(cx, view).len(), 5);
    super::choose_menu_entry(cx, handle, index);
}

#[gpui::test]
fn choosing_a_status_in_the_status_button_sets_it(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    choose_status(cx, handle, &view, 1);

    assert_eq!(
        stored_note(cx, &view, &seeded.newer).status,
        NoteStatus::Active
    );
}

#[gpui::test]
fn every_status_option_sets_its_status(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    for (index, status) in NoteStatus::ALL.into_iter().enumerate().rev() {
        choose_status(cx, handle, &view, index);
        assert_eq!(stored_note(cx, &view, &seeded.newer).status, status);
    }
}

#[gpui::test]
fn shift_alt_digits_set_the_status(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "title");

    for (index, status) in NoteStatus::ALL.into_iter().enumerate() {
        let other = NoteStatus::ALL[(index + 1) % NoteStatus::ALL.len()];
        view.update(cx, |view, _| {
            view.store.set_status(&seeded.newer, other, at(9)).unwrap();
        });
        press(cx, handle, &format!("shift-alt-{}", index + 1));
        assert_eq!(stored_note(cx, &view, &seeded.newer).status, status);
    }
}

#[gpui::test]
fn shift_alt_4_completes_the_note_and_keeps_it_open(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "title");

    press(cx, handle, "shift-alt-4");

    assert_eq!(
        stored_note(cx, &view, &seeded.newer).status,
        NoteStatus::Completed
    );
    cx.read(|cx| assert_eq!(view.read(cx).selected, Some(seeded.newer.clone())));
    assert!(!note_ids_after_opening(cx, &view).contains(&seeded.newer));
}

#[gpui::test]
fn changing_the_status_flushes_the_pending_edit_first(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    super::type_into_title(cx, handle, "!");

    press(cx, handle, "shift-alt-2");

    let note = stored_note(cx, &view, &seeded.newer);
    assert_eq!(note.title, "Newer!");
    assert_eq!(note.status, NoteStatus::Active);
}

fn store_with_tags() -> (NoteStore, NoteId, bourbon_core::Tag, bourbon_core::Tag) {
    let (store, seeded) = seeded_store();
    let rust = store.create_tag("rust", TagColor::Default, at(5)).unwrap();
    let ruby = store.create_tag("ruby", TagColor::Default, at(6)).unwrap();
    store
        .add_tag_to_note(&seeded.newer, &ruby.id, at(7))
        .unwrap();
    (store, seeded.newer, rust, ruby)
}

fn tag_names(cx: &mut TestAppContext, view: &Entity<Workspace>, id: &NoteId) -> Vec<String> {
    stored_note(cx, view, id)
        .tags
        .into_iter()
        .map(|tag| tag.name)
        .collect()
}

#[gpui::test]
fn the_header_shows_a_chip_for_each_tag_of_the_note(cx: &mut TestAppContext) {
    let (store, _, rust, ruby) = store_with_tags();
    let (handle, _) = open_view(cx, store);

    assert!(is_rendered(cx, handle, &format!("tag-chip-{}", ruby.id)));
    assert!(!is_rendered(cx, handle, &format!("tag-chip-{}", rust.id)));
}

#[gpui::test]
fn typing_offers_existing_tags_not_yet_on_the_note(cx: &mut TestAppContext) {
    let (store, _, rust, ruby) = store_with_tags();
    let (handle, _) = open_view(cx, store);

    type_into_tag_input(cx, handle, "ru");

    assert!(is_rendered(
        cx,
        handle,
        &format!("tag-suggestion-{}", rust.id)
    ));
    assert!(!is_rendered(
        cx,
        handle,
        &format!("tag-suggestion-{}", ruby.id)
    ));
}

#[gpui::test]
fn picking_a_suggestion_with_the_keyboard_adds_the_existing_tag(cx: &mut TestAppContext) {
    let (store, note, rust, _) = store_with_tags();
    let (handle, view) = open_view(cx, store);

    type_into_tag_input(cx, handle, "ru");
    press_in_window(cx, handle, "down");
    press_in_window(cx, handle, "enter");

    assert_eq!(tag_names(cx, &view, &note), ["ruby", "rust"]);
    cx.read(|cx| {
        assert_eq!(view.read(cx).tags.len(), 2);
        assert_eq!(view.read(cx).editor.tag_input_text(cx), "");
    });
    assert!(is_rendered(cx, handle, &format!("tag-chip-{}", rust.id)));
    assert!(!is_rendered(
        cx,
        handle,
        &format!("tag-suggestion-{}", rust.id)
    ));
}

#[gpui::test]
fn clicking_a_suggestion_adds_the_existing_tag(cx: &mut TestAppContext) {
    let (store, note, rust, _) = store_with_tags();
    let (handle, view) = open_view(cx, store);

    type_into_tag_input(cx, handle, "ru");
    click(cx, handle, &format!("tag-suggestion-{}", rust.id));

    assert_eq!(tag_names(cx, &view, &note), ["ruby", "rust"]);
}

fn store_with_rust_and_rustacean() -> (NoteStore, NoteId) {
    let (store, note, _, _) = store_with_tags();
    store
        .create_tag("rustacean", TagColor::Default, at(8))
        .unwrap();
    (store, note)
}

fn highlight_with_keys(cx: &mut TestAppContext, keys: &[&str]) -> Vec<String> {
    let (store, note) = store_with_rust_and_rustacean();
    let (handle, view) = open_view(cx, store);
    type_into_tag_input(cx, handle, "rust");
    for key in keys {
        press_in_window(cx, handle, key);
    }
    press_in_window(cx, handle, "enter");
    tag_names(cx, &view, &note)
}

#[gpui::test]
fn the_first_down_highlights_the_first_suggestion(cx: &mut TestAppContext) {
    assert_eq!(highlight_with_keys(cx, &["down"]), ["ruby", "rust"]);
}

#[gpui::test]
fn a_second_down_highlights_the_next_suggestion(cx: &mut TestAppContext) {
    assert_eq!(
        highlight_with_keys(cx, &["down", "down"]),
        ["ruby", "rustacean"]
    );
}

#[gpui::test]
fn the_highlight_stops_at_the_last_suggestion(cx: &mut TestAppContext) {
    assert_eq!(
        highlight_with_keys(cx, &["down", "down", "down", "down"]),
        ["ruby", "rustacean"]
    );
}

#[gpui::test]
fn up_moves_the_highlight_back_one_suggestion(cx: &mut TestAppContext) {
    assert_eq!(
        highlight_with_keys(cx, &["down", "down", "up"]),
        ["ruby", "rust"]
    );
}

#[gpui::test]
fn up_from_the_first_suggestion_clears_the_highlight(cx: &mut TestAppContext) {
    let (store, note) = store_with_rust_and_rustacean();
    let (handle, view) = open_view(cx, store);
    type_into_tag_input(cx, handle, "rus");
    for key in ["down", "up", "enter"] {
        press_in_window(cx, handle, key);
    }
    assert_eq!(tag_names(cx, &view, &note), ["ruby", "rus"]);
}

#[gpui::test]
fn typing_clears_the_highlight(cx: &mut TestAppContext) {
    let (store, note) = store_with_rust_and_rustacean();
    let (handle, view) = open_view(cx, store);
    type_into_tag_input(cx, handle, "rust");
    press_in_window(cx, handle, "down");
    type_text(cx, handle, "a");
    press_in_window(cx, handle, "enter");
    assert_eq!(tag_names(cx, &view, &note), ["ruby", "rusta"]);
}

#[gpui::test]
fn down_without_suggestions_highlights_nothing(cx: &mut TestAppContext) {
    let (store, note, _, _) = store_with_tags();
    let (handle, view) = open_view(cx, store);
    type_into_tag_input(cx, handle, "zzz");
    press_in_window(cx, handle, "down");
    press_in_window(cx, handle, "enter");
    assert_eq!(tag_names(cx, &view, &note), ["ruby", "zzz"]);
}

#[gpui::test]
fn enter_without_a_highlight_adds_the_tag_named_exactly_as_typed(cx: &mut TestAppContext) {
    let (store, note, _, _) = store_with_tags();
    let (handle, view) = open_view(cx, store);

    type_into_tag_input(cx, handle, "RUST");
    press_in_window(cx, handle, "enter");

    assert_eq!(tag_names(cx, &view, &note), ["ruby", "rust"]);
    cx.read(|cx| assert_eq!(view.read(cx).tags.len(), 2));
}

#[gpui::test]
fn enter_on_an_unknown_name_creates_the_tag_with_the_default_color(cx: &mut TestAppContext) {
    let (store, note, _, _) = store_with_tags();
    let (handle, view) = open_view(cx, store);

    type_into_tag_input(cx, handle, "  garden ");
    press_in_window(cx, handle, "enter");

    let tags = stored_note(cx, &view, &note).tags;
    let created = tags.iter().find(|tag| tag.name == "garden").unwrap();
    assert_eq!(created.color, TagColor::Default);
    cx.read(|cx| {
        let view = view.read(cx);
        assert_eq!(view.tags.len(), 3);
        assert_eq!(view.editor.tag_input_text(cx), "");
    });
    assert!(is_rendered(cx, handle, &format!("tag-chip-{}", created.id)));
}

#[gpui::test]
fn enter_on_an_empty_field_changes_nothing(cx: &mut TestAppContext) {
    let (store, note, _, _) = store_with_tags();
    let (handle, view) = open_view(cx, store);

    type_into_tag_input(cx, handle, "   ");
    press_in_window(cx, handle, "enter");

    assert_eq!(tag_names(cx, &view, &note), ["ruby"]);
    cx.read(|cx| assert_eq!(view.read(cx).tags.len(), 2));
}

#[gpui::test]
fn removing_a_chip_removes_the_tag_from_the_note_only(cx: &mut TestAppContext) {
    let (store, note, _, ruby) = store_with_tags();
    let (handle, view) = open_view(cx, store);

    click(cx, handle, &format!("remove-tag-{}", ruby.id));

    assert_eq!(tag_names(cx, &view, &note), Vec::<String>::new());
    cx.read(|cx| assert_eq!(view.read(cx).tags.len(), 2));
    assert!(!is_rendered(cx, handle, &format!("tag-chip-{}", ruby.id)));
}

#[gpui::test]
fn adding_a_tag_flushes_the_pending_edit_first(cx: &mut TestAppContext) {
    let (store, note, rust, _) = store_with_tags();
    let (handle, view) = open_view(cx, store);
    super::type_into_title(cx, handle, "!");
    type_into_tag_input(cx, handle, "ru");

    click(cx, handle, &format!("tag-suggestion-{}", rust.id));

    assert_eq!(stored_note(cx, &view, &note).title, "Newer!");
}

#[gpui::test]
fn a_header_action_failure_is_reported_in_the_banner(cx: &mut TestAppContext) {
    let (store, _, rust, _) = store_with_tags();
    let (handle, view) = open_view(cx, store);
    type_into_tag_input(cx, handle, "ru");
    view.update(cx, |view, _| view.store.delete_tag(&rust.id).unwrap());

    click(cx, handle, &format!("tag-suggestion-{}", rust.id));

    assert!(is_rendered(cx, handle, "error-banner"));
}

struct Three {
    oldest: NoteId,
    middle: NoteId,
    newest: NoteId,
}

fn three_notes() -> (NoteStore, Three) {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    let oldest = note_in(&store, &inbox, "Oldest", 1);
    let middle = note_in(&store, &inbox, "Middle", 2);
    let newest = note_in(&store, &inbox, "Newest", 3);
    (
        store,
        Three {
            oldest,
            middle,
            newest,
        },
    )
}

fn selected(cx: &mut TestAppContext, view: &Entity<Workspace>) -> NoteId {
    cx.read(|cx| view.read(cx).selected.clone().unwrap())
}

fn visit_middle_then_oldest(cx: &mut TestAppContext, handle: AnyWindowHandle, notes: &Three) {
    click(cx, handle, &format!("note-{}", notes.middle));
    click(cx, handle, &format!("note-{}", notes.oldest));
}

#[gpui::test]
fn alt_left_and_alt_right_walk_the_viewed_notes(cx: &mut TestAppContext) {
    let (store, notes) = three_notes();
    let (handle, view) = open_view(cx, store);
    visit_middle_then_oldest(cx, handle, &notes);

    press(cx, handle, "alt-left");
    assert_eq!(selected(cx, &view), notes.middle);
    press(cx, handle, "alt-left");
    assert_eq!(selected(cx, &view), notes.newest);
    press(cx, handle, "alt-left");
    assert_eq!(selected(cx, &view), notes.newest);
    press(cx, handle, "alt-right");
    assert_eq!(selected(cx, &view), notes.middle);
    press(cx, handle, "alt-right");
    assert_eq!(selected(cx, &view), notes.oldest);
    press(cx, handle, "alt-right");
    assert_eq!(selected(cx, &view), notes.oldest);
}

#[gpui::test]
fn going_back_does_not_add_history_entries(cx: &mut TestAppContext) {
    let (store, notes) = three_notes();
    let (handle, view) = open_view(cx, store);
    visit_middle_then_oldest(cx, handle, &notes);

    press(cx, handle, "alt-left");
    press(cx, handle, "alt-right");
    press(cx, handle, "alt-left");
    press(cx, handle, "alt-left");

    assert_eq!(selected(cx, &view), notes.newest);
}

#[gpui::test]
fn history_skips_trashed_notes(cx: &mut TestAppContext) {
    let (store, notes) = three_notes();
    let (handle, view) = open_view(cx, store);
    visit_middle_then_oldest(cx, handle, &notes);
    view.update(cx, |view, _| {
        view.store.trash_note(&notes.middle, at(9)).unwrap();
    });

    press(cx, handle, "alt-left");
    assert_eq!(selected(cx, &view), notes.newest);
    press(cx, handle, "alt-right");
    assert_eq!(selected(cx, &view), notes.oldest);
}

#[gpui::test]
fn history_skips_notes_that_no_longer_exist(cx: &mut TestAppContext) {
    let (store, notes) = three_notes();
    let (handle, view) = open_view(cx, store);
    visit_middle_then_oldest(cx, handle, &notes);
    view.update(cx, |view, _| {
        view.store.trash_note(&notes.middle, at(1)).unwrap();
        view.store
            .purge_expired_trash(bourbon_core::Timestamp::now())
            .unwrap();
    });

    press(cx, handle, "alt-left");

    assert_eq!(selected(cx, &view), notes.newest);
}

#[gpui::test]
fn going_back_flushes_the_pending_edit_first(cx: &mut TestAppContext) {
    let (store, notes) = three_notes();
    let (handle, view) = open_view(cx, store);
    visit_middle_then_oldest(cx, handle, &notes);
    super::type_into_title(cx, handle, "!");

    press(cx, handle, "alt-left");

    assert_eq!(stored_note(cx, &view, &notes.oldest).title, "Oldest!");
}

#[gpui::test]
fn walking_history_without_earlier_notes_changes_nothing(cx: &mut TestAppContext) {
    let (store, notes) = three_notes();
    let (handle, view) = open_view(cx, store);

    press(cx, handle, "alt-left");
    press(cx, handle, "alt-right");

    assert_eq!(selected(cx, &view), notes.newest);
}

fn id() -> NoteId {
    NoteId::generate()
}

fn any(_: &NoteId) -> bool {
    true
}

#[test]
fn visiting_after_going_back_drops_the_forward_entries() {
    let (first, second, third, fourth) = (id(), id(), id(), id());
    let mut history = History::default();
    history.visit(&first);
    history.visit(&second);
    history.visit(&third);
    assert_eq!(history.back(&any), Some(second.clone()));

    history.visit(&fourth);

    assert_eq!(history.forward(&any), None);
    assert_eq!(history.back(&any), Some(second));
    assert_eq!(history.back(&any), Some(first));
    assert_eq!(history.back(&any), None);
}

#[test]
fn visiting_the_current_note_again_adds_nothing() {
    let (first, second) = (id(), id());
    let mut history = History::default();
    history.visit(&first);
    history.visit(&second);
    history.visit(&second);

    assert_eq!(history.back(&any), Some(first));
    assert_eq!(history.back(&any), None);
}

#[test]
fn walking_skips_unusable_entries_in_both_directions() {
    let (first, second, third) = (id(), id(), id());
    let mut history = History::default();
    history.visit(&first);
    history.visit(&second);
    history.visit(&third);
    let not_second = |candidate: &NoteId| *candidate != second;

    assert_eq!(history.back(&not_second), Some(first.clone()));
    assert_eq!(history.forward(&not_second), Some(third));
    assert_eq!(history.back(&|_| false), None);
    assert_eq!(history.back(&not_second), Some(first));
}

#[gpui::test]
fn the_pickers_show_the_notebook_name_and_the_status_label(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let work = work_notebook(&store);
    store.move_note(&seeded.newer, &work, at(9)).unwrap();
    store
        .set_status(&seeded.newer, NoteStatus::OnHold, at(9))
        .unwrap();
    let (handle, _) = open_view(cx, store);

    assert!(super::shows_label(cx, handle, "Work"));
    assert!(super::shows_label(cx, handle, "On Hold"));
    assert!(!super::shows_label(cx, handle, "Inbox"));
}

#[gpui::test]
fn a_note_without_status_reads_status(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, _) = open_view(cx, store);

    assert!(super::shows_label(cx, handle, "Status"));
}

#[gpui::test]
fn down_on_a_name_already_on_the_note_does_not_break_the_exact_match(cx: &mut TestAppContext) {
    let (store, note, _, _) = store_with_tags();
    let (handle, view) = open_view(cx, store);

    type_into_tag_input(cx, handle, "ruby");
    press_in_window(cx, handle, "down");
    press_in_window(cx, handle, "enter");

    assert_eq!(tag_names(cx, &view, &note), ["ruby"]);
    cx.read(|cx| assert_eq!(view.read(cx).editor.tag_input_text(cx), ""));
    assert!(!is_rendered(cx, handle, "error-banner"));
}

#[gpui::test]
fn the_status_menu_checks_the_current_status(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    store
        .set_status(&seeded.newer, NoteStatus::OnHold, at(9))
        .unwrap();
    let (handle, view) = open_view(cx, store);

    click(cx, handle, "status-button");

    assert_eq!(super::checked_menu_labels(cx, &view), ["On Hold"]);
}

fn is_highlighted(cx: &mut TestAppContext, handle: AnyWindowHandle, id: &str) -> bool {
    is_rendered(cx, handle, &format!("{id}-highlight"))
}

#[gpui::test]
fn the_selector_checks_only_the_notebook_of_the_note(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    work_notebook(&store);
    let (handle, view) = open_view(cx, store);

    click(cx, handle, "notebook-selector");

    assert_eq!(super::checked_menu_labels(cx, &view), ["Inbox"]);
}

#[gpui::test]
fn only_the_highlighted_suggestion_is_marked_as_selected(cx: &mut TestAppContext) {
    let (store, _, rust, _) = store_with_tags();
    let rustacean = store
        .create_tag("rustacean", TagColor::Default, at(8))
        .unwrap();
    let (handle, _) = open_view(cx, store);

    type_into_tag_input(cx, handle, "rust");
    press_in_window(cx, handle, "down");

    assert!(is_highlighted(
        cx,
        handle,
        &format!("tag-suggestion-{}", rust.id)
    ));
    assert!(!is_highlighted(
        cx,
        handle,
        &format!("tag-suggestion-{}", rustacean.id)
    ));
}

#[gpui::test]
fn the_toolbar_arrows_walk_the_viewed_notes(cx: &mut TestAppContext) {
    let (store, notes) = three_notes();
    let (handle, view) = open_view(cx, store);
    visit_middle_then_oldest(cx, handle, &notes);

    click(cx, handle, "history-back");
    assert_eq!(selected(cx, &view), notes.middle);

    click(cx, handle, "history-forward");
    assert_eq!(selected(cx, &view), notes.oldest);
}
