use bourbon_core::{NotebookId, StoreError, TagColor, TagId, Timestamp};
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::{AppContext as _, Context, Entity, Window};

use super::Workspace;
use crate::navigation::Navigation;
use crate::sidebar::{DeletionTarget, NameTarget, Prompt, subtree_ids};

impl Workspace {
    pub(crate) fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        if self.focus_mode {
            return;
        }
        self.sidebar.is_visible = !self.sidebar.is_visible;
        self.settings.sidebar_visible = self.sidebar.is_visible;
        self.persist_settings(cx);
        cx.notify();
    }

    pub(crate) fn select_notebook(&mut self, notebook: &NotebookId, cx: &mut Context<Self>) {
        let depth = self.sidebar.depth_of(notebook);
        self.navigate(Navigation::Notebook(notebook.clone(), depth), cx);
    }

    pub(crate) fn toggle_notebook_expansion(
        &mut self,
        notebook: &NotebookId,
        cx: &mut Context<Self>,
    ) {
        self.sidebar.toggle_expansion(notebook);
        self.align_depth_with_expansion(notebook);
        self.reload_list(cx);
        cx.notify();
    }

    fn align_depth_with_expansion(&mut self, notebook: &NotebookId) {
        if matches!(&self.listing.navigation, Navigation::Notebook(current, _) if current == notebook)
        {
            let depth = self.sidebar.depth_of(notebook);
            self.listing.navigation = Navigation::Notebook(notebook.clone(), depth);
        }
    }

    pub(crate) fn select_tag(&mut self, tag: &TagId, is_additive: bool, cx: &mut Context<Self>) {
        let navigation = if is_additive {
            self.listing.navigation.with_tag_toggled(tag)
        } else {
            Navigation::Tags(vec![tag.clone()])
        };
        self.navigate(navigation, cx);
    }

    pub(crate) fn begin_create_notebook(
        &mut self,
        parent: Option<NotebookId>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.begin_naming(NameTarget::NewNotebook(parent), "", window, cx);
    }

    pub(crate) fn begin_rename_notebook(
        &mut self,
        notebook: &NotebookId,
        current_name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.begin_naming(
            NameTarget::RenameNotebook(notebook.clone()),
            current_name,
            window,
            cx,
        );
    }

    pub(crate) fn begin_rename_tag(
        &mut self,
        tag: &TagId,
        current_name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.begin_naming(NameTarget::RenameTag(tag.clone()), current_name, window, cx);
    }

    fn begin_naming(
        &mut self,
        target: NameTarget,
        initial: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let input: Entity<InputState> = cx.new(|cx| InputState::new(window, cx));
        input.update(cx, |input, cx| {
            input.set_value(initial.to_owned(), window, cx);
            input.focus(window, cx);
        });
        let subscription = cx.subscribe(&input, |workspace, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::PressEnter { .. }) {
                workspace.confirm_prompt(cx);
            }
        });
        self.sidebar.prompt = Some(Prompt::Name {
            target,
            input,
            _subscription: subscription,
        });
        cx.notify();
    }

    pub(crate) fn begin_delete_notebook(
        &mut self,
        notebook: &NotebookId,
        name: &str,
        cx: &mut Context<Self>,
    ) {
        self.sidebar.prompt = Some(Prompt::Deletion {
            target: DeletionTarget::Notebook(notebook.clone()),
            question: format!("Delete notebook {name}? Its notes move to Trash.").into(),
        });
        cx.notify();
    }

    pub(crate) fn begin_delete_tag(&mut self, tag: &TagId, name: &str, cx: &mut Context<Self>) {
        self.sidebar.prompt = Some(Prompt::Deletion {
            target: DeletionTarget::Tag(tag.clone()),
            question: format!("Delete tag {name}?").into(),
        });
        cx.notify();
    }

    pub(crate) fn cancel_prompt(&mut self, cx: &mut Context<Self>) {
        self.sidebar.prompt = None;
        cx.notify();
    }

    pub(crate) fn confirm_prompt(&mut self, cx: &mut Context<Self>) {
        let Some(prompt) = self.sidebar.prompt.take() else {
            return;
        };
        let result = match &prompt {
            Prompt::Name { target, input, .. } => {
                let name = input.read(cx).value().to_string();
                self.apply_name(target, &name)
            }
            Prompt::Deletion { target, .. } => self.apply_deletion(target, cx),
        };
        if result.is_err() {
            self.sidebar.prompt = Some(prompt);
        }
        self.finish_sidebar_change(result, cx);
    }

    fn apply_name(&mut self, target: &NameTarget, name: &str) -> Result<(), StoreError> {
        match target {
            NameTarget::NewNotebook(parent) => {
                self.store
                    .create_notebook(name, parent.as_ref(), Timestamp::now())?;
                if let Some(parent) = parent {
                    self.sidebar.expanded.insert(parent.clone());
                    self.align_depth_with_expansion(parent);
                }
                Ok(())
            }
            NameTarget::RenameNotebook(notebook) => self.store.rename_notebook(notebook, name),
            NameTarget::RenameTag(tag) => self.store.rename_tag(tag, name),
        }
    }

    fn apply_deletion(
        &mut self,
        target: &DeletionTarget,
        cx: &mut Context<Self>,
    ) -> Result<(), StoreError> {
        match target {
            DeletionTarget::Notebook(notebook) => {
                self.flush_pending_save(cx)?;
                let doomed = subtree_ids(&self.notebooks, notebook);
                self.store.delete_notebook(notebook, Timestamp::now())?;
                if self.listing.navigation.is_inside(&doomed) {
                    self.listing.navigation = Navigation::AllNotes;
                }
                self.clear_unavailable_selection();
                Ok(())
            }
            DeletionTarget::Tag(tag) => {
                self.store.delete_tag(tag)?;
                self.listing.navigation = self.listing.navigation.without_tag(tag);
                Ok(())
            }
        }
    }

    pub(crate) fn set_tag_color(&mut self, tag: &TagId, color: TagColor, cx: &mut Context<Self>) {
        let result = self.store.set_tag_color(tag, color);
        self.finish_sidebar_change(result, cx);
    }

    fn finish_sidebar_change(&mut self, result: Result<(), StoreError>, cx: &mut Context<Self>) {
        match result {
            Ok(()) => {
                self.error_message = None;
                self.refresh_listing(cx);
            }
            Err(error) => self.report(&error, cx),
        }
    }
}
