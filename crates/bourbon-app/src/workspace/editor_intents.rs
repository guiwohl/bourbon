use bourbon_core::{
    NoteId, NoteStatus, NoteStore, NotebookId, StoreError, TagColor, TagId, Timestamp,
};
use gpui_kit::{Context, Window};

use super::Workspace;
use super::load_note;
use crate::dragged_note::DraggedNote;
use crate::editor::History;

type HistoryStep = fn(&mut History, &dyn Fn(&NoteId) -> bool) -> Option<NoteId>;

impl Workspace {
    pub(crate) fn move_selected_to(&mut self, notebook: &NotebookId, cx: &mut Context<Self>) {
        self.editor.set_notebook_selector_open(false);
        if let Some(id) = self.selected.clone() {
            self.move_note_to(&id, notebook, cx);
        }
    }

    pub(crate) fn drop_note_on_notebook(
        &mut self,
        dragged: &DraggedNote,
        notebook: &NotebookId,
        cx: &mut Context<Self>,
    ) {
        self.move_note_to(&dragged.id, notebook, cx);
    }

    fn move_note_to(&mut self, id: &NoteId, notebook: &NotebookId, cx: &mut Context<Self>) {
        self.edit_note(id, cx, |store, note, now| {
            store.move_note(note, notebook, now)
        });
    }

    pub(crate) fn set_selected_status(&mut self, status: NoteStatus, cx: &mut Context<Self>) {
        self.edit_selected(cx, |store, note, now| store.set_status(note, status, now));
    }

    pub(crate) fn set_selected_template(&mut self, is_template: bool, cx: &mut Context<Self>) {
        self.edit_selected(cx, |store, note, _| {
            if is_template {
                store.mark_as_template(note)
            } else {
                store.unmark_template(note)
            }
        });
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
                store
                    .add_new_tag_to_note(note, &typed, TagColor::Default, now)
                    .map(drop)
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
        self.flush_then(cx, |this, cx| {
            let store = &this.store;
            let Some(id) = step(&mut this.history, &|id| is_openable(store, id)) else {
                return;
            };
            match load_note(&this.store, &id) {
                Ok(note) => this.show(note, window, cx),
                Err(error) => this.report(&error, cx),
            }
        });
    }

    fn edit_selected(
        &mut self,
        cx: &mut Context<Self>,
        change: impl FnOnce(&NoteStore, &NoteId, Timestamp) -> Result<(), StoreError>,
    ) -> bool {
        let Some(id) = self.selected.clone() else {
            return false;
        };
        self.edit_note(&id, cx, change)
    }

    fn edit_note(
        &mut self,
        id: &NoteId,
        cx: &mut Context<Self>,
        change: impl FnOnce(&NoteStore, &NoteId, Timestamp) -> Result<(), StoreError>,
    ) -> bool {
        self.flush_then(cx, |this, cx| {
            let changed =
                change(&this.store, id, Timestamp::now()).and_then(|()| load_note(&this.store, id));
            match changed {
                Ok(note) => {
                    if this.selected.as_ref() == Some(id) {
                        this.editor.show_header(&note);
                    }
                    this.refresh_listing(cx);
                    true
                }
                Err(error) => {
                    this.report(&error, cx);
                    false
                }
            }
        })
        .unwrap_or(false)
    }
}

fn is_openable(store: &NoteStore, id: &NoteId) -> bool {
    load_note(store, id).is_ok_and(|note| note.trashed_at.is_none())
}
