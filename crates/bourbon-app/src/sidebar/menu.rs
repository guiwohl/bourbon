use gpui_kit::component::ActiveTheme as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, div,
};

use crate::control_pane::PaneAction;
use crate::workspace::Workspace;

pub(super) struct Entry {
    pub(super) id: &'static str,
    pub(super) label: &'static str,
    pub(super) action: PaneAction,
}

pub(super) const ENTRIES: [Entry; 4] = [
    Entry {
        id: "app-menu-back-up",
        label: "Back up now",
        action: PaneAction::BackUpNow,
    },
    Entry {
        id: "app-menu-restore",
        label: "Restore from backup\u{2026}",
        action: PaneAction::RestoreFromBackup,
    },
    Entry {
        id: "app-menu-export",
        label: "Export to Markdown\u{2026}",
        action: PaneAction::ExportToMarkdown,
    },
    Entry {
        id: "app-menu-settings",
        label: "Settings",
        action: PaneAction::OpenSettings,
    },
];

pub(super) fn render(is_open: bool, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let toggle = div()
        .id("app-menu")
        .test_support()
        .px_2()
        .rounded_md()
        .cursor_pointer()
        .child("\u{22ef}")
        .on_click(cx.listener(|workspace, _, _, cx| workspace.toggle_app_menu(cx)));
    let entries = ENTRIES.map(|entry| {
        let action = entry.action;
        div()
            .id(entry.id)
            .test_support()
            .flex()
            .justify_between()
            .px_2()
            .py_1()
            .rounded_md()
            .cursor_pointer()
            .hover(|row| row.bg(cx.theme().sidebar_accent))
            .child(entry.label)
            .children(action.shortcut().map(|shortcut| {
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(shortcut)
            }))
            .on_click(cx.listener(move |workspace, _, window, cx| {
                workspace.run_menu_entry(action, window, cx);
            }))
    });
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().flex().justify_end().child(toggle))
        .when(is_open, |menu| menu.children(entries))
}
