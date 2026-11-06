use bourbon_core::{Appearance, Settings, ThemeSource};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    Context, Div, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, div,
};

use crate::workspace::Workspace;

pub(crate) fn render(settings: &Settings, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let follows_omarchy = settings.theme_source == ThemeSource::Omarchy;
    let theme_options = div()
        .flex()
        .gap_2()
        .child(option(
            "settings-theme-bourbon",
            "Bourbon",
            settings.theme_source == ThemeSource::Bourbon,
            true,
            cx,
            |view, cx| view.set_theme_source(ThemeSource::Bourbon, cx),
        ))
        .child(option(
            "settings-theme-omarchy",
            "Follow Omarchy",
            follows_omarchy,
            true,
            cx,
            |view, cx| view.set_theme_source(ThemeSource::Omarchy, cx),
        ));
    let appearance_options = div()
        .flex()
        .gap_2()
        .child(option(
            "settings-appearance-light",
            "Light",
            settings.appearance == Appearance::Light,
            !follows_omarchy,
            cx,
            |view, cx| view.set_appearance(Appearance::Light, cx),
        ))
        .child(option(
            "settings-appearance-dark",
            "Dark",
            settings.appearance == Appearance::Dark,
            !follows_omarchy,
            cx,
            |view, cx| view.set_appearance(Appearance::Dark, cx),
        ));
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
                .child(option(
                    "settings-close",
                    "Back to notes",
                    false,
                    true,
                    cx,
                    Workspace::close_settings,
                )),
        )
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Appearance"),
        )
        .child(labelled("Theme", theme_options))
        .child(labelled("Appearance", appearance_options))
}

fn labelled(label: &'static str, options: Div) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().child(label))
        .child(options)
}

fn option(
    id: &'static str,
    label: &'static str,
    selected: bool,
    enabled: bool,
    cx: &mut Context<Workspace>,
    on_click: impl Fn(&mut Workspace, &mut Context<Workspace>) + 'static,
) -> impl IntoElement {
    let theme = cx.theme();
    let (background, foreground) = if selected {
        (theme.primary, theme.primary_foreground)
    } else {
        (theme.muted, theme.foreground)
    };
    div()
        .id(SharedString::from(id))
        .test_support()
        .px_3()
        .py_1()
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(background)
        .text_color(foreground)
        .when(enabled, |button| {
            button
                .cursor_pointer()
                .on_click(cx.listener(move |view, _, _, cx| on_click(view, cx)))
        })
        .when(!enabled, |button| button.opacity(0.5))
        .child(label)
}
