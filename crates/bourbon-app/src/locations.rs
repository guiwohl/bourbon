use std::path::PathBuf;
use std::sync::Arc;

use bourbon_core::{SettingsFile, TokenFile, default_database_path, default_omarchy_colors_path};

use crate::backup_runner::{BackupRunner, GitBackupRunner};

pub(crate) struct Locations {
    pub(crate) settings: SettingsFile,
    pub(crate) omarchy_colors: PathBuf,
    pub(crate) database: PathBuf,
    pub(crate) backup_token: TokenFile,
    pub(crate) backup_work_dir: PathBuf,
    pub(crate) backup_runner: Arc<dyn BackupRunner>,
}

impl Locations {
    pub(crate) fn of_the_user() -> Self {
        Self {
            settings: SettingsFile::default_location(),
            omarchy_colors: default_omarchy_colors_path(),
            database: default_database_path(),
            backup_token: TokenFile::default_location(),
            backup_work_dir: default_backup_work_dir(),
            backup_runner: Arc::new(GitBackupRunner::github()),
        }
    }
}

fn default_backup_work_dir() -> PathBuf {
    resolve_backup_work_dir(
        std::env::var("XDG_CACHE_HOME").ok().as_deref(),
        std::env::var("HOME").ok().as_deref(),
    )
}

fn resolve_backup_work_dir(xdg_cache_home: Option<&str>, home: Option<&str>) -> PathBuf {
    let cache_directory = match (xdg_cache_home, home) {
        (Some(xdg), _) if !xdg.is_empty() => PathBuf::from(xdg),
        (_, Some(home)) => PathBuf::from(home).join(".cache"),
        _ => PathBuf::from(".cache"),
    };
    cache_directory.join("bourbon").join("backup")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xdg_cache_home_wins() {
        assert_eq!(
            resolve_backup_work_dir(Some("/x/cache"), Some("/home/u")),
            PathBuf::from("/x/cache/bourbon/backup")
        );
    }

    #[test]
    fn falls_back_to_home_dot_cache() {
        assert_eq!(
            resolve_backup_work_dir(None, Some("/home/u")),
            PathBuf::from("/home/u/.cache/bourbon/backup")
        );
        assert_eq!(
            resolve_backup_work_dir(Some(""), Some("/home/u")),
            PathBuf::from("/home/u/.cache/bourbon/backup")
        );
    }

    #[test]
    fn without_any_environment_the_path_is_relative() {
        assert_eq!(
            resolve_backup_work_dir(None, None),
            PathBuf::from(".cache/bourbon/backup")
        );
    }
}
