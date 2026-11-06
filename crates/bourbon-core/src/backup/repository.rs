use std::fmt;

use super::error::BackupError;

const MAX_OWNER_LENGTH: usize = 39;
const MAX_NAME_LENGTH: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubRepository {
    owner: String,
    name: String,
}

impl GitHubRepository {
    pub fn parse(text: &str) -> Result<Self, BackupError> {
        let invalid = || BackupError::InvalidRepository(text.to_owned());
        let (owner, name) = text.split_once('/').ok_or_else(invalid)?;
        let owner_is_valid = !owner.is_empty()
            && owner.len() <= MAX_OWNER_LENGTH
            && owner
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '-');
        let name_is_valid = !name.is_empty()
            && name.len() <= MAX_NAME_LENGTH
            && !matches!(name, "." | "..")
            && name.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
            });
        if owner_is_valid && name_is_valid {
            Ok(Self {
                owner: owner.to_owned(),
                name: name.to_owned(),
            })
        } else {
            Err(invalid())
        }
    }

    pub(super) fn https_url(&self) -> String {
        format!("https://github.com/{self}.git")
    }
}

impl fmt::Display for GitHubRepository {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}/{}", self.owner, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_owner_and_name() {
        let repository = GitHubRepository::parse("guiwohl/bourbon-backup").unwrap();
        assert_eq!(repository.to_string(), "guiwohl/bourbon-backup");
        assert_eq!(
            repository.https_url(),
            "https://github.com/guiwohl/bourbon-backup.git"
        );
    }

    #[test]
    fn accepts_dots_and_underscores_in_the_name() {
        assert!(GitHubRepository::parse("a/my_repo.v2").is_ok());
    }

    #[test]
    fn rejects_malformed_repositories() {
        for text in [
            "",
            "owner",
            "/name",
            "owner/",
            "a/b/c",
            "own er/name",
            "owner/na me",
            "owner/..",
            "owner/.",
            "ow_ner/name",
            "owner/na\"me",
            "owner/na@me",
            "https://github.com/a/b",
        ] {
            assert!(
                matches!(
                    GitHubRepository::parse(text),
                    Err(BackupError::InvalidRepository(_))
                ),
                "{text} should be rejected"
            );
        }
    }

    #[test]
    fn enforces_github_length_limits() {
        assert!(GitHubRepository::parse(&format!("{}/n", "o".repeat(39))).is_ok());
        assert!(GitHubRepository::parse(&format!("{}/n", "o".repeat(40))).is_err());
        assert!(GitHubRepository::parse(&format!("o/{}", "n".repeat(100))).is_ok());
        assert!(GitHubRepository::parse(&format!("o/{}", "n".repeat(101))).is_err());
    }
}
