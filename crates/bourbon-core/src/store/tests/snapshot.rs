use std::fs;
use std::path::Path;

use super::support::{inbox, store, titled_note};
use crate::store::verify_integrity;
use crate::{NoteQuery, NoteStore, StoreError};

fn titles(store: &NoteStore) -> Vec<String> {
    store
        .list_notes(&NoteQuery::default())
        .unwrap()
        .into_iter()
        .map(|summary| summary.title)
        .collect()
}

fn database_with_an_index_that_disagrees_with_its_table(path: &Path) {
    let connection = rusqlite::Connection::open(path).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE t (a TEXT, b TEXT);
             CREATE INDEX i ON t (b);
             INSERT INTO t VALUES ('aaaaaaaa', 'bbbbbbbb');",
        )
        .unwrap();
    drop(connection);
    let mut bytes = fs::read(path).unwrap();
    let position = bytes
        .windows(8)
        .position(|window| window == b"bbbbbbbb")
        .unwrap();
    bytes[position..position + 8].copy_from_slice(b"cccccccc");
    fs::write(path, bytes).unwrap();
}

#[test]
fn a_snapshot_holds_the_same_notes_as_the_store() {
    let store = store();
    let notebook = inbox(&store);
    titled_note(&store, &notebook, "First", 1, 1);
    titled_note(&store, &notebook, "Second", 2, 2);
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("copy.db");

    store.snapshot(&target).unwrap();

    let copy = NoteStore::open(&target).unwrap();
    assert_eq!(titles(&copy), titles(&store));
    assert_eq!(titles(&copy).len(), 2);
    assert_eq!(copy.default_notebook().unwrap(), notebook);
}

#[test]
fn a_snapshot_of_a_file_store_is_independent_of_later_writes() {
    let directory = tempfile::tempdir().unwrap();
    let store = NoteStore::open(&directory.path().join("live.db")).unwrap();
    titled_note(&store, &inbox(&store), "Before", 1, 1);
    let target = directory.path().join("copy.db");
    store.snapshot(&target).unwrap();
    titled_note(&store, &inbox(&store), "After", 2, 2);

    assert_eq!(titles(&NoteStore::open(&target).unwrap()), ["Before"]);
}

#[test]
fn an_existing_target_is_refused_and_kept() {
    let store = store();
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("copy.db");
    fs::write(&target, "precious").unwrap();

    assert!(matches!(
        store.snapshot(&target),
        Err(StoreError::Database(_))
    ));
    assert_eq!(fs::read_to_string(&target).unwrap(), "precious");
}

#[cfg(unix)]
#[test]
fn a_target_path_that_is_not_utf8_is_an_io_error() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let store = store();
    let target = Path::new(OsStr::from_bytes(b"/tmp/\xff.db"));
    assert!(matches!(store.snapshot(target), Err(StoreError::Io(_))));
}

#[test]
fn a_healthy_database_passes_the_integrity_check() {
    let store = store();
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("copy.db");
    store.snapshot(&target).unwrap();
    verify_integrity(&target).unwrap();
}

#[test]
fn an_inconsistent_database_fails_the_integrity_check() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("broken.db");
    database_with_an_index_that_disagrees_with_its_table(&path);

    let error = verify_integrity(&path).unwrap_err();

    assert!(matches!(error, StoreError::IntegrityCheckFailed(report) if report.contains("index")));
}

#[test]
fn a_file_that_is_not_a_database_fails_the_integrity_check() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("garbage.db");
    fs::write(&path, vec![0x42; 8192]).unwrap();
    assert!(verify_integrity(&path).is_err());
}
