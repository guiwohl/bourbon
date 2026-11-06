use bourbon_core::{
    Note, NoteId, NoteStatus, NoteStore, NotebookId, StoreError, TagColor, TagId, Timestamp,
};
use gpui_kit::{Context, Window};

use super::Workspace;
use crate::editor::History;

type HistoryStep = fn(&mut History, &dyn Fn(&NoteId) -> bool) -> Option<NoteId>;

impl Workspace {
    pub(crate) fn move_selected_to(&mut self, notebook: &NotebookId, cx: &mut Context<Self>) {
        self.editor.set_notebook_selector_open(false);
        self.edit_selected(cx, |store, note, now| store.move_note(note, notebook, now));
    }

    pub(crate) fn set_selected_status(&mut self, status: NoteStatus, cx: &mut Context<Self>) {
        self.edit_selected(cx, |store, note, now| store.set_status(note, status, now));
    }

    pub(crate) fn remove_tag_from_selected(&mut self, tag: &TagId, cx: &mut Context<Self>) {
        self.edit_selected(cx, |store, note, now| {
            store.remove_tag_from_note(note, tag, now)
        });
    }

    pub(crate) fn add_tag_to_selected(
        &mut self,
        tag: &TagId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let added =
            self.edit_selected(cx, |store, note, now| store.add_tag_to_note(note, tag, now));
        if added {
            self.editor.clear_tag_input(window, cx);
        }
    }

    pub(crate) fn commit_tag_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let typed = self.editor.tag_input_text(cx);
        if let Some(tag) = self.editor.chosen_tag(&typed, &self.tags) {
            self.add_tag_to_selected(&tag.id, window, cx);
        } else if !typed.trim().is_empty() {
            let created = self.edit_selected(cx, |store, note, now| {
                let tag = store.create_tag(&typed, TagColor::Default, now)?;
                store.add_tag_to_note(note, &tag.id, now)
            });
            if created {
                self.editor.clear_tag_input(window, cx);
            }
        }
    }

    pub(crate) fn tag_input_changed(&mut self, cx: &mut Context<Self>) {
        self.editor.forget_tag_highlight();
        cx.notify();
    }

    pub(crate) fn highlight_next_tag_suggestion(&mut self, cx: &mut Context<Self>) {
        let typed = self.editor.tag_input_text(cx);
        self.editor.highlight_next_tag(&typed, &self.tags);
        cx.notify();
    }

    pub(crate) fn highlight_previous_tag_suggestion(&mut self, cx: &mut Context<Self>) {
        self.editor.highlight_previous_tag();
        cx.notify();
    }

    pub(crate) fn set_notebook_selector_open(&mut self, open: bool, cx: &mut Context<Self>) {
        self.editor.set_notebook_selector_open(open);
        cx.notify();
    }

    pub(crate) fn open_notebook_selector(&mut self, cx: &mut Context<Self>) {
        if self.selected.is_some() {
            self.set_notebook_selector_open(true, cx);
        }
    }

    pub(crate) fn history_back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.walk_history(History::back, window, cx);
    }

    pub(crate) fn history_forward(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.walk_history(History::forward, window, cx);
    }

    fn walk_history(&mut self, step: HistoryStep, window: &mut Window, cx: &mut Context<Self>) {
        if let Err(error) = self.flush_pending_save(cx) {
            self.report(&error, cx);
            return;
        }
        let store = &self.store;
        let Some(id) = step(&mut self.history, &|id| is_openable(store, id)) else {
            return;
        };
        match load_note(&self.store, &id) {
            Ok(note) => self.show(note, window, cx),
            Err(error) => self.report(&error, cx),
        }
    }

    fn edit_selected(
        &mut self,
        cx: &mut Context<Self>,
        change: impl FnOnce(&NoteStore, &NoteId, Timestamp) -> Result<(), StoreError>,
    ) -> bool {
        let Some(id) = self.selected.clone() else {
            return false;
        };
        match self.apply_change(&id, change, cx) {
            Ok(note) => {
                self.editor.show_header(&note);
                self.reload_sidebar(cx);
                self.reload_list(cx);
                cx.notify();
                true
            }
            Err(error) => {
                self.report(&error, cx);
                false
            }
        }
    }

    fn apply_change(
        &mut self,
        id: &NoteId,
        change: impl FnOnce(&NoteStore, &NoteId, Timestamp) -> Result<(), StoreError>,
        cx: &mut Context<Self>,
    ) -> Result<Note, StoreError> {
        self.flush_pending_save(cx)?;
        change(&self.store, id, Timestamp::now())?;
        load_note(&self.store, id)
    }
}

fn load_note(store: &NoteStore, id: &NoteId) -> Result<Note, StoreError> {
    store
        .find_note(id)?
        .ok_or_else(|| StoreError::NoteNotFound(id.clone()))
}

fn is_openable(store: &NoteStore, id: &NoteId) -> bool {
    load_note(store, id).is_ok_and(|note| note.trashed_at.is_none())
}
