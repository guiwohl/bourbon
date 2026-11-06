use std::process::Command;

use bourbon_core::{NoteStore, Timestamp};

fn bourbon(data_home: &std::path::Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bourbon"))
        .args(arguments)
        .env("XDG_DATA_HOME", data_home)
        .output()
        .unwrap()
}

#[test]
fn get_prints_a_note_of_the_default_database() {
    let data_home = tempfile::tempdir().unwrap();
    let database = data_home.path().join("bourbon/bourbon.db");
    let store = NoteStore::open(&database).unwrap();
    let inbox = store.default_notebook().unwrap();
    let note = store
        .create_note(&inbox, Timestamp::from_millis(1))
        .unwrap();
    store
        .save_note(&note.id, "Hello", "body\n", Timestamp::from_millis(2))
        .unwrap();

    let output = bourbon(data_home.path(), &["get", note.id.as_str()]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        output.stdout,
        store.note_markdown(&note.id).unwrap().unwrap().into_bytes()
    );
    assert_eq!(output.stderr, b"");
}

#[test]
fn a_malformed_id_exits_with_two() {
    let data_home = tempfile::tempdir().unwrap();

    let output = bourbon(data_home.path(), &["get", "nope"]);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, b"");
}

#[test]
fn launching_without_the_app_installed_fails() {
    let isolated = tempfile::tempdir().unwrap();
    let copy = isolated.path().join("bourbon");
    std::fs::copy(env!("CARGO_BIN_EXE_bourbon"), &copy).unwrap();

    let output = Command::new(&copy)
        .env("PATH", isolated.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .starts_with("cannot launch bourbon-app: ")
    );
}
