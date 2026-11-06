use std::time::Duration;

use bourbon_core::{NoteId, NoteStore, Timestamp};
use gpui_kit::VisualTestContext;
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Entity, Point, TestAppContext, WindowBounds,
    WindowOptions, px, size,
};

use super::NotesView;

struct Seeded {
    older: NoteId,
    newer: NoteId,
}

fn at(millis: i64) -> Timestamp {
    Timestamp::from_millis(millis)
}

fn seeded_store() -> (NoteStore, Seeded) {
    let store = NoteStore::open_in_memory().unwrap();
    let older = store.create_note(at(1)).unwrap();
    store
        .save_note(&older.id, "Older", "older body", at(2))
        .unwrap();
    let newer = store.create_note(at(3)).unwrap();
    store
        .save_note(&newer.id, "Newer", "newer body", at(4))
        .unwrap();
    (
        store,
        Seeded {
            older: older.id,
            newer: newer.id,
        },
    )
}

fn open_view(cx: &mut TestAppContext, store: NoteStore) -> (AnyWindowHandle, Entity<NotesView>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::window::bind_keys(cx);
    });
    let bounds = Bounds {
        origin: Point::default(),
        size: size(px(900.), px(600.)),
    };
    cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..WindowOptions::default()
            },
            cx,
            |window, cx| crate::window::build_notes_view(store, window, cx),
        )
        .unwrap()
    })
}

fn stored_title(cx: &mut TestAppContext, view: &Entity<NotesView>, id: &NoteId) -> String {
    cx.read(|cx| view.read(cx).store.find_note(id).unwrap().unwrap().title)
}

fn type_into_title(cx: &mut TestAppContext, handle: AnyWindowHandle, text: &str) {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("title", cx);
        window.input(text, cx);
    })
    .unwrap();
}

fn click(cx: &mut TestAppContext, handle: AnyWindowHandle, id: &str) {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click(id.to_owned(), cx);
    })
    .unwrap();
}

#[gpui_kit::test]
fn startup_selects_the_most_recent_note_and_loads_it(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (_, view) = open_view(cx, store);

    cx.read(|cx| {
        let view = view.read(cx);
        assert_eq!(view.selected, Some(seeded.newer.clone()));
        assert_eq!(view.title_input.read(cx).value(), "Newer");
        assert_eq!(view.body_editor.read(cx).value(), "newer body");
    });
}

#[gpui_kit::test]
fn typing_a_title_is_saved_only_after_the_autosave_delay(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    type_into_title(cx, handle, "!");

    cx.executor().advance_clock(Duration::from_millis(499));
    cx.run_until_parked();
    assert_eq!(stored_title(cx, &view, &seeded.newer), "Newer");

    cx.executor().advance_clock(Duration::from_millis(1));
    cx.run_until_parked();
    assert_eq!(stored_title(cx, &view, &seeded.newer), "Newer!");
}

#[gpui_kit::test]
fn each_keystroke_restarts_the_autosave_delay(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    type_into_title(cx, handle, "!");
    cx.executor().advance_clock(Duration::from_millis(400));
    cx.update_window(handle, |_, window, cx| window.input("?", cx))
        .unwrap();
    cx.executor().advance_clock(Duration::from_millis(400));
    cx.run_until_parked();
    assert_eq!(stored_title(cx, &view, &seeded.newer), "Newer");

    cx.executor().advance_clock(Duration::from_millis(100));
    cx.run_until_parked();
    assert_eq!(stored_title(cx, &view, &seeded.newer), "Newer!?");
}

#[gpui_kit::test]
fn switching_notes_saves_the_pending_edit_immediately(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    type_into_title(cx, handle, "!");
    click(cx, handle, &format!("note-{}", seeded.older));

    assert_eq!(stored_title(cx, &view, &seeded.newer), "Newer!");
    cx.read(|cx| {
        let view = view.read(cx);
        assert_eq!(view.selected, Some(seeded.older.clone()));
        assert_eq!(view.title_input.read(cx).value(), "Older");
        assert_eq!(view.body_editor.read(cx).value(), "older body");
    });
}

#[gpui_kit::test]
fn creating_a_note_selects_it_on_top_of_the_list_and_focuses_the_title(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    click(cx, handle, "new-note");

    cx.read(|cx| {
        let view = view.read(cx);
        let created = view.selected.clone().unwrap();
        assert_ne!(created, seeded.newer);
        assert_ne!(created, seeded.older);
        assert_eq!(view.notes.len(), 3);
        assert_eq!(view.notes[0].id, created);
        assert_eq!(view.title_input.read(cx).value(), "");
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(window.find("title").focused(), Some(true));
    })
    .unwrap();
}

#[gpui_kit::test]
fn starting_without_notes_shows_the_empty_state_and_creating_leaves_it(cx: &mut TestAppContext) {
    let (handle, view) = open_view(cx, NoteStore::open_in_memory().unwrap());

    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("empty-new-note").is_some());
        assert!(window.try_find("title").is_none());
    })
    .unwrap();

    click(cx, handle, "empty-new-note");

    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("title").is_some());
    })
    .unwrap();
    cx.read(|cx| assert!(view.read(cx).selected.is_some()));
}

fn advance_past_autosave_delay(cx: &mut TestAppContext) {
    cx.executor().advance_clock(Duration::from_millis(500));
    cx.run_until_parked();
}

fn select_missing_note(cx: &mut TestAppContext, view: &Entity<NotesView>) {
    view.update(cx, |view, _| view.selected = Some(NoteId::generate()));
}

fn banner_is_shown(cx: &mut TestAppContext, handle: AnyWindowHandle) -> bool {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.try_find("error-banner").is_some()
    })
    .unwrap()
}

#[gpui_kit::test]
fn closing_the_window_saves_an_edit_typed_before_the_autosave_delay(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    type_into_title(cx, handle, "!");
    cx.executor().advance_clock(Duration::from_millis(100));
    let closed = VisualTestContext::from_window(handle, cx).simulate_close();

    assert!(closed);
    assert_eq!(stored_title(cx, &view, &seeded.newer), "Newer!");
}

#[gpui_kit::test]
fn closing_the_window_is_refused_while_the_edit_cannot_be_saved(cx: &mut TestAppContext) {
    let (store, _) = seeded_store();
    let (handle, view) = open_view(cx, store);

    type_into_title(cx, handle, "!");
    select_missing_note(cx, &view);
    let closed = VisualTestContext::from_window(handle, cx).simulate_close();

    assert!(!closed);
    assert!(banner_is_shown(cx, handle));
}

#[gpui_kit::test]
fn switching_notes_cancels_the_pending_autosave(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    type_into_title(cx, handle, "!");
    click(cx, handle, &format!("note-{}", seeded.older));
    cx.read(|cx| {
        view.read(cx)
            .store
            .save_note(&seeded.older, "Elsewhere", "older body", at(9))
            .unwrap();
    });
    advance_past_autosave_delay(cx);

    assert_eq!(stored_title(cx, &view, &seeded.older), "Elsewhere");
}

#[gpui_kit::test]
fn a_failed_save_shows_the_banner_until_the_next_successful_save(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    assert!(!banner_is_shown(cx, handle));

    select_missing_note(cx, &view);
    type_into_title(cx, handle, "!");
    advance_past_autosave_delay(cx);
    assert!(banner_is_shown(cx, handle));

    view.update(cx, |view, _| view.selected = Some(seeded.newer.clone()));
    cx.update_window(handle, |_, window, cx| window.input("?", cx))
        .unwrap();
    advance_past_autosave_delay(cx);
    assert!(!banner_is_shown(cx, handle));
    assert_eq!(stored_title(cx, &view, &seeded.newer), "Newer!?");
}

#[gpui_kit::test]
fn a_failed_save_blocks_switching_to_another_note(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    type_into_title(cx, handle, "!");
    select_missing_note(cx, &view);
    let missing = view.read_with(cx, |view, _| view.selected.clone());
    click(cx, handle, &format!("note-{}", seeded.older));

    cx.read(|cx| {
        let view = view.read(cx);
        assert_eq!(view.selected, missing);
        assert_eq!(view.title_input.read(cx).value(), "Newer!");
    });
    assert!(banner_is_shown(cx, handle));
}

#[gpui_kit::test]
fn ctrl_n_creates_and_selects_a_note(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    click(cx, handle, "title");

    VisualTestContext::from_window(handle, cx).simulate_keystrokes("ctrl-n");

    cx.read(|cx| {
        let view = view.read(cx);
        let created = view.selected.clone().unwrap();
        assert_ne!(created, seeded.newer);
        assert_ne!(created, seeded.older);
        assert_eq!(view.notes.len(), 3);
    });
}

#[gpui_kit::test]
fn ctrl_n_creates_a_note_when_nothing_is_focused(cx: &mut TestAppContext) {
    let (handle, view) = open_view(cx, NoteStore::open_in_memory().unwrap());

    VisualTestContext::from_window(handle, cx).simulate_keystrokes("ctrl-n");

    cx.read(|cx| {
        let view = view.read(cx);
        assert!(view.selected.is_some());
        assert_eq!(view.notes.len(), 1);
    });
}

#[gpui_kit::test]
fn quitting_the_app_saves_an_edit_typed_before_the_autosave_delay(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);

    type_into_title(cx, handle, "!");
    cx.update(gpui_kit::App::shutdown);
    cx.run_until_parked();

    assert_eq!(stored_title(cx, &view, &seeded.newer), "Newer!");
}
