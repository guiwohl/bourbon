mod destination;
mod document;
mod layout;
mod slug;

use std::path::Path;

use crate::identifier::NoteId;
use crate::store::{NoteStore, StoreError};

pub use layout::ExportSummary;

impl NoteStore {
    pub fn export_markdown(&self, destination: &Path) -> Result<ExportSummary, StoreError> {
        destination::ensure_available(destination)?;
        let plan = layout::plan(&self.all_notebooks()?, &self.exportable_notes()?);
        destination::write_atomically(&plan, destination)?;
        Ok(plan.summary)
    }

    pub fn note_markdown(&self, id: &NoteId) -> Result<Option<String>, StoreError> {
        Ok(self
            .find_note(id)?
            .filter(|note| note.trashed_at.is_none())
            .map(|note| document::render(&note)))
    }
}

#[cfg(test)]
mod tests {
    use crate::{NoteId, NoteStore, Timestamp};

    #[test]
    fn note_markdown_matches_the_exported_file_layout() {
        let store = NoteStore::open_in_memory().unwrap();
        let inbox = store.default_notebook().unwrap();
        let created = store
            .create_note(&inbox, Timestamp::from_millis(1))
            .unwrap();
        store
            .save_note(&created.id, "Hello", "body\n", Timestamp::from_millis(2))
            .unwrap();

        let markdown = store.note_markdown(&created.id).unwrap().unwrap();

        assert_eq!(
            markdown,
            format!(
                "---\nid: {}\ntitle: \"Hello\"\ntags: []\ncreated: 1970-01-01T00:00:00.001Z\nupdated: 1970-01-01T00:00:00.002Z\n---\n\nbody\n",
                created.id
            )
        );
    }

    #[test]
    fn note_markdown_of_an_unknown_note_is_none() {
        let store = NoteStore::open_in_memory().unwrap();

        assert_eq!(
            store
                .note_markdown(&NoteId::parse("abcdefghijkl").unwrap())
                .unwrap(),
            None
        );
    }

    #[test]
    fn note_markdown_of_a_trashed_note_is_none() {
        let store = NoteStore::open_in_memory().unwrap();
        let inbox = store.default_notebook().unwrap();
        let created = store
            .create_note(&inbox, Timestamp::from_millis(1))
            .unwrap();
        store
            .trash_note(&created.id, Timestamp::from_millis(2))
            .unwrap();

        assert_eq!(store.note_markdown(&created.id).unwrap(), None);
    }
}
