use bourbon_core::NoteSort;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::{Disableable as _, Sizable as _};
use gpui_kit::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, SharedString, Styled as _,
    div,
};

use crate::workspace::Workspace;

const SORTS: [(NoteSort, &str); 3] = [
    (NoteSort::UpdatedNewestFirst, "Updated"),
    (NoteSort::CreatedNewestFirst, "Created"),
    (NoteSort::TitleAlphabetical, "Title"),
];

pub(crate) struct HeaderData {
    pub(crate) title: SharedString,
    pub(crate) sort: NoteSort,
    pub(crate) in_trash: bool,
    pub(crate) note_count: usize,
}

pub(crate) fn sort_label(sort: NoteSort) -> &'static str {
    SORTS
        .iter()
        .find(|(candidate, _)| *candidate == sort)
        .map_or("", |(_, label)| label)
}

pub(crate) fn render(data: HeaderData, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let view = cx.entity();
    let sort_selector = Button::new("sort-selector")
        .ghost()
        .small()
        .label(sort_label(data.sort))
        .dropdown_caret(true)
        .dropdown_menu(move |menu, _, _| {
            SORTS.iter().fold(menu, |menu, (sort, label)| {
                let view = view.clone();
                let sort = *sort;
                menu.item(PopupMenuItem::new(*label).on_click(move |_, _, cx| {
                    view.update(cx, |workspace, cx| workspace.sort_by(sort, cx));
                }))
            })
        });
    let primary_action = if data.in_trash {
        Button::new("empty-trash")
            .small()
            .label("Empty trash")
            .disabled(data.note_count == 0)
            .on_click(cx.listener(|workspace, _, window, cx| {
                workspace.confirm_empty_trash(window, cx);
            }))
    } else {
        Button::new("new-note")
            .small()
            .label("New note")
            .on_click(cx.listener(|workspace, _, window, cx| {
                workspace.create_note(window, cx);
            }))
    };
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .p_2()
        .child(
            div()
                .id("note-list-title")
                .flex_1()
                .min_w_0()
                .truncate()
                .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                .child(data.title),
        )
        .child(sort_selector)
        .child(primary_action)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sort_has_its_own_label() {
        assert_eq!(sort_label(NoteSort::UpdatedNewestFirst), "Updated");
        assert_eq!(sort_label(NoteSort::CreatedNewestFirst), "Created");
        assert_eq!(sort_label(NoteSort::TitleAlphabetical), "Title");
    }
}
