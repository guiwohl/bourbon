#[cfg(test)]
mod tests;

mod control_pane_intents;
mod data_intents;
mod dialogs;
mod editor_intents;
mod menus;
mod note_list_intents;
mod settings_intents;
mod sidebar_intents;
mod writing_intents;

pub(crate) use menus::{MenuBuilder, OpenMenu};

use std::rc::Rc;
use std::time::Duration;

use bourbon_core::{
    Note, NoteId, NoteQuery, NoteScope, NoteSeed, NoteStore, NoteSummary, NotebookId, NotebookNode,
    Settings, StoreError, TagSummary, TemplateTimeZone, Timestamp, ViewMode,
};
use bourbon_editor::{TextField, TextFieldEvent};
use bourbon_ui::{IconName, Splitter, ThemeExt as _};
use gpui::prelude::FluentBuilder as _;
use gpui::{
    AnyElement, AppContext as _, Context, Entity, FocusHandle, Focusable, InteractiveElement as _,
    IntoElement, ListState, ParentElement as _, Pixels, Point, Render, ScrollHandle, SharedString,
    Styled as _, Subscription, Task, Window, div,
};

use crate::control_pane::{self, ControlPane};
use crate::editor::{self, EditorPane, EditorView, History};
use crate::locations::Locations;
use crate::navigation::{Listing, Navigation};
use crate::note_list::NoteListData;
use crate::settings_view::{SettingsData, SettingsPage};
use crate::sidebar::{SidebarCounts, SidebarData};
use crate::sidebar::{SidebarState, ToggleSidebar};
use crate::test_id::TestSupportExt as _;
use crate::theme::{self, ModificationWatch};
use crate::{autosave, note_list, settings_view, sidebar};

gpui::actions!(
    bourbon,
    [NewNote, OpenNoteSearch, OpenActionSearch, OpenSettings]
);

pub struct Workspace {
    store: NoteStore,
    listing: Listing,
    notes: Rc<[NoteSummary]>,
    note_list_state: ListState,
    templates: Vec<NoteSummary>,
    counts: SidebarCounts,
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
    settings_page: SettingsPage,
    repository_input: Entity<TextField>,
    token_input: Entity<TextField>,
    filter_input: Entity<TextField>,
    focus_handle: FocusHandle,
    sidebar_focus: FocusHandle,
    note_list_focus: FocusHandle,
    sidebar_scroll: ScrollHandle,
    open_menu: Option<OpenMenu>,
    dialogs: dialogs::DialogState,
    menu_origin: Point<Pixels>,
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
        let (repository_input, token_input) = backup_inputs(&settings, &mut subscriptions, cx);
        let filter_input = cx.new(|cx| {
            TextField::new(cx)
                .placeholder("Filter")
                .leading_icon(IconName::Search)
        });
        subscriptions.push(cx.subscribe_in(
            &filter_input,
            window,
            |view, _, event: &TextFieldEvent, window, cx| match event {
                TextFieldEvent::Changed(_) => view.filter_changed(cx),
                TextFieldEvent::Submit => view.focus_note_list(window, cx),
                TextFieldEvent::Cancel => view.clear_filter(cx),
            },
        ));
        let mut sidebar = SidebarState::new(store.default_notebook().ok());
        sidebar.is_visible = settings.sidebar_visible;
        let omarchy_watch = ModificationWatch::new(locations.omarchy_colors.clone());
        let mut view = Self {
            store,
            listing: Listing::default(),
            notes: Rc::from(Vec::new()),
            note_list_state: note_list::new_list_state(),
            templates: Vec::new(),
            counts: SidebarCounts::default(),
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
            settings_page: SettingsPage::default(),
            repository_input,
            token_input,
            filter_input,
            focus_handle: cx.focus_handle(),
            sidebar_focus: cx.focus_handle(),
            note_list_focus: cx.focus_handle(),
            sidebar_scroll: ScrollHandle::new(),
            open_menu: None,
            dialogs: dialogs::DialogState::new(cx.focus_handle()),
            menu_origin: Point::default(),
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
        match SidebarCounts::load(&self.store) {
            Ok(counts) => self.counts = counts,
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
        let matching = self.filter_matches(cx);
        match self.store.list_notes(&self.listing.query()) {
            Ok(notes) => {
                let previous = self.notes.len();
                self.notes = Rc::from(note_list::narrow(notes, matching.as_ref()));
                self.note_list_state.splice(0..previous, self.notes.len());
            }
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
        if let Some(position) = self.notes.iter().position(|listed| listed.id == note.id) {
            note_list::reveal(&self.note_list_state, position);
        }
        self.selected = Some(note.id);
        cx.notify();
    }

    fn report(&mut self, error: &StoreError, cx: &mut Context<Self>) {
        self.error_message = Some(error.to_string().into());
        cx.notify();
    }
}

impl Workspace {
    fn settings_screen(&self, cx: &mut Context<Self>) -> AnyElement {
        settings_view::render(
            &SettingsData {
                settings: &self.settings,
                page: self.settings_page,
                repository_input: &self.repository_input,
                token_input: &self.token_input,
                repository_error: self.data.repository_error.clone(),
                status: self.data.status.clone(),
            },
            cx,
        )
        .into_any_element()
    }

    fn open_pane(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.selected.is_some() {
            self.editor
                .render(
                    &EditorView {
                        notebooks: &self.notebooks,
                        tags: &self.tags,
                        mode: self.view_mode,
                        focus_mode: self.focus_mode,
                    },
                    cx,
                )
                .into_any_element()
        } else {
            editor::render_empty(cx).into_any_element()
        }
    }
}

fn backup_inputs(
    settings: &Settings,
    subscriptions: &mut Vec<Subscription>,
    cx: &mut Context<Workspace>,
) -> (Entity<TextField>, Entity<TextField>) {
    let repository = cx.new(|cx| {
        let mut field = TextField::new(cx).placeholder("owner/name");
        field.set_text(
            settings.backup_repository.as_deref().unwrap_or_default(),
            cx,
        );
        field
    });
    let token = cx.new(|cx| {
        TextField::new(cx)
            .masked(true)
            .placeholder("Personal access token")
    });
    subscriptions.push(
        cx.subscribe(&repository, |view, _, event: &TextFieldEvent, cx| {
            if matches!(event, TextFieldEvent::Changed(_)) {
                view.repository_changed(cx);
            }
        }),
    );
    subscriptions.push(cx.subscribe(&token, |view, _, event: &TextFieldEvent, cx| {
        if matches!(event, TextFieldEvent::Changed(_)) {
            view.token_changed(cx);
        }
    }));
    (repository, token)
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
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Workspace {
    fn sidebar_column(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Option<impl IntoElement + use<>> {
        if self.focus_mode {
            return None;
        }
        sidebar::render(
            &SidebarData {
                current: &self.listing.navigation,
                notebooks: &self.notebooks,
                tags: &self.tags,
                counts: &self.counts,
                state: &self.sidebar,
                width: f32::from(self.settings.sidebar_width()),
                is_focused: self.sidebar_focus.contains_focused(window, cx),
                focus: &self.sidebar_focus,
                scroll: &self.sidebar_scroll,
            },
            cx,
        )
    }

    fn note_list_column(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Option<impl IntoElement + use<>> {
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
                    filter: &self.filter_input,
                    width: f32::from(self.settings.note_list_width()),
                    is_focused: self.note_list_focus.contains_focused(window, cx),
                    focus: &self.note_list_focus,
                    state: &self.note_list_state,
                },
                cx,
            )
        })
    }

    fn sidebar_splitter(&self, cx: &mut Context<Self>) -> Option<impl IntoElement + use<>> {
        (!self.focus_mode && self.sidebar.is_visible).then(|| {
            let limits = cx.theme().tokens.scale.sidebar_width;
            let (resizing, finishing) = (cx.entity(), cx.entity());
            div().id("sidebar-splitter").test_support().child(
                Splitter::new(
                    "sidebar-splitter-handle",
                    gpui::px(f32::from(self.settings.sidebar_width())),
                    limits,
                )
                .on_resize(move |width, _, cx| {
                    resizing.update(cx, |workspace, cx| {
                        workspace.resize_sidebar(f32::from(width), cx);
                    });
                })
                .on_resize_end(move |_, cx| {
                    finishing.update(cx, Workspace::finish_resizing);
                }),
            )
        })
    }

    fn note_list_splitter(&self, cx: &mut Context<Self>) -> Option<impl IntoElement + use<>> {
        (!self.focus_mode).then(|| {
            let limits = cx.theme().tokens.scale.note_list_width;
            let (resizing, finishing) = (cx.entity(), cx.entity());
            div().id("note-list-splitter").test_support().child(
                Splitter::new(
                    "note-list-splitter-handle",
                    gpui::px(f32::from(self.settings.note_list_width())),
                    limits,
                )
                .on_resize(move |width, _, cx| {
                    resizing.update(cx, |workspace, cx| {
                        workspace.resize_note_list(f32::from(width), cx);
                    });
                })
                .on_resize_end(move |_, cx| {
                    finishing.update(cx, Workspace::finish_resizing);
                }),
            )
        })
    }
}

impl Workspace {
    fn columns(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let pane = self.open_pane(cx);
        div()
            .flex_1()
            .min_w_0()
            .flex()
            .children(self.sidebar_column(window, cx))
            .children(self.sidebar_splitter(cx))
            .children(self.note_list_column(window, cx))
            .children(self.note_list_splitter(cx))
            .child(pane)
            .into_any_element()
    }
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = if self.settings_open {
            self.settings_screen(cx)
        } else {
            self.columns(window, cx)
        };
        let menu = self.menu_popover(cx);
        let dialog = self.dialog(window, cx);
        div()
            .track_focus(&self.focus_handle)
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .bg(cx.theme().tokens.colors.page_background)
            .text_color(cx.theme().tokens.colors.text)
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
            .child(div().flex_1().min_h_0().flex().child(body))
            .children(
                self.control_pane
                    .as_ref()
                    .map(|open_pane| control_pane::render(open_pane, window, cx)),
            )
            .children(dialog)
            .children(menu)
    }
}
