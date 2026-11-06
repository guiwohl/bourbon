#[cfg(test)]
mod tests;

use bourbon_core::{
    Note, NoteId, NoteStore, NoteSummary, NotebookNode, StoreError, TagSummary, Timestamp,
};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    Context, FocusHandle, Focusable, InteractiveElement as _, IntoElement, ParentElement as _,
    Render, SharedString, Styled as _, Subscription, Task, Window, div,
};

use crate::editor::{self, EditorPane};
use crate::navigation::{Listing, Navigation};
use crate::sidebar::SidebarData;
use crate::{autosave, note_list, sidebar};

gpui_kit::actions!(bourbon, [NewNote]);

pub struct Workspace {
    store: NoteStore,
    listing: Listing,
    notes: Vec<NoteSummary>,
    notebooks: Vec<NotebookNode>,
    tags: Vec<TagSummary>,
    selected: Option<NoteId>,
    editor: EditorPane,
    pending_save: Option<Task<()>>,
    error_message: Option<SharedString>,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl Workspace {
    pub fn new(store: NoteStore, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor = EditorPane::new(window, cx);
        let mut subscriptions = editor.subscribe_to_edits(cx);
        subscriptions.push(cx.on_app_quit(|view, cx| {
            view.flush_before_closing(cx);
            std::future::ready(())
        }));
        let mut view = Self {
            store,
            listing: Listing::default(),
            notes: Vec::new(),
            notebooks: Vec::new(),
            tags: Vec::new(),
            selected: None,
            editor,
            pending_save: None,
            error_message: None,
            focus_handle: cx.focus_handle(),
            _subscriptions: subscriptions,
        };
        view.purge_expired_trash(cx);
        view.reload_sidebar(cx);
        view.reload_list(cx);
        if let Some(most_recent) = view.notes.first().map(|summary| summary.id.clone()) {
            view.select(&most_recent, window, cx);
        } else {
            window.focus(&view.focus_handle, cx);
        }
        view
    }

    pub(crate) fn restart_autosave(&mut self, cx: &mut Context<Self>) {
        self.pending_save = Some(autosave::after_delay(cx, |view, cx| {
            if let Err(error) = view.save_and_reload(cx) {
                view.report(&error, cx);
            }
        }));
    }

    fn purge_expired_trash(&mut self, cx: &mut Context<Self>) {
        if let Err(error) = self.store.purge_expired_trash(Timestamp::now()) {
            self.report(&error, cx);
        }
    }

    pub(crate) fn navigate(&mut self, navigation: Navigation, cx: &mut Context<Self>) {
        self.listing.navigation = navigation;
        self.reload_list(cx);
        cx.notify();
    }

    fn reload_sidebar(&mut self, cx: &mut Context<Self>) {
        match self.store.notebook_tree() {
            Ok(notebooks) => self.notebooks = notebooks,
            Err(error) => self.report(&error, cx),
        }
        match self.store.list_tags() {
            Ok(tags) => self.tags = tags,
            Err(error) => self.report(&error, cx),
        }
    }

    fn reload_list(&mut self, cx: &mut Context<Self>) {
        match self.store.list_notes(&self.listing.query()) {
            Ok(notes) => self.notes = notes,
            Err(error) => self.report(&error, cx),
        }
    }

    fn save_selected(&mut self, cx: &mut Context<Self>) -> Result<(), StoreError> {
        let Some(id) = self.selected.clone() else {
            return Ok(());
        };
        let title = self.editor.title(cx);
        let body = self.editor.body(cx);
        self.store.save_note(&id, &title, &body, Timestamp::now())?;
        self.error_message = None;
        cx.notify();
        Ok(())
    }

    fn save_and_reload(&mut self, cx: &mut Context<Self>) -> Result<(), StoreError> {
        self.save_selected(cx)?;
        self.reload_list(cx);
        Ok(())
    }

    fn flush_pending_save(&mut self, cx: &mut Context<Self>) -> Result<(), StoreError> {
        self.pending_save = None;
        self.save_and_reload(cx)
    }

    pub fn flush_before_closing(&mut self, cx: &mut Context<Self>) -> bool {
        match self.flush_pending_save(cx) {
            Ok(()) => true,
            Err(error) => {
                self.report(&error, cx);
                false
            }
        }
    }

    pub(crate) fn select(&mut self, id: &NoteId, window: &mut Window, cx: &mut Context<Self>) {
        if let Err(error) = self.flush_pending_save(cx) {
            self.report(&error, cx);
            return;
        }
        let loaded = self
            .store
            .find_note(id)
            .and_then(|found| found.ok_or_else(|| StoreError::NoteNotFound(id.clone())));
        match loaded {
            Ok(note) => self.show(note, window, cx),
            Err(error) => self.report(&error, cx),
        }
    }

    pub(crate) fn create_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Err(error) = self.flush_pending_save(cx) {
            self.report(&error, cx);
            return;
        }
        if self.listing.navigation == Navigation::Trash {
            self.listing.navigation = Navigation::AllNotes;
        }
        let created = self.create_note_for_navigation();
        self.reload_list(cx);
        match created {
            Ok(note) => {
                self.reload_sidebar(cx);
                self.show(note, window, cx);
                self.editor.focus_title(window, cx);
            }
            Err(error) => self.report(&error, cx),
        }
    }

    fn create_note_for_navigation(&self) -> Result<Note, StoreError> {
        let now = Timestamp::now();
        if let Navigation::Notebook(notebook) = &self.listing.navigation {
            return self.store.create_note(notebook, now);
        }
        let note = self
            .store
            .create_note(&self.store.default_notebook()?, now)?;
        match &self.listing.navigation {
            Navigation::Tags(tags) => {
                for tag in tags {
                    self.store.add_tag_to_note(&note.id, tag, now)?;
                }
            }
            Navigation::Status(status) => self.store.set_status(&note.id, *status, now)?,
            _ => {}
        }
        Ok(note)
    }

    fn show(&mut self, note: Note, window: &mut Window, cx: &mut Context<Self>) {
        self.editor.load(note.clone(), window, cx);
        self.selected = Some(note.id);
        cx.notify();
    }

    fn report(&mut self, error: &StoreError, cx: &mut Context<Self>) {
        self.error_message = Some(error.to_string().into());
        cx.notify();
    }
}

impl Focusable for Workspace {
    fn focus_handle(&self, _cx: &gpui_kit::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pane = if self.selected.is_some() {
            self.editor.render().into_any_element()
        } else {
            editor::render_empty(cx).into_any_element()
        };
        let sidebar = sidebar::render(
            &SidebarData {
                current: &self.listing.navigation,
                notebooks: &self.notebooks,
                tags: &self.tags,
            },
            cx,
        );
        let note_list = note_list::render(&self.notes, self.selected.as_ref(), cx);
        div()
            .track_focus(&self.focus_handle)
            .size_full()
            .flex()
            .flex_col()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .on_action(cx.listener(|view, _: &NewNote, window, cx| view.create_note(window, cx)))
            .when_some(self.error_message.clone(), |root, message| {
                root.child(editor::error_banner(message, cx))
            })
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(sidebar)
                    .child(note_list)
                    .child(pane),
            )
    }
}
