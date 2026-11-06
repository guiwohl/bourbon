use bourbon_core::{NoteStatus, NotebookNode, TagSummary};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, div, px,
};

use crate::navigation::Navigation;
use crate::workspace::Workspace;

const SIDEBAR_WIDTH: f32 = 240.;
const BASE_INDENT: f32 = 8.;
const INDENT_STEP: f32 = 12.;

const STATUS_ROWS: [(NoteStatus, &str); 4] = [
    (NoteStatus::Active, "Active"),
    (NoteStatus::OnHold, "On Hold"),
    (NoteStatus::Completed, "Completed"),
    (NoteStatus::Dropped, "Dropped"),
];

pub(crate) struct SidebarData<'a> {
    pub(crate) current: &'a Navigation,
    pub(crate) notebooks: &'a [NotebookNode],
    pub(crate) tags: &'a [TagSummary],
}

pub(crate) fn render(data: &SidebarData, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let mut rows = vec![
        row(
            "nav-all-notes".into(),
            "All Notes",
            0,
            Navigation::AllNotes,
            data.current,
            cx,
        )
        .into_any_element(),
    ];
    for (depth, node) in flattened(data.notebooks) {
        rows.push(
            row(
                format!("nav-notebook-{}", node.notebook.id).into(),
                &node.notebook.name,
                depth,
                Navigation::Notebook(node.notebook.id.clone()),
                data.current,
                cx,
            )
            .into_any_element(),
        );
    }
    rows.push(header("Status", cx).into_any_element());
    for (status, label) in STATUS_ROWS {
        rows.push(
            row(
                format!("nav-status-{}", status.as_stored()).into(),
                label,
                0,
                Navigation::Status(status),
                data.current,
                cx,
            )
            .into_any_element(),
        );
    }
    rows.push(header("Tags", cx).into_any_element());
    for summary in data.tags {
        rows.push(
            row(
                format!("nav-tag-{}", summary.tag.id).into(),
                &summary.tag.name,
                0,
                Navigation::Tags(vec![summary.tag.id.clone()]),
                data.current,
                cx,
            )
            .into_any_element(),
        );
    }
    rows.push(
        row(
            "nav-trash".into(),
            "Trash",
            0,
            Navigation::Trash,
            data.current,
            cx,
        )
        .into_any_element(),
    );
    div()
        .id("sidebar")
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
        .children(rows)
}

fn flattened(nodes: &[NotebookNode]) -> Vec<(u16, &NotebookNode)> {
    let mut rows = Vec::new();
    collect(nodes, 0, &mut rows);
    rows
}

fn collect<'a>(nodes: &'a [NotebookNode], depth: u16, rows: &mut Vec<(u16, &'a NotebookNode)>) {
    for node in nodes {
        rows.push((depth, node));
        collect(&node.children, depth + 1, rows);
    }
}

fn header(label: &'static str, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    div()
        .px_2()
        .pt_2()
        .text_xs()
        .text_color(cx.theme().muted_foreground)
        .child(label)
}

fn row(
    id: SharedString,
    label: &str,
    depth: u16,
    target: Navigation,
    current: &Navigation,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let is_selected = is_selected(current, &target);
    div()
        .id(id)
        .test_support()
        .pl(px(indent(depth)))
        .pr_2()
        .py_1()
        .rounded_md()
        .cursor_pointer()
        .truncate()
        .when(is_selected, |row| {
            row.bg(cx.theme().sidebar_accent)
                .text_color(cx.theme().sidebar_accent_foreground)
        })
        .child(label.to_owned())
        .on_click(cx.listener(move |workspace, _, _, cx| workspace.navigate(target.clone(), cx)))
}

fn indent(depth: u16) -> f32 {
    BASE_INDENT + INDENT_STEP * f32::from(depth)
}

fn is_selected(current: &Navigation, target: &Navigation) -> bool {
    match (current, target) {
        (Navigation::Tags(current), Navigation::Tags(target)) => {
            target.iter().all(|tag| current.contains(tag))
        }
        _ => current == target,
    }
}

#[cfg(test)]
mod tests {
    use bourbon_core::{Notebook, NotebookId, TagId, Timestamp};

    use super::*;

    fn node(name: &str, children: Vec<NotebookNode>) -> NotebookNode {
        NotebookNode {
            notebook: Notebook {
                id: NotebookId::generate(),
                name: name.to_owned(),
                parent: None,
                created_at: Timestamp::from_millis(0),
            },
            note_count: 0,
            children,
        }
    }

    #[test]
    fn the_tree_is_flattened_depth_first_with_depths() {
        let tree = vec![
            node(
                "A",
                vec![
                    node("A1", vec![node("A1a", Vec::new())]),
                    node("A2", Vec::new()),
                ],
            ),
            node("B", Vec::new()),
        ];
        let rows: Vec<_> = flattened(&tree)
            .into_iter()
            .map(|(depth, node)| (depth, node.notebook.name.as_str()))
            .collect();
        assert_eq!(rows, [(0, "A"), (1, "A1"), (2, "A1a"), (1, "A2"), (0, "B")]);
    }

    #[test]
    fn each_depth_indents_one_step_further() {
        assert!((indent(0) - 8.).abs() < f32::EPSILON);
        assert!((indent(1) - 20.).abs() < f32::EPSILON);
        assert!((indent(3) - 44.).abs() < f32::EPSILON);
    }

    #[test]
    fn a_row_is_selected_only_for_the_current_navigation() {
        let notebook = NotebookId::generate();
        let tag = TagId::generate();
        assert!(is_selected(&Navigation::Trash, &Navigation::Trash));
        assert!(!is_selected(&Navigation::Trash, &Navigation::AllNotes));
        assert!(is_selected(
            &Navigation::Notebook(notebook.clone()),
            &Navigation::Notebook(notebook)
        ));
        assert!(is_selected(
            &Navigation::Tags(vec![tag.clone(), TagId::generate()]),
            &Navigation::Tags(vec![tag.clone()])
        ));
        assert!(!is_selected(
            &Navigation::Tags(Vec::new()),
            &Navigation::Tags(vec![tag])
        ));
    }
}
