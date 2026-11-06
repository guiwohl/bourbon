use std::fs;
use std::io::{self, Write as _};
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};

use tempfile::NamedTempFile;

use super::secret::Secret;
use crate::parent_directory::parent_directory_of;
use crate::settings::default_settings_path;

const TOKEN_FILE_NAME: &str = "backup-token";
const OWNER_ONLY: u32 = 0o600;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenFile {
    path: PathBuf,
}

impl TokenFile {
    #[must_use]
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    #[must_use]
    pub fn default_location() -> Self {
        Self::at(default_settings_path().with_file_name(TOKEN_FILE_NAME))
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn save(&self, token: &str) -> io::Result<()> {
        let directory = parent_directory_of(&self.path);
        fs::create_dir_all(directory)?;
        let mut staged = NamedTempFile::new_in(directory)?;
        staged
            .as_file()
            .set_permissions(fs::Permissions::from_mode(OWNER_ONLY))?;
        staged.write_all(token.as_bytes())?;
        staged.persist(&self.path).map_err(|error| error.error)?;
        Ok(())
    }

    #[must_use]
    pub fn load(&self) -> Option<Secret> {
        let text = fs::read_to_string(&self.path).ok()?;
        let token = text.trim();
        (!token.is_empty()).then(|| Secret::new(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token_file(directory: &tempfile::TempDir) -> TokenFile {
        TokenFile::at(directory.path().join("nested").join("backup-token"))
    }

    #[test]
    fn a_saved_token_loads_back() {
        let directory = tempfile::tempdir().unwrap();
        let file = token_file(&directory);

        file.save("ghp_secret").unwrap();

        assert_eq!(file.load(), Some(Secret::new("ghp_secret")));
    }

    #[test]
    fn the_file_is_readable_by_the_owner_only() {
        let directory = tempfile::tempdir().unwrap();
        let file = token_file(&directory);

        file.save("ghp_secret").unwrap();
        file.save("ghp_other").unwrap();

        let mode = fs::metadata(file.path()).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
        assert_eq!(file.load(), Some(Secret::new("ghp_other")));
    }

    #[test]
    fn saving_leaves_no_temporary_files() {
        let directory = tempfile::tempdir().unwrap();
        let file = token_file(&directory);

        file.save("a").unwrap();

        assert_eq!(
            fs::read_dir(file.path().parent().unwrap()).unwrap().count(),
            1
        );
    }

    #[test]
    fn a_missing_or_blank_file_has_no_token() {
        let directory = tempfile::tempdir().unwrap();
        let file = token_file(&directory);
        assert_eq!(file.load(), None);

        file.save(" \n").unwrap();

        assert_eq!(file.load(), None);
    }

    #[test]
    fn surrounding_whitespace_is_trimmed_on_load() {
        let directory = tempfile::tempdir().unwrap();
        let file = token_file(&directory);

        file.save("  tok\n").unwrap();

        assert_eq!(file.load(), Some(Secret::new("tok")));
    }

    #[test]
    fn the_default_location_sits_next_to_the_settings() {
        assert!(
            TokenFile::default_location()
                .path()
                .ends_with("bourbon/backup-token")
        );
    }
}
