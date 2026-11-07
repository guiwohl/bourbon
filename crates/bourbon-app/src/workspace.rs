#[cfg(test)]
mod tests;

mod control_pane_intents;
mod data_intents;
mod editor_intents;
mod note_list_intents;
mod settings_intents;
mod sidebar_intents;
mod writing_intents;

use std::time::Duration;

use bourbon_core::{
    Note, NoteId, NoteQuery, NoteScope, NoteSeed, NoteStore, NoteSummary, NotebookId, NotebookNode,
    Settings, StoreError, TagSummary, TemplateTimeZone, Timestamp, ViewMode,
};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, AppContext as _, Context, DragMoveEvent, Entity, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, MouseButton, ParentElement as _, Render, SharedString,
    Styled as _, Subscription, Task, Window, div,
};

use crate::control_pane::{self, ControlPane};
use crate::editor::{self, EditorPane, EditorView, History};
use crate::locations::Locations;
use crate::navigation::{Listing, Navigation};
use crate::note_list::NoteListData;
use crate::panes::{self, Divider};
use crate::settings_view::SettingsData;
use crate::sidebar::SidebarData;
use crate::sidebar::{SidebarState, ToggleSidebar};
use crate::theme::{self, ModificationWatch};
use crate::{autosave, note_list, settings_view, sidebar};

gpui_kit::actions!(
    bourbon,
    [NewNote, OpenNoteSearch, OpenActionSearch, OpenSettings]
);

pub struct Workspace {
    store: NoteStore,
    listing: Listing,
    notes: Vec<NoteSummary>,
    templates: Vec<NoteSummary>,
    notebooks: Vec<NotebookNode>,
    tags: Vec<TagSummary>,
    selected: Option<NoteId>,
    sidebar: SidebarState,
    editor: EditorPane,
    history: History,
    control_pane: Option<ControlPane>,
    view_mode: ViewMode,
    focus_mode: bool,
    pending_save: Option<Task<()>>,
    locations: Locations,
    settings: Settings,
    settings_open: bool,
    widths_changed: bool,
    omarchy_watch: ModificationWatch,
    _omarchy_polling: Task<()>,
    error_message: Option<SharedString>,
    notice: Option<SharedString>,
    data: data_intents::DataState,
    repository_input: Entity<InputState>,
    token_input: Entity<InputState>,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl Workspace {
    pub(crate) fn new(
        store: NoteStore,
        locations: Locations,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let settings = locations.settings.load();
        theme::apply(&settings, &locations.omarchy_colors, cx);
        let editor = EditorPane::new(window, cx);
        let mut subscriptions = editor.subscribe_to_edits(cx);
        subscriptions.push(cx.on_app_quit(|view, cx| {
            view.flush_before_closing(cx);
            std::future::ready(())
        }));
        let repository_input = cx.new(|cx| InputState::new(window, cx).placeholder("owner/name"));
        repository_input.update(cx, |input, cx| {
            input.set_value(
                settings.backup_repository.clone().unwrap_or_default(),
                window,
                cx,
            );
        });
        let token_input = cx.new(|cx| {
            InputState::new(window, cx)
                .masked(true)
                .placeholder("Personal access token")
        });
        subscriptions.push(
            cx.subscribe(&repository_input, |view, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    view.repository_changed(cx);
                }
            }),
        );
        subscriptions.push(
            cx.subscribe(&token_input, |view, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    view.token_changed(cx);
                }
            }),
        );
        let mut sidebar = SidebarState::new(store.default_notebook().ok());
        sidebar.is_visible = settings.sidebar_visible;
        let omarchy_watch = ModificationWatch::new(locations.omarchy_colors.clone());
        let mut view = Self {
            store,
            listing: Listing::default(),
            notes: Vec::new(),
            templates: Vec::new(),
            notebooks: Vec::new(),
            tags: Vec::new(),
            selected: None,
            sidebar,
            editor,
            history: History::default(),
            control_pane: None,
            view_mode: settings.editor_view_mode,
            focus_mode: false,
            pending_save: None,
            locations,
            settings,
            settings_open: false,
            widths_changed: false,
            omarchy_watch,
            _omarchy_polling: poll_omarchy_theme(cx),
            error_message: None,
            notice: None,
            data: data_intents::DataState::default(),
            repository_input,
            token_input,
            focus_handle: cx.focus_handle(),
            _subscriptions: subscriptions,
        };
        view.purge_expired_trash(cx);
        view.reload_list(cx);
        view.reload_sidebar(cx);
        if let Some(most_recent) = view.notes.first().map(|summary| summary.id.clone()) {
            view.select(&most_recent, window, cx);
        }
        window.focus(&view.focus_handle, cx);
        view
    }

    pub(crate) fn restart_autosave(&mut self, cx: &mut Context<Self>) {
        self.note_data_changed(cx);
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
        let templates = NoteQuery {
            scope: NoteScope::Templates,
            sort: self.listing.sort,
        };
        match self.store.list_notes(&templates) {
            Ok(templates) => self.templates = templates,
            Err(error) => self.report(&error, cx),
        }
    }

    pub(super) fn refresh_listing(&mut self, cx: &mut Context<Self>) {
        self.note_data_changed(cx);
        self.reload_list(cx);
        self.reload_sidebar(cx);
        cx.notify();
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
        self.notice = None;
        cx.notify();
        Ok(())
    }

    fn save_and_reload(&mut self, cx: &mut Context<Self>) -> Result<(), StoreError> {
        self.save_selected(cx)?;
        self.reload_list(cx);
        Ok(())
    }

    pub(super) fn flush_pending_save(&mut self, cx: &mut Context<Self>) -> Result<(), StoreError> {
        self.pending_save = None;
        self.save_and_reload(cx)
    }

    pub(super) fn flush_then<T>(
        &mut self,
        cx: &mut Context<Self>,
        action: impl FnOnce(&mut Self, &mut Context<Self>) -> T,
    ) -> Option<T> {
        match self.flush_pending_save(cx) {
            Ok(()) => Some(action(self, cx)),
            Err(error) => {
                self.report(&error, cx);
                None
            }
        }
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
        self.flush_then(cx, |this, cx| match load_note(&this.store, id) {
            Ok(note) => this.show(note, window, cx),
            Err(error) => this.report(&error, cx),
        });
    }

    pub(crate) fn create_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.listing.navigation == Navigation::Templates {
            self.create_template(window, cx);
            return;
        }
        self.flush_then(cx, |this, cx| {
            this.leave_view_hiding_new_notes();
            let created = this.new_note_placement().and_then(|(notebook, seed)| {
                this.store
                    .create_note_with(&notebook, &seed, Timestamp::now())
            });
            this.show_created(created, window, cx);
        });
    }

    pub(crate) fn create_template(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.flush_then(cx, |this, cx| {
            this.listing.navigation = Navigation::Templates;
            let created = this
                .store
                .default_notebook()
                .and_then(|inbox| this.store.create_template(&inbox, Timestamp::now()));
            this.show_created(created, window, cx);
        });
    }

    pub(crate) fn create_note_from_template(
        &mut self,
        template: &NoteId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.flush_then(cx, |this, cx| {
            this.leave_view_hiding_new_notes();
            let created = this.new_note_placement().and_then(|(notebook, _)| {
                this.store.create_note_from_template(
                    template,
                    &notebook,
                    Timestamp::now(),
                    &TemplateTimeZone::system(),
                )
            });
            this.show_created(created, window, cx);
        });
    }

    fn leave_view_hiding_new_notes(&mut self) {
        if self.listing.navigation.hides_new_notes() {
            self.listing.navigation = Navigation::AllNotes;
        }
    }

    fn show_created(
        &mut self,
        created: Result<Note, StoreError>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.refresh_listing(cx);
        match created {
            Ok(note) => {
                self.show(note, window, cx);
                self.editor.focus_title(window, cx);
            }
            Err(error) => self.report(&error, cx),
        }
    }

    fn new_note_placement(&self) -> Result<(NotebookId, NoteSeed), StoreError> {
        Ok(match &self.listing.navigation {
            Navigation::Notebook(notebook, _) => (notebook.clone(), NoteSeed::default()),
            Navigation::Tags(tags) => (
                self.store.default_notebook()?,
                NoteSeed {
                    tags: tags.clone(),
                    ..NoteSeed::default()
                },
            ),
            Navigation::Status(status) => (
                self.store.default_notebook()?,
                NoteSeed {
                    status: *status,
                    ..NoteSeed::default()
                },
            ),
            _ => (self.store.default_notebook()?, NoteSeed::default()),
        })
    }

    fn show(&mut self, note: Note, window: &mut Window, cx: &mut Context<Self>) {
        self.settings_open = false;
        self.editor.link_picker = None;
        self.editor.load(note.clone(), window, cx);
        self.history.visit(&note.id);
        self.selected = Some(note.id);
        cx.notify();
    }

    fn report(&mut self, error: &StoreError, cx: &mut Context<Self>) {
        self.error_message = Some(error.to_string().into());
        cx.notify();
    }
}

impl Workspace {
    fn open_pane(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.settings_open {
            settings_view::render(
                &SettingsData {
                    settings: &self.settings,
                    repository_input: &self.repository_input,
                    token_input: &self.token_input,
                    repository_error: self.data.repository_error.clone(),
                    status: self.data.status.clone(),
                },
                cx,
            )
            .into_any_element()
        } else if self.selected.is_some() {
            self.editor
                .render(
                    &EditorView {
                        notebooks: &self.notebooks,
                        tags: &self.tags,
                        mode: self.view_mode,
                        focus_mode: self.focus_mode,
                        can_go_back: self.history.can_go_back(),
                        can_go_forward: self.history.can_go_forward(),
                    },
                    cx,
                )
                .into_any_element()
        } else {
            editor::render_empty(cx).into_any_element()
        }
    }
}

const OMARCHY_POLL_INTERVAL: Duration = Duration::from_secs(2);

fn poll_omarchy_theme(cx: &mut Context<Workspace>) -> Task<()> {
    cx.spawn(async move |view, cx| {
        loop {
            cx.background_executor().timer(OMARCHY_POLL_INTERVAL).await;
            let alive = view.update(cx, Workspace::reload_omarchy_theme_if_changed);
            if alive.is_err() {
                break;
            }
        }
    })
}

pub(super) fn load_note(store: &NoteStore, id: &NoteId) -> Result<Note, StoreError> {
    store
        .find_note(id)?
        .ok_or_else(|| StoreError::NoteNotFound(id.clone()))
}

impl Focusable for Workspace {
    fn focus_handle(&self, _cx: &gpui_kit::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Workspace {
    fn sidebar_column(&self, cx: &mut Context<Self>) -> Option<impl IntoElement + use<>> {
        if self.focus_mode {
            return None;
        }
        sidebar::render(
            &SidebarData {
                current: &self.listing.navigation,
                notebooks: &self.notebooks,
                tags: &self.tags,
                state: &self.sidebar,
                width: f32::from(self.settings.sidebar_width()),
            },
            cx,
        )
    }

    fn note_list_column(&self, cx: &mut Context<Self>) -> Option<impl IntoElement + use<>> {
        (!self.focus_mode).then(|| {
            note_list::render(
                &NoteListData {
                    navigation: &self.listing.navigation,
                    sort: self.listing.sort,
                    notebooks: &self.notebooks,
                    tags: &self.tags,
                    notes: &self.notes,
                    templates: &self.templates,
                    selected: self.selected.as_ref(),
                    width: f32::from(self.settings.note_list_width()),
                },
                cx,
            )
        })
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pane = self.open_pane(cx);
        let sidebar = self.sidebar_column(cx);
        let note_list = self.note_list_column(cx);
        div()
            .track_focus(&self.focus_handle)
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .on_drag_move::<Divider>(cx.listener(|view, event: &DragMoveEvent<Divider>, _, cx| {
                let pointer = f32::from(event.event.position.x);
                match event.drag(cx) {
                    Divider::AfterSidebar => view.resize_sidebar(pointer, cx),
                    Divider::AfterNoteList => {
                        view.resize_note_list_to_pointer(pointer, cx);
                    }
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|view, _, _, cx| view.finish_resizing(cx)),
            )
            .on_action(cx.listener(|view, _: &NewNote, window, cx| view.create_note(window, cx)))
            .on_action(cx.listener(|view, _: &ToggleSidebar, _, cx| view.toggle_sidebar(cx)))
            .on_action(cx.listener(|view, _: &OpenSettings, _, cx| view.open_settings(cx)))
            .on_action(cx.listener(|view, _: &OpenNoteSearch, window, cx| {
                view.open_note_search(window, cx);
            }))
            .on_action(cx.listener(|view, _: &OpenActionSearch, window, cx| {
                view.open_action_search(window, cx);
            }))
            .map(|root| editor::actions::register_writing(root, cx))
            .map(|root| editor::actions::register(root, cx))
            .when_some(self.error_message.clone(), |root, message| {
                root.child(editor::error_banner(message, cx))
            })
            .when_some(self.notice.clone(), |root, message| {
                root.child(settings_view::notice_banner(message, cx))
            })
            .when_some(self.restore_question(), |root, question| {
                root.child(settings_view::restore_confirmation(question, cx))
            })
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .children(sidebar.map(|sidebar| {
                        div().flex().child(sidebar).child(panes::divider(
                            "sidebar-divider",
                            Divider::AfterSidebar,
                            cx,
                        ))
                    }))
                    .children(note_list.map(|note_list| {
                        div().flex().child(note_list).child(panes::divider(
                            "note-list-divider",
                            Divider::AfterNoteList,
                            cx,
                        ))
                    }))
                    .child(pane),
            )
            .children(
                self.control_pane
                    .as_ref()
                    .map(|open_pane| control_pane::render(open_pane, cx)),
            )
    }
}
