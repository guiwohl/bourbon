use bourbon_core::{NotebookId, NotebookNode};
use bourbon_ui::{IconName, ListRow, MenuItem, ThemeExt as _};
use gpui::{
    Context, InteractiveElement as _, IntoElement, MouseButton, MouseDownEvent, ParentElement as _,
    SharedString, Styled as _, Window, div,
};

use super::state::{Section, SidebarState};
use super::tree::VisibleNotebook;
use super::{SidebarData, header};
use crate::dragged_note::DraggedNote;
use crate::test_id::TestSupportExt as _;
use crate::workspace::{MenuBuilder, Workspace};

pub(super) fn section_header(
    state: &SidebarState,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    header::render(
        &header::Spec {
            label: "Notebooks",
            section: Section::Notebooks,
            searchable: true,
            addable: true,
            collapsible: false,
        },
        state,
        cx,
    )
}

pub(super) fn disclosure_state(node: &NotebookNode, is_expanded: bool) -> Option<bool> {
    (!node.children.is_empty()).then_some(is_expanded)
}

pub(super) fn row(
    visible: &VisibleNotebook,
    data: &SidebarData,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let node = visible.node;
    let id = node.notebook.id.clone();
    let can_delete = data.state.inbox.as_ref() != Some(&id);
    let icon = if can_delete {
        IconName::Notebook
    } else {
        IconName::Inbox
    };
    let selecting = id.clone();
    let toggling = id.clone();
    let dropping = id.clone();
    let menu_id = id.clone();
    let name = node.notebook.name.clone();
    let view = cx.entity();
    let mut list_row = ListRow::new(
        SharedString::from(format!("nav-notebook-{id}-row")),
        name.clone(),
    )
    .icon(icon)
    .count(node.note_count)
    .depth(usize::from(visible.depth))
    .selected(data.current.shows_notebook(&id))
    .pane_focused(data.is_focused)
    .on_click(cx.listener(move |workspace, _, _, cx| workspace.select_notebook(&selecting, cx)))
    .on_toggle(move |_, _, cx| {
        view.update(cx, |workspace, cx| {
            workspace.toggle_notebook_expansion(&toggling, cx);
        });
    });
    if let Some(expanded) = disclosure_state(node, visible.is_expanded) {
        list_row = list_row.disclosure(expanded);
    }
    div()
        .id(SharedString::from(format!("nav-notebook-{id}")))
        .test_support()
        .drag_over::<DraggedNote>(|style, _, _, cx| {
            style.bg(cx.theme().tokens.colors.selection_active_sidebar)
        })
        .on_drop(cx.listener(move |workspace, dragged: &DraggedNote, _, cx| {
            workspace.drop_note_on_notebook(dragged, &dropping, cx);
        }))
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(move |workspace, event: &MouseDownEvent, window, cx| {
                workspace.open_menu(
                    event.position,
                    notebook_menu(&menu_id, &name, can_delete),
                    window,
                    cx,
                );
            }),
        )
        .child(list_row)
}

pub(super) fn notebook_menu(id: &NotebookId, name: &str, can_delete: bool) -> MenuBuilder {
    let creating = id.clone();
    let renaming = (id.clone(), name.to_owned());
    let defaulting = id.clone();
    let deleting = (id.clone(), name.to_owned());
    let menu = MenuBuilder::default()
        .item(
            MenuItem::new("New sub-notebook"),
            move |workspace, window: &mut Window, cx| {
                workspace.begin_create_notebook(Some(creating.clone()), window, cx);
            },
        )
        .item(MenuItem::new("Rename"), move |workspace, window, cx| {
            workspace.begin_rename_notebook(&renaming.0, &renaming.1, window, cx);
        });
    if can_delete {
        menu.item(
            MenuItem::new("Use for new notes"),
            move |workspace, _, cx| {
                workspace.make_default_notebook(&defaulting, cx);
            },
        )
        .item(MenuItem::new("Delete"), move |workspace, _, cx| {
            workspace.begin_delete_notebook(&deleting.0, &deleting.1, cx);
        })
    } else {
        menu
    }
}

#[cfg(test)]
mod tests {
    use bourbon_core::{Notebook, Timestamp};

    use super::*;

    fn node(children: Vec<NotebookNode>) -> NotebookNode {
        NotebookNode {
            notebook: Notebook {
                id: NotebookId::generate(),
                name: "Work".to_owned(),
                parent: None,
                created_at: Timestamp::from_millis(0),
            },
            note_count: 0,
            children,
        }
    }

    #[test]
    fn a_notebook_without_children_has_no_disclosure() {
        assert_eq!(disclosure_state(&node(Vec::new()), true), None);
    }

    #[test]
    fn a_notebook_with_children_discloses_in_its_expansion_state() {
        let parent = node(vec![node(Vec::new())]);
        assert_eq!(disclosure_state(&parent, true), Some(true));
        assert_eq!(disclosure_state(&parent, false), Some(false));
    }

    #[test]
    fn the_inbox_menu_cannot_delete_or_become_the_default() {
        let id = NotebookId::generate();
        assert_eq!(
            labels(&notebook_menu(&id, "Inbox", false)),
            ["New sub-notebook", "Rename"]
        );
        assert_eq!(
            labels(&notebook_menu(&id, "Work", true)),
            ["New sub-notebook", "Rename", "Use for new notes", "Delete"]
        );
    }

    fn labels(menu: &MenuBuilder) -> Vec<String> {
        menu.item_labels()
    }
}
