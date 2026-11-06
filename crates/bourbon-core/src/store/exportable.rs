use crate::identifier::NoteId;
use crate::note::Note;
use crate::store::{NoteStore, StoreError};

impl NoteStore {
    pub(crate) fn exportable_notes(&self) -> Result<Vec<Note>, StoreError> {
        let mut statement = self
            .connection
            .prepare("SELECT id FROM notes WHERE deleted_at IS NULL")?;
        let ids = statement
            .query_map([], |row| row.get::<_, NoteId>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        ids.iter()
            .filter_map(|id| self.find_note(id).transpose())
            .collect()
    }
}
