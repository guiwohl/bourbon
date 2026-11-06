use bourbon_core::{NoteId, NoteQuery, StoreError, format_note_link, parse_note_link};
use gpui_kit::{Context, Window};

use super::{Workspace, load_note};
use crate::editor::{LinkPicker, ViewMode, tasks};
use crate::editor::{closing_brackets_after, link_query_before_cursor};
use crate::navigation::Navigation;
use crate::presentation::display_title;

impl Workspace {
    pub(crate) fn set_view_mode(
        &mut self,
        mode: ViewMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.view_mode = mode;
        self.editor.link_picker = None;
        if mode.shows_editor() {
            self.editor.focus_body(window, cx);
        } else {
            window.focus(&self.focus_handle, cx);
        }
        cx.notify();
    }

    pub(crate) fn cycle_view_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_view_mode(self.view_mode.next(), window, cx);
    }

    pub(crate) fn toggle_task_on_line(
        &mut self,
        line: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let body = self.editor.body(cx);
        if let Some(edit) = tasks::toggle_task_on_line(&body, line) {
            self.edit_body(&edit, window, cx);
        }
    }

    pub(crate) fn toggle_task_at_cursor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let body = self.editor.body(cx);
        let cursor = self.editor.cursor(cx);
        if let Some(edit) = tasks::toggle_task_at_offset(&body, cursor) {
            self.edit_body(&edit, window, cx);
        }
    }

    fn edit_body(&mut self, edit: &tasks::TextEdit, window: &mut Window, cx: &mut Context<Self>) {
        self.editor.apply_edit(edit, window, cx);
        self.restart_autosave(cx);
        cx.notify();
    }

    pub(crate) fn body_edited(&mut self, cx: &mut Context<Self>) {
        self.restart_autosave(cx);
        let before_cursor = self.editor.text_before_cursor(cx);
        self.editor.link_picker = link_query_before_cursor(&before_cursor).and_then(|query| {
            let notes = self.store.list_notes(&NoteQuery::default()).ok()?;
            LinkPicker::matching(query.start, query.text, &notes)
        });
        cx.notify();
    }

    pub(crate) fn highlight_next_link(&mut self, cx: &mut Context<Self>) {
        if let Some(picker) = &mut self.editor.link_picker {
            picker.highlight_next();
            cx.notify();
        }
    }

    pub(crate) fn highlight_previous_link(&mut self, cx: &mut Context<Self>) {
        if let Some(picker) = &mut self.editor.link_picker {
            picker.highlight_previous();
            cx.notify();
        }
    }

    pub(crate) fn cancel_link_picker(&mut self, cx: &mut Context<Self>) {
        self.editor.link_picker = None;
        cx.notify();
    }

    pub(crate) fn confirm_link(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let highlight = self
            .editor
            .link_picker
            .as_ref()
            .map(|picker| picker.highlight);
        if let Some(index) = highlight {
            self.pick_link_suggestion(index, window, cx);
        }
    }

    pub(crate) fn pick_link_suggestion(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(picker) = self.editor.link_picker.take() else {
            return;
        };
        let Some(note) = picker.candidate(index) else {
            return;
        };
        let link = format_note_link(display_title(&note.title), &note.id);
        let end =
            self.editor.cursor(cx) + closing_brackets_after(&self.editor.text_after_cursor(cx));
        self.editor
            .replace_range(picker.start..end, &link, window, cx);
        self.editor.focus_body(window, cx);
        self.restart_autosave(cx);
        cx.notify();
    }

    pub(crate) fn open_link(&mut self, url: &str, window: &mut Window, cx: &mut Context<Self>) {
        match parse_note_link(url) {
            Some(id) => self.open_linked_note(&id, window, cx),
            None => cx.open_url(url),
        }
    }

    fn open_linked_note(&mut self, id: &NoteId, window: &mut Window, cx: &mut Context<Self>) {
        self.flush_then(cx, |this, cx| match load_live_note(this, id) {
            Ok(note) => {
                if !this.notes.iter().any(|listed| listed.id == note.id) {
                    this.navigate(Navigation::AllNotes, cx);
                }
                this.show(note, window, cx);
            }
            Err(error) => this.report(&error, cx),
        });
    }

    pub(crate) fn find_in_note(
        &mut self,
        replace_mode: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selected.is_none() {
            return;
        }
        if !self.view_mode.shows_editor() {
            self.set_view_mode(ViewMode::Edit, window, cx);
        }
        self.editor.open_search(replace_mode, window, cx);
    }

    pub(crate) fn toggle_focus_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.focus_mode {
            self.exit_focus_mode(window, cx);
        } else {
            self.focus_mode = true;
            cx.notify();
        }
    }

    pub(crate) fn exit_focus_mode(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> bool {
        let was_focused = self.focus_mode;
        self.focus_mode = false;
        cx.notify();
        was_focused
    }
}

fn load_live_note(workspace: &Workspace, id: &NoteId) -> Result<bourbon_core::Note, StoreError> {
    let note = load_note(&workspace.store, id)?;
    if note.trashed_at.is_some() {
        return Err(StoreError::NoteNotFound(id.clone()));
    }
    Ok(note)
}
