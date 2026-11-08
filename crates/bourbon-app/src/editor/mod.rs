use std::time::Instant;

use bourbon_core::{Note, NotebookNode, Tag, TagSummary, ViewMode};
use bourbon_editor::{Editor, EditorEvent, PreviewEvent, PreviewView, TextField, TextFieldEvent};
use bourbon_ui::{Appearance, Button, ButtonVariant, ThemeExt as _};
use gpui::{
    App, AppContext as _, Context, Entity, Hsla, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, Styled as _, Subscription, Window, div, px,
};

use crate::presentation::UNTITLED;
use crate::test_id::TestSupportExt as _;
use crate::workspace::Workspace;

pub(crate) mod actions;
mod body;
mod header;
mod history;
mod link_picker;
mod link_trigger;
mod menu_anchor;
mod scroll_sync;
mod themes;
mod toolbar;
mod word_count;

pub(crate) use header::{note_actions_menu, notebook_menu, status_menu};
pub(crate) use history::History;
pub(crate) use link_picker::{LinkPicker, MAX_SUGGESTIONS};
pub(crate) use link_trigger::{closing_brackets_after, link_query_before_cursor};
#[cfg(test)]
pub(crate) use toolbar::mode_index;
pub(crate) use word_count::WordCount;

use body::BodyPanes;
use header::{HeaderView, NoteHeader};
use menu_anchor::MenuAnchors;
use scroll_sync::ScrollSync;
use toolbar::ToolbarData;

const TITLE_SIZE: f32 = 21.;
const TITLE_HEIGHT: f32 = 36.;
const TITLE_PADDING_LEFT: f32 = 10.5;
const TITLE_PADDING_RIGHT: f32 = 8.;
const STATUS_BAR_PADDING: f32 = 12.;
const STATUS_BAR_HEIGHT: f32 = 24.;
const BANNER_PADDING_VERTICAL: f32 = 4.;

type ThemeKey = (Appearance, Hsla);

pub(crate) struct EditorPane {
    title: Entity<TextField>,
    body: Entity<Editor>,
    preview: Entity<PreviewView>,
    tag_field: Entity<TextField>,
    note_header: Option<NoteHeader>,
    tag_highlight: Option<usize>,
    pub(crate) link_picker: Option<LinkPicker>,
    pub(crate) anchors: MenuAnchors,
    scroll_sync: ScrollSync,
    applied_theme: ThemeKey,
}

fn theme_key(cx: &App) -> ThemeKey {
    let theme = cx.theme();
    (theme.appearance, theme.tokens.colors.editor_background)
}

impl EditorPane {
    pub(crate) fn new(cx: &mut App) -> Self {
        let themes = themes::pane_themes(cx.theme());
        Self {
            title: cx.new(title_field),
            body: cx.new(|cx| {
                let mut body = Editor::new("", themes.editor, cx);
                body.set_line_numbers(true, cx);
                body
            }),
            preview: cx.new(|cx| PreviewView::new("", themes.preview, cx)),
            tag_field: cx.new(tag_field),
            note_header: None,
            tag_highlight: None,
            link_picker: None,
            anchors: MenuAnchors::default(),
            scroll_sync: ScrollSync::default(),
            applied_theme: theme_key(cx),
        }
    }

    pub(crate) fn subscribe_to_edits(
        &self,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) -> Vec<Subscription> {
        vec![
            cx.subscribe_in(
                &self.title,
                window,
                |workspace, _, event: &TextFieldEvent, window, cx| match event {
                    TextFieldEvent::Changed(_) => workspace.restart_autosave(cx),
                    TextFieldEvent::Submit => workspace.leave_title(window, cx),
                    TextFieldEvent::Cancel => {}
                },
            ),
            cx.subscribe_in(
                &self.tag_field,
                window,
                |workspace, _, event: &TextFieldEvent, _, cx| match event {
                    TextFieldEvent::Changed(_) => workspace.tag_input_changed(cx),
                    TextFieldEvent::Submit => workspace.commit_tag_input(cx),
                    TextFieldEvent::Cancel => {}
                },
            ),
            cx.subscribe(&self.body, |workspace, _, event, cx| {
                Self::forward_body_event(workspace, event, cx);
            }),
            cx.subscribe(&self.preview, |workspace, _, event, cx| {
                Self::forward_preview_event(workspace, event, cx);
            }),
        ]
    }

    fn forward_body_event(
        workspace: &mut Workspace,
        event: &EditorEvent,
        cx: &mut Context<Workspace>,
    ) {
        match event {
            EditorEvent::Changed(_) => workspace.body_edited(cx),
            EditorEvent::Scrolled => workspace.editor_scrolled(cx),
            EditorEvent::TaskToggled { .. } => {}
        }
    }

    fn forward_preview_event(
        workspace: &mut Workspace,
        event: &PreviewEvent,
        cx: &mut Context<Workspace>,
    ) {
        match event {
            PreviewEvent::ToggleTask { source_line } => {
                workspace.toggle_task_on_line(*source_line, cx);
            }
            PreviewEvent::OpenLink(url) => workspace.open_link(url, cx),
            PreviewEvent::OpenNote(id) => workspace.open_note_by_id(id, cx),
            PreviewEvent::Scrolled => workspace.preview_scrolled(cx),
        }
    }

    pub(crate) fn follow_editor_scroll(&mut self, cx: &mut App) {
        let line = self.body.read(cx).top_line();
        let anchors = self.preview.read(cx).scroll_anchors();
        if let Some(offset) = self
            .scroll_sync
            .preview_offset_for(line, &anchors, Instant::now())
        {
            self.preview
                .update(cx, |preview, cx| preview.set_scroll_top(px(offset), cx));
        }
    }

    pub(crate) fn follow_preview_scroll(&mut self, cx: &mut App) {
        let offset = f32::from(self.preview.read(cx).scroll_top());
        let anchors = self.preview.read(cx).scroll_anchors();
        if let Some(line) = self
            .scroll_sync
            .editor_line_for(offset, &anchors, Instant::now())
        {
            self.body
                .update(cx, |body, cx| body.scroll_to_line(line, cx));
        }
    }

    pub(crate) fn refresh_preview(&self, cx: &mut App) {
        let text = self.body(cx);
        self.preview
            .update(cx, |preview, cx| preview.set_source(&text, cx));
    }

    pub(crate) fn toggle_task_on_line(&self, line: usize, cx: &mut App) {
        self.body
            .update(cx, |body, cx| body.toggle_task_at_line(line, cx));
    }

    pub(crate) fn sync_theme(&mut self, cx: &mut App) {
        let key = theme_key(cx);
        if key == self.applied_theme {
            return;
        }
        self.applied_theme = key;
        let themes = themes::pane_themes(cx.theme());
        self.body
            .update(cx, |body, cx| body.set_theme(themes.editor, cx));
        self.preview
            .update(cx, |preview, cx| preview.set_theme(themes.preview, cx));
    }

    pub(crate) fn title(&self, cx: &App) -> String {
        self.title.read(cx).text()
    }

    pub(crate) fn body(&self, cx: &App) -> String {
        self.body.read(cx).text()
    }

    pub(crate) fn load(&mut self, note: &Note, cx: &mut App) {
        self.show_header(note);
        self.title
            .update(cx, |title, cx| title.set_text(&note.title, cx));
        self.body
            .update(cx, |body, cx| body.set_text(&note.body, cx));
        self.preview.update(cx, |preview, cx| {
            preview.set_source(&note.body, cx);
            preview.set_scroll_top(px(0.), cx);
        });
    }

    pub(crate) fn cursor(&self, cx: &App) -> usize {
        self.body.read(cx).selected_range().end
    }

    pub(crate) fn text_before_cursor(&self, cx: &App) -> String {
        self.body(cx)[..self.cursor(cx)].to_owned()
    }

    pub(crate) fn text_after_cursor(&self, cx: &App) -> String {
        self.body(cx)[self.cursor(cx)..].to_owned()
    }

    pub(crate) fn replace_range(&self, range: std::ops::Range<usize>, text: &str, cx: &mut App) {
        self.body
            .update(cx, |body, cx| body.replace_range(range, text, cx));
    }

    pub(crate) fn show_header(&mut self, note: &Note) -> &NoteHeader {
        self.note_header.insert(NoteHeader::of(note))
    }

    pub(crate) fn header(&self) -> Option<&NoteHeader> {
        self.note_header.as_ref()
    }

    pub(crate) fn tag_input_text(&self, cx: &App) -> String {
        self.tag_field.read(cx).text()
    }

    pub(crate) fn clear_tag_input(&mut self, cx: &mut App) {
        self.tag_field
            .update(cx, |field, cx| field.set_text("", cx));
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

    pub(crate) fn focus_body(&self, window: &mut Window, cx: &mut App) {
        let handle = self.body.read(cx).focus_handle().clone();
        window.focus(&handle, cx);
    }

    pub(crate) fn focus_title(&self, window: &mut Window, cx: &mut App) {
        let handle = gpui::Focusable::focus_handle(self.title.read(cx), cx);
        window.focus(&handle, cx);
    }

    #[cfg(test)]
    pub(crate) fn title_is_focused(&self, window: &Window, cx: &App) -> bool {
        gpui::Focusable::focus_handle(self.title.read(cx), cx).is_focused(window)
    }

    #[cfg(test)]
    pub(crate) fn preview(&self) -> &Entity<PreviewView> {
        &self.preview
    }

    #[cfg(test)]
    pub(crate) fn body_editor(&self) -> &Entity<Editor> {
        &self.body
    }

    #[cfg(test)]
    pub(crate) fn place_cursor_at(&self, offset: usize, cx: &mut App) {
        self.body
            .update(cx, |body, cx| body.set_selection(offset, offset, cx));
    }

    #[cfg(test)]
    pub(crate) fn place_cursor_at_end(&self, cx: &mut App) {
        let end = self.body(cx).len();
        self.place_cursor_at(end, cx);
    }

    #[cfg(test)]
    pub(crate) fn place_cursor_on_line(&self, line: usize, cx: &mut App) {
        let offset = self
            .body(cx)
            .split('\n')
            .take(line)
            .map(|text| text.len() + 1)
            .sum();
        self.place_cursor_at(offset, cx);
    }

    pub(crate) fn render(
        &self,
        view: &EditorView<'_>,
        cx: &mut Context<Workspace>,
    ) -> impl IntoElement + use<> {
        let tokens = cx.theme().tokens.clone();
        let typed_tag = self.tag_input_text(cx);
        let header = self.note_header.as_ref().map(|header| {
            header::render(
                &HeaderView {
                    header,
                    notebooks: view.notebooks,
                    tags: view.tags,
                    tag_field: &self.tag_field,
                    typed_tag: &typed_tag,
                    highlight: self.tag_highlight,
                    anchors: &self.anchors,
                },
                cx,
            )
        });
        let toolbar = toolbar::render(&view.toolbar, &self.anchors.more, cx);
        let body = body::render(
            &BodyPanes {
                mode: view.toolbar.mode,
                editor: &self.body,
                preview: &self.preview,
                link_picker: self.link_picker.as_ref(),
            },
            cx,
        );
        let word_count = WordCount::of(&self.body(cx)).label();
        let word_count_text = word_count.clone();
        div()
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .flex_col()
            .bg(tokens.colors.editor_background)
            .child(toolbar)
            .child(
                div()
                    .flex_none()
                    .pr(px(TITLE_PADDING_RIGHT))
                    .text_color(tokens.colors.input_text)
                    .child(
                        div()
                            .id("title")
                            .test_support()
                            .flex()
                            .items_center()
                            .h(px(TITLE_HEIGHT))
                            .pl(px(TITLE_PADDING_LEFT))
                            .child(self.title.clone()),
                    ),
            )
            .children(header)
            .child(body)
            .child(
                div()
                    .id("word-count")
                    .test_support()
                    .flex()
                    .flex_none()
                    .justify_end()
                    .items_center()
                    .h(px(STATUS_BAR_HEIGHT))
                    .px(px(STATUS_BAR_PADDING))
                    .text_size(tokens.scale.mini)
                    .font_family(tokens.fonts.ui_family.clone())
                    .text_color(tokens.colors.muted_text)
                    .child(
                        div()
                            .debug_selector(move || format!("word-count: {word_count}"))
                            .child(word_count_text),
                    ),
            )
    }
}

fn title_field(cx: &mut Context<TextField>) -> TextField {
    TextField::new(cx)
        .placeholder(UNTITLED)
        .borderless()
        .text_size(px(TITLE_SIZE))
}

fn tag_field(cx: &mut Context<TextField>) -> TextField {
    TextField::new(cx)
        .placeholder("Add Tags")
        .borderless()
        .text_size(px(header::META_TEXT_SIZE))
}

pub(crate) struct EditorView<'a> {
    pub(crate) notebooks: &'a [NotebookNode],
    pub(crate) tags: &'a [TagSummary],
    pub(crate) toolbar: ToolbarData,
}

pub(crate) fn toolbar_data(mode: ViewMode, focus_mode: bool, history: &History) -> ToolbarData {
    ToolbarData {
        mode,
        focus_mode,
        can_go_back: history.can_go_back(),
        can_go_forward: history.can_go_forward(),
    }
}

pub(crate) fn render_empty(cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let tokens = cx.theme().tokens.clone();
    div()
        .flex_1()
        .h_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(tokens.scale.row_gap)
        .bg(tokens.colors.editor_background)
        .font_family(tokens.fonts.ui_family.clone())
        .child(
            div()
                .text_size(tokens.scale.massive)
                .text_color(tokens.colors.disabled_text)
                .child("No notes yet"),
        )
        .child(
            div().id("empty-new-note").test_support().child(
                Button::new("empty-new-note-button")
                    .variant(ButtonVariant::Primary)
                    .label("New note")
                    .on_click(cx.listener(|workspace, _, window, cx| {
                        workspace.create_note(window, cx);
                    })),
            ),
        )
}

pub(crate) fn error_banner(
    message: SharedString,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let tokens = cx.theme().tokens.clone();
    div()
        .id("error-banner")
        .test_support()
        .px(px(STATUS_BAR_PADDING))
        .py(px(BANNER_PADDING_VERTICAL))
        .bg(tokens.colors.negative)
        .text_color(tokens.colors.inverted_text)
        .font_family(tokens.fonts.ui_family.clone())
        .child(message)
}
