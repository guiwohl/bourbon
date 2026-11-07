mod header;
mod menu;
mod notebooks;
mod prompt;
mod state;
mod tags;
mod tree;

use bourbon_core::{NoteStatus, NotebookNode, TagSummary};
use gpui_kit::assets::IconName;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::label::Label;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, Context, Div, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, Stateful, StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, div,
    px,
};

pub(crate) use state::{DeletionTarget, NameTarget, Prompt, Section, SidebarState};
pub(crate) use tree::subtree_ids;

use crate::icons::{glyph, status_glyph};
use crate::navigation::Navigation;
use crate::presentation::status_label;
use crate::workspace::Workspace;

gpui_kit::actions!(bourbon, [ToggleSidebar]);

const BASE_INDENT: f32 = 8.;
const ROW_HEIGHT: f32 = 30.;
const INDENT_STEP: f32 = 12.;

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
    pub(crate) state: &'a SidebarState,
    pub(crate) width: f32,
}

pub(crate) fn render(
    data: &SidebarData,
    cx: &mut Context<Workspace>,
) -> Option<impl IntoElement + use<>> {
    if !data.state.is_visible {
        return None;
    }
    let mut rows = vec![menu::render(cx).into_any_element()];
    if let Some(prompt) = &data.state.prompt {
        rows.push(prompt::render(prompt, cx).into_any_element());
    }
    rows.push(
        simple_row(
            "nav-all-notes",
            "All Notes",
            IconName::FileText,
            Navigation::AllNotes,
            data.current,
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
            data.current,
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
            data.current,
            cx,
        )
        .into_any_element(),
    );
    Some(
        div()
            .id("sidebar")
            .test_support()
            .w(px(data.width))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .gap_px()
            .px(px(6.))
            .pb_2()
            .overflow_y_scroll()
            .bg(cx.theme().sidebar)
            .text_color(cx.theme().sidebar_foreground)
            .border_r_1()
            .border_color(cx.theme().sidebar_border)
            .children(rows),
    )
}

fn push_notebooks(rows: &mut Vec<AnyElement>, data: &SidebarData, cx: &mut Context<Workspace>) {
    rows.push(notebooks::section_header(data.state, cx).into_any_element());
    if data.state.is_collapsed(Section::Notebooks) {
        return;
    }
    for visible in tree::visible_notebooks(data.notebooks, &data.state.expanded) {
        rows.push(
            notebooks::row(&visible, data.state.inbox.as_ref(), data.current, cx)
                .into_any_element(),
        );
    }
}

fn push_statuses(rows: &mut Vec<AnyElement>, data: &SidebarData, cx: &mut Context<Workspace>) {
    let spec = header::Spec {
        label: "Status",
        section: Section::Status,
        searchable: false,
        addable: false,
    };
    rows.push(header::render(&spec, data.state, cx).into_any_element());
    if data.state.is_collapsed(Section::Status) {
        return;
    }
    for status in STATUS_ROWS {
        rows.push(status_row(status, data.current, cx).into_any_element());
    }
}

fn push_tags(rows: &mut Vec<AnyElement>, data: &SidebarData, cx: &mut Context<Workspace>) {
    let spec = header::Spec {
        label: "Tags",
        section: Section::Tags,
        searchable: true,
        addable: false,
    };
    rows.push(header::render(&spec, data.state, cx).into_any_element());
    if data.state.is_collapsed(Section::Tags) {
        return;
    }
    for summary in data.tags {
        rows.push(tags::row(summary, data.current, cx).into_any_element());
    }
}

fn status_row(
    status: NoteStatus,
    current: &Navigation,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let target = Navigation::Status(status);
    let icon = status_glyph(status).map(IntoElement::into_any_element);
    nav_row(
        format!("nav-status-{}", status.as_stored()).into(),
        0,
        is_current(current, &target),
        cx,
    )
    .test_support()
    .children(icon)
    .child(div().flex_1().truncate().child(status_label(status)))
    .on_click(cx.listener(move |workspace, _, _, cx| workspace.navigate(target.clone(), cx)))
}

fn simple_row(
    id: &str,
    label: &'static str,
    icon: IconName,
    target: Navigation,
    current: &Navigation,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let muted = cx.theme().muted_foreground;
    nav_row(id.to_owned().into(), 0, is_current(current, &target), cx)
        .test_support()
        .child(glyph(icon).text_color(muted))
        .child(div().flex_1().truncate().child(label))
        .on_click(cx.listener(move |workspace, _, _, cx| workspace.navigate(target.clone(), cx)))
}

fn count_label(count: usize, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    Label::new(count.to_string())
        .flex_none()
        .text_size(px(11.))
        .text_color(cx.theme().muted_foreground)
}

fn nav_row(
    id: SharedString,
    depth: u16,
    is_selected: bool,
    cx: &mut Context<Workspace>,
) -> Stateful<Div> {
    div()
        .id(id)
        .h(px(ROW_HEIGHT))
        .flex_none()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .pl(px(indent(depth)))
        .pr(px(8.))
        .rounded(px(6.))
        .text_size(px(14.))
        .cursor_pointer()
        .when(is_selected, |row| {
            row.bg(cx.theme().sidebar_accent)
                .text_color(cx.theme().sidebar_accent_foreground)
        })
}

fn is_current(current: &Navigation, target: &Navigation) -> bool {
    current == target
}

fn indent(depth: u16) -> f32 {
    BASE_INDENT + INDENT_STEP * f32::from(depth)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_simple_row_is_current_only_for_its_own_navigation() {
        assert!(is_current(&Navigation::Trash, &Navigation::Trash));
        assert!(!is_current(&Navigation::Trash, &Navigation::AllNotes));
        assert!(is_current(
            &Navigation::Status(NoteStatus::Active),
            &Navigation::Status(NoteStatus::Active)
        ));
        assert!(!is_current(
            &Navigation::Status(NoteStatus::Active),
            &Navigation::Status(NoteStatus::Dropped)
        ));
    }

    #[test]
    fn each_depth_indents_one_step_further() {
        assert_eq!(indent(0), BASE_INDENT);
        assert_eq!(indent(3), BASE_INDENT + 3. * INDENT_STEP);
    }
}
