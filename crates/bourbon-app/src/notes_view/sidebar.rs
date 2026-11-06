use bourbon_core::NoteSummary;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::button::Button;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, div, px,
};

use super::{NotesView, UNTITLED};

const SIDEBAR_WIDTH: f32 = 260.;

impl NotesView {
    pub(super) fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let rows: Vec<_> = self
            .notes
            .iter()
            .map(|summary| self.render_note_row(summary, cx))
            .collect();
        div()
            .w(px(SIDEBAR_WIDTH))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .gap_2()
            .p_2()
            .bg(cx.theme().sidebar)
            .text_color(cx.theme().sidebar_foreground)
            .border_r_1()
            .border_color(cx.theme().sidebar_border)
            .child(
                Button::new("new-note")
                    .label("New note")
                    .on_click(cx.listener(|view, _, window, cx| view.create_note(window, cx))),
            )
            .child(
                div()
                    .id("note-list")
                    .flex_1()
                    .overflow_y_scroll()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .children(rows),
            )
    }

    fn render_note_row(
        &self,
        summary: &NoteSummary,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let id = summary.id.clone();
        let is_selected = self.selected.as_ref() == Some(&summary.id);
        let label = if summary.title.is_empty() {
            UNTITLED
        } else {
            &summary.title
        };
        div()
            .id(SharedString::from(format!("note-{id}")))
            .test_support()
            .px_2()
            .py_1()
            .rounded_md()
            .cursor_pointer()
            .truncate()
            .when(is_selected, |row| {
                row.bg(cx.theme().sidebar_accent)
                    .text_color(cx.theme().sidebar_accent_foreground)
            })
            .child(label.to_owned())
            .on_click(cx.listener(move |view, _, window, cx| view.select(&id, window, cx)))
    }
}
