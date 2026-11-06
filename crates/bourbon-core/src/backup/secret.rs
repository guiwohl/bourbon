use std::fmt;
use std::process::Command;

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
        if !output.status.success() {
            return Err(BackupError::GhCliUnavailable(
                "gh auth token exited with an error".to_owned(),
            ));
        }
        Self::from_gh_output(&String::from_utf8_lossy(&output.stdout))
    }

    fn from_gh_output(stdout: &str) -> Result<Self, BackupError> {
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

    #[test]
    fn gh_output_is_trimmed() {
        let secret = Secret::from_gh_output("  gho_abc123\n").unwrap();
        assert_eq!(secret.expose(), "gho_abc123");
    }

    #[test]
    fn empty_gh_output_is_an_error() {
        assert!(matches!(
            Secret::from_gh_output(" \n"),
            Err(BackupError::GhCliUnavailable(_))
        ));
    }
}
