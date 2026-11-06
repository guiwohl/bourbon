mod editor_pane;
mod sidebar;
#[cfg(test)]
mod tests;

use bourbon_core::{Note, NoteId, NoteStore, NoteSummary, StoreError, Timestamp};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::input::{EditorState, InputEvent, InputState};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AppContext as _, Context, Entity, FocusHandle, Focusable, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, Styled as _, Subscription, Task, Window, div,
};

use crate::autosave;
use editor_pane::error_banner;

const UNTITLED: &str = "Untitled";

gpui_kit::actions!(bourbon, [NewNote]);

pub struct NotesView {
    store: NoteStore,
    notes: Vec<NoteSummary>,
    selected: Option<NoteId>,
    title_input: Entity<InputState>,
    body_editor: Entity<EditorState>,
    pending_save: Option<Task<()>>,
    error_message: Option<SharedString>,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl NotesView {
    pub fn new(store: NoteStore, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let title_input = cx.new(|cx| InputState::new(window, cx).placeholder(UNTITLED));
        let body_editor = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("markdown")
                .soft_wrap(true)
                .line_number(false)
        });
        let subscriptions = vec![
            cx.subscribe(&title_input, Self::restart_autosave_on_change::<InputState>),
            cx.subscribe(
                &body_editor,
                Self::restart_autosave_on_change::<EditorState>,
            ),
            cx.on_app_quit(|view, cx| {
                view.flush_before_closing(cx);
                std::future::ready(())
            }),
        ];
        let mut view = Self {
            store,
            notes: Vec::new(),
            selected: None,
            title_input,
            body_editor,
            pending_save: None,
            error_message: None,
            focus_handle: cx.focus_handle(),
            _subscriptions: subscriptions,
        };
        view.reload_list(cx);
        if let Some(most_recent) = view.notes.first().map(|summary| summary.id.clone()) {
            view.select(&most_recent, window, cx);
        } else {
            window.focus(&view.focus_handle, cx);
        }
        view
    }

    fn restart_autosave_on_change<Source: 'static>(
        &mut self,
        _source: Entity<Source>,
        event: &InputEvent,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputEvent::Change) {
            self.pending_save = Some(autosave::after_delay(cx, |view, cx| {
                if let Err(error) = view.save_and_reload_list(cx) {
                    view.report(&error, cx);
                }
            }));
        }
    }

    fn reload_list(&mut self, cx: &mut Context<Self>) {
        match self.store.list_notes() {
            Ok(notes) => self.notes = notes,
            Err(error) => self.report(&error, cx),
        }
    }

    fn save_selected(&mut self, cx: &mut Context<Self>) -> Result<(), StoreError> {
        let Some(id) = self.selected.clone() else {
            return Ok(());
        };
        let title = self.title_input.read(cx).value();
        let body = self.body_editor.read(cx).value();
        self.store.save_note(&id, &title, &body, Timestamp::now())?;
        self.error_message = None;
        cx.notify();
        Ok(())
    }

    fn save_and_reload_list(&mut self, cx: &mut Context<Self>) -> Result<(), StoreError> {
        self.save_selected(cx)?;
        self.reload_list(cx);
        Ok(())
    }

    fn flush_pending_save(&mut self, cx: &mut Context<Self>) -> Result<(), StoreError> {
        self.pending_save = None;
        self.save_and_reload_list(cx)
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

    fn select(&mut self, id: &NoteId, window: &mut Window, cx: &mut Context<Self>) {
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

    fn create_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Err(error) = self.flush_pending_save(cx) {
            self.report(&error, cx);
            return;
        }
        match self.store.create_note(Timestamp::now()) {
            Ok(note) => {
                self.reload_list(cx);
                self.show(note, window, cx);
                self.title_input
                    .update(cx, |title, cx| title.focus(window, cx));
            }
            Err(error) => self.report(&error, cx),
        }
    }

    fn show(&mut self, note: Note, window: &mut Window, cx: &mut Context<Self>) {
        self.title_input
            .update(cx, |title, cx| title.set_value(note.title, window, cx));
        self.body_editor
            .update(cx, |body, cx| body.set_value(note.body, window, cx));
        self.selected = Some(note.id);
        cx.notify();
    }

    fn report(&mut self, error: &StoreError, cx: &mut Context<Self>) {
        self.error_message = Some(error.to_string().into());
        cx.notify();
    }
}

impl Focusable for NotesView {
    fn focus_handle(&self, _cx: &gpui_kit::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for NotesView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pane = if self.selected.is_some() {
            self.render_editor_pane().into_any_element()
        } else {
            Self::render_empty_pane(cx).into_any_element()
        };
        div()
            .track_focus(&self.focus_handle)
            .size_full()
            .flex()
            .flex_col()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .on_action(cx.listener(|view, _: &NewNote, window, cx| view.create_note(window, cx)))
            .when_some(self.error_message.clone(), |root, message| {
                root.child(error_banner(message, cx))
            })
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(self.render_sidebar(cx))
                    .child(pane),
            )
    }
}
