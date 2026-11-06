mod destination;
mod document;
mod layout;
mod slug;

use std::path::Path;

use crate::store::{NoteStore, StoreError};

pub use layout::ExportSummary;

impl NoteStore {
    pub fn export_markdown(&self, destination: &Path) -> Result<ExportSummary, StoreError> {
        destination::ensure_available(destination)?;
        let plan = layout::plan(&self.all_notebooks()?, &self.exportable_notes()?);
        destination::write_atomically(&plan, destination)?;
        Ok(plan.summary)
    }
}
