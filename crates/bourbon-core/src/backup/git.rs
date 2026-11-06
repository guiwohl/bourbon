use std::io;
use std::path::Path;
use std::process::Command;

use super::error::BackupError;
use super::remote::BackupRemote;

const COMMITTER_NAME: &str = "Bourbon";
const COMMITTER_EMAIL: &str = "bourbon@localhost";

pub fn run(
    directory: &Path,
    remote: Option<&BackupRemote>,
    arguments: &[&str],
) -> Result<String, BackupError> {
    let mut command = Command::new("git");
    command
        .current_dir(directory)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", COMMITTER_NAME)
        .env("GIT_AUTHOR_EMAIL", COMMITTER_EMAIL)
        .env("GIT_COMMITTER_NAME", COMMITTER_NAME)
        .env("GIT_COMMITTER_EMAIL", COMMITTER_EMAIL);
    if let Some(remote) = remote {
        command.args(remote.authentication_arguments());
    }
    let output = command.args(arguments).output().map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            BackupError::GitNotInstalled
        } else {
            BackupError::Io(error)
        }
    })?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let message = remote.map_or_else(|| stderr.to_string(), |remote| remote.scrubbed(&stderr));
    Err(BackupError::GitFailed {
        operation: arguments.first().copied().unwrap_or_default().to_owned(),
        message: message.trim().to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_commands_return_their_output() {
        let directory = tempfile::tempdir().unwrap();
        let output = run(directory.path(), None, &["--version"]).unwrap();
        assert!(output.starts_with("git version"));
    }

    #[test]
    fn failures_name_the_operation_and_carry_stderr() {
        let directory = tempfile::tempdir().unwrap();
        let error = run(directory.path(), None, &["rev-parse", "HEAD"]).unwrap_err();
        match error {
            BackupError::GitFailed { operation, message } => {
                assert_eq!(operation, "rev-parse");
                assert_ne!(message, "");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn commits_use_the_fixed_bourbon_identity() {
        let directory = tempfile::tempdir().unwrap();
        run(directory.path(), None, &["init", "-b", "main"]).unwrap();
        run(
            directory.path(),
            None,
            &["commit", "--allow-empty", "-m", "x"],
        )
        .unwrap();
        let author = run(
            directory.path(),
            None,
            &["log", "-1", "--format=%an <%ae> | %cn <%ce>"],
        )
        .unwrap();
        assert_eq!(
            author.trim(),
            "Bourbon <bourbon@localhost> | Bourbon <bourbon@localhost>"
        );
    }
}
