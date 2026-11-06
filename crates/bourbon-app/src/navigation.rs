use bourbon_core::{NoteQuery, NoteScope, NoteSort, NoteStatus, NotebookDepth, NotebookId, TagId};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum Navigation {
    #[default]
    AllNotes,
    Notebook(NotebookId, NotebookDepth),
    Status(NoteStatus),
    Tags(Vec<TagId>),
    Templates,
    Trash,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Listing {
    pub(crate) navigation: Navigation,
    pub(crate) sort: NoteSort,
}

impl Navigation {
    pub(crate) fn with_tag_toggled(&self, tag: &TagId) -> Self {
        match self {
            Self::Tags(tags) if tags.contains(tag) => self.without_tag(tag),
            Self::Tags(tags) => Self::Tags([tags.as_slice(), std::slice::from_ref(tag)].concat()),
            _ => Self::Tags(vec![tag.clone()]),
        }
    }

    pub(crate) fn without_tag(&self, tag: &TagId) -> Self {
        let Self::Tags(tags) = self else {
            return self.clone();
        };
        let remaining: Vec<TagId> = tags.iter().filter(|kept| *kept != tag).cloned().collect();
        if remaining.is_empty() {
            Self::AllNotes
        } else {
            Self::Tags(remaining)
        }
    }

    pub(crate) fn hides_new_notes(&self) -> bool {
        matches!(self, Self::Trash | Self::Templates)
    }

    pub(crate) fn shows_notebook(&self, notebook: &NotebookId) -> bool {
        matches!(self, Self::Notebook(shown, _) if shown == notebook)
    }

    pub(crate) fn shows_tag(&self, tag: &TagId) -> bool {
        matches!(self, Self::Tags(shown) if shown.contains(tag))
    }

    pub(crate) fn is_inside(&self, notebooks: &[NotebookId]) -> bool {
        matches!(self, Self::Notebook(id, _) if notebooks.contains(id))
    }
}

impl Listing {
    pub(crate) fn query(&self) -> NoteQuery {
        let scope = match &self.navigation {
            Navigation::AllNotes => NoteScope::AllNotes,
            Navigation::Notebook(id, depth) => NoteScope::Notebook(id.clone(), *depth),
            Navigation::Status(status) => NoteScope::Status(*status),
            Navigation::Tags(ids) => NoteScope::Tags(ids.clone()),
            Navigation::Templates => NoteScope::Templates,
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
                Navigation::Notebook(notebook.clone(), NotebookDepth::WithDescendants),
                NoteScope::Notebook(notebook.clone(), NotebookDepth::WithDescendants),
            ),
            (
                Navigation::Notebook(notebook.clone(), NotebookDepth::DirectOnly),
                NoteScope::Notebook(notebook, NotebookDepth::DirectOnly),
            ),
            (
                Navigation::Status(NoteStatus::OnHold),
                NoteScope::Status(NoteStatus::OnHold),
            ),
            (Navigation::Tags(tags.clone()), NoteScope::Tags(tags)),
            (Navigation::Templates, NoteScope::Templates),
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

    #[test]
    fn toggling_a_tag_adds_it_after_the_selected_ones_and_removes_it_again() {
        let first = TagId::generate();
        let second = TagId::generate();
        let one = Navigation::Tags(vec![first.clone()]);
        let both = one.with_tag_toggled(&second);
        assert_eq!(both, Navigation::Tags(vec![first.clone(), second.clone()]));
        assert_eq!(
            both.with_tag_toggled(&first),
            Navigation::Tags(vec![second])
        );
    }

    #[test]
    fn toggling_a_tag_outside_a_tag_selection_starts_one() {
        let tag = TagId::generate();
        assert_eq!(
            Navigation::Trash.with_tag_toggled(&tag),
            Navigation::Tags(vec![tag])
        );
    }

    #[test]
    fn removing_the_last_selected_tag_goes_to_all_notes() {
        let tag = TagId::generate();
        assert_eq!(
            Navigation::Tags(vec![tag.clone()]).with_tag_toggled(&tag),
            Navigation::AllNotes
        );
        assert_eq!(
            Navigation::Tags(vec![tag.clone()]).without_tag(&tag),
            Navigation::AllNotes
        );
    }

    #[test]
    fn removing_a_tag_leaves_other_navigations_alone() {
        let tag = TagId::generate();
        assert_eq!(Navigation::Trash.without_tag(&tag), Navigation::Trash);
        let other = TagId::generate();
        let tags = Navigation::Tags(vec![other]);
        assert_eq!(tags.without_tag(&tag), tags);
    }

    #[test]
    fn only_trash_and_templates_hide_new_notes() {
        assert!(Navigation::Trash.hides_new_notes());
        assert!(Navigation::Templates.hides_new_notes());
        assert!(!Navigation::AllNotes.hides_new_notes());
        assert!(!Navigation::Status(NoteStatus::Active).hides_new_notes());
        assert!(!Navigation::Tags(vec![TagId::generate()]).hides_new_notes());
    }

    #[test]
    fn a_navigation_is_inside_only_the_notebooks_it_points_to() {
        let notebook = NotebookId::generate();
        let navigation = Navigation::Notebook(notebook.clone(), NotebookDepth::DirectOnly);
        assert!(navigation.is_inside(&[NotebookId::generate(), notebook]));
        assert!(!navigation.is_inside(&[NotebookId::generate()]));
        assert!(!Navigation::AllNotes.is_inside(&[NotebookId::generate()]));
    }

    #[test]
    fn a_navigation_shows_the_notebook_whatever_the_depth() {
        let notebook = NotebookId::generate();
        let direct = Navigation::Notebook(notebook.clone(), NotebookDepth::DirectOnly);
        let nested = Navigation::Notebook(notebook.clone(), NotebookDepth::WithDescendants);
        assert!(direct.shows_notebook(&notebook));
        assert!(nested.shows_notebook(&notebook));
        assert!(!direct.shows_notebook(&NotebookId::generate()));
        assert!(!Navigation::AllNotes.shows_notebook(&notebook));
    }

    #[test]
    fn a_navigation_shows_every_selected_tag() {
        let first = TagId::generate();
        let second = TagId::generate();
        let both = Navigation::Tags(vec![first.clone(), second.clone()]);
        assert!(both.shows_tag(&first));
        assert!(both.shows_tag(&second));
        assert!(!both.shows_tag(&TagId::generate()));
        assert!(!Navigation::Trash.shows_tag(&first));
    }
}
