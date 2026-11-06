use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use super::*;
use crate::{NoteQuery, NotebookId, Timestamp};

const TOKEN: &str = "ghp_s3cr3tT0ken";

struct Sandbox {
    root: TempDir,
}

impl Sandbox {
    fn new() -> Self {
        Self {
            root: tempfile::tempdir().unwrap(),
        }
    }

    fn bare_remote(&self) -> BackupRemote {
        let path = self.root.path().join("remote.git");
        git::run(
            self.root.path(),
            None,
            &["init", "--bare", "-b", "main", "remote.git"],
        )
        .unwrap();
        Self::remote_at(&path)
    }

    fn remote_at(path: &Path) -> BackupRemote {
        BackupRemote {
            url: RemoteUrl::local_file(path),
            token: Secret::new(TOKEN),
        }
    }

    fn work_dir(&self) -> PathBuf {
        self.root.path().join("work")
    }

    fn remote_git(&self, arguments: &[&str]) -> String {
        let mut full = vec!["--git-dir", "remote.git"];
        full.extend_from_slice(arguments);
        git::run(self.root.path(), None, &full).unwrap()
    }
}

fn store_with_note(title: &str) -> NoteStore {
    let store = NoteStore::open_in_memory().unwrap();
    add_note(&store, title);
    store
}

fn add_note(store: &NoteStore, title: &str) {
    let notebook: NotebookId = store.default_notebook().unwrap();
    let note = store
        .create_note(&notebook, Timestamp::from_millis(1))
        .unwrap();
    store
        .save_note(&note.id, title, "", Timestamp::from_millis(2))
        .unwrap();
}

fn titles(store: &NoteStore) -> Vec<String> {
    store
        .list_notes(&NoteQuery::default())
        .unwrap()
        .into_iter()
        .map(|summary| summary.title)
        .collect()
}

#[test]
fn a_backup_pushes_one_commit_holding_only_the_database() {
    let sandbox = Sandbox::new();
    let remote = sandbox.bare_remote();
    let store = store_with_note("Hello");

    let outcome = backup(&store, &remote, &sandbox.work_dir()).unwrap();

    assert_eq!(outcome, BackupOutcome::Pushed);
    assert_eq!(
        sandbox.remote_git(&["rev-list", "--count", "main"]).trim(),
        "1"
    );
    assert_eq!(
        sandbox.remote_git(&["ls-tree", "-r", "--name-only", "main"]),
        "bourbon.db\n"
    );
    assert_eq!(
        sandbox
            .remote_git(&["log", "-1", "--format=%an <%ae>", "main"])
            .trim(),
        "Bourbon <bourbon@localhost>"
    );
}

#[test]
fn a_backup_leaves_only_the_state_file_in_the_work_dir() {
    let sandbox = Sandbox::new();
    let remote = sandbox.bare_remote();
    backup(&store_with_note("Hello"), &remote, &sandbox.work_dir()).unwrap();

    let entries: Vec<_> = fs::read_dir(sandbox.work_dir())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(entries, ["last-pushed.sha256"]);
}

#[test]
fn an_unchanged_database_is_not_pushed_again() {
    let sandbox = Sandbox::new();
    let remote = sandbox.bare_remote();
    let store = store_with_note("Hello");
    backup(&store, &remote, &sandbox.work_dir()).unwrap();
    let pushed_commit = sandbox.remote_git(&["rev-parse", "main"]);

    let outcome = backup(&store, &remote, &sandbox.work_dir()).unwrap();

    assert_eq!(outcome, BackupOutcome::Unchanged);
    assert_eq!(sandbox.remote_git(&["rev-parse", "main"]), pushed_commit);
}

#[test]
fn a_changed_database_replaces_the_single_commit() {
    let sandbox = Sandbox::new();
    let remote = sandbox.bare_remote();
    let store = store_with_note("Hello");
    backup(&store, &remote, &sandbox.work_dir()).unwrap();
    let first_commit = sandbox.remote_git(&["rev-parse", "main"]);
    add_note(&store, "Another");

    let outcome = backup(&store, &remote, &sandbox.work_dir()).unwrap();

    assert_eq!(outcome, BackupOutcome::Pushed);
    assert_ne!(sandbox.remote_git(&["rev-parse", "main"]), first_commit);
    assert_eq!(
        sandbox.remote_git(&["rev-list", "--count", "main"]).trim(),
        "1"
    );
}

#[test]
fn a_failed_push_is_not_recorded_so_the_next_backup_retries() {
    let sandbox = Sandbox::new();
    let missing = Sandbox::remote_at(&sandbox.root.path().join("missing.git"));
    let store = store_with_note("Hello");
    assert!(backup(&store, &missing, &sandbox.work_dir()).is_err());

    let remote = sandbox.bare_remote();
    let outcome = backup(&store, &remote, &sandbox.work_dir()).unwrap();

    assert_eq!(outcome, BackupOutcome::Pushed);
}

#[test]
fn a_damaged_database_is_never_pushed() {
    let sandbox = Sandbox::new();
    let remote = sandbox.bare_remote();
    let live = sandbox.root.path().join("live.db");
    {
        let store = NoteStore::open(&live).unwrap();
        for index in 0..50 {
            add_note(&store, &format!("Note {index}"));
        }
    }
    let mut bytes = fs::read(&live).unwrap();
    for byte in &mut bytes[4096..12288] {
        *byte = 0xA5;
    }
    fs::write(&live, bytes).unwrap();
    let damaged = NoteStore::open(&live).unwrap();

    let result = backup(&damaged, &remote, &sandbox.work_dir());

    assert!(result.is_err());
    assert_eq!(sandbox.remote_git(&["for-each-ref"]), "");
    assert_eq!(last_push::recorded(&sandbox.work_dir()), None);
}

#[test]
fn restore_brings_back_the_backed_up_notes() {
    let sandbox = Sandbox::new();
    let remote = sandbox.bare_remote();
    backup(&store_with_note("Precious"), &remote, &sandbox.work_dir()).unwrap();
    let destination = sandbox.root.path().join("restored/bourbon.db");

    restore(&remote, &destination, &sandbox.work_dir()).unwrap();

    assert_eq!(
        titles(&NoteStore::open(&destination).unwrap()),
        ["Precious"]
    );
}

#[test]
fn restore_replaces_an_existing_database() {
    let sandbox = Sandbox::new();
    let remote = sandbox.bare_remote();
    backup(&store_with_note("Remote"), &remote, &sandbox.work_dir()).unwrap();
    let destination = sandbox.root.path().join("bourbon.db");
    drop(store_with_note("Local").snapshot(&destination));

    restore(&remote, &destination, &sandbox.work_dir()).unwrap();

    assert_eq!(titles(&NoteStore::open(&destination).unwrap()), ["Remote"]);
}

#[test]
fn restore_from_an_unreachable_remote_leaves_the_destination_intact() {
    let sandbox = Sandbox::new();
    let destination = sandbox.root.path().join("bourbon.db");
    fs::write(&destination, "mine").unwrap();
    let missing = Sandbox::remote_at(&sandbox.root.path().join("missing.git"));

    assert!(restore(&missing, &destination, &sandbox.work_dir()).is_err());

    assert_eq!(fs::read_to_string(&destination).unwrap(), "mine");
}

#[test]
fn restore_of_a_corrupted_backup_leaves_the_destination_intact() {
    let sandbox = Sandbox::new();
    let remote = sandbox.bare_remote();
    let seed = sandbox.root.path().join("seed");
    fs::create_dir(&seed).unwrap();
    fs::write(seed.join("bourbon.db"), vec![0x42; 8192]).unwrap();
    git::run(&seed, None, &["init", "-b", "main"]).unwrap();
    git::run(&seed, None, &["add", "bourbon.db"]).unwrap();
    git::run(&seed, None, &["commit", "-m", "bad"]).unwrap();
    git::run(
        &seed,
        None,
        &["push", &remote.url.as_url(), "HEAD:refs/heads/main"],
    )
    .unwrap();
    let destination = sandbox.root.path().join("bourbon.db");
    fs::write(&destination, "mine").unwrap();

    assert!(restore(&remote, &destination, &sandbox.work_dir()).is_err());

    assert_eq!(fs::read_to_string(&destination).unwrap(), "mine");
    let siblings: Vec<_> = fs::read_dir(sandbox.root.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with(".tmp"))
        .collect();
    assert_eq!(siblings, Vec::<String>::new());
}

#[test]
fn restore_from_a_repository_without_the_database_leaves_the_destination_intact() {
    let sandbox = Sandbox::new();
    let remote = sandbox.bare_remote();
    let seed = sandbox.root.path().join("seed");
    fs::create_dir(&seed).unwrap();
    fs::write(seed.join("other.txt"), "x").unwrap();
    git::run(&seed, None, &["init", "-b", "main"]).unwrap();
    git::run(&seed, None, &["add", "other.txt"]).unwrap();
    git::run(&seed, None, &["commit", "-m", "other"]).unwrap();
    git::run(
        &seed,
        None,
        &["push", &remote.url.as_url(), "HEAD:refs/heads/main"],
    )
    .unwrap();
    let destination = sandbox.root.path().join("bourbon.db");
    fs::write(&destination, "mine").unwrap();

    assert!(restore(&remote, &destination, &sandbox.work_dir()).is_err());

    assert_eq!(fs::read_to_string(&destination).unwrap(), "mine");
}

#[test]
fn errors_never_contain_the_token() {
    let sandbox = Sandbox::new();
    let missing = Sandbox::remote_at(&sandbox.root.path().join("missing.git"));
    let store = store_with_note("Hello");
    let destination = sandbox.root.path().join("bourbon.db");

    let backup_error = backup(&store, &missing, &sandbox.work_dir()).unwrap_err();
    let restore_error = restore(&missing, &destination, &sandbox.work_dir()).unwrap_err();

    for error in [backup_error, restore_error] {
        let text = format!("{error} {error:?}");
        assert!(!text.contains(TOKEN), "{text}");
        assert!(!text.contains("eC1hY2Nlc3MtdG9rZW"), "{text}");
    }
}
