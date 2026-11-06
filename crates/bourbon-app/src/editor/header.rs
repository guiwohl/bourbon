use bourbon_core::{Note, NoteId, NoteStatus, NotebookId, NotebookNode, Tag, TagId, TagSummary};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputState, MoveDown, MoveUp};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::popover::Popover;
use gpui_kit::component::tag::Tag as TagChip;
use gpui_kit::component::{IconName, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    Action, Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, WeakEntity, div, px,
};

use super::actions::{
    SetStatusActive, SetStatusCompleted, SetStatusDropped, SetStatusNone, SetStatusOnHold,
};
use crate::cli_command::copy_cli_command;
use crate::presentation::{chip_color, status_label};
use crate::workspace::Workspace;

const INDENT_STEP: f32 = 12.;
const TAG_INPUT_WIDTH: f32 = 160.;

struct StatusOption {
    status: NoteStatus,
    action: fn() -> Box<dyn Action>,
}

const STATUS_OPTIONS: [StatusOption; 5] = [
    StatusOption {
        status: NoteStatus::None,
        action: || Box::new(SetStatusNone),
    },
    StatusOption {
        status: NoteStatus::Active,
        action: || Box::new(SetStatusActive),
    },
    StatusOption {
        status: NoteStatus::OnHold,
        action: || Box::new(SetStatusOnHold),
    },
    StatusOption {
        status: NoteStatus::Completed,
        action: || Box::new(SetStatusCompleted),
    },
    StatusOption {
        status: NoteStatus::Dropped,
        action: || Box::new(SetStatusDropped),
    },
];

pub(crate) struct NoteHeader {
    id: NoteId,
    notebook: NotebookId,
    status: NoteStatus,
    tags: Vec<Tag>,
    is_template: bool,
}

impl NoteHeader {
    pub(crate) fn of(note: &Note) -> Self {
        Self {
            id: note.id.clone(),
            notebook: note.notebook.clone(),
            status: note.status,
            tags: note.tags.clone(),
            is_template: note.is_template,
        }
    }
}

pub(crate) struct HeaderView<'a> {
    pub(crate) header: &'a NoteHeader,
    pub(crate) notebooks: &'a [NotebookNode],
    pub(crate) tags: &'a [TagSummary],
    pub(crate) tag_input: &'a Entity<InputState>,
    pub(crate) typed_tag: &'a str,
    pub(crate) highlight: Option<usize>,
    pub(crate) notebook_selector_open: bool,
}

struct NotebookRow {
    depth: u16,
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

fn notebook_rows(nodes: &[NotebookNode], depth: u16, rows: &mut Vec<NotebookRow>) {
    for node in nodes {
        rows.push(NotebookRow {
            depth,
            id: node.notebook.id.clone(),
            name: node.notebook.name.clone(),
        });
        notebook_rows(&node.children, depth + 1, rows);
    }
}

pub(crate) fn render(view: &HeaderView, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let mut rows = Vec::new();
    notebook_rows(view.notebooks, 0, &mut rows);
    let suggested = suggestions(view.typed_tag, view.tags, view.header);
    div()
        .flex()
        .flex_col()
        .gap_1()
        .capture_action(cx.listener(|workspace, _: &MoveDown, _, cx| {
            workspace.highlight_next_tag_suggestion(cx);
            cx.stop_propagation();
        }))
        .capture_action(cx.listener(|workspace, _: &MoveUp, _, cx| {
            workspace.highlight_previous_tag_suggestion(cx);
            cx.stop_propagation();
        }))
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap_2()
                .child(notebook_selector(view, rows, cx))
                .child(status_button(view.header.status))
                .child(note_actions_button(
                    view.header.is_template,
                    view.header.id.clone(),
                    cx,
                ))
                .children(
                    view.header
                        .tags
                        .iter()
                        .map(|tag| tag_chip(tag, cx).into_any_element()),
                )
                .child(
                    div().w(px(TAG_INPUT_WIDTH)).child(
                        Input::new(view.tag_input)
                            .id("tag-input")
                            .aria_label("Add tags"),
                    ),
                ),
        )
        .children(suggested.iter().enumerate().map(|(index, tag)| {
            suggestion_row(tag, view.highlight == Some(index), cx).into_any_element()
        }))
}

fn notebook_selector(
    view: &HeaderView,
    rows: Vec<NotebookRow>,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let current = view.header.notebook.clone();
    let name = rows
        .iter()
        .find(|row| row.id == current)
        .map(|row| row.name.clone())
        .unwrap_or_default();
    let owner = cx.entity().downgrade();
    let toggled_owner = owner.clone();
    Popover::new("notebook-selector-popover")
        .open(view.notebook_selector_open)
        .on_open_change(move |open, _, cx| {
            toggled_owner
                .update(cx, |workspace, cx| {
                    workspace.set_notebook_selector_open(*open, cx);
                })
                .ok();
        })
        .trigger(Button::new("notebook-selector").ghost().small().label(name))
        .content(move |_, _, cx| {
            let options = rows.iter().map(|row| {
                notebook_option(row, row.id == current, owner.clone(), cx).into_any_element()
            });
            div()
                .flex()
                .flex_col()
                .min_w(px(200.))
                .children(options.collect::<Vec<_>>())
        })
}

fn notebook_option(
    row: &NotebookRow,
    is_current: bool,
    owner: WeakEntity<Workspace>,
    cx: &mut gpui_kit::App,
) -> impl IntoElement + use<> {
    let id = row.id.clone();
    let (background, foreground) = (cx.theme().accent, cx.theme().accent_foreground);
    div()
        .id(format!("notebook-option-{}", row.id))
        .aria_selected(is_current)
        .test_support()
        .ml(px(INDENT_STEP * f32::from(row.depth)))
        .pr_2()
        .py_1()
        .rounded_md()
        .cursor_pointer()
        .when(is_current, |option| {
            option.bg(background).text_color(foreground)
        })
        .child(row.name.clone())
        .on_click(move |_, _, cx| {
            owner
                .update(cx, |workspace, cx| workspace.move_selected_to(&id, cx))
                .ok();
        })
}

fn status_button(current: NoteStatus) -> impl IntoElement + use<> {
    Button::new("status-button")
        .ghost()
        .small()
        .label(status_label(current))
        .dropdown_menu(move |menu, _, _| {
            STATUS_OPTIONS.iter().fold(menu, |menu, option| {
                menu.menu_with_check(
                    status_label(option.status),
                    option.status == current,
                    (option.action)(),
                )
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

fn note_actions_button(
    is_template: bool,
    id: NoteId,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let owner = cx.entity();
    Button::new("note-actions")
        .ghost()
        .small()
        .icon(IconName::Ellipsis)
        .dropdown_menu(move |menu, _, _| {
            let template_owner = owner.clone();
            let id = id.clone();
            menu.item(
                PopupMenuItem::new(template_action_label(is_template)).on_click(move |_, _, cx| {
                    template_owner.update(cx, |workspace, cx| {
                        workspace.set_selected_template(!is_template, cx);
                    });
                }),
            )
            .item(
                PopupMenuItem::new("Copy CLI command").on_click(move |_, _, cx| {
                    copy_cli_command(&id, cx);
                }),
            )
        })
}

fn tag_chip(tag: &Tag, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let id: TagId = tag.id.clone();
    div()
        .id(format!("tag-chip-{}", tag.id))
        .test_support()
        .child(
            TagChip::color(chip_color(tag.color))
                .small()
                .child(tag.name.clone())
                .child(
                    Button::new(format!("remove-tag-{}", tag.id))
                        .ghost()
                        .xsmall()
                        .label("×")
                        .on_click(cx.listener(move |workspace, _, _, cx| {
                            workspace.remove_tag_from_selected(&id, cx);
                        })),
                ),
        )
}

fn suggestion_row(
    tag: &Tag,
    is_highlighted: bool,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let id = tag.id.clone();
    let highlight = cx.theme().accent;
    div()
        .id(format!("tag-suggestion-{}", tag.id))
        .aria_selected(is_highlighted)
        .test_support()
        .px_2()
        .py_1()
        .rounded_md()
        .cursor_pointer()
        .when(is_highlighted, |row| row.bg(highlight))
        .child(tag.name.clone())
        .on_click(cx.listener(move |workspace, _, window, cx| {
            workspace.add_tag_to_selected(&id, window, cx);
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_action_offers_the_opposite_of_the_current_state() {
        assert_eq!(template_action_label(false), "Mark as template");
        assert_eq!(template_action_label(true), "Unmark template");
    }
}
