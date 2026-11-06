use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use super::repository::GitHubRepository;
use super::secret::Secret;

const REDACTION: &str = "<redacted>";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteUrl {
    GitHub(GitHubRepository),
    LocalFile(PathBuf),
}

impl RemoteUrl {
    #[must_use]
    pub fn local_file(path: &Path) -> Self {
        Self::LocalFile(path.to_path_buf())
    }

    pub(super) fn as_url(&self) -> String {
        match self {
            Self::GitHub(repository) => repository.https_url(),
            Self::LocalFile(path) => format!("file://{}", path.display()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackupRemote {
    pub url: RemoteUrl,
    pub token: Secret,
}

impl BackupRemote {
    #[must_use]
    pub fn github(repository: GitHubRepository, token: Secret) -> Self {
        Self {
            url: RemoteUrl::GitHub(repository),
            token,
        }
    }

    pub(super) fn authentication_arguments(&self) -> [String; 2] {
        [
            "-c".to_owned(),
            format!(
                "http.extraHeader=Authorization: Basic {}",
                self.encoded_credentials()
            ),
        ]
    }

    pub(super) fn scrubbed(&self, text: &str) -> String {
        let token = self.token.expose();
        let without_token = if token.is_empty() {
            text.to_owned()
        } else {
            text.replace(token, REDACTION)
        };
        without_token.replace(&self.encoded_credentials(), REDACTION)
    }

    fn encoded_credentials(&self) -> String {
        STANDARD.encode(format!("x-access-token:{}", self.token.expose()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn remote(token: &str) -> BackupRemote {
        BackupRemote::github(GitHubRepository::parse("o/r").unwrap(), Secret::new(token))
    }

    #[test]
    fn github_remote_uses_an_https_url_without_credentials() {
        let url = remote("tok").url.as_url();
        assert_eq!(url, "https://github.com/o/r.git");
        assert!(!url.contains("tok"));
    }

    #[test]
    fn local_remote_uses_a_file_url() {
        let url = RemoteUrl::local_file(Path::new("/tmp/x.git")).as_url();
        let expected_rest = format!("{}tmp/x.git", "/".repeat(3));
        assert_eq!(url.strip_prefix("file:"), Some(expected_rest.as_str()));
    }

    #[test]
    fn authentication_is_a_basic_extra_header() {
        let [flag, header] = remote("tok").authentication_arguments();
        assert_eq!(flag, "-c");
        assert_eq!(
            header,
            "http.extraHeader=Authorization: Basic eC1hY2Nlc3MtdG9rZW46dG9r"
        );
    }

    #[test]
    fn scrubbing_hides_the_token_and_its_encoded_form() {
        let remote = remote("s3cr3t");
        let encoded = STANDARD.encode("x-access-token:s3cr3t");
        let leaked = format!("fatal: bad s3cr3t and header Basic {encoded} end");
        let scrubbed = remote.scrubbed(&leaked);
        assert_eq!(
            scrubbed,
            "fatal: bad <redacted> and header Basic <redacted> end"
        );
    }

    #[test]
    fn an_empty_token_does_not_garble_the_text() {
        assert_eq!(remote("").scrubbed("abc"), "abc");
    }

    #[test]
    fn scrubbing_leaves_other_text_alone() {
        assert_eq!(remote("tok").scrubbed("nothing here"), "nothing here");
    }
}
