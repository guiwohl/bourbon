use bourbon_core::{Note, NoteId, NoteStatus, NotebookId, NotebookNode, Tag, TagId, TagSummary};
use bourbon_editor::TextField;
use bourbon_ui::{Icon, IconName, MenuItem, Pill, ThemeExt as _};
use gpui::prelude::FluentBuilder as _;
use gpui::{
    Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, deferred, div, px,
};

use super::actions::{TagSuggestionNext, TagSuggestionPrevious};
use super::menu_anchor::MenuAnchors;
use crate::cli_command::copy_cli_command;
use crate::presentation::{dot_color, status_icon, status_label};
use crate::test_id::TestSupportExt as _;
use crate::workspace::{MenuBuilder, Workspace};

const NOTEBOOK_INDENT: &str = "    ";
const META_PADDING: f32 = 12.;
pub(super) const META_TEXT_SIZE: f32 = 12.6;
const PICKER_HEIGHT: f32 = 28.;
const PICKER_GAP: f32 = 4.;
const CHEVRON_SIZE: f32 = 10.;
const META_ICON_SIZE: f32 = 14.;
const TAGS_MIN_HEIGHT: f32 = 32.;
const TAG_MARGIN: f32 = 2.;
const TAG_INPUT_MIN_WIDTH: f32 = 84.;
pub(crate) const TAG_INPUT_CONTEXT: &str = "TagInput";
const TAG_INPUT_HEIGHT: f32 = 24.;
const TAG_INPUT_PADDING: f32 = 4.;
const SUGGESTIONS_WIDTH: f32 = 220.;
const SUGGESTION_DOT: f32 = 8.;
const SUGGESTION_GAP: f32 = 8.;
const REMOVE_ICON_SIZE: f32 = 10.;
const REMOVE_IDLE_OPACITY: f32 = 0.4;

const STATUSES: [NoteStatus; 5] = [
    NoteStatus::None,
    NoteStatus::Active,
    NoteStatus::OnHold,
    NoteStatus::Completed,
    NoteStatus::Dropped,
];

pub(crate) struct NoteHeader {
    pub(crate) id: NoteId,
    pub(crate) notebook: NotebookId,
    pub(crate) status: NoteStatus,
    pub(crate) tags: Vec<Tag>,
    pub(crate) is_template: bool,
    pub(crate) is_pinned: bool,
}

impl NoteHeader {
    pub(crate) fn of(note: &Note) -> Self {
        Self {
            id: note.id.clone(),
            notebook: note.notebook.clone(),
            status: note.status,
            tags: note.tags.clone(),
            is_template: note.is_template,
            is_pinned: note.pinned,
        }
    }
}

pub(crate) struct HeaderView<'a> {
    pub(crate) header: &'a NoteHeader,
    pub(crate) notebooks: &'a [NotebookNode],
    pub(crate) tags: &'a [TagSummary],
    pub(crate) tag_field: &'a Entity<TextField>,
    pub(crate) typed_tag: &'a str,
    pub(crate) highlight: Option<usize>,
    pub(crate) anchors: &'a MenuAnchors,
}

struct NotebookRow {
    depth: usize,
    id: NotebookId,
    name: String,
}

pub(super) fn suggestions<'a>(
    typed: &str,
    tags: &'a [TagSummary],
    header: &NoteHeader,
) -> Vec<&'a Tag> {
    let typed = typed.trim().to_lowercase();
    if typed.is_empty() {
        return Vec::new();
    }
    tags.iter()
        .map(|summary| &summary.tag)
        .filter(|tag| !header.tags.iter().any(|own| own.id == tag.id))
        .filter(|tag| tag.name.to_lowercase().contains(&typed))
        .collect()
}

pub(super) fn tag_named<'a>(name: &str, tags: &'a [TagSummary]) -> Option<&'a Tag> {
    let name = name.trim().to_lowercase();
    tags.iter()
        .map(|summary| &summary.tag)
        .find(|tag| tag.name.to_lowercase() == name)
}

fn notebook_rows(nodes: &[NotebookNode]) -> Vec<NotebookRow> {
    let mut rows = Vec::new();
    let mut pending: Vec<(usize, &NotebookNode)> =
        nodes.iter().rev().map(|node| (0, node)).collect();
    while let Some((depth, node)) = pending.pop() {
        rows.push(NotebookRow {
            depth,
            id: node.notebook.id.clone(),
            name: node.notebook.name.clone(),
        });
        pending.extend(node.children.iter().rev().map(|child| (depth + 1, child)));
    }
    rows
}

fn notebook_name(nodes: &[NotebookNode], id: &NotebookId) -> String {
    notebook_rows(nodes)
        .into_iter()
        .find(|row| &row.id == id)
        .map(|row| row.name)
        .unwrap_or_default()
}

fn indented(name: &str, depth: usize) -> String {
    format!("{}{name}", NOTEBOOK_INDENT.repeat(depth))
}

pub(crate) fn notebook_menu(nodes: &[NotebookNode], current: &NotebookId) -> MenuBuilder {
    notebook_rows(nodes)
        .into_iter()
        .fold(MenuBuilder::default(), |menu, row| {
            let item = MenuItem::new(indented(&row.name, row.depth)).checked(&row.id == current);
            let target = row.id;
            menu.item(item, move |workspace, _, cx| {
                workspace.move_selected_to(&target, cx);
            })
        })
}

fn status_shortcut(position: usize) -> String {
    format!("Shift+Alt+{}", position + 1)
}

pub(crate) fn status_menu(current: NoteStatus) -> MenuBuilder {
    STATUSES
        .into_iter()
        .enumerate()
        .fold(MenuBuilder::default(), |menu, (position, status)| {
            let item = MenuItem::new(status_label(status))
                .shortcut(status_shortcut(position))
                .checked(status == current);
            menu.item(item, move |workspace, _, cx| {
                workspace.set_selected_status(status, cx);
            })
        })
}

fn template_action_label(is_template: bool) -> &'static str {
    if is_template {
        "Unmark template"
    } else {
        "Mark as template"
    }
}

pub(crate) fn note_actions_menu(header: &NoteHeader) -> MenuBuilder {
    let is_template = header.is_template;
    let copied = header.id.clone();
    crate::note_list::note_menu(&header.id, header.is_pinned, false)
        .separator()
        .item(
            MenuItem::new(template_action_label(is_template)),
            move |workspace, _, cx| workspace.set_selected_template(!is_template, cx),
        )
        .item(MenuItem::new("Copy CLI command"), move |_, _, cx| {
            copy_cli_command(&copied, cx);
        })
}

fn status_text(status: NoteStatus) -> &'static str {
    match status {
        NoteStatus::None => "Status",
        _ => status_label(status),
    }
}

pub(crate) fn render(view: &HeaderView, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let tokens = cx.theme().tokens.clone();
    let suggested = suggestions(view.typed_tag, view.tags, view.header);
    let tag_input = div()
        .id("tag-input")
        .test_support()
        .key_context(TAG_INPUT_CONTEXT)
        .flex()
        .items_center()
        .h(px(TAG_INPUT_HEIGHT))
        .pl(px(TAG_INPUT_PADDING))
        .min_w(px(TAG_INPUT_MIN_WIDTH))
        .flex_1()
        .on_action(cx.listener(|workspace, _: &TagSuggestionNext, _, cx| {
            workspace.highlight_next_tag_suggestion(cx);
        }))
        .on_action(cx.listener(|workspace, _: &TagSuggestionPrevious, _, cx| {
            workspace.highlight_previous_tag_suggestion(cx);
        }))
        .child(view.tag_field.clone());
    let suggestion_rows: Vec<_> = suggested
        .iter()
        .enumerate()
        .map(|(index, tag)| {
            suggestion_row(tag, view.highlight == Some(index), cx).into_any_element()
        })
        .collect();
    div()
        .relative()
        .flex()
        .flex_none()
        .flex_wrap()
        .items_center()
        .px(px(META_PADDING))
        .text_size(px(META_TEXT_SIZE))
        .font_family(tokens.fonts.ui_family.clone())
        .text_color(tokens.colors.text)
        .child(view.anchors.notebook.wrap(notebook_picker(view, cx)))
        .child(
            view.anchors
                .status
                .wrap(status_picker(view.header.status, cx)),
        )
        .child(
            div()
                .flex()
                .flex_1()
                .flex_wrap()
                .items_center()
                .min_h(px(TAGS_MIN_HEIGHT))
                .children(
                    view.header
                        .tags
                        .iter()
                        .map(|tag| tag_chip(tag, cx).into_any_element()),
                )
                .child(tag_input),
        )
        .when(!suggestion_rows.is_empty(), |row| {
            row.child(deferred(
                div()
                    .id("tag-suggestions")
                    .occlude()
                    .absolute()
                    .top_full()
                    .right(px(META_PADDING))
                    .w(px(SUGGESTIONS_WIDTH))
                    .py(px(PICKER_GAP))
                    .rounded(tokens.scale.radius_large)
                    .border_1()
                    .border_color(tokens.colors.border)
                    .bg(tokens.colors.menu_background)
                    .shadow(tokens.shadows.raised.clone())
                    .children(suggestion_rows),
            ))
        })
}

fn picker(id: &'static str, cx: &mut Context<Workspace>) -> gpui::Stateful<gpui::Div> {
    let tokens = cx.theme().tokens.clone();
    div()
        .id(id)
        .flex()
        .flex_none()
        .items_center()
        .gap(px(PICKER_GAP))
        .min_h(px(PICKER_HEIGHT))
        .mr(px(PICKER_GAP))
        .cursor_pointer()
        .text_color(tokens.colors.text)
}

pub(super) fn highlight_marker(selector: String) -> gpui::Div {
    div()
        .debug_selector(move || selector.clone())
        .absolute()
        .size_full()
}

fn picker_label(text: String) -> gpui::Div {
    let selector = format!("picker-label: {text}");
    div().debug_selector(move || selector.clone()).child(text)
}

fn chevron(cx: &mut Context<Workspace>) -> Icon {
    Icon::new(IconName::ChevronDown)
        .size(px(CHEVRON_SIZE))
        .color(cx.theme().tokens.colors.text)
}

fn notebook_picker(view: &HeaderView, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let name = notebook_name(view.notebooks, &view.header.notebook);
    let weight = cx.theme().tokens.scale.weight_bold;
    let color = cx.theme().tokens.colors.text;
    let icon = Icon::new(IconName::Notebook)
        .size(px(META_ICON_SIZE))
        .color(color);
    let chevron = chevron(cx);
    picker("notebook-selector", cx)
        .test_support()
        .child(icon)
        .child(picker_label(name).font_weight(weight))
        .child(chevron)
        .on_click(cx.listener(|workspace, _, window, cx| {
            workspace.open_notebook_selector(window, cx);
        }))
}

fn status_picker(status: NoteStatus, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let colors = cx.theme().tokens.colors;
    let (icon, color) =
        status_icon(status, &colors).unwrap_or((IconName::Circle, colors.muted_text));
    let chevron = chevron(cx);
    picker("status-button", cx)
        .test_support()
        .child(Icon::new(icon).size(px(META_ICON_SIZE)).color(color))
        .child(picker_label(status_text(status).to_owned()))
        .child(chevron)
        .on_click(cx.listener(|workspace, _, window, cx| {
            workspace.open_status_menu(window, cx);
        }))
}

fn tag_chip(tag: &Tag, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let id: TagId = tag.id.clone();
    let group = format!("tag-chip-group-{}", tag.id);
    let muted = cx.theme().tokens.colors.muted_text;
    div()
        .id(format!("tag-chip-{}", tag.id))
        .test_support()
        .group(group.clone())
        .flex()
        .items_center()
        .mx(px(TAG_MARGIN))
        .child(Pill::new(tag.name.clone(), dot_color(tag.color)))
        .child(
            div()
                .id(format!("remove-tag-{}", tag.id))
                .test_support()
                .opacity(REMOVE_IDLE_OPACITY)
                .group_hover(group, |style| style.opacity(1.))
                .cursor_pointer()
                .child(
                    Icon::new(IconName::X)
                        .size(px(REMOVE_ICON_SIZE))
                        .color(muted),
                )
                .on_click(cx.listener(move |workspace, _, _, cx| {
                    workspace.remove_tag_from_selected(&id, cx);
                })),
        )
}

fn suggestion_row(
    tag: &Tag,
    is_highlighted: bool,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let id = tag.id.clone();
    let highlighted_id = tag.id.clone();
    let tokens = cx.theme().tokens.clone();
    div()
        .id(format!("tag-suggestion-{}", tag.id))
        .relative()
        .aria_selected(is_highlighted)
        .test_support()
        .flex()
        .items_center()
        .gap(px(SUGGESTION_GAP))
        .px(tokens.scale.menu_item_padding_horizontal)
        .py(tokens.scale.menu_item_padding_vertical)
        .cursor_pointer()
        .when(is_highlighted, |row| {
            row.bg(tokens.colors.menu_hover)
                .child(highlight_marker(format!(
                    "tag-suggestion-{highlighted_id}-highlight"
                )))
        })
        .child(
            div()
                .size(px(SUGGESTION_DOT))
                .rounded_full()
                .bg(dot_color(tag.color)),
        )
        .child(tag.name.clone())
        .on_click(cx.listener(move |workspace, _, _, cx| {
            workspace.add_tag_to_selected(&id, cx);
        }))
}

#[cfg(test)]
mod tests {
    use bourbon_core::{Notebook, Timestamp};

    use super::*;

    fn node(id: &str, name: &str, children: Vec<NotebookNode>) -> NotebookNode {
        NotebookNode {
            notebook: Notebook {
                id: NotebookId::parse(id).unwrap(),
                name: name.to_owned(),
                parent: None,
                created_at: Timestamp::from_millis(0),
            },
            children,
            note_count: 0,
        }
    }

    fn tree() -> Vec<NotebookNode> {
        vec![
            node(
                "aaaaaaaaaaaa",
                "Inbox",
                vec![node("bbbbbbbbbbbb", "Work", vec![])],
            ),
            node("cccccccccccc", "Home", vec![]),
        ]
    }

    #[test]
    fn the_notebook_rows_walk_the_tree_depth_first() {
        let rows: Vec<(usize, String)> = notebook_rows(&tree())
            .into_iter()
            .map(|row| (row.depth, row.name))
            .collect();
        assert_eq!(
            rows,
            [
                (0, "Inbox".to_owned()),
                (1, "Work".to_owned()),
                (0, "Home".to_owned())
            ]
        );
    }

    #[test]
    fn the_notebook_menu_indents_children_and_checks_the_current_one() {
        let current = NotebookId::parse("bbbbbbbbbbbb").unwrap();
        assert_eq!(
            notebook_menu(&tree(), &current).item_labels(),
            ["Inbox", "    Work", "Home"]
        );
        assert_eq!(
            notebook_menu(&tree(), &current).checked_labels(),
            ["    Work"]
        );
    }

    #[test]
    fn the_picker_names_the_notebook_of_the_note() {
        let work = NotebookId::parse("bbbbbbbbbbbb").unwrap();
        let missing = NotebookId::parse("dddddddddddd").unwrap();
        assert_eq!(notebook_name(&tree(), &work), "Work");
        assert_eq!(notebook_name(&tree(), &missing), "");
    }

    #[test]
    fn the_status_menu_lists_every_status_with_its_shortcut() {
        let menu = status_menu(NoteStatus::OnHold);
        assert_eq!(
            menu.item_labels(),
            ["None", "Active", "On Hold", "Completed", "Dropped"]
        );
        assert_eq!(menu.checked_labels(), ["On Hold"]);
        assert_eq!(
            menu.shortcuts(),
            [
                "Shift+Alt+1",
                "Shift+Alt+2",
                "Shift+Alt+3",
                "Shift+Alt+4",
                "Shift+Alt+5"
            ]
        );
    }

    #[test]
    fn an_unset_status_reads_status() {
        assert_eq!(status_text(NoteStatus::None), "Status");
        assert_eq!(status_text(NoteStatus::Active), "Active");
    }

    #[test]
    fn the_action_offers_the_opposite_of_the_current_state() {
        assert_eq!(template_action_label(false), "Mark as template");
        assert_eq!(template_action_label(true), "Unmark template");
    }
}
