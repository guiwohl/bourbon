use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Editor, Input};
use gpui_kit::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, SharedString, Styled as _,
    TestSupportExt as _, div,
};

use super::NotesView;

impl NotesView {
    pub(super) fn render_editor_pane(&self) -> impl IntoElement {
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

    pub(super) fn render_empty_pane(cx: &mut Context<Self>) -> impl IntoElement {
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
                    .on_click(cx.listener(|view, _, window, cx| view.create_note(window, cx))),
            )
    }
}

pub(super) fn error_banner(message: SharedString, cx: &mut Context<NotesView>) -> impl IntoElement {
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
