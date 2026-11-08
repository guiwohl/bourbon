use bourbon_core::{TagColor, TagSummary};
use bourbon_ui::{ListRow, MenuItem};
use gpui_kit::{
    ClickEvent, Context, InteractiveElement as _, IntoElement, MouseButton, MouseDownEvent,
    ParentElement as _, SharedString, TestSupportExt as _, div,
};

use super::SidebarData;
use crate::presentation::{color_label, dot_color, is_current_color};
use crate::workspace::{MenuBuilder, Workspace};

pub(super) fn row(
    summary: &TagSummary,
    data: &SidebarData,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let tag = summary.tag.clone();
    let selecting = tag.id.clone();
    let menu_tag = tag.clone();
    let list_row = ListRow::new(
        SharedString::from(format!("nav-tag-{}-row", tag.id)),
        tag.name.clone(),
    )
    .dot(dot_color(tag.color))
    .count(summary.note_count)
    .selected(data.current.shows_tag(&tag.id))
    .pane_focused(data.is_focused)
    .on_click(cx.listener(move |workspace, event: &ClickEvent, _, cx| {
        workspace.select_tag(&selecting, event.modifiers().control, cx);
    }));
    div()
        .id(SharedString::from(format!("nav-tag-{}", tag.id)))
        .test_support()
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(move |workspace, event: &MouseDownEvent, window, cx| {
                workspace.open_menu(event.position, tag_menu(&menu_tag), window, cx);
            }),
        )
        .child(list_row)
}

pub(super) fn tag_menu(tag: &bourbon_core::Tag) -> MenuBuilder {
    let renaming = tag.clone();
    let colouring = tag.clone();
    let deleting = tag.clone();
    MenuBuilder::default()
        .item(MenuItem::new("Rename"), move |workspace, window, cx| {
            workspace.begin_rename_tag(&renaming.id, &renaming.name, window, cx);
        })
        .item(
            MenuItem::new("Change colour").submenu(),
            move |workspace, window, cx| {
                let position = workspace.last_menu_position();
                workspace.open_menu(position, colour_menu(&colouring), window, cx);
            },
        )
        .item(MenuItem::new("Delete"), move |workspace, _, cx| {
            workspace.begin_delete_tag(&deleting.id, &deleting.name, cx);
        })
}

fn colour_menu(tag: &bourbon_core::Tag) -> MenuBuilder {
    TagColor::ALL
        .into_iter()
        .fold(MenuBuilder::default(), |menu, color| {
            let id = tag.id.clone();
            menu.item(
                MenuItem::new(color_label(color)).checked(is_current_color(tag.color, color)),
                move |workspace, _, cx| workspace.set_tag_color(&id, color, cx),
            )
        })
}
