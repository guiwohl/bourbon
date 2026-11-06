use std::io::{self, Write};
use std::path::Path;

use bourbon_core::{NoteId, NoteStore};

use crate::command::{self, Command};

pub const USAGE: &str = "Usage:\n  bourbon              open the Bourbon app\n  bourbon get <id>     print a note as Markdown\n  bourbon --help       show this help\n  bourbon --version    show the version\n";

const FAILURE: u8 = 1;
const USAGE_ERROR: u8 = 2;

pub fn run(
    arguments: &[String],
    database_path: &Path,
    launch_app: impl FnOnce() -> io::Result<()>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> u8 {
    let outcome = match command::parse(arguments) {
        Command::LaunchApp => {
            launch_app().map_err(|error| (FAILURE, format!("cannot launch bourbon-app: {error}\n")))
        }
        Command::Get(id) => get(&id, database_path, stdout),
        Command::Help => write_text(stdout, USAGE),
        Command::Version => write_text(stdout, &format!("bourbon {}\n", env!("CARGO_PKG_VERSION"))),
        Command::Unknown => Err((USAGE_ERROR, USAGE.to_owned())),
    };
    match outcome {
        Ok(()) => 0,
        Err((status, message)) => {
            let _ = stderr.write_all(message.as_bytes());
            status
        }
    }
}

type Outcome = Result<(), (u8, String)>;

fn write_text(stdout: &mut impl Write, text: &str) -> Outcome {
    stdout
        .write_all(text.as_bytes())
        .map_err(|error| (FAILURE, format!("cannot write output: {error}\n")))
}

fn get(id: &str, database_path: &Path, stdout: &mut impl Write) -> Outcome {
    let id = NoteId::parse(id).map_err(|_| (USAGE_ERROR, format!("invalid note id: {id}\n")))?;
    let store = NoteStore::open(database_path)
        .map_err(|error| (FAILURE, format!("cannot open database: {error}\n")))?;
    let markdown = store
        .note_markdown(&id)
        .map_err(|error| (FAILURE, format!("cannot read note: {error}\n")))?
        .ok_or_else(|| (FAILURE, format!("note not found: {id}\n")))?;
    write_text(stdout, &markdown)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bourbon_core::Timestamp;

    struct Invocation {
        status: u8,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    }

    fn invoke(arguments: &[&str], database: &Path) -> Invocation {
        invoke_launching(arguments, database, || Ok(()))
    }

    fn invoke_launching(
        arguments: &[&str],
        database: &Path,
        launch: impl FnOnce() -> io::Result<()>,
    ) -> Invocation {
        let owned: Vec<String> = arguments
            .iter()
            .map(|argument| (*argument).to_owned())
            .collect();
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        let status = run(&owned, database, launch, &mut stdout, &mut stderr);
        Invocation {
            status,
            stdout,
            stderr,
        }
    }

    fn database_with_note(
        directory: &tempfile::TempDir,
    ) -> (std::path::PathBuf, NoteStore, NoteId) {
        let path = directory.path().join("bourbon.db");
        let store = NoteStore::open(&path).unwrap();
        let inbox = store.default_notebook().unwrap();
        let note = store
            .create_note(&inbox, Timestamp::from_millis(1))
            .unwrap();
        store
            .save_note(&note.id, "Hello", "body\n", Timestamp::from_millis(2))
            .unwrap();
        (path, store, note.id)
    }

    #[test]
    fn get_prints_exactly_the_exported_markdown() {
        let directory = tempfile::tempdir().unwrap();
        let (path, store, id) = database_with_note(&directory);

        let outcome = invoke(&["get", id.as_str()], &path);

        assert_eq!(outcome.status, 0);
        assert_eq!(
            outcome.stdout,
            store.note_markdown(&id).unwrap().unwrap().into_bytes()
        );
        assert!(
            String::from_utf8(outcome.stdout)
                .unwrap()
                .ends_with("---\n\nbody\n")
        );
        assert_eq!(outcome.stderr, b"");
    }

    #[test]
    fn get_works_for_templates() {
        let directory = tempfile::tempdir().unwrap();
        let (path, store, id) = database_with_note(&directory);
        store.mark_as_template(&id).unwrap();

        let outcome = invoke(&["get", id.as_str()], &path);

        assert_eq!(outcome.status, 0);
        assert!(
            String::from_utf8(outcome.stdout)
                .unwrap()
                .contains("title: \"Hello\"")
        );
    }

    #[test]
    fn get_of_an_unknown_note_fails_on_stderr() {
        let directory = tempfile::tempdir().unwrap();
        let (path, ..) = database_with_note(&directory);

        let outcome = invoke(&["get", "abcdefghijkl"], &path);

        assert_eq!(outcome.status, 1);
        assert_eq!(outcome.stdout, b"");
        assert_eq!(outcome.stderr, b"note not found: abcdefghijkl\n");
    }

    #[test]
    fn get_of_a_trashed_note_fails_on_stderr() {
        let directory = tempfile::tempdir().unwrap();
        let (path, store, id) = database_with_note(&directory);
        store.trash_note(&id, Timestamp::from_millis(3)).unwrap();

        let outcome = invoke(&["get", id.as_str()], &path);

        assert_eq!(outcome.status, 1);
        assert_eq!(outcome.stdout, b"");
        assert_eq!(
            outcome.stderr,
            format!("note not found: {id}\n").into_bytes()
        );
    }

    #[test]
    fn get_of_a_malformed_id_is_a_usage_error() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("bourbon.db");

        let outcome = invoke(&["get", "NOT-AN-ID"], &path);

        assert_eq!(outcome.status, 2);
        assert_eq!(outcome.stdout, b"");
        assert_eq!(outcome.stderr, b"invalid note id: NOT-AN-ID\n");
    }

    #[test]
    fn get_with_an_unopenable_database_fails() {
        let directory = tempfile::tempdir().unwrap();
        let blocked = directory.path().join("file");
        std::fs::write(&blocked, "").unwrap();

        let outcome = invoke(&["get", "abcdefghijkl"], &blocked.join("bourbon.db"));

        assert_eq!(outcome.status, 1);
        assert_eq!(outcome.stdout, b"");
        assert!(
            String::from_utf8(outcome.stderr)
                .unwrap()
                .starts_with("cannot open database: ")
        );
    }

    #[test]
    fn help_prints_usage_on_stdout() {
        let directory = tempfile::tempdir().unwrap();

        let outcome = invoke(&["--help"], &directory.path().join("x.db"));

        assert_eq!(outcome.status, 0);
        assert_eq!(outcome.stdout, USAGE.as_bytes());
        assert_eq!(outcome.stderr, b"");
    }

    #[test]
    fn version_prints_the_package_version() {
        let directory = tempfile::tempdir().unwrap();

        let outcome = invoke(&["-V"], &directory.path().join("x.db"));

        assert_eq!(outcome.status, 0);
        assert_eq!(
            outcome.stdout,
            format!("bourbon {}\n", env!("CARGO_PKG_VERSION")).into_bytes()
        );
    }

    #[test]
    fn unknown_commands_print_usage_on_stderr() {
        let directory = tempfile::tempdir().unwrap();

        let outcome = invoke(&["bogus"], &directory.path().join("x.db"));

        assert_eq!(outcome.status, 2);
        assert_eq!(outcome.stdout, b"");
        assert_eq!(outcome.stderr, USAGE.as_bytes());
    }

    #[test]
    fn no_arguments_launch_the_app_silently() {
        let directory = tempfile::tempdir().unwrap();
        let launched = std::cell::Cell::new(false);

        let outcome = invoke_launching(&[], &directory.path().join("x.db"), || {
            launched.set(true);
            Ok(())
        });

        assert!(launched.get());
        assert_eq!(outcome.status, 0);
        assert_eq!(outcome.stdout, b"");
        assert_eq!(outcome.stderr, b"");
    }

    #[test]
    fn a_failed_launch_reports_on_stderr() {
        let directory = tempfile::tempdir().unwrap();

        let outcome = invoke_launching(&[], &directory.path().join("x.db"), || {
            Err(io::Error::new(io::ErrorKind::NotFound, "missing"))
        });

        assert_eq!(outcome.status, 1);
        assert_eq!(outcome.stdout, b"");
        assert_eq!(outcome.stderr, b"cannot launch bourbon-app: missing\n");
    }
}
