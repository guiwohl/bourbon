use std::path::PathBuf;

#[must_use]
pub fn default_database_path() -> PathBuf {
    resolve_database_path(
        std::env::var("XDG_DATA_HOME").ok().as_deref(),
        std::env::var("HOME").ok().as_deref(),
    )
}

fn resolve_database_path(xdg_data_home: Option<&str>, home: Option<&str>) -> PathBuf {
    let data_directory = match (xdg_data_home, home) {
        (Some(xdg), _) if !xdg.is_empty() => PathBuf::from(xdg),
        (_, Some(home)) => PathBuf::from(home).join(".local/share"),
        _ => PathBuf::from(".local/share"),
    };
    data_directory.join("bourbon").join("bourbon.db")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn xdg_data_home_wins() {
        assert_eq!(
            resolve_database_path(Some("/x/data"), Some("/home/u")),
            PathBuf::from("/x/data/bourbon/bourbon.db")
        );
    }

    #[test]
    fn falls_back_to_home_local_share() {
        assert_eq!(
            resolve_database_path(None, Some("/home/u")),
            PathBuf::from("/home/u/.local/share/bourbon/bourbon.db")
        );
    }

    #[test]
    fn empty_xdg_data_home_is_ignored() {
        assert_eq!(
            resolve_database_path(Some(""), Some("/home/u")),
            PathBuf::from("/home/u/.local/share/bourbon/bourbon.db")
        );
    }

    #[test]
    fn default_path_ends_with_bourbon_db() {
        assert!(default_database_path().ends_with("bourbon/bourbon.db"));
    }
}
