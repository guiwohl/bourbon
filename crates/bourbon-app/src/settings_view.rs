use bourbon_core::{Appearance, BackupAuth, Settings, ThemeSource};
use bourbon_editor::TextField;
use bourbon_ui::popover::below;
use bourbon_ui::{Button, Checkbox, IconButton, IconName, MenuItem, Select, ThemeExt as _, Tokens};
use gpui::prelude::FluentBuilder as _;
use gpui::{
    AnyElement, Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, div, px,
};

use crate::overlay::tagged;
use crate::test_id::TestSupportExt as _;
use crate::workspace::{MenuBuilder, Workspace};

const MENU_WIDTH: f32 = 160.;
const MENU_ITEM_HEIGHT: f32 = 36.;
const MENU_ITEM_PADDING: f32 = 14.;
const MENU_TOP_PADDING: f32 = 8.;
const MENU_ACCENT_WIDTH: f32 = 3.;
const INACTIVE_ITEM_OPACITY: f32 = 0.6;
const CONTENT_PADDING: f32 = 12.;
const CONTENT_MAX_WIDTH: f32 = 640.;
const SECTION_GAP: f32 = 24.;
const FIELD_GAP: f32 = 6.;
const ACTION_GAP: f32 = 7.;
const NOTICE_TINT: f32 = 0.1;
const NOTICE_PADDING_VERTICAL: f32 = 8.;
const NOTICE_PADDING_HORIZONTAL: f32 = 14.;

type Apply = fn(&mut Workspace, &mut Context<Workspace>);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SettingsPage {
    #[default]
    General,
    Backup,
}

impl SettingsPage {
    const ALL: [Self; 2] = [Self::General, Self::Backup];

    fn label(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Backup => "Backup",
        }
    }

    fn id(self) -> &'static str {
        match self {
            Self::General => "settings-page-general",
            Self::Backup => "settings-page-backup",
        }
    }
}

#[derive(Clone, Copy)]
struct Choice {
    label: &'static str,
    selected: bool,
    apply: Apply,
}

fn theme_choices(settings: &Settings) -> [Choice; 2] {
    let source = settings.theme_source;
    [
        Choice {
            label: "Bourbon",
            selected: source == ThemeSource::Bourbon,
            apply: |view, cx| view.set_theme_source(ThemeSource::Bourbon, cx),
        },
        Choice {
            label: "Follow Omarchy",
            selected: source == ThemeSource::Omarchy,
            apply: |view, cx| view.set_theme_source(ThemeSource::Omarchy, cx),
        },
    ]
}

fn appearance_choices(settings: &Settings) -> [Choice; 2] {
    [
        Choice {
            label: "Light",
            selected: settings.appearance == Appearance::Light,
            apply: |view, cx| view.set_appearance(Appearance::Light, cx),
        },
        Choice {
            label: "Dark",
            selected: settings.appearance == Appearance::Dark,
            apply: |view, cx| view.set_appearance(Appearance::Dark, cx),
        },
    ]
}

fn backup_auth_choices(settings: &Settings) -> [Choice; 2] {
    [
        Choice {
            label: "GitHub CLI",
            selected: settings.backup_auth == BackupAuth::GhCli,
            apply: |view, cx| view.set_backup_auth(BackupAuth::GhCli, cx),
        },
        Choice {
            label: "Personal access token",
            selected: settings.backup_auth == BackupAuth::Token,
            apply: |view, cx| view.set_backup_auth(BackupAuth::Token, cx),
        },
    ]
}

fn appearance_is_editable(settings: &Settings) -> bool {
    settings.theme_source == ThemeSource::Bourbon
}

fn appearance_hint(settings: &Settings) -> Option<&'static str> {
    (!appearance_is_editable(settings))
        .then_some("Omarchy decides the appearance while you follow its theme.")
}

fn menu_item_opacity(active: bool) -> f32 {
    if active { 1. } else { INACTIVE_ITEM_OPACITY }
}

fn current_label(choices: &[Choice; 2]) -> &'static str {
    choices
        .iter()
        .find(|choice| choice.selected)
        .map_or(choices[0].label, |choice| choice.label)
}

fn choices_menu(choices: [Choice; 2]) -> MenuBuilder {
    choices
        .into_iter()
        .fold(MenuBuilder::default(), |menu, choice| {
            let apply = choice.apply;
            menu.item(
                MenuItem::new(choice.label).checked(choice.selected),
                move |view, _, cx| apply(view, cx),
            )
        })
}

pub(crate) struct SettingsData<'a> {
    pub(crate) settings: &'a Settings,
    pub(crate) page: SettingsPage,
    pub(crate) repository_input: &'a Entity<TextField>,
    pub(crate) token_input: &'a Entity<TextField>,
    pub(crate) repository_error: Option<SharedString>,
    pub(crate) status: Option<SharedString>,
}

pub(crate) fn render(data: &SettingsData, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let tokens = cx.theme().tokens.clone();
    let page = match data.page {
        SettingsPage::General => general_page(data.settings, &tokens, cx),
        SettingsPage::Backup => backup_page(data, &tokens, cx),
    };
    let content = div()
        .id("settings-content")
        .flex_1()
        .min_w_0()
        .h_full()
        .overflow_y_scroll()
        .p(px(CONTENT_PADDING))
        .bg(tokens.colors.page_background)
        .child(
            div()
                .flex()
                .justify_between()
                .items_start()
                .child(
                    div()
                        .flex_1()
                        .max_w(px(CONTENT_MAX_WIDTH))
                        .flex()
                        .flex_col()
                        .gap(px(SECTION_GAP))
                        .child(page),
                )
                .child(
                    div().id("settings-close").test_support().child(
                        IconButton::new("settings-close-button", IconName::X)
                            .tooltip("Back to notes")
                            .on_click(cx.listener(|view, _, _, cx| view.close_settings(cx))),
                    ),
                ),
        );
    div()
        .id("settings")
        .test_support()
        .flex_1()
        .h_full()
        .flex()
        .text_color(tokens.colors.text)
        .font_family(tokens.fonts.ui_family.clone())
        .text_size(tokens.scale.medium)
        .child(page_menu(data.page, &tokens, cx))
        .child(content)
}

fn page_menu(
    current: SettingsPage,
    tokens: &Tokens,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let colors = tokens.colors;
    div()
        .flex_none()
        .w(px(MENU_WIDTH))
        .h_full()
        .pt(px(MENU_TOP_PADDING))
        .flex()
        .flex_col()
        .bg(colors.sidebar_background)
        .border_r_1()
        .border_color(colors.sidebar_border)
        .children(SettingsPage::ALL.map(|page| {
            let active = page == current;
            div()
                .id(page.id())
                .test_support()
                .relative()
                .flex()
                .items_center()
                .h(px(MENU_ITEM_HEIGHT))
                .px(px(MENU_ITEM_PADDING))
                .cursor_pointer()
                .opacity(menu_item_opacity(active))
                .when(active, |item| {
                    item.bg(colors.selection_inactive).child(
                        div()
                            .debug_selector(move || format!("{}-active", page.id()))
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .left_0()
                            .w(px(MENU_ACCENT_WIDTH))
                            .bg(colors.primary),
                    )
                })
                .on_click(cx.listener(move |view, _, _, cx| view.show_settings_page(page, cx)))
                .child(page.label())
        }))
}

fn heading(title: &'static str, tokens: &Tokens) -> impl IntoElement + use<> {
    div()
        .text_size(tokens.scale.medium)
        .font_weight(tokens.scale.weight_semibold)
        .child(title)
}

fn field(
    label: &'static str,
    control: impl IntoElement,
    description: Option<&'static str>,
    tokens: &Tokens,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(FIELD_GAP))
        .child(div().child(label))
        .child(control)
        .when_some(description, |field, description| {
            field.child(
                div()
                    .text_size(tokens.scale.small)
                    .text_color(tokens.colors.muted_text)
                    .child(description),
            )
        })
        .into_any_element()
}

fn choice_select(
    id: &'static str,
    choices: [Choice; 2],
    enabled: bool,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let select = div().id(id).test_support().child(
        Select::new(
            SharedString::from(format!("{id}-select")),
            current_label(&choices),
        )
        .disabled(!enabled)
        .on_open({
            let view = cx.entity();
            move |bounds, window, cx| {
                view.update(cx, |view, cx| {
                    view.open_menu(below(bounds), choices_menu(choices), window, cx);
                });
            }
        }),
    );
    div().flex().child(select)
}

fn general_page(settings: &Settings, tokens: &Tokens, cx: &mut Context<Workspace>) -> AnyElement {
    let editable = appearance_is_editable(settings);
    div()
        .flex()
        .flex_col()
        .gap(px(SECTION_GAP))
        .child(heading("Appearance", tokens))
        .child(field(
            "Theme",
            choice_select("settings-theme", theme_choices(settings), true, cx),
            None,
            tokens,
        ))
        .child(field(
            "Appearance",
            choice_select(
                "settings-appearance",
                appearance_choices(settings),
                editable,
                cx,
            ),
            appearance_hint(settings),
            tokens,
        ))
        .into_any_element()
}

fn backup_page(data: &SettingsData, tokens: &Tokens, cx: &mut Context<Workspace>) -> AnyElement {
    let settings = data.settings;
    let repository = div()
        .flex()
        .flex_col()
        .gap(px(FIELD_GAP))
        .child(tagged(
            "settings-backup-repository",
            data.repository_input.clone(),
        ))
        .children(data.repository_error.clone().map(|message| {
            div()
                .id("settings-backup-repository-error")
                .test_support()
                .text_size(tokens.scale.small)
                .text_color(tokens.colors.negative)
                .child(message)
        }));
    let actions = div()
        .flex()
        .justify_end()
        .gap(px(ACTION_GAP))
        .child(tagged(
            "settings-backup-now",
            Button::new("settings-backup-now-button")
                .label("Back up now")
                .on_click(cx.listener(|view, _, _, cx| view.back_up_now(cx))),
        ))
        .child(tagged(
            "settings-restore",
            Button::new("settings-restore-button")
                .label("Restore from backup\u{2026}")
                .on_click(cx.listener(|view, _, _, cx| view.begin_restore(cx))),
        ));
    div()
        .flex()
        .flex_col()
        .gap(px(SECTION_GAP))
        .child(heading("Backup", tokens))
        .child(field(
            "Repository",
            repository,
            Some("The GitHub repository that keeps your notes, as owner/name."),
            tokens,
        ))
        .child(field(
            "Sign in with",
            choice_select(
                "settings-backup-auth",
                backup_auth_choices(settings),
                true,
                cx,
            ),
            None,
            tokens,
        ))
        .when(settings.backup_auth == BackupAuth::Token, |page| {
            page.child(field(
                "Token",
                tagged("settings-backup-token", data.token_input.clone()),
                None,
                tokens,
            ))
        })
        .child(tagged(
            "settings-backup-enabled",
            Checkbox::new(
                "settings-backup-enabled-checkbox",
                "Back up automatically",
                settings.backup_enabled,
            )
            .description("Pushes your notes after changes and when Bourbon closes.")
            .on_toggle({
                let view = cx.entity();
                move |_, _, cx| view.update(cx, Workspace::toggle_backup_enabled)
            }),
        ))
        .child(actions)
        .children(data.status.clone().map(|status| {
            div()
                .id("settings-backup-status")
                .test_support()
                .text_size(tokens.scale.small)
                .text_color(tokens.colors.muted_text)
                .child(status)
        }))
        .into_any_element()
}

pub(crate) fn notice_banner(
    message: SharedString,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let tokens = cx.theme().tokens.clone();
    div()
        .id("notice-banner")
        .test_support()
        .py(px(NOTICE_PADDING_VERTICAL))
        .px(px(NOTICE_PADDING_HORIZONTAL))
        .bg(tokens.colors.info.opacity(NOTICE_TINT))
        .border_b_1()
        .border_color(tokens.colors.internal_border)
        .text_color(tokens.colors.text)
        .font_family(tokens.fonts.ui_family.clone())
        .text_size(tokens.scale.small)
        .child(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selected(choices: [Choice; 2]) -> [bool; 2] {
        choices.map(|choice| choice.selected)
    }

    #[test]
    fn the_theme_choice_marks_the_current_source() {
        let mut settings = Settings::default();
        assert_eq!(selected(theme_choices(&settings)), [true, false]);
        settings.theme_source = ThemeSource::Omarchy;
        assert_eq!(selected(theme_choices(&settings)), [false, true]);
    }

    #[test]
    fn the_appearance_choice_marks_the_current_appearance() {
        let mut settings = Settings::default();
        assert_eq!(selected(appearance_choices(&settings)), [true, false]);
        settings.appearance = Appearance::Dark;
        assert_eq!(selected(appearance_choices(&settings)), [false, true]);
    }

    #[test]
    fn the_appearance_is_editable_only_with_the_bourbon_theme() {
        let mut settings = Settings::default();
        assert!(appearance_is_editable(&settings));
        settings.theme_source = ThemeSource::Omarchy;
        assert!(!appearance_is_editable(&settings));
    }

    #[test]
    fn the_appearance_hint_shows_only_while_following_omarchy() {
        let mut settings = Settings::default();
        assert_eq!(appearance_hint(&settings), None);
        settings.theme_source = ThemeSource::Omarchy;
        assert!(appearance_hint(&settings).is_some());
    }

    #[test]
    fn inactive_pages_are_dimmed() {
        assert!((menu_item_opacity(true) - 1.).abs() < f32::EPSILON);
        assert!((menu_item_opacity(false) - INACTIVE_ITEM_OPACITY).abs() < f32::EPSILON);
    }

    #[test]
    fn the_backup_auth_choice_marks_the_current_login() {
        let mut settings = Settings::default();
        assert_eq!(selected(backup_auth_choices(&settings)), [true, false]);
        settings.backup_auth = BackupAuth::Token;
        assert_eq!(selected(backup_auth_choices(&settings)), [false, true]);
    }

    #[test]
    fn the_select_shows_the_selected_choice() {
        let mut settings = Settings::default();
        settings.backup_auth = BackupAuth::Token;
        assert_eq!(
            current_label(&backup_auth_choices(&settings)),
            "Personal access token"
        );
        assert_eq!(current_label(&theme_choices(&settings)), "Bourbon");
    }

    #[test]
    fn the_choice_menu_lists_every_choice_in_order() {
        let settings = Settings::default();
        assert_eq!(
            choices_menu(theme_choices(&settings)).item_labels(),
            ["Bourbon", "Follow Omarchy"]
        );
    }

    #[test]
    fn the_pages_are_general_then_backup() {
        let labels = SettingsPage::ALL.map(SettingsPage::label);
        assert_eq!(labels, ["General", "Backup"]);
        assert_eq!(SettingsPage::default(), SettingsPage::General);
    }
}
