use bourbon_core::{NoteId, NoteSummary, Tag, TagId, Timestamp};
use gpui_kit::assets::{IconName, icon_assets};
use gpui_kit::component::menu::{ContextMenuExt as _, PopupMenuItem};
use gpui_kit::component::tag::Tag as TagChip;
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AppContext as _, AssetSource as _, Context, Entity, InteractiveElement as _, IntoElement,
    MouseButton, ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _,
    TestSupportExt as _, div,
};
use jiff::tz::TimeZone;

use super::date::short_date;
use crate::dragged_note::DraggedNote;
use crate::presentation::{chip_color, display_title};
use crate::workspace::Workspace;

icon_assets!(PinIcon, [Pin]);

pub(crate) struct RowData<'a> {
    pub(crate) summary: &'a NoteSummary,
    pub(crate) is_selected: bool,
    pub(crate) in_trash: bool,
}

pub(crate) fn render(data: &RowData, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let summary = data.summary;
    let id = summary.id.clone();
    let view = cx.entity();
    let title = display_title(&summary.title);
    let updated_on = short_date(summary.updated_at, Timestamp::now(), &TimeZone::system());
    let pinned = summary.pinned;
    let in_trash = data.in_trash;
    let menu_id = id.clone();
    let click_id = id.clone();
    div()
        .id(SharedString::from(format!("note-{id}")))
        .test_support()
        .px_2()
        .py_1()
        .rounded_md()
        .cursor_pointer()
        .flex()
        .flex_col()
        .gap_1()
        .when(data.is_selected, |row| {
            row.bg(cx.theme().accent)
                .text_color(cx.theme().accent_foreground)
        })
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .when(pinned, |line| {
                    line.child(
                        div()
                            .id(SharedString::from(format!("pin-{id}")))
                            .test_support()
                            .child(pin_icon().xsmall()),
                    )
                })
                .child(div().flex_1().min_w_0().truncate().child(title.to_owned()))
                .child(
                    div()
                        .flex_none()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(updated_on),
                ),
        )
        .child(
            div()
                .truncate()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(summary.excerpt.clone()),
        )
        .when(!summary.tags.is_empty(), |row| {
            row.child(tag_chips(&summary.id, &summary.tags, cx))
        })
        .on_click(
            cx.listener(move |workspace, _, window, cx| workspace.select(&click_id, window, cx)),
        )
        .when(!in_trash, |row| {
            row.on_drag(DraggedNote::of(summary), |dragged, _, _, cx| {
                cx.new(|_| dragged.clone())
            })
        })
        .context_menu(move |menu, _, _| {
            if in_trash {
                menu.item(restore_item(&menu_id, &view))
            } else {
                menu.item(pin_item(&menu_id, pinned, &view))
                    .item(duplicate_item(&menu_id, &view))
                    .item(trash_item(&menu_id, &view))
            }
        })
}

fn tag_chips(note: &NoteId, tags: &[Tag], cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let chips: Vec<_> = tags.iter().map(|tag| tag_chip(note, tag, cx)).collect();
    div().flex().flex_row().flex_wrap().gap_1().children(chips)
}

fn tag_chip(note: &NoteId, tag: &Tag, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let tag_id: TagId = tag.id.clone();
    div()
        .id(SharedString::from(format!("chip-{note}-{}", tag.id)))
        .test_support()
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(cx.listener(move |workspace, _, _, cx| workspace.select_tag(&tag_id, false, cx)))
        .child(
            TagChip::color(chip_color(tag.color))
                .small()
                .child(tag.name.clone()),
        )
}

fn pin_item(id: &NoteId, pinned: bool, view: &Entity<Workspace>) -> PopupMenuItem {
    let (id, view) = (id.clone(), view.clone());
    PopupMenuItem::new(if pinned { "Unpin" } else { "Pin" }).on_click(move |_, _, cx| {
        view.update(cx, |workspace, cx| workspace.toggle_pin(&id, cx));
    })
}

fn duplicate_item(id: &NoteId, view: &Entity<Workspace>) -> PopupMenuItem {
    let (id, view) = (id.clone(), view.clone());
    PopupMenuItem::new("Duplicate").on_click(move |_, window, cx| {
        view.update(cx, |workspace, cx| workspace.duplicate(&id, window, cx));
    })
}

fn trash_item(id: &NoteId, view: &Entity<Workspace>) -> PopupMenuItem {
    let (id, view) = (id.clone(), view.clone());
    PopupMenuItem::new("Move to Trash").on_click(move |_, window, cx| {
        view.update(cx, |workspace, cx| workspace.move_to_trash(&id, window, cx));
    })
}

fn restore_item(id: &NoteId, view: &Entity<Workspace>) -> PopupMenuItem {
    let (id, view) = (id.clone(), view.clone());
    PopupMenuItem::new("Restore").on_click(move |_, _, cx| {
        view.update(cx, |workspace, cx| workspace.restore(&id, cx));
    })
}

fn pin_icon() -> Icon {
    let svg = PinIcon.load(&IconName::Pin.path()).ok().flatten();
    svg.map_or_else(Icon::empty, |bytes| Icon::empty().data(&bytes))
}
