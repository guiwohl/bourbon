mod notebooks;
mod prompt;
mod state;
mod tags;
mod tree;

use bourbon_core::{NoteStatus, NotebookNode, TagSummary};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    Context, Div, InteractiveElement as _, IntoElement, ParentElement as _, SharedString, Stateful,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, div, px,
};

pub(crate) use state::{DeletionTarget, NameTarget, Prompt, SidebarState};
pub(crate) use tree::subtree_ids;

use crate::navigation::Navigation;
use crate::presentation::status_label;
use crate::workspace::Workspace;

gpui_kit::actions!(bourbon, [ToggleSidebar]);

const SIDEBAR_WIDTH: f32 = 240.;
const BASE_INDENT: f32 = 8.;
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
}

pub(crate) fn render(
    data: &SidebarData,
    cx: &mut Context<Workspace>,
) -> Option<impl IntoElement + use<>> {
    if !data.state.is_visible {
        return None;
    }
    let mut rows = Vec::new();
    if let Some(prompt) = &data.state.prompt {
        rows.push(prompt::render(prompt, cx).into_any_element());
    }
    rows.push(
        simple_row(
            "nav-all-notes",
            "All Notes",
            Navigation::AllNotes,
            data.current,
            cx,
        )
        .into_any_element(),
    );
    rows.push(notebooks::section_header(cx).into_any_element());
    for visible in tree::visible_notebooks(data.notebooks, &data.state.expanded) {
        rows.push(
            notebooks::row(&visible, data.state.inbox.as_ref(), data.current, cx)
                .into_any_element(),
        );
    }
    rows.push(header("Status", cx).into_any_element());
    for status in STATUS_ROWS {
        rows.push(
            simple_row(
                &format!("nav-status-{}", status.as_stored()),
                status_label(status),
                Navigation::Status(status),
                data.current,
                cx,
            )
            .into_any_element(),
        );
    }
    rows.push(header("Tags", cx).into_any_element());
    for summary in data.tags {
        rows.push(tags::row(summary, data.current, cx).into_any_element());
    }
    rows.push(
        simple_row("nav-trash", "Trash", Navigation::Trash, data.current, cx).into_any_element(),
    );
    Some(
        div()
            .id("sidebar")
            .test_support()
            .w(px(SIDEBAR_WIDTH))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .gap_1()
            .p_2()
            .overflow_y_scroll()
            .bg(cx.theme().sidebar)
            .text_color(cx.theme().sidebar_foreground)
            .border_r_1()
            .border_color(cx.theme().sidebar_border)
            .children(rows),
    )
}

fn header(label: &'static str, cx: &mut Context<Workspace>) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .px_2()
        .pt_2()
        .text_xs()
        .text_color(cx.theme().muted_foreground)
        .child(label)
}

fn simple_row(
    id: &str,
    label: &'static str,
    target: Navigation,
    current: &Navigation,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    nav_row(id.to_owned().into(), 0, is_current(current, &target), cx)
        .test_support()
        .child(label)
        .on_click(cx.listener(move |workspace, _, _, cx| workspace.navigate(target.clone(), cx)))
}

fn nav_row(
    id: SharedString,
    depth: u16,
    is_selected: bool,
    cx: &mut Context<Workspace>,
) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .pl(px(indent(depth)))
        .pr_2()
        .py_1()
        .rounded_md()
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
