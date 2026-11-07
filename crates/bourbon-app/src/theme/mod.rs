mod palette;
mod watch;

use std::path::Path;

use bourbon_core::{Appearance, OmarchyColors, Settings};
use gpui_kit::App;
use gpui_kit::component::{Theme, ThemeMode};

#[cfg(test)]
pub(crate) const BOURBON_DARK_BACKGROUND: u32 = 0x001c_1613;

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
    Theme::change(mode, None, cx);
    Theme::update(cx, |theme| {
        palette.paint(&mut theme.colors);
        crate::editor::paint_markdown_highlight(theme);
    });
}
