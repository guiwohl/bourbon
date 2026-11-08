use std::collections::HashSet;

use bourbon_core::{
    NoteScope, NoteStatus, NoteStore, NotebookDepth, NotebookId, StoreError, TagId,
};
use bourbon_editor::TextField;
use gpui_kit::{Entity, SharedString, Subscription};

use super::tree;

pub(crate) enum NameTarget {
    NewNotebook(Option<NotebookId>),
    RenameNotebook(NotebookId),
    RenameTag(TagId),
}

impl NameTarget {
    pub(super) fn title(&self) -> &'static str {
        match self {
            Self::NewNotebook(None) => "New notebook",
            Self::NewNotebook(Some(_)) => "New sub-notebook",
            Self::RenameNotebook(_) => "Rename notebook",
            Self::RenameTag(_) => "Rename tag",
        }
    }
}

pub(crate) enum DeletionTarget {
    Notebook(NotebookId),
    Tag(TagId),
}

pub(crate) enum Prompt {
    Name {
        target: NameTarget,
        input: Entity<TextField>,
        _subscription: Subscription,
    },
    Deletion {
        target: DeletionTarget,
        question: SharedString,
    },
}

impl Prompt {
    pub(crate) fn has_input(&self) -> bool {
        matches!(self, Self::Name { .. })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Section {
    Notebooks,
    Status,
    Tags,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct SidebarCounts {
    pub(crate) all_notes: usize,
    pub(crate) templates: usize,
    pub(crate) by_status: Vec<(NoteStatus, usize)>,
}

impl SidebarCounts {
    pub(crate) fn load(store: &NoteStore) -> Result<Self, StoreError> {
        let by_status = NoteStatus::ALL
            .into_iter()
            .filter(|status| *status != NoteStatus::None)
            .map(|status| Ok((status, store.count_notes(&NoteScope::Status(status))?)))
            .collect::<Result<_, StoreError>>()?;
        Ok(Self {
            all_notes: store.count_notes(&NoteScope::AllNotes)?,
            templates: store.count_notes(&NoteScope::Templates)?,
            by_status,
        })
    }

    pub(crate) fn of_status(&self, status: NoteStatus) -> usize {
        self.by_status
            .iter()
            .find_map(|(candidate, count)| (*candidate == status).then_some(*count))
            .unwrap_or_default()
    }
}

pub(crate) struct SidebarState {
    pub(crate) is_visible: bool,
    pub(crate) expanded: HashSet<NotebookId>,
    pub(crate) inbox: Option<NotebookId>,
    pub(crate) prompt: Option<Prompt>,
    pub(crate) collapsed: HashSet<Section>,
}

impl SidebarState {
    pub(crate) fn new(inbox: Option<NotebookId>) -> Self {
        Self {
            is_visible: true,
            expanded: HashSet::new(),
            inbox,
            prompt: None,
            collapsed: HashSet::new(),
        }
    }

    pub(crate) fn depth_of(&self, notebook: &NotebookId) -> NotebookDepth {
        tree::depth_for(self.expanded.contains(notebook))
    }

    pub(crate) fn is_collapsed(&self, section: Section) -> bool {
        self.collapsed.contains(&section)
    }

    pub(crate) fn toggle_section(&mut self, section: Section) {
        if !self.collapsed.remove(&section) {
            self.collapsed.insert(section);
        }
    }

    pub(crate) fn toggle_expansion(&mut self, notebook: &NotebookId) {
        if !self.expanded.remove(notebook) {
            self.expanded.insert(notebook.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_sidebar_is_visible_with_everything_collapsed() {
        let state = SidebarState::new(None);
        let notebook = NotebookId::generate();
        assert!(state.is_visible);
        assert_eq!(state.depth_of(&notebook), NotebookDepth::WithDescendants);
    }

    #[test]
    fn toggling_expansion_switches_the_depth_back_and_forth() {
        let mut state = SidebarState::new(None);
        let notebook = NotebookId::generate();
        state.toggle_expansion(&notebook);
        assert_eq!(state.depth_of(&notebook), NotebookDepth::DirectOnly);
        state.toggle_expansion(&notebook);
        assert_eq!(state.depth_of(&notebook), NotebookDepth::WithDescendants);
    }

    #[test]
    fn a_status_count_is_looked_up_by_status() {
        let counts = SidebarCounts {
            all_notes: 0,
            templates: 0,
            by_status: vec![(NoteStatus::Active, 3), (NoteStatus::Dropped, 5)],
        };
        assert_eq!(counts.of_status(NoteStatus::Dropped), 5);
        assert_eq!(counts.of_status(NoteStatus::Active), 3);
        assert_eq!(counts.of_status(NoteStatus::Completed), 0);
    }

    #[test]
    fn loaded_counts_cover_all_notes_templates_and_every_status() {
        let store = NoteStore::open_in_memory().unwrap();
        let counts = SidebarCounts::load(&store).unwrap();
        assert_eq!(counts.all_notes, 0);
        assert_eq!(counts.templates, 0);
        assert_eq!(counts.by_status.len(), 4);
        assert_eq!(counts.of_status(NoteStatus::Active), 0);
    }

    #[test]
    fn sections_start_expanded_and_toggle_independently() {
        let mut state = SidebarState::new(None);
        assert!(!state.is_collapsed(Section::Tags));
        state.toggle_section(Section::Tags);
        assert!(state.is_collapsed(Section::Tags));
        assert!(!state.is_collapsed(Section::Status));
        state.toggle_section(Section::Tags);
        assert!(!state.is_collapsed(Section::Tags));
    }

    #[test]
    fn prompts_are_titled_by_what_they_do() {
        let notebook = NotebookId::generate();
        assert_eq!(NameTarget::NewNotebook(None).title(), "New notebook");
        assert_eq!(
            NameTarget::NewNotebook(Some(notebook.clone())).title(),
            "New sub-notebook"
        );
        assert_eq!(
            NameTarget::RenameNotebook(notebook).title(),
            "Rename notebook"
        );
        assert_eq!(
            NameTarget::RenameTag(TagId::generate()).title(),
            "Rename tag"
        );
    }
}
