use bourbon_core::{Appearance, BackupAuth, Settings, ThemeSource};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, div,
};

use crate::workspace::Workspace;

type Apply = fn(&mut Workspace, &mut Context<Workspace>);

#[derive(Clone, Copy)]
struct Choice {
    id: &'static str,
    label: &'static str,
    selected: bool,
    enabled: bool,
    apply: Apply,
}

fn theme_choices(settings: &Settings) -> [Choice; 2] {
    let source = settings.theme_source;
    [
        Choice {
            id: "settings-theme-bourbon",
            label: "Bourbon",
            selected: source == ThemeSource::Bourbon,
            enabled: true,
            apply: |view, cx| view.set_theme_source(ThemeSource::Bourbon, cx),
        },
        Choice {
            id: "settings-theme-omarchy",
            label: "Follow Omarchy",
            selected: source == ThemeSource::Omarchy,
            enabled: true,
            apply: |view, cx| view.set_theme_source(ThemeSource::Omarchy, cx),
        },
    ]
}

fn appearance_choices(settings: &Settings) -> [Choice; 2] {
    let follows_omarchy = settings.theme_source == ThemeSource::Omarchy;
    [
        Choice {
            id: "settings-appearance-light",
            label: "Light",
            selected: settings.appearance == Appearance::Light,
            enabled: !follows_omarchy,
            apply: |view, cx| view.set_appearance(Appearance::Light, cx),
        },
        Choice {
            id: "settings-appearance-dark",
            label: "Dark",
            selected: settings.appearance == Appearance::Dark,
            enabled: !follows_omarchy,
            apply: |view, cx| view.set_appearance(Appearance::Dark, cx),
        },
    ]
}

fn opacity_of(enabled: bool) -> f32 {
    if enabled { 1. } else { 0.5 }
}

pub(crate) struct SettingsData<'a> {
    pub(crate) settings: &'a Settings,
    pub(crate) repository_input: &'a Entity<InputState>,
    pub(crate) token_input: &'a Entity<InputState>,
    pub(crate) repository_error: Option<SharedString>,
    pub(crate) status: Option<SharedString>,
}

fn backup_auth_choices(settings: &Settings) -> [Choice; 2] {
    [
        Choice {
            id: "settings-backup-auth-gh",
            label: "GitHub CLI",
            selected: settings.backup_auth == BackupAuth::GhCli,
            enabled: true,
            apply: |view, cx| view.set_backup_auth(BackupAuth::GhCli, cx),
        },
        Choice {
            id: "settings-backup-auth-token",
            label: "Personal access token",
            selected: settings.backup_auth == BackupAuth::Token,
            enabled: true,
            apply: |view, cx| view.set_backup_auth(BackupAuth::Token, cx),
        },
    ]
}

fn backup_enabled_choice(settings: &Settings) -> Choice {
    Choice {
        id: "settings-backup-enabled",
        label: if settings.backup_enabled {
            "Automatic backup: on"
        } else {
            "Automatic backup: off"
        },
        selected: settings.backup_enabled,
        enabled: true,
        apply: Workspace::toggle_backup_enabled,
    }
}

fn backup_section(data: &SettingsData, cx: &mut Context<Workspace>) -> AnyElement {
    let settings = data.settings;
    let auth_options = div()
        .flex()
        .gap_2()
        .children(backup_auth_choices(settings).map(|choice| button(choice, cx)));
    let actions = div()
        .flex()
        .gap_2()
        .child(
            Button::new("settings-backup-now")
                .label("Back up now")
                .on_click(cx.listener(|view, _, _, cx| view.back_up_now(cx))),
        )
        .child(
            Button::new("settings-restore")
                .label("Restore from backup\u{2026}")
                .on_click(cx.listener(|view, _, _, cx| view.begin_restore(cx))),
        );
    let repository = div()
        .flex()
        .flex_col()
        .gap_1()
        .child(Input::new(data.repository_input).id("settings-backup-repository"))
        .children(data.repository_error.clone().map(|message| {
            div()
                .id("settings-backup-repository-error")
                .test_support()
                .text_sm()
                .text_color(cx.theme().danger)
                .child(message)
        }));
    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Backup"),
        )
        .child(labelled("Repository", repository))
        .child(labelled("Sign in with", auth_options))
        .when(settings.backup_auth == BackupAuth::Token, |section| {
            section.child(labelled(
                "Token",
                Input::new(data.token_input).id("settings-backup-token"),
            ))
        })
        .child(button(backup_enabled_choice(settings), cx))
        .child(actions)
        .children(data.status.clone().map(|status| {
            div()
                .id("settings-backup-status")
                .test_support()
                .text_sm()
                .child(status)
        }))
        .into_any_element()
}

pub(crate) fn restore_confirmation(
    question: String,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    div()
        .id("restore-confirmation")
        .test_support()
        .flex()
        .flex_col()
        .gap_2()
        .p_3()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().muted)
        .child(question)
        .child(
            div()
                .flex()
                .gap_2()
                .child(
                    Button::new("restore-confirm")
                        .label("Restore")
                        .danger()
                        .on_click(cx.listener(|view, _, _, cx| view.confirm_restore(cx))),
                )
                .child(
                    Button::new("restore-cancel")
                        .label("Cancel")
                        .on_click(cx.listener(|view, _, _, cx| view.cancel_restore(cx))),
                ),
        )
}

pub(crate) fn notice_banner(
    message: SharedString,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    div()
        .id("notice-banner")
        .test_support()
        .px_3()
        .py_1()
        .rounded_md()
        .bg(cx.theme().muted)
        .text_color(cx.theme().foreground)
        .child(message)
}

pub(crate) fn render(data: &SettingsData, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let settings = data.settings;
    let theme_options = div()
        .flex()
        .gap_2()
        .children(theme_choices(settings).map(|choice| button(choice, cx)));
    let appearance_options = div()
        .flex()
        .gap_2()
        .children(appearance_choices(settings).map(|choice| button(choice, cx)));
    let back = Choice {
        id: "settings-close",
        label: "Back to notes",
        selected: false,
        enabled: true,
        apply: Workspace::close_settings,
    };
    div()
        .id("settings")
        .test_support()
        .flex_1()
        .h_full()
        .flex()
        .flex_col()
        .gap_4()
        .p_6()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(div().text_xl().child("Settings"))
                .child(button(back, cx)),
        )
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Appearance"),
        )
        .child(labelled("Theme", theme_options))
        .child(labelled("Appearance", appearance_options))
        .child(backup_section(data, cx))
}

fn labelled(label: &'static str, options: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().child(label))
        .child(options)
}

fn button(choice: Choice, cx: &mut Context<Workspace>) -> AnyElement {
    let theme = cx.theme();
    let (background, foreground) = if choice.selected {
        (theme.primary, theme.primary_foreground)
    } else {
        (theme.muted, theme.foreground)
    };
    let apply = choice.apply;
    let look = div()
        .id(SharedString::from(choice.id))
        .test_support()
        .px_3()
        .py_1()
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(background)
        .text_color(foreground)
        .opacity(opacity_of(choice.enabled))
        .child(choice.label);
    if choice.enabled {
        look.cursor_pointer()
            .on_click(cx.listener(move |view, _, _, cx| apply(view, cx)))
            .into_any_element()
    } else {
        look.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flags(choices: [Choice; 2]) -> [(bool, bool); 2] {
        choices.map(|choice| (choice.selected, choice.enabled))
    }

    #[test]
    fn the_theme_choice_marks_the_current_source() {
        let mut settings = Settings::default();
        assert_eq!(
            flags(theme_choices(&settings)),
            [(true, true), (false, true)]
        );
        settings.theme_source = ThemeSource::Omarchy;
        assert_eq!(
            flags(theme_choices(&settings)),
            [(false, true), (true, true)]
        );
    }

    #[test]
    fn the_appearance_choice_marks_the_current_appearance_and_is_enabled_for_bourbon() {
        let mut settings = Settings::default();
        assert_eq!(
            flags(appearance_choices(&settings)),
            [(false, true), (true, true)]
        );
        settings.appearance = Appearance::Light;
        assert_eq!(
            flags(appearance_choices(&settings)),
            [(true, true), (false, true)]
        );
    }

    #[test]
    fn the_appearance_choice_is_disabled_while_following_omarchy() {
        let mut settings = Settings::default();
        settings.theme_source = ThemeSource::Omarchy;
        assert_eq!(
            flags(appearance_choices(&settings)),
            [(false, false), (true, false)]
        );
    }

    #[test]
    fn the_backup_auth_choice_marks_the_current_login() {
        let mut settings = Settings::default();
        assert_eq!(
            flags(backup_auth_choices(&settings)),
            [(true, true), (false, true)]
        );
        settings.backup_auth = BackupAuth::Token;
        assert_eq!(
            flags(backup_auth_choices(&settings)),
            [(false, true), (true, true)]
        );
    }

    #[test]
    fn disabled_buttons_are_dimmed() {
        assert!((opacity_of(true) - 1.).abs() < f32::EPSILON);
        assert!(opacity_of(false) < 1.);
    }
}
