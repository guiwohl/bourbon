use std::fmt;
use std::process::{Command, Output};

use super::error::BackupError;

#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn from_gh_cli() -> Result<Self, BackupError> {
        let output = Command::new("gh")
            .args(["auth", "token"])
            .output()
            .map_err(|error| BackupError::GhCliUnavailable(error.to_string()))?;
        Self::from_gh_output(&output)
    }

    fn from_gh_output(output: &Output) -> Result<Self, BackupError> {
        if !output.status.success() {
            return Err(BackupError::GhCliUnavailable(
                "gh auth token exited with an error".to_owned(),
            ));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let token = stdout.trim();
        if token.is_empty() {
            Err(BackupError::GhCliUnavailable(
                "gh printed no token".to_owned(),
            ))
        } else {
            Ok(Self::new(token))
        }
    }

    pub(super) fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Secret(<redacted>)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_never_prints_the_value() {
        let secret = Secret::new("ghp_supersecret");
        assert_eq!(format!("{secret:?}"), "Secret(<redacted>)");
        assert!(!format!("{secret:#?}").contains("supersecret"));
    }

    #[test]
    fn expose_returns_the_value() {
        assert_eq!(Secret::new("abc").expose(), "abc");
    }

    #[cfg(unix)]
    fn gh_output(exit_code: i32, stdout: &str) -> Output {
        use std::os::unix::process::ExitStatusExt;

        Output {
            status: std::process::ExitStatus::from_raw(exit_code << 8),
            stdout: stdout.as_bytes().to_vec(),
            stderr: Vec::new(),
        }
    }

    #[cfg(unix)]
    #[test]
    fn gh_output_is_trimmed() {
        let secret = Secret::from_gh_output(&gh_output(0, "  gho_abc123\n")).unwrap();
        assert_eq!(secret.expose(), "gho_abc123");
    }

    #[cfg(unix)]
    #[test]
    fn empty_gh_output_is_an_error() {
        assert!(matches!(
            Secret::from_gh_output(&gh_output(0, " \n")),
            Err(BackupError::GhCliUnavailable(_))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn a_failing_gh_is_an_error_even_if_it_printed_something() {
        assert!(matches!(
            Secret::from_gh_output(&gh_output(1, "gho_abc123")),
            Err(BackupError::GhCliUnavailable(_))
        ));
    }
}
