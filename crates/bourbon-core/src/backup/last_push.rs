use std::fmt::Write;
use std::fs;
use std::io;
use std::path::Path;

use sha2::{Digest, Sha256};

const STATE_FILE: &str = "last-pushed.sha256";

pub fn fingerprint_of(file: &Path) -> io::Result<String> {
    let digest = Sha256::digest(fs::read(file)?);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(hex, "{byte:02x}");
    }
    Ok(hex)
}

pub fn recorded(work_dir: &Path) -> Option<String> {
    fs::read_to_string(work_dir.join(STATE_FILE))
        .ok()
        .map(|content| content.trim().to_owned())
}

pub fn record(work_dir: &Path, fingerprint: &str) -> io::Result<()> {
    fs::write(work_dir.join(STATE_FILE), fingerprint)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_the_lowercase_hex_sha256() {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("f");
        fs::write(&file, "abc").unwrap();
        assert_eq!(
            fingerprint_of(&file).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn nothing_is_recorded_at_first() {
        let directory = tempfile::tempdir().unwrap();
        assert_eq!(recorded(directory.path()), None);
    }

    #[test]
    fn a_recorded_fingerprint_is_read_back() {
        let directory = tempfile::tempdir().unwrap();
        record(directory.path(), "abc123").unwrap();
        assert_eq!(recorded(directory.path()), Some("abc123".to_owned()));
        record(directory.path(), "def456").unwrap();
        assert_eq!(recorded(directory.path()), Some("def456".to_owned()));
    }
}
