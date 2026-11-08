use std::sync::Arc;
use std::time::Duration;

use bourbon_core::{NoteId, NoteStatus, NoteStore, NotebookId, TagColor, Timestamp};
use gpui_kit::VisualTestContext;
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Entity, Point, TestAppContext, WindowBounds,
    WindowOptions, px, size,
};

use super::Workspace;
use crate::backup_runner::BackupRunner;
use crate::locations::Locations;
use crate::navigation::Navigation;

mod control_pane;
mod data;
mod drag;
mod editor;
mod find;
mod focus;
mod links;
mod tasks;
mod view_modes;

struct Seeded {
    older: NoteId,
    newer: NoteId,
}

fn at(millis: i64) -> Timestamp {
    Timestamp::from_millis(millis)
}

fn seeded_store() -> (NoteStore, Seeded) {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    let older = store.create_note(&inbox, at(1)).unwrap();
    store
        .save_note(&older.id, "Older", "older body", at(2))
        .unwrap();
    let newer = store.create_note(&inbox, at(3)).unwrap();
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

fn isolated_locations() -> Locations {
    locations_with_runner(Arc::new(data::FakeRunner::default()))
}

fn locations_with_runner(runner: Arc<dyn BackupRunner>) -> Locations {
    let directory = tempfile::tempdir().unwrap().keep();
    Locations {
        settings: bourbon_core::SettingsFile::at(directory.join("settings.toml")),
        omarchy_colors: directory.join("colors.toml"),
        database: directory.join("bourbon.db"),
        backup_token: bourbon_core::TokenFile::at(directory.join("backup-token")),
        backup_work_dir: directory.join("backup"),
        backup_runner: runner,
    }
}

fn open_view(cx: &mut TestAppContext, store: NoteStore) -> (AnyWindowHandle, Entity<Workspace>) {
    open_view_with(cx, store, isolated_locations())
}

fn open_view_with(
    cx: &mut TestAppContext,
    store: NoteStore,
    locations: Locations,
) -> (AnyWindowHandle, Entity<Workspace>) {
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
            |window, cx| crate::window::build_workspace(store, locations, window, cx),
        )
        .unwrap()
    })
}

fn stored_title(cx: &mut TestAppContext, view: &Entity<Workspace>, id: &NoteId) -> String {
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

fn choose_popup_item(cx: &mut TestAppContext, handle: AnyWindowHandle, index: usize) {
    cx.update_window(handle, |_, window, cx| {
        window.within("popup-menu").click(index, cx);
    })
    .unwrap();
    cx.run_until_parked();
}

fn popup_labels(cx: &mut TestAppContext, handle: AnyWindowHandle) -> Vec<String> {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let menu = window.within("popup-menu");
        (0usize..)
            .map_while(|index| menu.try_find(index))
            .map(|item| item.label().unwrap_or_default().to_string())
            .collect()
    })
    .unwrap()
}

fn menu_labels(cx: &mut TestAppContext, view: &Entity<Workspace>) -> Vec<String> {
    cx.read(|cx| {
        view.read(cx)
            .open_menu
            .as_ref()
            .map(|open| {
                open.menu()
                    .read(cx)
                    .entries()
                    .iter()
                    .filter_map(|entry| match entry {
                        bourbon_ui::MenuEntry::Item(item) => Some(item.label.to_string()),
                        bourbon_ui::MenuEntry::Separator => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    })
}

fn choose_menu_entry(cx: &mut TestAppContext, handle: AnyWindowHandle, position: usize) {
    cx.update_window(handle, |_, window, cx| {
        for _ in 0..=position {
            window.press("down", cx);
        }
        window.press("enter", cx);
        window.render_frame(cx);
    })
    .unwrap();
    cx.run_until_parked();
}

fn choose_setting(cx: &mut TestAppContext, handle: AnyWindowHandle, select: &str, position: usize) {
    click(cx, handle, select);
    cx.run_until_parked();
    choose_menu_entry(cx, handle, position);
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
        assert_eq!(view.editor.title(cx), "Newer");
        assert_eq!(view.editor.body(cx), "newer body");
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
        assert_eq!(view.editor.title(cx), "Older");
        assert_eq!(view.editor.body(cx), "older body");
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
        assert_eq!(view.editor.title(cx), "");
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

fn select_missing_note(cx: &mut TestAppContext, view: &Entity<Workspace>) {
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
        assert_eq!(view.editor.title(cx), "Newer!");
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

fn listed_titles(cx: &mut TestAppContext, view: &Entity<Workspace>) -> Vec<String> {
    cx.read(|cx| {
        view.read(cx)
            .notes
            .iter()
            .map(|summary| summary.title.clone())
            .collect()
    })
}

fn note_in(store: &NoteStore, notebook: &NotebookId, title: &str, at_millis: i64) -> NoteId {
    let note = store.create_note(notebook, at(at_millis)).unwrap();
    store.save_note(&note.id, title, "", at(at_millis)).unwrap();
    note.id
}

#[gpui_kit::test]
fn clicking_a_notebook_lists_only_its_notes(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    let work = store.create_notebook("Work", None, at(1)).unwrap();
    note_in(&store, &inbox, "In inbox", 2);
    note_in(&store, &work.id, "In work", 3);
    let (handle, view) = open_view(cx, store);
    assert_eq!(listed_titles(cx, &view).len(), 2);

    click(cx, handle, &format!("nav-notebook-{}", work.id));

    assert_eq!(listed_titles(cx, &view), ["In work"]);
}

#[gpui_kit::test]
fn clicking_all_notes_after_a_notebook_lists_everything_again(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    let work = store.create_notebook("Work", None, at(1)).unwrap();
    note_in(&store, &inbox, "In inbox", 2);
    note_in(&store, &work.id, "In work", 3);
    let (handle, view) = open_view(cx, store);

    click(cx, handle, &format!("nav-notebook-{}", work.id));
    click(cx, handle, "nav-all-notes");

    assert_eq!(listed_titles(cx, &view), ["In work", "In inbox"]);
}

#[gpui_kit::test]
fn clicking_a_notebook_lists_the_notes_of_its_sub_notebooks(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
    let work = store.create_notebook("Work", None, at(1)).unwrap();
    let nested = store
        .create_notebook("Nested", Some(&work.id), at(1))
        .unwrap();
    note_in(&store, &nested.id, "Deep", 2);
    let (handle, view) = open_view(cx, store);

    click(cx, handle, &format!("nav-notebook-{}", work.id));

    assert_eq!(listed_titles(cx, &view), ["Deep"]);
}

#[gpui_kit::test]
fn clicking_a_status_lists_notes_with_that_status(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    note_in(&store, &inbox, "Plain", 1);
    let done = note_in(&store, &inbox, "Done", 2);
    store
        .set_status(&done, NoteStatus::Completed, at(3))
        .unwrap();
    let (handle, view) = open_view(cx, store);

    click(cx, handle, "nav-status-completed");

    assert_eq!(listed_titles(cx, &view), ["Done"]);
}

#[gpui_kit::test]
fn clicking_a_tag_lists_notes_carrying_it(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    note_in(&store, &inbox, "Untagged", 1);
    let tagged = note_in(&store, &inbox, "Tagged", 2);
    let tag = store.create_tag("idea", TagColor::Red, at(3)).unwrap();
    store.add_tag_to_note(&tagged, &tag.id, at(4)).unwrap();
    let (handle, view) = open_view(cx, store);

    click(cx, handle, &format!("nav-tag-{}", tag.id));

    assert_eq!(listed_titles(cx, &view), ["Tagged"]);
}

#[gpui_kit::test]
fn clicking_trash_lists_only_trashed_notes(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    note_in(&store, &inbox, "Alive", 1);
    let trashed = note_in(&store, &inbox, "Trashed", 2);
    store.trash_note(&trashed, Timestamp::now()).unwrap();
    let (handle, view) = open_view(cx, store);
    assert_eq!(listed_titles(cx, &view), ["Alive"]);

    click(cx, handle, "nav-trash");

    assert_eq!(listed_titles(cx, &view), ["Trashed"]);
}

#[gpui_kit::test]
fn creating_a_note_while_a_notebook_is_selected_puts_it_there(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
    let work = store.create_notebook("Work", None, at(1)).unwrap();
    let (handle, view) = open_view(cx, store);

    click(cx, handle, &format!("nav-notebook-{}", work.id));
    click(cx, handle, "new-note");

    cx.read(|cx| {
        let workspace = view.read(cx);
        let created = workspace.selected.clone().unwrap();
        let note = workspace.store.find_note(&created).unwrap().unwrap();
        assert_eq!(note.notebook, work.id);
        assert_eq!(workspace.notes.len(), 1);
    });
}

fn created_note(cx: &mut TestAppContext, view: &Entity<Workspace>) -> bourbon_core::Note {
    cx.read(|cx| {
        let workspace = view.read(cx);
        let created = workspace.selected.clone().unwrap();
        assert!(workspace.notes.iter().any(|summary| summary.id == created));
        workspace.store.find_note(&created).unwrap().unwrap()
    })
}

#[gpui_kit::test]
fn creating_a_note_on_all_notes_puts_it_in_the_inbox_and_lists_it(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    store.create_notebook("Work", None, at(1)).unwrap();
    let (handle, view) = open_view(cx, store);

    click(cx, handle, "new-note");

    assert_eq!(created_note(cx, &view).notebook, inbox);
}

#[gpui_kit::test]
fn creating_a_note_on_trash_switches_to_all_notes_and_lists_it(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    let (handle, view) = open_view(cx, store);

    click(cx, handle, "nav-trash");
    VisualTestContext::from_window(handle, cx).simulate_keystrokes("ctrl-n");

    assert_eq!(created_note(cx, &view).notebook, inbox);
    cx.read(|cx| assert_eq!(view.read(cx).listing.navigation, Navigation::AllNotes));
}

#[gpui_kit::test]
fn creating_a_note_on_a_status_gives_it_that_status_and_lists_it(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    let (handle, view) = open_view(cx, store);

    click(cx, handle, "nav-status-completed");
    click(cx, handle, "new-note");

    let note = created_note(cx, &view);
    assert_eq!(note.notebook, inbox);
    assert_eq!(note.status, NoteStatus::Completed);
}

#[gpui_kit::test]
fn creating_a_note_on_a_tag_adds_the_tag_and_lists_it(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    let tag = store.create_tag("idea", TagColor::Red, at(1)).unwrap();
    let (handle, view) = open_view(cx, store);

    click(cx, handle, &format!("nav-tag-{}", tag.id));
    click(cx, handle, "new-note");

    let note = created_note(cx, &view);
    assert_eq!(note.notebook, inbox);
    assert_eq!(note.tags, [tag]);
}

#[gpui_kit::test]
fn creating_a_note_with_several_tags_adds_every_tag(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
    let first = store.create_tag("one", TagColor::Red, at(1)).unwrap();
    let second = store.create_tag("two", TagColor::Blue, at(1)).unwrap();
    let (handle, view) = open_view(cx, store);
    view.update(cx, |view, cx| {
        view.navigate(
            Navigation::Tags(vec![first.id.clone(), second.id.clone()]),
            cx,
        );
    });

    click(cx, handle, "new-note");

    let tag_ids: Vec<_> = created_note(cx, &view)
        .tags
        .into_iter()
        .map(|tag| tag.id)
        .collect();
    assert_eq!(tag_ids.len(), 2);
    assert!(tag_ids.contains(&first.id));
    assert!(tag_ids.contains(&second.id));
}

#[gpui_kit::test]
fn saving_reloads_the_list_and_leaves_the_sidebar_data_unchanged(cx: &mut TestAppContext) {
    let (store, seeded) = seeded_store();
    let (handle, view) = open_view(cx, store);
    let sidebar_before = cx.read(|cx| {
        let view = view.read(cx);
        (view.notebooks.clone(), view.tags.clone())
    });

    type_into_title(cx, handle, "!");
    advance_past_autosave_delay(cx);

    assert_eq!(listed_titles(cx, &view)[0], "Newer!");
    cx.read(|cx| {
        let view = view.read(cx);
        assert_eq!((view.notebooks.clone(), view.tags.clone()), sidebar_before);
        assert_eq!(view.selected, Some(seeded.newer.clone()));
    });
}

#[gpui_kit::test]
fn creating_a_note_refreshes_the_sidebar_counts(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
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
    assert_eq!(count(cx), 0);

    click(cx, handle, "new-note");

    assert_eq!(count(cx), 1);
}

#[gpui_kit::test]
fn startup_purges_trash_older_than_the_retention(cx: &mut TestAppContext) {
    let store = NoteStore::open_in_memory().unwrap();
    let inbox = store.default_notebook().unwrap();
    let expired = note_in(&store, &inbox, "Expired", 1);
    store.trash_note(&expired, at(1)).unwrap();
    let recent = note_in(&store, &inbox, "Recent", 2);
    store.trash_note(&recent, Timestamp::now()).unwrap();
    let (_, view) = open_view(cx, store);

    cx.read(|cx| {
        let store = &view.read(cx).store;
        assert!(store.find_note(&expired).unwrap().is_none());
        assert!(store.find_note(&recent).unwrap().is_some());
    });
}

mod note_list;
mod settings;
mod sidebar;
mod templates;
