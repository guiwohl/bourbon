use bourbon_core::{Note, NotebookNode, Tag, TagSummary, ViewMode};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Editor, EditorState, Input, InputEvent, InputState};
use gpui_kit::component::label::Label;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    App, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, Styled as _, Subscription, TestSupportExt as _, Window, div,
    px,
};

use crate::presentation::UNTITLED;
use crate::workspace::Workspace;

pub(crate) mod actions;
mod header;
mod history;
mod link_picker;
mod link_trigger;
mod preview;
mod preview_blocks;
pub(crate) mod tasks;
mod toolbar;
mod word_count;

pub(crate) use history::History;
pub(crate) use link_picker::{LinkPicker, MAX_SUGGESTIONS};
pub(crate) use link_trigger::{closing_brackets_after, link_query_before_cursor};
pub(crate) use tasks::TextEdit;

pub(crate) use word_count::WordCount;

use header::{HeaderView, NoteHeader};

const FOCUS_COLUMN_WIDTH: f32 = 720.;

pub(crate) struct EditorPane {
    title_input: Entity<InputState>,
    body_editor: Entity<EditorState>,
    tag_input: Entity<InputState>,
    note_header: Option<NoteHeader>,
    tag_highlight: Option<usize>,
    notebook_selector_open: bool,
    pub(crate) link_picker: Option<LinkPicker>,
    _subscriptions: Vec<Subscription>,
}

impl EditorPane {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Workspace>) -> Self {
        let title_input = cx.new(|cx| InputState::new(window, cx).placeholder(UNTITLED));
        let body_editor = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("markdown")
                .soft_wrap(true)
                .line_number(false)
        });
        let tag_input = cx.new(|cx| InputState::new(window, cx).placeholder("Add tags"));
        let tag_events = cx.subscribe_in(
            &tag_input,
            window,
            |workspace, _, event: &InputEvent, window, cx| match event {
                InputEvent::Change => workspace.tag_input_changed(cx),
                InputEvent::PressEnter { .. } => workspace.commit_tag_input(window, cx),
                _ => {}
            },
        );
        Self {
            title_input,
            body_editor,
            tag_input,
            note_header: None,
            tag_highlight: None,
            notebook_selector_open: false,
            link_picker: None,
            _subscriptions: vec![tag_events],
        }
    }

    pub(crate) fn subscribe_to_edits(&self, cx: &mut Context<Workspace>) -> Vec<Subscription> {
        vec![
            cx.subscribe(&self.title_input, Self::forward_title_edit),
            cx.subscribe(&self.body_editor, Self::forward_body_edit),
        ]
    }

    fn forward_title_edit(
        workspace: &mut Workspace,
        _source: Entity<InputState>,
        event: &InputEvent,
        cx: &mut Context<Workspace>,
    ) {
        if matches!(event, InputEvent::Change) {
            workspace.restart_autosave(cx);
        }
    }

    fn forward_body_edit(
        workspace: &mut Workspace,
        _source: Entity<EditorState>,
        event: &InputEvent,
        cx: &mut Context<Workspace>,
    ) {
        if matches!(event, InputEvent::Change) {
            workspace.body_edited(cx);
        }
    }

    pub(crate) fn title(&self, cx: &App) -> String {
        self.title_input.read(cx).value().to_string()
    }

    pub(crate) fn body(&self, cx: &App) -> String {
        self.body_editor.read(cx).value().to_string()
    }

    pub(crate) fn load(&mut self, note: Note, window: &mut Window, cx: &mut Context<Workspace>) {
        self.show_header(&note);
        self.title_input
            .update(cx, |title, cx| title.set_value(note.title, window, cx));
        self.body_editor
            .update(cx, |body, cx| body.set_value(note.body, window, cx));
    }

    pub(crate) fn text_before_cursor(&self, cx: &App) -> String {
        let body = self.body_editor.read(cx);
        body.value()[..body.cursor()].to_owned()
    }

    pub(crate) fn text_after_cursor(&self, cx: &App) -> String {
        let body = self.body_editor.read(cx);
        body.value()[body.cursor()..].to_owned()
    }

    pub(crate) fn cursor(&self, cx: &App) -> usize {
        self.body_editor.read(cx).cursor()
    }

    pub(crate) fn replace_range(
        &self,
        range: std::ops::Range<usize>,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) {
        self.body_editor.update(cx, |body, cx| {
            body.set_selected_range(range, cx);
            body.replace(text.to_owned(), window, cx);
        });
    }

    pub(crate) fn apply_edit(
        &self,
        edit: &TextEdit,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) {
        let cursor = self.cursor(cx);
        self.replace_range(edit.range.clone(), &edit.text, window, cx);
        let moved = if cursor >= edit.range.end {
            cursor + edit.text.len() - edit.range.len()
        } else {
            cursor
        };
        self.body_editor
            .update(cx, |body, cx| body.set_selected_range(moved..moved, cx));
    }

    pub(crate) fn open_search(
        &self,
        replace_mode: bool,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) {
        self.body_editor.update(cx, |body, cx| {
            body.focus(window, cx);
            body.open_search(replace_mode, cx);
        });
    }

    pub(crate) fn show_header(&mut self, note: &Note) {
        self.note_header = Some(NoteHeader::of(note));
    }

    pub(crate) fn tag_input_text(&self, cx: &App) -> String {
        self.tag_input.read(cx).value().to_string()
    }

    pub(crate) fn clear_tag_input(&mut self, window: &mut Window, cx: &mut Context<Workspace>) {
        self.tag_input
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.tag_highlight = None;
    }

    pub(crate) fn forget_tag_highlight(&mut self) {
        self.tag_highlight = None;
    }

    pub(crate) fn highlight_next_tag(&mut self, typed: &str, tags: &[TagSummary]) {
        let Some(header) = &self.note_header else {
            return;
        };
        let count = header::suggestions(typed, tags, header).len();
        if count > 0 {
            self.tag_highlight = Some(
                self.tag_highlight
                    .map_or(0, |index| (index + 1).min(count - 1)),
            );
        }
    }

    pub(crate) fn highlight_previous_tag(&mut self) {
        self.tag_highlight = self.tag_highlight.and_then(|index| index.checked_sub(1));
    }

    pub(crate) fn chosen_tag(&self, typed: &str, tags: &[TagSummary]) -> Option<Tag> {
        let header = self.note_header.as_ref()?;
        match self.tag_highlight {
            Some(index) => header::suggestions(typed, tags, header)
                .get(index)
                .map(|tag| (*tag).clone()),
            None => header::tag_named(typed, tags).cloned(),
        }
    }

    pub(crate) fn set_notebook_selector_open(&mut self, open: bool) {
        self.notebook_selector_open = open;
    }

    #[cfg(test)]
    pub(crate) fn notebook_selector_is_open(&self) -> bool {
        self.notebook_selector_open
    }

    pub(crate) fn focus_body(&self, window: &mut Window, cx: &mut Context<Workspace>) {
        self.body_editor
            .update(cx, |body, cx| body.focus(window, cx));
    }

    #[cfg(test)]
    pub(crate) fn place_cursor_at(&self, offset: usize, cx: &mut Context<Workspace>) {
        self.body_editor
            .update(cx, |body, cx| body.set_selected_range(offset..offset, cx));
    }

    #[cfg(test)]
    pub(crate) fn place_cursor_at_end(&self, cx: &mut Context<Workspace>) {
        let end = self.body(cx).len();
        self.body_editor
            .update(cx, |body, cx| body.set_selected_range(end..end, cx));
    }

    #[cfg(test)]
    pub(crate) fn place_cursor_on_line(
        &self,
        line: usize,
        _window: &mut Window,
        cx: &mut Context<Workspace>,
    ) {
        let offset = self
            .body(cx)
            .split('\n')
            .take(line)
            .map(|text| text.len() + 1)
            .sum();
        self.body_editor
            .update(cx, |body, cx| body.set_selected_range(offset..offset, cx));
    }

    #[cfg(test)]
    pub(crate) fn search_is_open(&self, cx: &App) -> bool {
        self.body_editor.read(cx).search_session().open
    }

    #[cfg(test)]
    pub(crate) fn search_is_replacing(&self, cx: &App) -> bool {
        self.body_editor.read(cx).search_session().replace_mode
    }

    pub(crate) fn focus_title(&self, window: &mut Window, cx: &mut Context<Workspace>) {
        self.title_input
            .update(cx, |title, cx| title.focus(window, cx));
    }

    pub(crate) fn render(
        &self,
        view: &EditorView<'_>,
        cx: &mut Context<Workspace>,
    ) -> impl IntoElement + use<> {
        let typed_tag = self.tag_input_text(cx);
        let header = self.note_header.as_ref().map(|header| {
            header::render(
                &HeaderView {
                    header,
                    notebooks: view.notebooks,
                    tags: view.tags,
                    tag_input: &self.tag_input,
                    typed_tag: &typed_tag,
                    highlight: self.tag_highlight,
                    notebook_selector_open: self.notebook_selector_open,
                },
                cx,
            )
        });
        let body = self.body(cx);
        let mut panes = div().flex_1().min_h_0().flex().gap_2();
        if view.mode.shows_editor() {
            panes = panes.child(self.render_body_editor(cx));
        }
        if view.mode.shows_preview() {
            panes = panes.child(preview::render(&body, cx));
        }
        div().flex_1().h_full().flex().justify_center().child(
            div()
                .w_full()
                .h_full()
                .flex()
                .flex_col()
                .gap_2()
                .p_3()
                .when(view.focus_mode, |column| {
                    column.max_w(px(FOCUS_COLUMN_WIDTH))
                })
                .child(
                    Input::new(&self.title_input)
                        .id("title")
                        .aria_label("Title"),
                )
                .children(header)
                .child(toolbar::render(view.mode, cx))
                .child(panes)
                .child(
                    div().id("word-count").test_support().child(
                        Label::new(WordCount::of(&body).label())
                            .text_sm()
                            .text_color(cx.theme().muted_foreground),
                    ),
                ),
        )
    }

    fn render_body_editor(&self, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
        div()
            .id("body-editor")
            .test_support()
            .when(self.link_picker.is_some(), |wrapper| {
                wrapper.key_context("LinkPicker")
            })
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .flex_col()
            .gap_1()
            .children(
                self.link_picker
                    .as_ref()
                    .map(|picker| link_picker::render(picker, cx)),
            )
            .child(
                Editor::new(&self.body_editor)
                    .aria_label("Body")
                    .h_full()
                    .flex_1(),
            )
    }
}

pub(crate) struct EditorView<'a> {
    pub(crate) notebooks: &'a [NotebookNode],
    pub(crate) tags: &'a [TagSummary],
    pub(crate) mode: ViewMode,
    pub(crate) focus_mode: bool,
}

pub(crate) fn render_empty(cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    div()
        .flex_1()
        .h_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_3()
        .child(
            div()
                .text_color(cx.theme().muted_foreground)
                .child("No notes yet"),
        )
        .child(
            Button::new("empty-new-note")
                .primary()
                .label("New note")
                .on_click(cx.listener(|workspace, _, window, cx| {
                    workspace.create_note(window, cx);
                })),
        )
}

pub(crate) fn error_banner(
    message: SharedString,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    div()
        .id("error-banner")
        .test_support()
        .px_3()
        .py_1()
        .rounded_md()
        .bg(cx.theme().danger)
        .text_color(cx.theme().danger_foreground)
        .child(message)
}
