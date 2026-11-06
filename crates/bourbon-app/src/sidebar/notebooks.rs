use bourbon_core::{NotebookId, NotebookNode};
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::label::Label;
use gpui_kit::component::menu::{ContextMenuExt as _, PopupMenuItem};
use gpui_kit::component::{ActiveTheme as _, Icon, IconName, Sizable as _};
use gpui_kit::{
    Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, Window, div, px,
};

use super::tree::VisibleNotebook;
use super::{header, nav_row};
use crate::navigation::Navigation;
use crate::workspace::Workspace;

pub(super) fn section_header(cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    header("Notebooks", cx).child(
        Button::new("new-notebook")
            .ghost()
            .xsmall()
            .icon(IconName::Plus)
            .on_click(cx.listener(|workspace, _, window, cx| {
                workspace.begin_create_notebook(None, window, cx);
            })),
    )
}

pub(super) fn row(
    visible: &VisibleNotebook,
    inbox: Option<&NotebookId>,
    current: &Navigation,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let node = visible.node;
    let id = node.notebook.id.clone();
    let is_selected = current.shows_notebook(&id);
    let can_delete = inbox != Some(&id);
    let selecting = id.clone();
    nav_row(
        format!("nav-notebook-{id}").into(),
        visible.depth,
        is_selected,
        cx,
    )
    .test_support()
    .child(disclosure(node, visible.is_expanded, cx))
    .child(div().flex_1().truncate().child(node.notebook.name.clone()))
    .child(Label::new(node.note_count.to_string()).text_color(cx.theme().muted_foreground))
    .on_click(cx.listener(move |workspace, _, _, cx| workspace.select_notebook(&selecting, cx)))
    .context_menu(context_menu(id, node.notebook.name.clone(), can_delete, cx))
}

fn disclosure(
    node: &NotebookNode,
    is_expanded: bool,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let slot = div().w(px(16.)).flex_none();
    if node.children.is_empty() {
        return slot.into_any_element();
    }
    let id = node.notebook.id.clone();
    let icon = if is_expanded {
        IconName::ChevronDown
    } else {
        IconName::ChevronRight
    };
    slot.id(SharedString::from(format!("toggle-notebook-{id}")))
        .test_support()
        .child(Icon::new(icon).small())
        .on_click(cx.listener(move |workspace, _, _, cx| {
            cx.stop_propagation();
            workspace.toggle_notebook_expansion(&id, cx);
        }))
        .into_any_element()
}

fn context_menu(
    id: NotebookId,
    name: String,
    can_delete: bool,
    cx: &mut Context<Workspace>,
) -> impl Fn(
    gpui_kit::component::menu::PopupMenu,
    &mut Window,
    &mut Context<gpui_kit::component::menu::PopupMenu>,
) -> gpui_kit::component::menu::PopupMenu
+ 'static {
    let workspace = cx.entity();
    move |menu, _, _| {
        let creating = id.clone();
        let renaming = (id.clone(), name.clone());
        let deleting = (id.clone(), name.clone());
        let menu = menu
            .item(item(
                "New sub-notebook",
                &workspace,
                move |workspace, window, cx| {
                    workspace.begin_create_notebook(Some(creating.clone()), window, cx);
                },
            ))
            .item(item("Rename", &workspace, move |workspace, window, cx| {
                workspace.begin_rename_notebook(&renaming.0, &renaming.1, window, cx);
            }));
        if can_delete {
            menu.item(item("Delete", &workspace, move |workspace, _, cx| {
                workspace.begin_delete_notebook(&deleting.0, &deleting.1, cx);
            }))
        } else {
            menu
        }
    }
}

pub(super) fn item(
    label: impl Into<SharedString>,
    workspace: &Entity<Workspace>,
    action: impl Fn(&mut Workspace, &mut Window, &mut Context<Workspace>) + 'static,
) -> PopupMenuItem {
    let workspace = workspace.clone();
    PopupMenuItem::new(label).on_click(move |_, window, cx| {
        workspace.update(cx, |workspace, cx| action(workspace, window, cx));
    })
}
