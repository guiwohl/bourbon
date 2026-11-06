use std::collections::HashSet;

use bourbon_core::{NotebookDepth, NotebookId, TagId};
use gpui_kit::component::input::InputState;
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
        input: Entity<InputState>,
        _subscription: Subscription,
    },
    Deletion {
        target: DeletionTarget,
        question: SharedString,
    },
}

pub(crate) struct SidebarState {
    pub(crate) is_visible: bool,
    pub(crate) expanded: HashSet<NotebookId>,
    pub(crate) inbox: Option<NotebookId>,
    pub(crate) prompt: Option<Prompt>,
    pub(crate) menu_open: bool,
}

impl SidebarState {
    pub(crate) fn new(inbox: Option<NotebookId>) -> Self {
        Self {
            is_visible: true,
            expanded: HashSet::new(),
            inbox,
            prompt: None,
            menu_open: false,
        }
    }

    pub(crate) fn depth_of(&self, notebook: &NotebookId) -> NotebookDepth {
        tree::depth_for(self.expanded.contains(notebook))
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
