use gpui_kit::assets::IconName;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::Sizable as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::{Context, Entity, IntoElement, ParentElement as _, Styled as _, div, px};

use crate::control_pane::PaneAction;
use crate::icons::glyph;
use crate::workspace::Workspace;

pub(super) const ENTRIES: [(&str, PaneAction); 4] = [
    ("Settings", PaneAction::OpenSettings),
    ("Back up now", PaneAction::BackUpNow),
    ("Restore from backup\u{2026}", PaneAction::RestoreFromBackup),
    ("Export to Markdown\u{2026}", PaneAction::ExportToMarkdown),
];

const TOP_ROW_HEIGHT: f32 = 40.;
const EDGE_GAP: f32 = 12.;

pub(super) fn render(cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let workspace = cx.entity();
    let muted = cx.theme().muted_foreground;
    let gear = Button::new("app-menu")
        .ghost()
        .small()
        .icon(glyph(IconName::Settings))
        .dropdown_menu(move |menu, _, _| {
            let shortcut_color = muted;
            ENTRIES.iter().fold(menu, |menu, (label, action)| {
                menu.item(entry(label, *action, shortcut_color, &workspace))
            })
        });
    div()
        .h(px(TOP_ROW_HEIGHT))
        .flex_none()
        .flex()
        .flex_row()
        .items_center()
        .justify_end()
        .pr(px(EDGE_GAP))
        .child(gear)
}

fn entry(
    label: &'static str,
    action: PaneAction,
    shortcut_color: gpui_kit::Hsla,
    workspace: &Entity<Workspace>,
) -> PopupMenuItem {
    let workspace = workspace.clone();
    PopupMenuItem::element(move |_, _| {
        div()
            .flex()
            .flex_row()
            .justify_between()
            .gap_4()
            .child(label)
            .children(
                action
                    .shortcut()
                    .map(|shortcut| div().text_xs().text_color(shortcut_color).child(shortcut)),
            )
    })
    .on_click(move |_, window, cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.run_pane_action(action, window, cx);
        });
    })
}
