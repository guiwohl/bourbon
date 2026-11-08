mod header;
mod menu;
mod notebooks;
mod prompt;
mod state;
mod tags;
mod tree;

use bourbon_core::{NoteStatus, NotebookNode, TagSummary};
use bourbon_ui::{IconName, ListRow, ScrollArea, ThemeExt as _};
use gpui_kit::{
    AnyElement, Context, FocusHandle, InteractiveElement as _, IntoElement, ParentElement as _,
    ScrollHandle, SharedString, Styled as _, TestSupportExt as _, div, px,
};

pub(crate) use state::{DeletionTarget, NameTarget, Prompt, Section, SidebarCounts, SidebarState};
pub(crate) use tree::subtree_ids;

use crate::navigation::Navigation;
use crate::presentation::{status_icon, status_label};
use crate::workspace::Workspace;

gpui_kit::actions!(bourbon, [ToggleSidebar]);

const STATUS_ROWS: [NoteStatus; 4] = [
    NoteStatus::Active,
    NoteStatus::OnHold,
    NoteStatus::Completed,
    NoteStatus::Dropped,
];

pub(crate) struct SidebarData<'a> {
    pub(crate) current: &'a Navigation,
    pub(crate) notebooks: &'a [NotebookNode],
    pub(crate) tags: &'a [TagSummary],
    pub(crate) counts: &'a SidebarCounts,
    pub(crate) state: &'a SidebarState,
    pub(crate) width: f32,
    pub(crate) is_focused: bool,
    pub(crate) focus: &'a FocusHandle,
    pub(crate) scroll: &'a ScrollHandle,
}

pub(crate) fn render(
    data: &SidebarData,
    cx: &mut Context<Workspace>,
) -> Option<impl IntoElement + use<>> {
    if !data.state.is_visible {
        return None;
    }
    let mut rows: Vec<AnyElement> = Vec::new();
    if let Some(prompt) = &data.state.prompt {
        rows.push(prompt::render(prompt, cx).into_any_element());
    }
    rows.push(
        simple_row(
            "nav-all-notes",
            "All Notes",
            IconName::FileText,
            Navigation::AllNotes,
            Some(data.counts.all_notes),
            data,
            cx,
        )
        .into_any_element(),
    );
    rows.push(
        simple_row(
            "nav-templates",
            "Note Templates",
            IconName::Copy,
            Navigation::Templates,
            Some(data.counts.templates),
            data,
            cx,
        )
        .into_any_element(),
    );
    push_notebooks(&mut rows, data, cx);
    push_statuses(&mut rows, data, cx);
    push_tags(&mut rows, data, cx);
    rows.push(
        simple_row(
            "nav-trash",
            "Trash",
            IconName::Trash,
            Navigation::Trash,
            None,
            data,
            cx,
        )
        .into_any_element(),
    );
    let tokens = cx.theme().tokens.clone();
    Some(
        div()
            .id("sidebar")
            .test_support()
            .key_context("Sidebar")
            .track_focus(data.focus)
            .w(px(data.width))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .bg(tokens.colors.sidebar_background)
            .text_color(tokens.colors.sidebar_item_text)
            .font_family(tokens.fonts.ui_family.clone())
            .child(menu::control_strip(cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .child(ScrollArea::new("sidebar-scroll", data.scroll).children(rows)),
            ),
    )
}

fn push_notebooks(rows: &mut Vec<AnyElement>, data: &SidebarData, cx: &mut Context<Workspace>) {
    rows.push(notebooks::section_header(data.state, cx).into_any_element());
    for visible in tree::visible_notebooks(data.notebooks, &data.state.expanded) {
        rows.push(notebooks::row(&visible, data, cx).into_any_element());
    }
}

fn push_statuses(rows: &mut Vec<AnyElement>, data: &SidebarData, cx: &mut Context<Workspace>) {
    let spec = header::Spec {
        label: "Status",
        section: Section::Status,
        searchable: false,
        addable: false,
        collapsible: true,
    };
    rows.push(header::render(&spec, data.state, cx).into_any_element());
    if data.state.is_collapsed(Section::Status) {
        return;
    }
    for status in STATUS_ROWS {
        rows.push(status_row(status, data, cx).into_any_element());
    }
}

fn push_tags(rows: &mut Vec<AnyElement>, data: &SidebarData, cx: &mut Context<Workspace>) {
    let spec = header::Spec {
        label: "Tags",
        section: Section::Tags,
        searchable: true,
        addable: false,
        collapsible: true,
    };
    rows.push(header::render(&spec, data.state, cx).into_any_element());
    if data.state.is_collapsed(Section::Tags) {
        return;
    }
    for summary in data.tags {
        rows.push(tags::row(summary, data, cx).into_any_element());
    }
}

fn status_row(
    status: NoteStatus,
    data: &SidebarData,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let target = Navigation::Status(status);
    let id = format!("nav-status-{}", status.as_stored());
    let mut row = ListRow::new(
        SharedString::from(format!("{id}-row")),
        status_label(status),
    )
    .count(data.counts.of_status(status))
    .selected(data.current == &target)
    .pane_focused(data.is_focused)
    .on_click(cx.listener(move |workspace, _, _, cx| workspace.navigate(target.clone(), cx)));
    if let Some((icon, color)) = status_icon(status, &cx.theme().tokens.colors) {
        row = row.icon(icon).icon_color(color);
    }
    div().id(SharedString::from(id)).test_support().child(row)
}

fn simple_row(
    id: &'static str,
    label: &'static str,
    icon: IconName,
    target: Navigation,
    count: Option<usize>,
    data: &SidebarData,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let mut row = ListRow::new(SharedString::from(format!("{id}-row")), label)
        .icon(icon)
        .selected(data.current == &target)
        .pane_focused(data.is_focused)
        .on_click(cx.listener(move |workspace, _, _, cx| workspace.navigate(target.clone(), cx)));
    if let Some(count) = count {
        row = row.count(count);
    }
    div().id(id).test_support().child(row)
}
