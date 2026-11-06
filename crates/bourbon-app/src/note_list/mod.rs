use bourbon_core::{NoteId, NoteSummary};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::button::Button;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, div, px,
};

use crate::UNTITLED;
use crate::workspace::Workspace;

const NOTE_LIST_WIDTH: f32 = 320.;

pub(crate) fn render(
    notes: &[NoteSummary],
    selected: Option<&NoteId>,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let rows: Vec<_> = notes
        .iter()
        .map(|summary| note_row(summary, is_selected(selected, &summary.id), cx))
        .collect();
    div()
        .w(px(NOTE_LIST_WIDTH))
        .h_full()
        .flex_none()
        .flex()
        .flex_col()
        .border_r_1()
        .border_color(cx.theme().border)
        .child(
            div().flex().flex_row().justify_end().p_2().child(
                Button::new("new-note")
                    .label("New note")
                    .on_click(cx.listener(|workspace, _, window, cx| {
                        workspace.create_note(window, cx);
                    })),
            ),
        )
        .child(
            div()
                .id("note-list")
                .flex_1()
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .gap_1()
                .p_2()
                .children(rows),
        )
}

fn note_row(
    summary: &NoteSummary,
    is_selected: bool,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let id = summary.id.clone();
    let title = if summary.title.is_empty() {
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
        .flex()
        .flex_col()
        .when(is_selected, |row| {
            row.bg(cx.theme().accent)
                .text_color(cx.theme().accent_foreground)
        })
        .child(div().truncate().child(title.to_owned()))
        .child(
            div()
                .truncate()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(summary.excerpt.clone()),
        )
        .on_click(cx.listener(move |workspace, _, window, cx| workspace.select(&id, window, cx)))
}

fn is_selected(selected: Option<&NoteId>, id: &NoteId) -> bool {
    selected == Some(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_selected_note_is_highlighted() {
        let selected = NoteId::generate();
        assert!(is_selected(Some(&selected), &selected));
        assert!(!is_selected(Some(&selected), &NoteId::generate()));
        assert!(!is_selected(None, &selected));
    }
}
