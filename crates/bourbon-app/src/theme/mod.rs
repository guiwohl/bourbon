mod palette;
mod tokens;
mod watch;

use std::path::Path;

use bourbon_core::{Appearance, OmarchyColors, Settings};
use gpui_kit::App;
use gpui_kit::component::{Theme, ThemeMode};

use crate::fonts;

#[cfg(test)]
pub(crate) const BOURBON_LIGHT_BACKGROUND: u32 = 0x00f7_f7f8;

pub(crate) use watch::ModificationWatch;

pub(crate) fn apply(settings: &Settings, omarchy_colors_path: &Path, cx: &mut App) {
    let omarchy = std::fs::read_to_string(omarchy_colors_path)
        .ok()
        .and_then(|text| OmarchyColors::parse(&text));
    let (palette, appearance) =
        palette::resolve(settings.theme_source, settings.appearance, omarchy.as_ref());
    let mode = match appearance {
        Appearance::Dark => ThemeMode::Dark,
        Appearance::Light => ThemeMode::Light,
    };
    let (ui_tokens, ui_appearance) =
        tokens::tokens_for(settings.theme_source, settings.appearance, omarchy.as_ref());
    bourbon_ui::Theme::install_tokens(ui_tokens, ui_appearance, cx);
    Theme::change(mode, None, cx);
    Theme::update(cx, |theme| {
        palette.paint(&mut theme.colors);
        theme.font_family = fonts::UI_FAMILY.into();
    });
}
