use bourbon_core::{NoteId, NoteSort};
use bourbon_ui::{IconButton, IconName, MenuItem, ThemeExt as _};
use gpui_kit::{
    ClickEvent, Context, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, div, px,
};

use crate::navigation::Navigation;
use crate::presentation::display_title;
use crate::sidebar::ToggleSidebar;
use crate::workspace::{MenuBuilder, Workspace};

const SORTS: [(NoteSort, &str); 3] = [
    (NoteSort::UpdatedNewestFirst, "Sort by updated"),
    (NoteSort::CreatedNewestFirst, "Sort by created"),
    (NoteSort::TitleAlphabetical, "Sort by title"),
];
const TITLE_SIZE: f32 = 16.8;
const TITLE_LINE_HEIGHT: f32 = 21.84;
const HORIZONTAL_PADDING: f32 = 2.;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ListView {
    Notes,
    Templates,
    Trash,
}

impl ListView {
    pub(crate) fn shows_dates(self) -> bool {
        self != Self::Templates
    }

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
    let tokens = cx.theme().tokens.clone();
    let colors = tokens.colors;
    let sort = data.sort;
    let templates = data.templates;
    let sidebar_toggle = div().id("toggle-sidebar").test_support().child(
        IconButton::new("toggle-sidebar-button", IconName::PanelLeft)
            .tooltip_action("Toggle sidebar", &ToggleSidebar)
            .on_click(cx.listener(|workspace, _, _, cx| workspace.toggle_sidebar(cx))),
    );
    let title = div()
        .id("note-list-title")
        .test_support()
        .flex()
        .items_center()
        .justify_center()
        .min_w_0()
        .cursor_pointer()
        .text_size(px(TITLE_SIZE))
        .font_weight(tokens.scale.weight_medium)
        .line_height(px(TITLE_LINE_HEIGHT))
        .text_color(colors.text)
        .on_click(
            cx.listener(move |workspace, event: &ClickEvent, window, cx| {
                workspace.open_menu(event.position(), title_menu(sort, &templates), window, cx);
            }),
        )
        .child(div().truncate().child(data.title));
    let primary_action = match data.view {
        ListView::Trash => div().id("empty-trash").test_support().child(
            IconButton::new("empty-trash-button", IconName::Trash)
                .tooltip("Empty trash")
                .disabled(data.note_count == 0)
                .on_click(cx.listener(|workspace, _, window, cx| {
                    workspace.confirm_empty_trash(window, cx);
                })),
        ),
        ListView::Notes | ListView::Templates => div().id("new-note").test_support().child(
            IconButton::new("new-note-button", IconName::SquarePen)
                .tooltip_action("New note", &crate::workspace::NewNote)
                .on_click(cx.listener(|workspace, _, window, cx| {
                    workspace.create_note(window, cx);
                })),
        ),
    };
    div()
        .flex()
        .flex_none()
        .items_center()
        .min_h(tokens.scale.header_height)
        .px(px(HORIZONTAL_PADDING))
        .font_family(tokens.fonts.ui_family.clone())
        .child(sidebar_toggle)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .justify_center()
                .child(title),
        )
        .child(primary_action)
}

fn is_current_sort(sort: NoteSort, current: NoteSort) -> bool {
    sort == current
}

pub(crate) fn title_menu(current: NoteSort, templates: &[(NoteId, String)]) -> MenuBuilder {
    let menu = SORTS
        .iter()
        .fold(MenuBuilder::default(), |menu, (sort, label)| {
            let sort = *sort;
            menu.item(
                MenuItem::new(*label).checked(is_current_sort(sort, current)),
                move |workspace, _, cx| workspace.sort_by(sort, cx),
            )
        });
    if templates.is_empty() {
        return menu;
    }
    templates.iter().fold(
        menu.separator().item(
            MenuItem::new("New from template").disabled(true),
            |_, _, _| {},
        ),
        |menu, (id, title)| {
            let id = id.clone();
            menu.item(
                MenuItem::new(display_title(title).to_owned()),
                move |workspace, window, cx| {
                    workspace.create_note_from_template(&id, window, cx);
                },
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
    fn templates_are_the_only_view_without_dates() {
        assert!(ListView::Notes.shows_dates());
        assert!(ListView::Trash.shows_dates());
        assert!(!ListView::Templates.shows_dates());
    }

    #[test]
    fn the_list_view_follows_the_navigation() {
        assert_eq!(ListView::of(&Navigation::Trash), ListView::Trash);
        assert_eq!(ListView::of(&Navigation::Templates), ListView::Templates);
        assert_eq!(ListView::of(&Navigation::AllNotes), ListView::Notes);
    }
}
