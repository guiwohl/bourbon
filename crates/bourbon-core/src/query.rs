use crate::identifier::{NotebookId, TagId};
use crate::status::NoteStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotebookDepth {
    DirectOnly,
    WithDescendants,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum NoteScope {
    #[default]
    AllNotes,
    Notebook(NotebookId, NotebookDepth),
    Status(NoteStatus),
    Tags(Vec<TagId>),
    Templates,
    Trash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NoteSort {
    #[default]
    UpdatedNewestFirst,
    CreatedNewestFirst,
    TitleAlphabetical,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NoteQuery {
    pub scope: NoteScope,
    pub sort: NoteSort,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_query_lists_all_notes_by_most_recently_updated() {
        let query = NoteQuery::default();
        assert_eq!(query.scope, NoteScope::AllNotes);
        assert_eq!(query.sort, NoteSort::UpdatedNewestFirst);
    }
}
