use std::path::PathBuf;

use bourbon_core::{
    BackupAuth, BackupOutcome, BackupRemote, GitHubRepository, NoteStore, RemoteUrl, Secret,
    TokenFile, backup, restore,
};

#[derive(Debug, Clone)]
pub(crate) struct BackupJob {
    pub(crate) repository: GitHubRepository,
    pub(crate) auth: BackupAuth,
    pub(crate) token_file: TokenFile,
    pub(crate) database: PathBuf,
    pub(crate) work_dir: PathBuf,
}

pub(crate) trait BackupRunner: Send + Sync {
    fn back_up(&self, job: &BackupJob) -> Result<BackupOutcome, String>;
    fn restore(&self, job: &BackupJob) -> Result<(), String>;
}

pub(crate) struct GitBackupRunner {
    remote_url: Option<RemoteUrl>,
}

impl GitBackupRunner {
    pub(crate) fn github() -> Self {
        Self { remote_url: None }
    }

    #[cfg(test)]
    pub(crate) fn at(remote_url: RemoteUrl) -> Self {
        Self {
            remote_url: Some(remote_url),
        }
    }

    fn remote_for(&self, job: &BackupJob) -> Result<BackupRemote, String> {
        let token = match job.auth {
            BackupAuth::GhCli => Secret::from_gh_cli().map_err(|error| error.to_string())?,
            BackupAuth::Token => job
                .token_file
                .load()
                .ok_or_else(|| "no access token is saved".to_owned())?,
        };
        let url = self
            .remote_url
            .clone()
            .unwrap_or_else(|| RemoteUrl::GitHub(job.repository.clone()));
        Ok(BackupRemote { url, token })
    }
}

impl BackupRunner for GitBackupRunner {
    fn back_up(&self, job: &BackupJob) -> Result<BackupOutcome, String> {
        let remote = self.remote_for(job)?;
        let store = NoteStore::open(&job.database).map_err(|error| error.to_string())?;
        backup(&store, &remote, &job.work_dir).map_err(|error| error.to_string())
    }

    fn restore(&self, job: &BackupJob) -> Result<(), String> {
        let remote = self.remote_for(job)?;
        restore(&remote, &job.database, &job.work_dir).map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use super::*;

    fn git(directory: &Path, arguments: &[&str]) {
        let status = Command::new("git")
            .current_dir(directory)
            .args(arguments)
            .status()
            .unwrap();
        assert!(status.success());
    }

    struct Fixture {
        _root: tempfile::TempDir,
        job: BackupJob,
        runner: GitBackupRunner,
    }

    fn fixture(token: Option<&str>) -> Fixture {
        let root = tempfile::tempdir().unwrap();
        let bare = root.path().join("remote.git");
        std::fs::create_dir(&bare).unwrap();
        git(&bare, &["init", "--bare", "-b", "main"]);
        let token_file = TokenFile::at(root.path().join("token"));
        if let Some(token) = token {
            token_file.save(token).unwrap();
        }
        let job = BackupJob {
            repository: GitHubRepository::parse("owner/notes").unwrap(),
            auth: BackupAuth::Token,
            token_file,
            database: root.path().join("bourbon.db"),
            work_dir: root.path().join("work"),
        };
        Fixture {
            runner: GitBackupRunner::at(RemoteUrl::local_file(&bare)),
            job,
            _root: root,
        }
    }

    fn titles(path: &Path) -> Vec<String> {
        let store = NoteStore::open(path).unwrap();
        store
            .list_notes(&bourbon_core::NoteQuery::default())
            .unwrap()
            .into_iter()
            .map(|summary| summary.title)
            .collect()
    }

    #[test]
    fn a_backup_then_a_restore_round_trips_the_notes() {
        let fixture = fixture(Some("tok"));
        let (job, runner) = (&fixture.job, &fixture.runner);
        let store = NoteStore::open(&job.database).unwrap();
        let inbox = store.default_notebook().unwrap();
        let note = store
            .create_note(&inbox, bourbon_core::Timestamp::from_millis(1))
            .unwrap();
        store
            .save_note(
                &note.id,
                "Kept",
                "",
                bourbon_core::Timestamp::from_millis(2),
            )
            .unwrap();
        drop(store);

        assert_eq!(runner.back_up(job).unwrap(), BackupOutcome::Pushed);
        assert_eq!(runner.back_up(job).unwrap(), BackupOutcome::Unchanged);
        std::fs::remove_file(&job.database).unwrap();
        runner.restore(job).unwrap();

        assert_eq!(titles(&job.database), ["Kept"]);
    }

    #[test]
    fn without_a_saved_token_both_directions_fail_with_a_clear_message() {
        let fixture = fixture(None);
        let (job, runner) = (&fixture.job, &fixture.runner);

        assert_eq!(runner.back_up(job).unwrap_err(), "no access token is saved");
        assert_eq!(runner.restore(job).unwrap_err(), "no access token is saved");
    }
}
