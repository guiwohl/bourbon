use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bourbon_core::{BackupOutcome, NoteQuery, NoteStore, Settings};
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext, VisualTestContext};
use jiff::civil::date;

use super::{
    choose_popup_item, click, locations_with_runner, note_in, open_view_with, popup_labels,
    seeded_store, type_into_title,
};
use crate::backup_runner::{BackupJob, BackupRunner};
use crate::locations::Locations;
use crate::navigation::Navigation;
use crate::workspace::Workspace;

pub(super) struct FakeRunner {
    backups: Mutex<Vec<BackupJob>>,
    restores: Mutex<Vec<BackupJob>>,
    backup_result: Mutex<Result<BackupOutcome, String>>,
    restore_source: Mutex<Result<PathBuf, String>>,
}

impl Default for FakeRunner {
    fn default() -> Self {
        Self {
            backups: Mutex::new(Vec::new()),
            restores: Mutex::new(Vec::new()),
            backup_result: Mutex::new(Ok(BackupOutcome::Pushed)),
            restore_source: Mutex::new(Err("no backup".to_owned())),
        }
    }
}

impl FakeRunner {
    fn backup_count(&self) -> usize {
        self.backups.lock().unwrap().len()
    }
}

impl BackupRunner for FakeRunner {
    fn back_up(&self, job: &BackupJob) -> Result<BackupOutcome, String> {
        self.backups.lock().unwrap().push(job.clone());
        self.backup_result.lock().unwrap().clone()
    }

    fn restore(&self, job: &BackupJob) -> Result<(), String> {
        self.restores.lock().unwrap().push(job.clone());
        let source = self.restore_source.lock().unwrap().clone()?;
        std::fs::copy(source, &job.database).map_err(|error| error.to_string())?;
        Ok(())
    }
}

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

struct Setup {
    runner: Arc<FakeRunner>,
    handle: AnyWindowHandle,
    view: Entity<Workspace>,
    locations: Locations,
}

fn settings_with(enabled: bool) -> Settings {
    let mut settings = Settings::default();
    settings.backup_repository = Some("owner/notes".to_owned());
    settings.backup_enabled = enabled;
    settings
}

fn open_with_settings(
    cx: &mut TestAppContext,
    store: NoteStore,
    settings: &Settings,
    runner: Arc<FakeRunner>,
) -> Setup {
    let locations = locations_with_runner(runner.clone());
    locations.settings.save(settings).unwrap();
    let view_locations = Locations {
        settings: locations.settings.clone(),
        omarchy_colors: locations.omarchy_colors.clone(),
        database: locations.database.clone(),
        backup_token: locations.backup_token.clone(),
        backup_work_dir: locations.backup_work_dir.clone(),
        backup_runner: runner.clone(),
    };
    let (handle, view) = open_view_with(cx, store, view_locations);
    Setup {
        runner,
        handle,
        view,
        locations,
    }
}

fn open_default(cx: &mut TestAppContext) -> Setup {
    open_with_settings(
        cx,
        seeded_store().0,
        &settings_with(false),
        Arc::new(FakeRunner::default()),
    )
}

fn is_present(cx: &mut TestAppContext, handle: AnyWindowHandle, id: &str) -> bool {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.try_find(id.to_owned()).is_some()
    })
    .unwrap()
}

fn status(cx: &mut TestAppContext, view: &Entity<Workspace>) -> Option<String> {
    view.read_with(cx, |view, _| {
        view.data.status.as_ref().map(ToString::to_string)
    })
}

fn banner(cx: &mut TestAppContext, view: &Entity<Workspace>) -> Option<String> {
    view.read_with(cx, |view, _| {
        view.error_message.as_ref().map(ToString::to_string)
    })
}

fn titles(cx: &mut TestAppContext, view: &Entity<Workspace>) -> Vec<String> {
    view.read_with(cx, |view, _| {
        view.store
            .list_notes(&NoteQuery::default())
            .unwrap()
            .into_iter()
            .map(|summary| summary.title)
            .collect()
    })
}

#[gpui_kit::test]
fn the_app_menu_lists_its_four_entries_once_opened(cx: &mut TestAppContext) {
    let setup = open_default(cx);

    click(cx, setup.handle, "app-menu");

    assert_eq!(popup_labels(cx, setup.handle).len(), 4);
}

#[gpui_kit::test]
fn the_menu_settings_entry_opens_settings(cx: &mut TestAppContext) {
    let setup = open_default(cx);
    click(cx, setup.handle, "app-menu");

    choose_popup_item(cx, setup.handle, 0);

    assert!(setup.view.read_with(cx, |view, _| view.settings_open));
}

#[gpui_kit::test]
fn the_menu_back_up_entry_runs_a_backup_of_the_configured_repository(cx: &mut TestAppContext) {
    let setup = open_default(cx);
    click(cx, setup.handle, "app-menu");

    choose_popup_item(cx, setup.handle, 1);
    cx.run_until_parked();

    let jobs = setup.runner.backups.lock().unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].repository.to_string(), "owner/notes");
    assert_eq!(jobs[0].database, setup.locations.database);
    assert_eq!(jobs[0].work_dir, setup.locations.backup_work_dir);
}

#[gpui_kit::test]
fn the_menu_restore_entry_asks_before_replacing_anything(cx: &mut TestAppContext) {
    let setup = open_default(cx);
    click(cx, setup.handle, "app-menu");

    choose_popup_item(cx, setup.handle, 2);

    assert_eq!(
        setup.view.read_with(cx, |view, _| view.restore_question()),
        Some(
            "Replace all notes with the backup from owner/notes? Your current notes are saved to bourbon.db.before-restore."
                .to_owned()
        )
    );
    assert!(setup.runner.restores.lock().unwrap().is_empty());
}

#[gpui_kit::test]
fn the_action_search_offers_and_runs_back_up_now(cx: &mut TestAppContext) {
    let setup = open_default(cx);
    press(cx, setup.handle, "ctrl-p");
    type_text(cx, setup.handle, "back up now");

    let offered = setup
        .view
        .read_with(cx, |view, _| view.control_pane.as_ref().unwrap().actions());
    assert_eq!(offered, [crate::control_pane::PaneAction::BackUpNow]);
    press(cx, setup.handle, "enter");
    cx.run_until_parked();

    assert_eq!(setup.runner.backup_count(), 1);
}

#[gpui_kit::test]
fn the_action_search_runs_restore_from_backup_with_a_confirmation(cx: &mut TestAppContext) {
    let setup = open_default(cx);
    press(cx, setup.handle, "ctrl-p");
    type_text(cx, setup.handle, "restore");

    press(cx, setup.handle, "enter");

    assert!(
        setup
            .view
            .read_with(cx, |view, _| view.restore_question().is_some())
    );
}

#[gpui_kit::test]
fn a_manual_backup_reports_the_time_and_an_unchanged_one_says_up_to_date(cx: &mut TestAppContext) {
    let setup = open_default(cx);

    setup.view.update(cx, Workspace::back_up_now);
    cx.run_until_parked();
    let pushed = status(cx, &setup.view).unwrap();
    *setup.runner.backup_result.lock().unwrap() = Ok(BackupOutcome::Unchanged);
    setup.view.update(cx, Workspace::back_up_now);
    cx.run_until_parked();

    assert!(pushed.starts_with("Last backup "));
    assert_eq!(status(cx, &setup.view).as_deref(), Some("Up to date"));
    assert_eq!(banner(cx, &setup.view), None);
}

#[gpui_kit::test]
fn a_failed_backup_goes_to_the_status_line_and_the_banner(cx: &mut TestAppContext) {
    let setup = open_default(cx);
    *setup.runner.backup_result.lock().unwrap() = Err("push rejected".to_owned());

    setup.view.update(cx, Workspace::back_up_now);
    cx.run_until_parked();

    assert_eq!(status(cx, &setup.view).as_deref(), Some("push rejected"));
    assert_eq!(banner(cx, &setup.view).as_deref(), Some("push rejected"));
}

#[gpui_kit::test]
fn backing_up_without_a_repository_asks_for_one(cx: &mut TestAppContext) {
    let setup = open_with_settings(
        cx,
        seeded_store().0,
        &Settings::default(),
        Arc::new(FakeRunner::default()),
    );

    setup.view.update(cx, Workspace::back_up_now);
    cx.run_until_parked();

    assert_eq!(setup.runner.backup_count(), 0);
    assert_eq!(
        status(cx, &setup.view).as_deref(),
        Some("Set a backup repository in Settings first")
    );
}

#[gpui_kit::test]
fn two_backups_never_run_at_once(cx: &mut TestAppContext) {
    let setup = open_default(cx);

    setup.view.update(cx, |view, cx| {
        view.back_up_now(cx);
        view.back_up_now(cx);
    });
    cx.run_until_parked();

    assert_eq!(setup.runner.backup_count(), 1);
    setup.view.update(cx, Workspace::back_up_now);
    cx.run_until_parked();
    assert_eq!(setup.runner.backup_count(), 2);
}

#[gpui_kit::test]
fn an_edit_triggers_an_automatic_backup_five_minutes_later(cx: &mut TestAppContext) {
    let setup = open_with_settings(
        cx,
        seeded_store().0,
        &settings_with(true),
        Arc::new(FakeRunner::default()),
    );
    type_into_title(cx, setup.handle, "Edited");

    cx.executor().advance_clock(Duration::from_secs(299));
    cx.run_until_parked();
    assert_eq!(setup.runner.backup_count(), 0);
    cx.executor().advance_clock(Duration::from_secs(2));
    cx.run_until_parked();

    assert_eq!(setup.runner.backup_count(), 1);
}

#[gpui_kit::test]
fn a_new_edit_restarts_the_five_minutes(cx: &mut TestAppContext) {
    let setup = open_with_settings(
        cx,
        seeded_store().0,
        &settings_with(true),
        Arc::new(FakeRunner::default()),
    );
    type_into_title(cx, setup.handle, "One");
    cx.executor().advance_clock(Duration::from_secs(200));
    type_into_title(cx, setup.handle, "Two");

    cx.executor().advance_clock(Duration::from_secs(200));
    cx.run_until_parked();
    assert_eq!(setup.runner.backup_count(), 0);
    cx.executor().advance_clock(Duration::from_secs(101));
    cx.run_until_parked();

    assert_eq!(setup.runner.backup_count(), 1);
}

#[gpui_kit::test]
fn edits_never_back_up_while_automatic_backup_is_off(cx: &mut TestAppContext) {
    let setup = open_default(cx);
    type_into_title(cx, setup.handle, "Edited");

    cx.executor().advance_clock(Duration::from_mins(15));
    cx.run_until_parked();

    assert_eq!(setup.runner.backup_count(), 0);
}

#[gpui_kit::test]
fn closing_after_an_edit_waits_for_a_backup_then_closes(cx: &mut TestAppContext) {
    let setup = open_with_settings(
        cx,
        seeded_store().0,
        &settings_with(true),
        Arc::new(FakeRunner::default()),
    );
    type_into_title(cx, setup.handle, "Edited");

    let closed = VisualTestContext::from_window(setup.handle, cx).simulate_close();
    assert!(!closed);
    cx.run_until_parked();

    assert_eq!(setup.runner.backup_count(), 1);
    assert!(cx.update_window(setup.handle, |_, _, _| ()).is_err());
}

#[gpui_kit::test]
fn closing_after_an_edit_still_closes_when_the_backup_fails(cx: &mut TestAppContext) {
    let setup = open_with_settings(
        cx,
        seeded_store().0,
        &settings_with(true),
        Arc::new(FakeRunner::default()),
    );
    *setup.runner.backup_result.lock().unwrap() = Err("offline".to_owned());
    type_into_title(cx, setup.handle, "Edited");

    VisualTestContext::from_window(setup.handle, cx).simulate_close();
    cx.run_until_parked();

    assert!(cx.update_window(setup.handle, |_, _, _| ()).is_err());
}

#[gpui_kit::test]
fn closing_without_edits_or_without_automatic_backup_does_not_back_up(cx: &mut TestAppContext) {
    let untouched = open_with_settings(
        cx,
        seeded_store().0,
        &settings_with(true),
        Arc::new(FakeRunner::default()),
    );
    assert!(VisualTestContext::from_window(untouched.handle, cx).simulate_close());
    assert_eq!(untouched.runner.backup_count(), 0);

    let disabled = open_default(cx);
    type_into_title(cx, disabled.handle, "Edited");
    assert!(VisualTestContext::from_window(disabled.handle, cx).simulate_close());
    assert_eq!(disabled.runner.backup_count(), 0);
}

fn file_store_with(locations: &Locations, title: &str) -> NoteStore {
    let store = NoteStore::open(&locations.database).unwrap();
    let inbox = store.default_notebook().unwrap();
    note_in(&store, &inbox, title, 10);
    store
}

fn backup_file_with(title: &str) -> PathBuf {
    let path = tempfile::tempdir().unwrap().keep().join("backup.db");
    let store = NoteStore::open(&path).unwrap();
    let inbox = store.default_notebook().unwrap();
    note_in(&store, &inbox, title, 20);
    store.snapshot(&path.with_file_name("snapshot.db")).unwrap();
    path.with_file_name("snapshot.db")
}

fn open_for_restore(cx: &mut TestAppContext, restore_source: Result<PathBuf, String>) -> Setup {
    let runner = Arc::new(FakeRunner::default());
    *runner.restore_source.lock().unwrap() = restore_source;
    let locations = locations_with_runner(runner.clone());
    locations.settings.save(&settings_with(false)).unwrap();
    let store = file_store_with(&locations, "Current");
    let (handle, view) = open_view_with(
        cx,
        store,
        Locations {
            settings: locations.settings.clone(),
            omarchy_colors: locations.omarchy_colors.clone(),
            database: locations.database.clone(),
            backup_token: locations.backup_token.clone(),
            backup_work_dir: locations.backup_work_dir.clone(),
            backup_runner: runner.clone(),
        },
    );
    Setup {
        runner,
        handle,
        view,
        locations,
    }
}

#[gpui_kit::test]
fn restoring_swaps_the_store_keeps_a_copy_and_starts_over(cx: &mut TestAppContext) {
    let setup = open_for_restore(cx, Ok(backup_file_with("From backup")));
    setup
        .view
        .update(cx, |view, cx| view.navigate(Navigation::Trash, cx));
    setup.view.update(cx, Workspace::begin_restore);

    click(cx, setup.handle, "restore-confirm");
    cx.run_until_parked();

    assert_eq!(titles(cx, &setup.view), ["From backup"]);
    let kept = NoteStore::open(
        &setup
            .locations
            .database
            .with_file_name("bourbon.db.before-restore"),
    )
    .unwrap();
    let kept_titles: Vec<String> = kept
        .list_notes(&NoteQuery::default())
        .unwrap()
        .into_iter()
        .map(|summary| summary.title)
        .collect();
    assert_eq!(kept_titles, ["Current"]);
    setup.view.read_with(cx, |view, _| {
        assert_eq!(view.listing.navigation, Navigation::AllNotes);
        assert_eq!(view.selected, None);
        assert_eq!(view.notes.len(), 1);
        assert_eq!(view.restore_question(), None);
    });
    assert_eq!(
        status(cx, &setup.view).as_deref(),
        Some("Restored from backup")
    );
}

#[gpui_kit::test]
fn cancelling_the_restore_changes_nothing(cx: &mut TestAppContext) {
    let setup = open_for_restore(cx, Ok(backup_file_with("From backup")));
    setup.view.update(cx, Workspace::begin_restore);

    click(cx, setup.handle, "restore-cancel");
    cx.run_until_parked();

    assert_eq!(titles(cx, &setup.view), ["Current"]);
    assert!(setup.runner.restores.lock().unwrap().is_empty());
    assert_eq!(
        setup.view.read_with(cx, |view, _| view.restore_question()),
        None
    );
}

#[gpui_kit::test]
fn a_failed_restore_reopens_the_old_notes_and_says_why(cx: &mut TestAppContext) {
    let setup = open_for_restore(cx, Err("repository not found".to_owned()));
    setup.view.update(cx, Workspace::begin_restore);

    click(cx, setup.handle, "restore-confirm");
    cx.run_until_parked();

    assert_eq!(titles(cx, &setup.view), ["Current"]);
    assert_eq!(
        banner(cx, &setup.view).as_deref(),
        Some("repository not found")
    );
    assert!(
        setup
            .locations
            .database
            .with_file_name("bourbon.db.before-restore")
            .exists()
    );
}

#[gpui_kit::test]
fn restoring_saves_the_pending_edit_first(cx: &mut TestAppContext) {
    let setup = open_for_restore(cx, Err("nope".to_owned()));
    type_into_title(cx, setup.handle, "Typed");
    setup.view.update(cx, Workspace::begin_restore);

    click(cx, setup.handle, "restore-confirm");
    cx.run_until_parked();

    assert!(
        titles(cx, &setup.view)
            .iter()
            .any(|title| title.contains("Typed"))
    );
}

#[gpui_kit::test]
fn a_restore_is_refused_while_a_backup_is_running(cx: &mut TestAppContext) {
    let setup = open_for_restore(cx, Ok(backup_file_with("From backup")));
    setup.view.update(cx, |view, cx| {
        view.back_up_now(cx);
        view.begin_restore(cx);
        view.confirm_restore(cx);
    });
    cx.run_until_parked();

    assert!(setup.runner.restores.lock().unwrap().is_empty());
    assert_eq!(titles(cx, &setup.view), ["Current"]);
}

#[gpui_kit::test]
fn exporting_writes_every_note_into_a_dated_folder(cx: &mut TestAppContext) {
    let setup = open_default(cx);
    let target = tempfile::tempdir().unwrap();
    let now = date(2026, 11, 6).at(14, 3, 0, 0).in_tz("UTC").unwrap();

    setup.view.update(cx, |view, cx| {
        view.export_into(target.path(), &now, cx);
    });

    let folder = target.path().join("Bourbon export 2026-11-06 1403");
    let mut files: Vec<String> = walk(&folder);
    files.sort();
    assert_eq!(files, ["newer.md", "older.md"]);
    let notice = setup
        .view
        .read_with(cx, |view, _| view.notice.as_ref().map(ToString::to_string));
    assert_eq!(
        notice,
        Some(format!("Exported 2 notes to {}", folder.display()))
    );
}

#[gpui_kit::test]
fn an_export_that_cannot_be_written_is_reported(cx: &mut TestAppContext) {
    let setup = open_default(cx);
    let target = tempfile::tempdir().unwrap();
    let now = date(2026, 11, 6).at(14, 3, 0, 0).in_tz("UTC").unwrap();
    setup
        .view
        .update(cx, |view, cx| view.export_into(target.path(), &now, cx));

    setup
        .view
        .update(cx, |view, cx| view.export_into(target.path(), &now, cx));

    assert!(banner(cx, &setup.view).is_some());
}

fn walk(directory: &std::path::Path) -> Vec<String> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            found.extend(walk(&path));
        } else {
            found.push(path.file_name().unwrap().to_string_lossy().into_owned());
        }
    }
    found
}

#[gpui_kit::test]
fn the_settings_repository_field_validates_and_saves(cx: &mut TestAppContext) {
    let setup = open_with_settings(
        cx,
        seeded_store().0,
        &Settings::default(),
        Arc::new(FakeRunner::default()),
    );
    press(cx, setup.handle, "ctrl-,");
    click(cx, setup.handle, "settings-backup-repository");

    type_text(cx, setup.handle, "not a repo");
    let error = setup
        .view
        .read_with(cx, |view, _| view.data.repository_error.is_some());
    assert!(error);
    assert_eq!(setup.locations.settings.load().backup_repository, None);

    click(cx, setup.handle, "settings-backup-repository");
    press(cx, setup.handle, "ctrl-a");
    type_text(cx, setup.handle, "me/notes");

    assert_eq!(
        setup.locations.settings.load().backup_repository.as_deref(),
        Some("me/notes")
    );
    assert!(
        !setup
            .view
            .read_with(cx, |view, _| view.data.repository_error.is_some())
    );
}

#[gpui_kit::test]
fn choosing_the_token_login_shows_a_token_field_whose_text_is_saved_privately(
    cx: &mut TestAppContext,
) {
    let setup = open_default(cx);
    press(cx, setup.handle, "ctrl-,");
    let hidden = !is_present(cx, setup.handle, "settings-backup-token");

    click(cx, setup.handle, "settings-backup-auth-token");
    click(cx, setup.handle, "settings-backup-token");
    type_text(cx, setup.handle, "ghp_abc");

    assert!(hidden);
    assert_eq!(
        setup.locations.settings.load().backup_auth,
        bourbon_core::BackupAuth::Token
    );
    let token_path = setup.locations.backup_token.path().to_path_buf();
    assert_eq!(std::fs::read_to_string(&token_path).unwrap(), "ghp_abc");
    assert_eq!(
        std::fs::metadata(&token_path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let settings_text = std::fs::read_to_string(setup.locations.settings.path()).unwrap();
    assert!(!settings_text.contains("ghp_abc"));
}

#[gpui_kit::test]
fn the_settings_toggle_and_buttons_drive_the_backup(cx: &mut TestAppContext) {
    let setup = open_default(cx);
    press(cx, setup.handle, "ctrl-,");

    click(cx, setup.handle, "settings-backup-enabled");
    click(cx, setup.handle, "settings-backup-now");
    cx.run_until_parked();
    click(cx, setup.handle, "settings-restore");

    assert!(setup.locations.settings.load().backup_enabled);
    assert_eq!(setup.runner.backup_count(), 1);
    assert!(
        setup
            .view
            .read_with(cx, |view, _| view.restore_question().is_some())
    );
}

#[gpui_kit::test]
fn starting_an_export_dismisses_the_previous_notice(cx: &mut TestAppContext) {
    let setup = open_default(cx);
    setup
        .view
        .update(cx, |view, _| view.notice = Some("old".into()));

    setup.view.update(cx, Workspace::export_to_markdown);

    assert_eq!(
        setup.view.read_with(cx, |view, _| view.notice.clone()),
        None
    );
}
