use bourbon_core::ViewMode;
use bourbon_core::{NoteId, StoreError, format_note_link, parse_note_link};
use gpui::{Context, Window};

use super::{Workspace, load_note};
use crate::editor::{LinkPicker, MAX_SUGGESTIONS};
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
        self.settings.editor_view_mode = mode;
        self.persist_settings(cx);
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

    pub(crate) fn leave_title(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor.focus_body(window, cx);
    }

    pub(crate) fn toggle_task_on_line(&mut self, line: usize, cx: &mut Context<Self>) {
        self.editor.toggle_task_on_line(line, cx);
    }

    pub(crate) fn body_edited(&mut self, cx: &mut Context<Self>) {
        self.editor.refresh_preview(cx);
        self.restart_autosave(cx);
        let before_cursor = self.editor.text_before_cursor(cx);
        self.editor.link_picker = link_query_before_cursor(&before_cursor).and_then(|query| {
            let hits = self.store.search_notes(query.text, MAX_SUGGESTIONS).ok()?;
            LinkPicker::from_hits(query.start, hits)
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
        self.editor.replace_range(picker.start..end, &link, cx);
        self.editor.focus_body(window, cx);
        self.restart_autosave(cx);
        cx.notify();
    }

    pub(crate) fn open_link(&mut self, url: &str, cx: &mut Context<Self>) {
        match parse_note_link(url) {
            Some(id) => self.open_linked_note(&id, cx),
            None => cx.open_url(url),
        }
    }

    pub(crate) fn open_note_by_id(&mut self, id: &str, cx: &mut Context<Self>) {
        if let Ok(id) = NoteId::parse(id) {
            self.open_linked_note(&id, cx);
        }
    }

    pub(crate) fn editor_scrolled(&mut self, cx: &mut Context<Self>) {
        if self.view_mode == ViewMode::SideBySide {
            self.editor.follow_editor_scroll(cx);
        }
    }

    pub(crate) fn preview_scrolled(&mut self, cx: &mut Context<Self>) {
        if self.view_mode == ViewMode::SideBySide {
            self.editor.follow_preview_scroll(cx);
        }
    }

    fn open_linked_note(&mut self, id: &NoteId, cx: &mut Context<Self>) {
        self.flush_then(cx, |this, cx| match load_live_note(this, id) {
            Ok(note) => {
                if !this.notes.iter().any(|listed| listed.id == note.id) {
                    this.navigate(Navigation::AllNotes, cx);
                }
                this.show(note, cx);
            }
            Err(error) => this.report(&error, cx),
        });
    }

    pub(crate) fn find_in_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected.is_none() {
            return;
        }
        if self.view_mode.shows_editor() {
            self.editor.focus_body(window, cx);
        } else {
            self.set_view_mode(ViewMode::Edit, window, cx);
        }
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
