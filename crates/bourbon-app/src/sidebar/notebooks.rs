use bourbon_core::{NotebookId, NotebookNode};
use gpui_kit::assets::IconName as GlyphName;
use gpui_kit::component::menu::{ContextMenuExt as _, PopupMenuItem};
use gpui_kit::component::{ActiveTheme as _, Icon, IconName, Sizable as _};
use gpui_kit::{
    Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, Window, div, px,
};

use super::state::{Section, SidebarState};
use super::tree::VisibleNotebook;
use super::{count_label, header, nav_row};
use crate::dragged_note::DraggedNote;
use crate::icons::glyph;
use crate::navigation::Navigation;
use crate::workspace::Workspace;

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
        },
        state,
        cx,
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
    let icon = if can_delete {
        GlyphName::Notebook
    } else {
        GlyphName::Inbox
    };
    let selecting = id.clone();
    let dropping = id.clone();
    nav_row(
        format!("nav-notebook-{id}").into(),
        visible.depth,
        is_selected,
        cx,
    )
    .test_support()
    .child(disclosure(node, visible.is_expanded, cx))
    .child(glyph(icon).text_color(cx.theme().muted_foreground))
    .child(div().flex_1().truncate().child(node.notebook.name.clone()))
    .child(count_label(node.note_count, cx))
    .on_click(cx.listener(move |workspace, _, _, cx| workspace.select_notebook(&selecting, cx)))
    .drag_over::<DraggedNote>(|style, _, _, cx| style.bg(cx.theme().drop_target))
    .on_drop(cx.listener(move |workspace, dragged: &DraggedNote, _, cx| {
        workspace.drop_note_on_notebook(dragged, &dropping, cx);
    }))
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
        let defaulting = id.clone();
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
            menu.item(item(
                "Use for new notes",
                &workspace,
                move |workspace, _, cx| workspace.make_default_notebook(&defaulting, cx),
            ))
            .item(item("Delete", &workspace, move |workspace, _, cx| {
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
