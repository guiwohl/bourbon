use bourbon_core::{
    NoteId, NoteSort, NoteStatus, NoteSummary, Tag, TagId, TaskProgress, Timestamp,
};
use bourbon_ui::list_row::RowAccent;
use bourbon_ui::{Colors, Icon, IconName, Pill, ThemeExt as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, App, AppContext as _, Entity, Hsla, InteractiveElement as _, IntoElement,
    MouseButton, MouseDownEvent, ParentElement as _, SharedString, StatefulInteractiveElement as _,
    Styled as _, TestSupportExt as _, div, px,
};
use jiff::tz::TimeZone;

use super::date::date_label;
use crate::dragged_note::DraggedNote;
use crate::presentation::{display_title, dot_color, status_icon};
use crate::workspace::{MenuBuilder, Workspace};

const PROGRESS_INNER_WIDTH: f32 = 30.;
const PROGRESS_HEIGHT: f32 = 6.;
const TITLE_MARGIN: f32 = 8.;
const META_GAP: f32 = 5.;
const META_ICON_SIZE: f32 = 14.;
const LEFT_PADDING: f32 = 14.;
const TOP_PADDING: f32 = 2.;
const BOTTOM_PADDING: f32 = 10.;
const ACCENT_BAR_WIDTH: f32 = 5.;
const TITLE_ICON_SIZE: f32 = 14.;
const EXCERPT_TOP_PADDING: f32 = 3.;

pub(crate) struct RowData<'a> {
    pub(crate) summary: &'a NoteSummary,
    pub(crate) shown: RowAccent,
    pub(crate) in_trash: bool,
    pub(crate) shows_date: bool,
    pub(crate) sort: NoteSort,
    pub(crate) now: Timestamp,
    pub(crate) zone: &'a TimeZone,
}

pub(crate) fn progress_fill(progress: TaskProgress) -> f32 {
    if progress.total == 0 {
        return 0.;
    }
    let done = u16::try_from(progress.done).unwrap_or(u16::MAX);
    let total = u16::try_from(progress.total).unwrap_or(u16::MAX);
    f32::from(done.min(total)) / f32::from(total)
}

fn progress_bar_width(progress: TaskProgress) -> f32 {
    PROGRESS_INNER_WIDTH * progress_fill(progress)
}

fn is_complete(progress: TaskProgress) -> bool {
    progress.total > 0 && progress.done >= progress.total
}

fn dated_by(summary: &NoteSummary, sort: NoteSort) -> Timestamp {
    match sort {
        NoteSort::CreatedNewestFirst => summary.created_at,
        _ => summary.updated_at,
    }
}

pub(crate) fn render(data: &RowData, view: &Entity<Workspace>, cx: &mut App) -> AnyElement {
    let summary = data.summary;
    let tokens = cx.theme().tokens.clone();
    let colors = tokens.colors;
    let shown = data.shown;
    let id = summary.id.clone();
    let clicking = (id.clone(), view.clone());
    let menuing = (id, view.clone(), summary.pinned, data.in_trash);
    div()
        .id(SharedString::from(format!("note-{}", summary.id)))
        .test_support()
        .relative()
        .w_full()
        .flex()
        .flex_col()
        .pl(px(LEFT_PADDING))
        .pt(px(TOP_PADDING))
        .pb(px(BOTTOM_PADDING))
        .border_b_1()
        .border_color(colors.note_list_item_separator)
        .font_family(tokens.fonts.ui_family.clone())
        .text_size(tokens.scale.medium)
        .text_color(colors.text)
        .cursor_pointer()
        .when(shown.tinted, |row| row.bg(colors.selection_inactive))
        .when(shown.bar, |row| {
            row.child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left_0()
                    .w(px(ACCENT_BAR_WIDTH))
                    .bg(colors.selection_active_note_list),
            )
        })
        .child(title_line(summary, &colors, cx))
        .child(meta_line(data, &colors, view))
        .when(!summary.excerpt.is_empty(), |row| {
            row.child(
                div()
                    .truncate()
                    .mr(px(TITLE_MARGIN))
                    .pt(px(EXCERPT_TOP_PADDING))
                    .text_color(colors.muted_text)
                    .child(summary.excerpt.clone()),
            )
        })
        .on_click(move |_, window, cx| {
            let (id, view) = &clicking;
            view.update(cx, |workspace, cx| workspace.select(id, window, cx));
        })
        .on_mouse_down(
            MouseButton::Right,
            move |event: &MouseDownEvent, window, cx| {
                let (id, view, pinned, in_trash) = &menuing;
                view.update(cx, |workspace, cx| {
                    workspace.open_menu(
                        event.position,
                        note_menu(id, *pinned, *in_trash),
                        window,
                        cx,
                    );
                });
            },
        )
        .when(!data.in_trash, |row| {
            row.on_drag(DraggedNote::of(summary), |dragged, _, _, cx| {
                cx.new(|_| dragged.clone())
            })
        })
        .into_any_element()
}

fn title_line(summary: &NoteSummary, colors: &Colors, cx: &App) -> impl IntoElement + use<> {
    let tokens = &cx.theme().tokens;
    let id = &summary.id;
    let status = status_icon(summary.status, colors);
    div()
        .flex()
        .items_start()
        .gap(px(4.))
        .mt(px(TITLE_MARGIN))
        .mr(px(TITLE_MARGIN))
        .mb(px(TITLE_MARGIN))
        .when(summary.pinned, |line| {
            line.child(
                div()
                    .id(SharedString::from(format!("pin-{id}")))
                    .test_support()
                    .flex_none()
                    .child(
                        Icon::new(IconName::Pin)
                            .size(px(TITLE_ICON_SIZE))
                            .color(colors.primary),
                    ),
            )
        })
        .children(status.map(|(icon, color): (IconName, Hsla)| {
            div()
                .flex_none()
                .child(Icon::new(icon).size(px(TITLE_ICON_SIZE)).color(color))
        }))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .font_weight(tokens.scale.weight_semibold)
                .line_clamp(2)
                .when(summary.status == NoteStatus::Dropped, |title| {
                    title.line_through()
                })
                .child(display_title(&summary.title).to_owned()),
        )
}

fn meta_line(
    data: &RowData,
    colors: &Colors,
    view: &Entity<Workspace>,
) -> impl IntoElement + use<> {
    let summary = data.summary;
    let stamp = dated_by(summary, data.sort);
    let shown_date = data
        .shows_date
        .then(|| date_label(stamp, data.now, data.zone));
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(px(META_GAP))
        .mr(px(TITLE_MARGIN))
        .text_size(px(12.))
        .line_height(px(16.2))
        .children(shown_date.map(|date| {
            div()
                .whitespace_nowrap()
                .text_color(colors.note_list_date)
                .child(date)
        }))
        .children(
            summary
                .task_progress
                .map(|progress| progress_view(progress, colors)),
        )
        .children(
            summary
                .tags
                .iter()
                .map(|tag| tag_chip(&summary.id, tag, view)),
        )
}

fn progress_label(progress: TaskProgress) -> String {
    format!("{} of {}", progress.done, progress.total)
}

fn progress_view(progress: TaskProgress, colors: &Colors) -> impl IntoElement + use<> {
    div()
        .flex()
        .items_center()
        .gap(px(META_GAP))
        .text_color(colors.muted_text)
        .child(
            Icon::new(IconName::ListChecks)
                .size(px(META_ICON_SIZE))
                .color(colors.muted_text),
        )
        .child(progress_bar(progress, colors))
        .child(progress_label(progress))
}

fn progress_bar(progress: TaskProgress, colors: &Colors) -> impl IntoElement + use<> {
    let fill = if is_complete(progress) {
        colors.positive
    } else {
        colors.primary
    };
    div()
        .w(px(PROGRESS_INNER_WIDTH + 2.))
        .h(px(PROGRESS_HEIGHT))
        .mx(px(4.))
        .my(px(2.))
        .flex_none()
        .rounded(px(12.))
        .overflow_hidden()
        .border_1()
        .border_color(colors.strong_border)
        .child(div().h_full().w(px(progress_bar_width(progress))).bg(fill))
}

fn tag_chip(note: &NoteId, tag: &Tag, view: &Entity<Workspace>) -> impl IntoElement + use<> {
    let tag_id: TagId = tag.id.clone();
    let view = view.clone();
    div()
        .id(SharedString::from(format!("chip-{note}-{}", tag.id)))
        .test_support()
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(move |_, _, cx| {
            view.update(cx, |workspace, cx| workspace.select_tag(&tag_id, false, cx));
        })
        .child(Pill::new(tag.name.clone(), dot_color(tag.color)).compact())
}

pub(crate) fn note_menu(id: &NoteId, pinned: bool, in_trash: bool) -> MenuBuilder {
    use bourbon_ui::MenuItem;
    if in_trash {
        let restoring = id.clone();
        return MenuBuilder::default().item(MenuItem::new("Restore"), move |workspace, _, cx| {
            workspace.restore(&restoring, cx);
        });
    }
    let pinning = id.clone();
    let duplicating = id.clone();
    let trashing = id.clone();
    MenuBuilder::default()
        .item(
            MenuItem::new(if pinned { "Unpin" } else { "Pin" }),
            move |workspace, _, cx| workspace.toggle_pin(&pinning, cx),
        )
        .item(MenuItem::new("Duplicate"), move |workspace, window, cx| {
            workspace.duplicate(&duplicating, window, cx);
        })
        .item(
            MenuItem::new("Move to Trash"),
            move |workspace, window, cx| {
                workspace.move_to_trash(&trashing, window, cx);
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_reads_done_of_total() {
        assert_eq!(progress_label(TaskProgress { done: 1, total: 2 }), "1 of 2");
    }

    #[test]
    fn the_filled_bar_is_a_share_of_the_full_width() {
        let half = progress_bar_width(TaskProgress { done: 1, total: 2 });
        assert!((half - PROGRESS_INNER_WIDTH / 2.).abs() < f32::EPSILON);
        let empty = progress_bar_width(TaskProgress { done: 0, total: 2 });
        assert!(empty.abs() < f32::EPSILON);
    }

    #[test]
    fn the_bar_fills_in_proportion_to_the_tasks_done() {
        assert!((progress_fill(TaskProgress { done: 1, total: 4 }) - 0.25).abs() < f32::EPSILON);
        assert!((progress_fill(TaskProgress { done: 0, total: 3 })).abs() < f32::EPSILON);
        assert!((progress_fill(TaskProgress { done: 3, total: 3 }) - 1.).abs() < f32::EPSILON);
    }

    #[test]
    fn only_a_note_with_every_task_done_is_complete() {
        assert!(is_complete(TaskProgress { done: 3, total: 3 }));
        assert!(!is_complete(TaskProgress { done: 2, total: 3 }));
        assert!(!is_complete(TaskProgress { done: 0, total: 0 }));
    }

    #[test]
    fn the_date_follows_the_sort_key() {
        let summary = NoteSummary {
            id: NoteId::generate(),
            title: String::new(),
            excerpt: String::new(),
            created_at: Timestamp::from_millis(1),
            updated_at: Timestamp::from_millis(2),
            pinned: false,
            status: NoteStatus::None,
            tags: Vec::new(),
            task_progress: None,
        };
        assert_eq!(
            dated_by(&summary, NoteSort::CreatedNewestFirst),
            Timestamp::from_millis(1)
        );
        assert_eq!(
            dated_by(&summary, NoteSort::UpdatedNewestFirst),
            Timestamp::from_millis(2)
        );
        assert_eq!(
            dated_by(&summary, NoteSort::TitleAlphabetical),
            Timestamp::from_millis(2)
        );
    }
}
