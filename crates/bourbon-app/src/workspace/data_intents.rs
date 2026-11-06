use std::path::Path;
use std::time::Duration;

use bourbon_core::{BackupOutcome, GitHubRepository, NoteStore, StoreError};
use gpui_kit::{AnyWindowHandle, Context, PathPromptOptions, SharedString, Task, Window};
use jiff::Zoned;

use super::Workspace;
use crate::backup_runner::BackupJob;
use crate::editor::History;
use crate::navigation::Navigation;

const AUTOMATIC_BACKUP_DELAY: Duration = Duration::from_secs(300);
const CLOSE_BACKUP_LIMIT: Duration = Duration::from_secs(20);
const BEFORE_RESTORE_FILE: &str = "bourbon.db.before-restore";

#[derive(Default)]
pub(super) struct DataState {
    running: bool,
    dirty: bool,
    pub(super) status: Option<SharedString>,
    pub(super) repository_error: Option<SharedString>,
    restore_confirmation: bool,
    timer: Option<Task<()>>,
    task: Option<Task<()>>,
    closing: Option<AnyWindowHandle>,
    close_deadline: Option<Task<()>>,
}

pub(super) fn success_text(outcome: BackupOutcome, now: &Zoned) -> String {
    match outcome {
        BackupOutcome::Pushed => format!("Last backup {}", now.strftime("%H:%M")),
        BackupOutcome::Unchanged => "Up to date".to_owned(),
    }
}

pub(super) fn export_folder_name(now: &Zoned) -> String {
    now.strftime("Bourbon export %Y-%m-%d %H%M").to_string()
}

impl Workspace {
    pub(crate) fn restore_question(&self) -> Option<String> {
        let repository = self.settings.backup_repository.as_deref()?;
        self.data.restore_confirmation.then(|| {
            format!(
                "Replace all notes with the backup from {repository}? Your current notes are saved to {BEFORE_RESTORE_FILE}."
            )
        })
    }

    fn report_data(&mut self, message: String, cx: &mut Context<Self>) {
        self.data.status = Some(message.clone().into());
        self.error_message = Some(message.into());
        cx.notify();
    }

    fn backup_job(&mut self, cx: &mut Context<Self>) -> Option<BackupJob> {
        let Some(text) = self.settings.backup_repository.clone() else {
            self.report_data("Set a backup repository in Settings first".to_owned(), cx);
            return None;
        };
        match GitHubRepository::parse(&text) {
            Ok(repository) => Some(BackupJob {
                repository,
                auth: self.settings.backup_auth,
                token_file: self.locations.backup_token.clone(),
                database: self.locations.database.clone(),
                work_dir: self.locations.backup_work_dir.clone(),
            }),
            Err(error) => {
                self.report_data(error.to_string(), cx);
                None
            }
        }
    }

    pub(crate) fn back_up_now(&mut self, cx: &mut Context<Self>) {
        self.flush_then(cx, Self::start_backup);
    }

    fn start_backup(&mut self, cx: &mut Context<Self>) {
        if self.data.running {
            return;
        }
        let Some(job) = self.backup_job(cx) else {
            return;
        };
        self.data.running = true;
        self.data.status = Some("Backing up\u{2026}".into());
        let runner = self.locations.backup_runner.clone();
        self.data.task = Some(cx.spawn(async move |view, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { runner.back_up(&job) })
                .await;
            view.update(cx, |view, cx| view.backup_finished(result, cx))
                .ok();
        }));
        cx.notify();
    }

    fn backup_finished(&mut self, result: Result<BackupOutcome, String>, cx: &mut Context<Self>) {
        self.data.running = false;
        match result {
            Ok(outcome) => {
                self.data.dirty = false;
                self.data.status = Some(success_text(outcome, &Zoned::now()).into());
            }
            Err(message) => self.report_data(message, cx),
        }
        self.close_window_now(cx);
        cx.notify();
    }

    pub(super) fn note_data_changed(&mut self, cx: &mut Context<Self>) {
        self.data.dirty = true;
        if !self.settings.backup_enabled {
            return;
        }
        self.data.timer = Some(cx.spawn(async move |view, cx| {
            cx.background_executor().timer(AUTOMATIC_BACKUP_DELAY).await;
            view.update(cx, Workspace::back_up_automatically).ok();
        }));
    }

    fn back_up_automatically(&mut self, cx: &mut Context<Self>) {
        self.flush_then(cx, Self::start_backup);
    }

    pub(crate) fn request_close(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if self.data.closing.is_some() || !self.flush_before_closing(cx) {
            return false;
        }
        if !(self.settings.backup_enabled && self.data.dirty) {
            return true;
        }
        self.data.closing = Some(window.window_handle());
        self.start_backup(cx);
        if !self.data.running {
            self.data.closing = None;
            return true;
        }
        self.data.close_deadline = Some(cx.spawn(async move |view, cx| {
            cx.background_executor().timer(CLOSE_BACKUP_LIMIT).await;
            view.update(cx, Workspace::close_window_now).ok();
        }));
        false
    }

    fn close_window_now(&mut self, cx: &mut Context<Self>) {
        if let Some(handle) = self.data.closing.take() {
            cx.spawn(async move |_, cx| {
                handle
                    .update(cx, |_, window, _| window.remove_window())
                    .ok();
            })
            .detach();
        }
    }

    pub(crate) fn begin_restore(&mut self, cx: &mut Context<Self>) {
        if self.backup_job(cx).is_some() {
            self.data.restore_confirmation = true;
            cx.notify();
        }
    }

    pub(crate) fn cancel_restore(&mut self, cx: &mut Context<Self>) {
        self.data.restore_confirmation = false;
        cx.notify();
    }

    pub(crate) fn confirm_restore(&mut self, cx: &mut Context<Self>) {
        self.data.restore_confirmation = false;
        if self.data.running {
            self.report_data("A backup is running, try again in a moment".to_owned(), cx);
            return;
        }
        let Some(job) = self.backup_job(cx) else {
            return;
        };
        self.flush_then(cx, |this, cx| this.restore_with(job, cx));
    }

    fn restore_with(&mut self, job: BackupJob, cx: &mut Context<Self>) {
        let placeholder = NoteStore::open_in_memory().and_then(|placeholder| {
            self.keep_copy_of_current_notes(&job.database)?;
            Ok(placeholder)
        });
        let placeholder = match placeholder {
            Ok(placeholder) => placeholder,
            Err(error) => {
                self.report(&error, cx);
                return;
            }
        };
        self.store = placeholder;
        self.selected = None;
        self.data.running = true;
        self.data.status = Some("Restoring\u{2026}".into());
        let runner = self.locations.backup_runner.clone();
        self.data.task = Some(cx.spawn(async move |view, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { runner.restore(&job) })
                .await;
            view.update(cx, |view, cx| view.restore_finished(result, cx))
                .ok();
        }));
        cx.notify();
    }

    fn keep_copy_of_current_notes(&self, database: &Path) -> Result<(), StoreError> {
        let copy = database.with_file_name(BEFORE_RESTORE_FILE);
        if let Some(directory) = copy.parent() {
            std::fs::create_dir_all(directory)?;
        }
        if copy.exists() {
            std::fs::remove_file(&copy)?;
        }
        self.store.snapshot(&copy)
    }

    fn restore_finished(&mut self, result: Result<(), String>, cx: &mut Context<Self>) {
        self.data.running = false;
        match NoteStore::open(&self.locations.database) {
            Ok(store) => self.store = store,
            Err(error) => self.report(&error, cx),
        }
        match result {
            Ok(()) => self.start_over_with_restored_notes(),
            Err(message) => self.report_data(message, cx),
        }
        self.reload_list(cx);
        self.reload_sidebar(cx);
        cx.notify();
    }

    fn start_over_with_restored_notes(&mut self) {
        self.listing.navigation = Navigation::AllNotes;
        self.selected = None;
        self.history = History::default();
        self.sidebar.expanded.clear();
        self.sidebar.inbox = self.store.default_notebook().ok();
        self.data.dirty = false;
        self.data.status = Some("Restored from backup".into());
    }

    pub(crate) fn export_to_markdown(&mut self, cx: &mut Context<Self>) {
        self.notice = None;
        let chosen = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Export here".into()),
        });
        cx.spawn(async move |view, cx| {
            if let Ok(Ok(Some(paths))) = chosen.await
                && let Some(directory) = paths.into_iter().next()
            {
                view.update(cx, |view, cx| {
                    view.export_into(&directory, &Zoned::now(), cx);
                })
                .ok();
            }
        })
        .detach();
    }

    pub(super) fn export_into(&mut self, directory: &Path, now: &Zoned, cx: &mut Context<Self>) {
        self.flush_then(cx, |this, cx| {
            let destination = directory.join(export_folder_name(now));
            match this.store.export_markdown(&destination) {
                Ok(summary) => {
                    this.notice = Some(
                        format!(
                            "Exported {} notes to {}",
                            summary.notes,
                            destination.display()
                        )
                        .into(),
                    );
                    cx.notify();
                }
                Err(error) => this.report(&error, cx),
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;

    use super::*;

    fn afternoon() -> Zoned {
        date(2026, 11, 6).at(14, 3, 9, 0).in_tz("UTC").unwrap()
    }

    #[test]
    fn a_pushed_backup_reports_the_time() {
        assert_eq!(
            success_text(BackupOutcome::Pushed, &afternoon()),
            "Last backup 14:03"
        );
    }

    #[test]
    fn an_unchanged_backup_reports_up_to_date() {
        assert_eq!(
            success_text(BackupOutcome::Unchanged, &afternoon()),
            "Up to date"
        );
    }

    #[test]
    fn the_export_folder_carries_the_local_date_and_minute() {
        assert_eq!(
            export_folder_name(&afternoon()),
            "Bourbon export 2026-11-06 1403"
        );
    }
}
