mod error;
mod git;
mod last_push;
mod remote;
mod repository;
mod secret;
#[cfg(test)]
mod tests;

use std::fs;
use std::path::Path;

use crate::store::{NoteStore, verify_integrity};

pub use error::BackupError;
pub use remote::{BackupRemote, RemoteUrl};
pub use repository::GitHubRepository;
pub use secret::Secret;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupOutcome {
    Pushed,
    Unchanged,
}

const DATABASE_FILE: &str = "bourbon.db";
const BACKUP_BRANCH: &str = "main";

pub fn backup(
    store: &NoteStore,
    remote: &BackupRemote,
    work_dir: &Path,
) -> Result<BackupOutcome, BackupError> {
    fs::create_dir_all(work_dir)?;
    let staging = tempfile::tempdir_in(work_dir)?;
    store.snapshot(&staging.path().join(DATABASE_FILE))?;
    let fingerprint = last_push::fingerprint_of(&staging.path().join(DATABASE_FILE))?;
    if last_push::recorded(work_dir).as_deref() == Some(fingerprint.as_str()) {
        return Ok(BackupOutcome::Unchanged);
    }
    git::run(staging.path(), None, &["init", "-b", BACKUP_BRANCH])?;
    git::run(staging.path(), None, &["add", DATABASE_FILE])?;
    git::run(staging.path(), None, &["commit", "-m", "Bourbon backup"])?;
    let url = remote.url.as_url();
    let target = format!("HEAD:refs/heads/{BACKUP_BRANCH}");
    git::run(
        staging.path(),
        Some(remote),
        &["push", "--force", &url, &target],
    )?;
    last_push::record(work_dir, &fingerprint)?;
    Ok(BackupOutcome::Pushed)
}

pub fn restore(
    remote: &BackupRemote,
    destination_db: &Path,
    work_dir: &Path,
) -> Result<(), BackupError> {
    fs::create_dir_all(work_dir)?;
    let staging = tempfile::tempdir_in(work_dir)?;
    let url = remote.url.as_url();
    git::run(
        staging.path(),
        Some(remote),
        &[
            "clone",
            "--depth",
            "1",
            "--branch",
            BACKUP_BRANCH,
            &url,
            "fetched",
        ],
    )?;
    let fetched = staging.path().join("fetched").join(DATABASE_FILE);
    verify_integrity(&fetched)?;
    let parent = match destination_db.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    fs::create_dir_all(parent)?;
    let replacement = tempfile::NamedTempFile::new_in(parent)?;
    fs::copy(&fetched, replacement.path())?;
    replacement
        .persist(destination_db)
        .map_err(|error| error.error)?;
    Ok(())
}
