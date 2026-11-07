use bourbon_core::{NoteId, NoteSummary, Tag, TagId, TaskProgress, Timestamp};
use gpui_kit::assets::IconName;
use gpui_kit::component::menu::{ContextMenuExt as _, PopupMenuItem};
use gpui_kit::component::{ActiveTheme as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AppContext as _, Context, Entity, FontWeight, InteractiveElement as _, IntoElement,
    MouseButton, ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _,
    TestSupportExt as _, div, px,
};

use super::date::relative_age;
use crate::dragged_note::DraggedNote;
use crate::icons::{glyph, status_glyph};
use crate::presentation::{display_title, dot_color, pill_text_color};
use crate::workspace::Workspace;

const PROGRESS_BAR_WIDTH: f32 = 30.;
const PILL_TINT: f32 = 0.15;

pub(crate) struct RowData<'a> {
    pub(crate) summary: &'a NoteSummary,
    pub(crate) is_selected: bool,
    pub(crate) in_trash: bool,
}

pub(crate) fn progress_label(progress: TaskProgress) -> String {
    format!("{} of {}", progress.done, progress.total)
}

pub(crate) fn progress_fill(progress: TaskProgress) -> f32 {
    if progress.total == 0 {
        return 0.;
    }
    let done = u16::try_from(progress.done).unwrap_or(u16::MAX);
    let total = u16::try_from(progress.total).unwrap_or(u16::MAX);
    f32::from(done.min(total)) / f32::from(total)
}

pub(crate) fn render(data: &RowData, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let summary = data.summary;
    let id = summary.id.clone();
    let view = cx.entity();
    let title = display_title(&summary.title);
    let age = relative_age(summary.updated_at, Timestamp::now());
    let pinned = summary.pinned;
    let in_trash = data.in_trash;
    let menu_id = id.clone();
    let click_id = id.clone();
    let theme = cx.theme().clone();
    div()
        .id(SharedString::from(format!("note-{id}")))
        .test_support()
        .px(px(12.))
        .pt(px(10.))
        .pb(px(12.))
        .border_b_1()
        .border_color(theme.border)
        .cursor_pointer()
        .flex()
        .flex_col()
        .gap_1()
        .when(data.is_selected, |row| row.bg(theme.list_active))
        .when(!data.is_selected, |row| {
            row.hover(|row| row.bg(theme.list_hover))
        })
        .child(title_line(summary, title, cx))
        .child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap_1()
                .mt_1()
                .text_size(px(12.))
                .child(div().text_color(theme.primary).child(age))
                .children(
                    summary
                        .task_progress
                        .map(|progress| progress_view(progress, cx)),
                )
                .children(
                    summary
                        .tags
                        .iter()
                        .map(|tag| tag_pill(&summary.id, tag, cx)),
                ),
        )
        .when(!summary.excerpt.is_empty(), |row| {
            row.child(
                div()
                    .truncate()
                    .text_size(px(13.))
                    .text_color(theme.muted_foreground)
                    .child(summary.excerpt.clone()),
            )
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

fn title_line(
    summary: &NoteSummary,
    title: &str,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let theme = cx.theme();
    let id = &summary.id;
    div()
        .flex()
        .flex_row()
        .items_start()
        .gap_1()
        .when(summary.pinned, |line| {
            line.child(
                div()
                    .id(SharedString::from(format!("pin-{id}")))
                    .test_support()
                    .flex_none()
                    .child(glyph(IconName::Pin).small().text_color(theme.primary)),
            )
        })
        .children(status_glyph(summary.status).map(|icon| div().flex_none().child(icon.small())))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(px(14.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.foreground)
                .line_clamp(2)
                .child(title.to_owned()),
        )
}

fn progress_view(progress: TaskProgress, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let theme = cx.theme();
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .text_color(theme.muted_foreground)
        .child(glyph(IconName::ListChecks).xsmall())
        .child(
            div()
                .w(px(PROGRESS_BAR_WIDTH))
                .h(px(4.))
                .rounded_full()
                .bg(theme.border)
                .child(
                    div()
                        .h_full()
                        .w(px(PROGRESS_BAR_WIDTH * progress_fill(progress)))
                        .rounded_full()
                        .bg(theme.primary),
                ),
        )
        .child(progress_label(progress))
}

fn tag_pill(note: &NoteId, tag: &Tag, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let tag_id: TagId = tag.id.clone();
    let color = dot_color(tag.color);
    let text = pill_text_color(color, cx.theme().background, cx.theme().is_dark());
    div()
        .id(SharedString::from(format!("chip-{note}-{}", tag.id)))
        .test_support()
        .cursor_pointer()
        .px(px(7.))
        .py(px(1.))
        .rounded_full()
        .text_size(px(11.))
        .text_color(text)
        .bg(color.opacity(PILL_TINT))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(cx.listener(move |workspace, _, _, cx| workspace.select_tag(&tag_id, false, cx)))
        .child(tag.name.clone())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_reads_done_of_total() {
        assert_eq!(progress_label(TaskProgress { done: 2, total: 4 }), "2 of 4");
    }

    #[test]
    fn the_bar_fills_in_proportion_to_the_tasks_done() {
        assert!((progress_fill(TaskProgress { done: 1, total: 4 }) - 0.25).abs() < f32::EPSILON);
        assert!((progress_fill(TaskProgress { done: 0, total: 3 })).abs() < f32::EPSILON);
        assert!((progress_fill(TaskProgress { done: 3, total: 3 }) - 1.).abs() < f32::EPSILON);
    }
}
