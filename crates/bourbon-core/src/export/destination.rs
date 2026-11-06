use std::fs;
use std::io;
use std::path::Path;

use super::layout::ExportPlan;
use crate::parent_directory::parent_directory_of;
use crate::store::StoreError;

pub fn ensure_available(destination: &Path) -> Result<(), StoreError> {
    let available = if destination.is_dir() {
        fs::read_dir(destination)?.next().is_none()
    } else {
        !destination.exists()
    };
    if available {
        Ok(())
    } else {
        Err(StoreError::ExportDestinationNotEmpty(
            destination.to_path_buf(),
        ))
    }
}

pub fn write_atomically(plan: &ExportPlan, destination: &Path) -> io::Result<()> {
    let parent = parent_directory_of(destination);
    fs::create_dir_all(parent)?;
    let staging = tempfile::Builder::new()
        .prefix(".bourbon-export-")
        .tempdir_in(parent)?;
    for directory in &plan.directories {
        fs::create_dir_all(staging.path().join(directory))?;
    }
    for file in &plan.files {
        fs::write(staging.path().join(&file.path), &file.contents)?;
    }
    if destination.is_dir() {
        fs::remove_dir(destination)?;
    }
    fs::rename(staging.path(), destination)
}
