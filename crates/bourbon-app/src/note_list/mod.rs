mod date;
mod header;
mod row;
mod title;

use bourbon_core::{NoteId, NoteSort, NoteSummary, NotebookNode, TagSummary};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, div, px,
};

use crate::navigation::Navigation;
use crate::workspace::Workspace;
use header::ListView;

const NOTE_LIST_WIDTH: f32 = 320.;

pub(crate) struct NoteListData<'a> {
    pub(crate) navigation: &'a Navigation,
    pub(crate) sort: NoteSort,
    pub(crate) notebooks: &'a [NotebookNode],
    pub(crate) tags: &'a [TagSummary],
    pub(crate) notes: &'a [NoteSummary],
    pub(crate) templates: &'a [NoteSummary],
    pub(crate) selected: Option<&'a NoteId>,
}

pub(crate) fn render(data: &NoteListData, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let in_trash = *data.navigation == Navigation::Trash;
    let header = header::render(
        header::HeaderData {
            title: title::view_title(data.navigation, data.notebooks, data.tags).into(),
            sort: data.sort,
            view: ListView::of(data.navigation),
            note_count: data.notes.len(),
            templates: data
                .templates
                .iter()
                .map(|template| (template.id.clone(), template.title.clone()))
                .collect(),
        },
        cx,
    );
    let rows: Vec<_> = data
        .notes
        .iter()
        .map(|summary| {
            row::render(
                &row::RowData {
                    summary,
                    is_selected: is_selected(data.selected, &summary.id),
                    in_trash,
                },
                cx,
            )
        })
        .collect();
    div()
        .w(px(NOTE_LIST_WIDTH))
        .h_full()
        .flex_none()
        .flex()
        .flex_col()
        .border_r_1()
        .border_color(cx.theme().border)
        .child(header)
        .child(
            div()
                .id("note-list")
                .flex_1()
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .gap_1()
                .p_2()
                .children(rows)
                .when(data.notes.is_empty(), |list| {
                    list.child(
                        div()
                            .id("no-notes")
                            .test_support()
                            .p_2()
                            .text_color(cx.theme().muted_foreground)
                            .child("No notes"),
                    )
                }),
        )
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
