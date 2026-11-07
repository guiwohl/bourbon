use bourbon_core::{NoteId, NoteSummary};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::{Context, IntoElement, ParentElement as _, Render, Styled as _, Window, div};

use crate::presentation::display_title;

#[derive(Clone)]
pub(crate) struct DraggedNote {
    pub(crate) id: NoteId,
    pub(crate) title: String,
}

impl DraggedNote {
    pub(crate) fn of(summary: &NoteSummary) -> Self {
        Self {
            id: summary.id.clone(),
            title: display_title(&summary.title).to_owned(),
        }
    }
}

impl Render for DraggedNote {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_2()
            .py_1()
            .rounded_md()
            .max_w(gpui_kit::px(220.))
            .truncate()
            .bg(cx.theme().popover)
            .text_color(cx.theme().popover_foreground)
            .border_1()
            .border_color(cx.theme().border)
            .child(self.title.clone())
    }
}

#[cfg(test)]
mod tests {
    use bourbon_core::{NoteStatus, Timestamp};

    use super::*;

    fn summary(title: &str) -> NoteSummary {
        NoteSummary {
            id: NoteId::generate(),
            title: title.to_owned(),
            excerpt: String::new(),
            pinned: false,
            updated_at: Timestamp::from_millis(0),
            created_at: Timestamp::from_millis(0),
            status: NoteStatus::None,
            tags: Vec::new(),
            task_progress: None,
        }
    }

    #[test]
    fn the_dragged_note_carries_the_id_and_title() {
        let note = summary("Plan");
        let dragged = DraggedNote::of(&note);
        assert_eq!(dragged.id, note.id);
        assert_eq!(dragged.title, "Plan");
    }

    #[test]
    fn an_empty_title_is_shown_as_untitled() {
        assert_eq!(DraggedNote::of(&summary("")).title, "Untitled");
    }
}
