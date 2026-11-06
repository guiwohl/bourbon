use bourbon_core::Note;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Editor, EditorState, Input, InputEvent, InputState};
use gpui_kit::{
    App, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, Styled as _, Subscription, TestSupportExt as _, Window, div,
};

use crate::UNTITLED;
use crate::workspace::Workspace;

pub(crate) struct EditorPane {
    title_input: Entity<InputState>,
    body_editor: Entity<EditorState>,
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
        Self {
            title_input,
            body_editor,
        }
    }

    pub(crate) fn subscribe_to_edits(&self, cx: &mut Context<Workspace>) -> Vec<Subscription> {
        vec![
            cx.subscribe(&self.title_input, Self::forward_edit::<InputState>),
            cx.subscribe(&self.body_editor, Self::forward_edit::<EditorState>),
        ]
    }

    fn forward_edit<Source: 'static>(
        workspace: &mut Workspace,
        _source: Entity<Source>,
        event: &InputEvent,
        cx: &mut Context<Workspace>,
    ) {
        if matches!(event, InputEvent::Change) {
            workspace.restart_autosave(cx);
        }
    }

    pub(crate) fn title(&self, cx: &App) -> String {
        self.title_input.read(cx).value().to_string()
    }

    pub(crate) fn body(&self, cx: &App) -> String {
        self.body_editor.read(cx).value().to_string()
    }

    pub(crate) fn load(&self, note: Note, window: &mut Window, cx: &mut Context<Workspace>) {
        self.title_input
            .update(cx, |title, cx| title.set_value(note.title, window, cx));
        self.body_editor
            .update(cx, |body, cx| body.set_value(note.body, window, cx));
    }

    pub(crate) fn focus_title(&self, window: &mut Window, cx: &mut Context<Workspace>) {
        self.title_input
            .update(cx, |title, cx| title.focus(window, cx));
    }

    pub(crate) fn render(&self) -> impl IntoElement + use<> {
        div()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .child(
                Input::new(&self.title_input)
                    .id("title")
                    .aria_label("Title"),
            )
            .child(
                Editor::new(&self.body_editor)
                    .aria_label("Body")
                    .h_full()
                    .flex_1(),
            )
    }
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
