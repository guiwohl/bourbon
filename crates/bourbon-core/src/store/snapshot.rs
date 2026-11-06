use std::fs;
use std::io;
use std::path::Path;

use rusqlite::{Connection, OpenFlags, params};

use crate::store::{NoteStore, StoreError};

impl NoteStore {
    pub fn snapshot(&self, target: &Path) -> Result<(), StoreError> {
        let target_text = target.to_str().ok_or_else(|| {
            StoreError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "snapshot path is not valid UTF-8",
            ))
        })?;
        self.connection
            .execute("VACUUM INTO ?1", params![target_text])?;
        if let Err(error) = verify_integrity(target) {
            fs::remove_file(target)?;
            return Err(error);
        }
        Ok(())
    }
}

pub(crate) fn verify_integrity(database: &Path) -> Result<(), StoreError> {
    let connection = Connection::open_with_flags(database, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut statement = connection.prepare("PRAGMA integrity_check")?;
    let report = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    if report == ["ok"] {
        Ok(())
    } else {
        Err(StoreError::IntegrityCheckFailed(report.join("; ")))
    }
}
