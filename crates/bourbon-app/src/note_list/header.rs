use bourbon_core::{NoteId, NoteSort};
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::{Disableable as _, Sizable as _};
use gpui_kit::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, SharedString, Styled as _,
    div,
};

use crate::navigation::Navigation;
use crate::presentation::display_title;
use crate::workspace::Workspace;

const SORTS: [(NoteSort, &str); 3] = [
    (NoteSort::UpdatedNewestFirst, "Updated"),
    (NoteSort::CreatedNewestFirst, "Created"),
    (NoteSort::TitleAlphabetical, "Title"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ListView {
    Notes,
    Templates,
    Trash,
}

impl ListView {
    pub(crate) fn of(navigation: &Navigation) -> Self {
        match navigation {
            Navigation::Trash => Self::Trash,
            Navigation::Templates => Self::Templates,
            _ => Self::Notes,
        }
    }
}

pub(crate) struct HeaderData {
    pub(crate) title: SharedString,
    pub(crate) sort: NoteSort,
    pub(crate) view: ListView,
    pub(crate) note_count: usize,
    pub(crate) templates: Vec<(NoteId, String)>,
}

pub(crate) fn sort_label(sort: NoteSort) -> &'static str {
    SORTS
        .iter()
        .find(|(candidate, _)| *candidate == sort)
        .map_or("", |(_, label)| label)
}

pub(crate) fn render(data: HeaderData, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let view = cx.entity();
    let sort_owner = view.clone();
    let sort_selector = Button::new("sort-selector")
        .ghost()
        .small()
        .label(sort_label(data.sort))
        .dropdown_caret(true)
        .dropdown_menu(move |menu, _, _| {
            SORTS.iter().fold(menu, |menu, (sort, label)| {
                let view = sort_owner.clone();
                let sort = *sort;
                menu.item(PopupMenuItem::new(*label).on_click(move |_, _, cx| {
                    view.update(cx, |workspace, cx| workspace.sort_by(sort, cx));
                }))
            })
        });
    let primary_action = match data.view {
        ListView::Trash => Button::new("empty-trash")
            .small()
            .label("Empty trash")
            .disabled(data.note_count == 0)
            .on_click(cx.listener(|workspace, _, window, cx| {
                workspace.confirm_empty_trash(window, cx);
            })),
        ListView::Templates => Button::new("list-new-template")
            .small()
            .label("New template")
            .on_click(cx.listener(|workspace, _, window, cx| {
                workspace.create_template(window, cx);
            })),
        ListView::Notes => Button::new("new-note")
            .small()
            .label("New note")
            .on_click(cx.listener(|workspace, _, window, cx| {
                workspace.create_note(window, cx);
            })),
    };
    let from_template = (!data.templates.is_empty()).then(|| {
        let templates = data.templates;
        Button::new("new-from-template")
            .small()
            .label("New from template")
            .dropdown_caret(true)
            .dropdown_menu(move |menu, _, _| {
                templates.iter().fold(menu, |menu, (id, title)| {
                    let view = view.clone();
                    let id = id.clone();
                    menu.item(
                        PopupMenuItem::new(display_title(title).to_owned()).on_click(
                            move |_, window, cx| {
                                view.update(cx, |workspace, cx| {
                                    workspace.create_note_from_template(&id, window, cx);
                                });
                            },
                        ),
                    )
                })
            })
    });
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
        .children(from_template)
        .child(primary_action)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_view_follows_the_navigation() {
        assert_eq!(ListView::of(&Navigation::Trash), ListView::Trash);
        assert_eq!(ListView::of(&Navigation::Templates), ListView::Templates);
        assert_eq!(ListView::of(&Navigation::AllNotes), ListView::Notes);
    }

    #[test]
    fn every_sort_has_its_own_label() {
        assert_eq!(sort_label(NoteSort::UpdatedNewestFirst), "Updated");
        assert_eq!(sort_label(NoteSort::CreatedNewestFirst), "Created");
        assert_eq!(sort_label(NoteSort::TitleAlphabetical), "Title");
    }
}
