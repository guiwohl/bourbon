use bourbon_ui::{IconButton, IconName, MenuItem, ThemeExt as _};
use gpui_kit::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, SharedString, Styled as _,
    TestSupportExt as _, div, px,
};

use crate::control_pane::PaneAction;
use crate::workspace::{MenuBuilder, Workspace};

pub(super) const ENTRIES: [(&str, PaneAction); 4] = [
    ("Settings", PaneAction::OpenSettings),
    ("Back up now", PaneAction::BackUpNow),
    ("Restore from backup\u{2026}", PaneAction::RestoreFromBackup),
    ("Export to Markdown\u{2026}", PaneAction::ExportToMarkdown),
];

const TITLE: &str = "Bourbon";
const TITLE_OPACITY: f32 = 0.6;
const TITLE_GAP: f32 = 6.;
const TITLE_MARGIN_LEFT: f32 = 11.;
const TRAILING_PADDING: f32 = 2.;

pub(super) fn builder() -> MenuBuilder {
    ENTRIES
        .iter()
        .fold(MenuBuilder::default(), |menu, (label, action)| {
            let action = *action;
            let item = action.shortcut().map_or_else(
                || MenuItem::new(*label),
                |shortcut| MenuItem::new(*label).shortcut(SharedString::from(shortcut)),
            );
            menu.item(item, move |workspace, window, cx| {
                workspace.run_pane_action(action, window, cx);
            })
        })
}

pub(super) fn control_strip(cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let tokens = cx.theme().tokens.clone();
    let gear = div().id("app-menu").test_support().child(
        IconButton::new("app-menu-button", IconName::Settings)
            .tooltip("Settings")
            .on_click(
                cx.listener(|workspace, event: &gpui_kit::ClickEvent, window, cx| {
                    workspace.open_menu(event.position(), builder(), window, cx);
                }),
            ),
    );
    div()
        .flex()
        .flex_none()
        .items_center()
        .min_h(tokens.scale.header_height)
        .pr(px(TRAILING_PADDING))
        .font_family(tokens.fonts.ui_family.clone())
        .child(
            div()
                .flex_1()
                .min_w_0()
                .ml(px(TITLE_MARGIN_LEFT))
                .mr(px(TITLE_GAP))
                .opacity(TITLE_OPACITY)
                .text_size(tokens.scale.medium)
                .text_color(tokens.colors.text)
                .child(TITLE),
        )
        .child(gear)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_app_menu_offers_settings_backup_restore_and_export_in_order() {
        let labels: Vec<_> = ENTRIES.iter().map(|(label, _)| *label).collect();
        assert_eq!(
            labels,
            [
                "Settings",
                "Back up now",
                "Restore from backup\u{2026}",
                "Export to Markdown\u{2026}"
            ]
        );
    }
}
