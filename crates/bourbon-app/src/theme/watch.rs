use std::path::PathBuf;
use std::time::SystemTime;

pub(crate) struct ModificationWatch {
    path: PathBuf,
    seen: Option<SystemTime>,
}

impl ModificationWatch {
    pub(crate) fn new(path: PathBuf) -> Self {
        let seen = modified_at(&path);
        Self { path, seen }
    }

    pub(crate) fn has_changed(&mut self) -> bool {
        let now = modified_at(&self.path);
        let changed = now != self.seen;
        self.seen = now;
        changed
    }
}

fn modified_at(path: &PathBuf) -> Option<SystemTime> {
    std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn touch(path: &PathBuf, seconds_after_epoch: u64) {
        let file = std::fs::File::options().write(true).open(path).unwrap();
        file.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds_after_epoch))
            .unwrap();
    }

    #[test]
    fn an_untouched_file_does_not_report_a_change() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("colors.toml");
        std::fs::write(&path, "a").unwrap();
        let mut watch = ModificationWatch::new(path);
        assert!(!watch.has_changed());
    }

    #[test]
    fn a_new_modification_time_reports_once() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("colors.toml");
        std::fs::write(&path, "a").unwrap();
        touch(&path, 1_000);
        let mut watch = ModificationWatch::new(path.clone());
        touch(&path, 2_000);
        assert!(watch.has_changed());
        assert!(!watch.has_changed());
    }

    #[test]
    fn a_file_appearing_or_disappearing_counts_as_a_change() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("colors.toml");
        let mut watch = ModificationWatch::new(path.clone());
        assert!(!watch.has_changed());
        std::fs::write(&path, "a").unwrap();
        assert!(watch.has_changed());
        std::fs::remove_file(&path).unwrap();
        assert!(watch.has_changed());
    }
}
