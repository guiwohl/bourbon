use bourbon_core::{NoteId, NoteSort};
use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenu, PopupMenuItem};
use gpui_kit::component::{Disableable as _, Sizable as _};
use gpui_kit::{
    Context, Entity, FontWeight, IntoElement, ParentElement as _, SharedString, Styled as _, div,
    px,
};

use crate::icons::glyph;
use crate::navigation::Navigation;
use crate::presentation::display_title;
use crate::workspace::Workspace;

const SORTS: [(NoteSort, &str); 3] = [
    (NoteSort::UpdatedNewestFirst, "Sort by updated"),
    (NoteSort::CreatedNewestFirst, "Sort by created"),
    (NoteSort::TitleAlphabetical, "Sort by title"),
];

const HEADER_HEIGHT: f32 = 40.;

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

pub(crate) fn render(data: HeaderData, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let view = cx.entity();
    let sidebar_toggle = Button::new("toggle-sidebar")
        .ghost()
        .small()
        .icon(glyph(IconName::PanelLeft))
        .on_click(cx.listener(|workspace, _, _, cx| workspace.toggle_sidebar(cx)));
    let title_menu = Button::new("note-list-title")
        .ghost()
        .label(data.title)
        .text_size(px(15.))
        .font_weight(FontWeight::MEDIUM)
        .dropdown_menu(move |menu, _, _| title_menu_items(menu, data.sort, &data.templates, &view));
    let primary_action = match data.view {
        ListView::Trash => Button::new("empty-trash")
            .ghost()
            .small()
            .icon(glyph(IconName::Trash))
            .disabled(data.note_count == 0)
            .on_click(cx.listener(|workspace, _, window, cx| {
                workspace.confirm_empty_trash(window, cx);
            })),
        ListView::Notes | ListView::Templates => Button::new("new-note")
            .ghost()
            .small()
            .icon(glyph(IconName::SquarePen))
            .on_click(cx.listener(|workspace, _, window, cx| {
                workspace.create_note(window, cx);
            })),
    };
    div()
        .flex()
        .flex_row()
        .items_center()
        .h(px(HEADER_HEIGHT))
        .flex_none()
        .px_2()
        .child(sidebar_toggle)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .justify_center()
                .child(title_menu),
        )
        .child(primary_action)
}

fn is_current_sort(sort: NoteSort, current: NoteSort) -> bool {
    sort == current
}

fn title_menu_items(
    menu: PopupMenu,
    current: NoteSort,
    templates: &[(NoteId, String)],
    view: &Entity<Workspace>,
) -> PopupMenu {
    let menu = SORTS.iter().fold(menu, |menu, (sort, label)| {
        let view = view.clone();
        let sort = *sort;
        menu.item(
            PopupMenuItem::new(*label)
                .checked(is_current_sort(sort, current))
                .on_click(move |_, _, cx| {
                    view.update(cx, |workspace, cx| workspace.sort_by(sort, cx));
                }),
        )
    });
    if templates.is_empty() {
        return menu;
    }
    templates.iter().fold(
        menu.separator()
            .item(PopupMenuItem::label("New from template")),
        |menu, (id, title)| {
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
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_current_sort_is_checked() {
        assert!(is_current_sort(SORTS[0].0, SORTS[0].0));
        assert!(!is_current_sort(SORTS[0].0, SORTS[1].0));
    }

    #[test]
    fn the_list_view_follows_the_navigation() {
        assert_eq!(ListView::of(&Navigation::Trash), ListView::Trash);
        assert_eq!(ListView::of(&Navigation::Templates), ListView::Templates);
        assert_eq!(ListView::of(&Navigation::AllNotes), ListView::Notes);
    }
}
