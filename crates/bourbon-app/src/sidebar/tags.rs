use bourbon_core::{TagColor, TagSummary};
use gpui_kit::component::menu::{ContextMenuExt as _, PopupMenu};
use gpui_kit::{
    Context, IntoElement, ParentElement as _, StatefulInteractiveElement as _, Styled as _,
    TestSupportExt as _, Window, div, px,
};

use super::notebooks::item;
use super::{count_label, nav_row};
use crate::navigation::Navigation;
use crate::presentation::{color_label, dot_color, is_current_color};
use crate::workspace::Workspace;

pub(super) fn row(
    summary: &TagSummary,
    current: &Navigation,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let tag = &summary.tag;
    let id = tag.id.clone();
    let is_selected = current.shows_tag(&id);
    let selecting = id.clone();
    nav_row(format!("nav-tag-{id}").into(), 0, is_selected, cx)
        .test_support()
        .child(
            div()
                .size(px(10.))
                .flex_none()
                .rounded_full()
                .bg(dot_color(tag.color)),
        )
        .child(div().flex_1().truncate().child(tag.name.clone()))
        .child(count_label(summary.note_count, cx))
        .on_click(
            cx.listener(move |workspace, event: &gpui_kit::ClickEvent, _, cx| {
                workspace.select_tag(&selecting, event.modifiers().control, cx);
            }),
        )
        .context_menu(context_menu(summary, cx))
}

fn context_menu(
    summary: &TagSummary,
    cx: &mut Context<Workspace>,
) -> impl Fn(PopupMenu, &mut Window, &mut Context<PopupMenu>) -> PopupMenu + 'static {
    let workspace = cx.entity();
    let tag = summary.tag.clone();
    move |menu, window, cx| {
        let renaming = tag.clone();
        let deleting = tag.clone();
        let colouring = tag.clone();
        let colour_workspace = workspace.clone();
        menu.item(item("Rename", &workspace, move |workspace, window, cx| {
            workspace.begin_rename_tag(&renaming.id, &renaming.name, window, cx);
        }))
        .submenu("Change colour", window, cx, move |submenu, _, _| {
            TagColor::ALL.into_iter().fold(submenu, |submenu, color| {
                let id = colouring.id.clone();
                submenu.item(
                    item(
                        color_label(color),
                        &colour_workspace,
                        move |workspace, _, cx| {
                            workspace.set_tag_color(&id, color, cx);
                        },
                    )
                    .checked(is_current_color(colouring.color, color)),
                )
            })
        })
        .item(item("Delete", &workspace, move |workspace, _, cx| {
            workspace.begin_delete_tag(&deleting.id, &deleting.name, cx);
        }))
    }
}
