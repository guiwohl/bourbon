use bourbon_core::{NoteQuery, NoteScope, NoteSort, NoteStatus, NotebookDepth, NotebookId, TagId};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum Navigation {
    #[default]
    AllNotes,
    Notebook(NotebookId),
    Status(NoteStatus),
    Tags(Vec<TagId>),
    Trash,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Listing {
    pub(crate) navigation: Navigation,
    pub(crate) sort: NoteSort,
}

impl Listing {
    pub(crate) fn query(&self) -> NoteQuery {
        let scope = match &self.navigation {
            Navigation::AllNotes => NoteScope::AllNotes,
            Navigation::Notebook(id) => {
                NoteScope::Notebook(id.clone(), NotebookDepth::WithDescendants)
            }
            Navigation::Status(status) => NoteScope::Status(*status),
            Navigation::Tags(ids) => NoteScope::Tags(ids.clone()),
            Navigation::Trash => NoteScope::Trash,
        };
        NoteQuery {
            scope,
            sort: self.sort,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listing(navigation: Navigation) -> Listing {
        Listing {
            navigation,
            sort: NoteSort::default(),
        }
    }

    #[test]
    fn default_listing_is_all_notes_newest_updated_first() {
        assert_eq!(Listing::default().query(), NoteQuery::default());
    }

    #[test]
    fn every_navigation_maps_to_its_scope() {
        let notebook = NotebookId::generate();
        let tags = vec![TagId::generate(), TagId::generate()];
        let cases = [
            (Navigation::AllNotes, NoteScope::AllNotes),
            (
                Navigation::Notebook(notebook.clone()),
                NoteScope::Notebook(notebook, NotebookDepth::WithDescendants),
            ),
            (
                Navigation::Status(NoteStatus::OnHold),
                NoteScope::Status(NoteStatus::OnHold),
            ),
            (Navigation::Tags(tags.clone()), NoteScope::Tags(tags)),
            (Navigation::Trash, NoteScope::Trash),
        ];
        for (navigation, scope) in cases {
            assert_eq!(listing(navigation).query().scope, scope);
        }
    }

    #[test]
    fn the_sort_is_carried_into_the_query() {
        let listing = Listing {
            navigation: Navigation::Trash,
            sort: NoteSort::TitleAlphabetical,
        };
        assert_eq!(listing.query().sort, NoteSort::TitleAlphabetical);
    }
}
