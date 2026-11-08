mod tokens;
mod watch;

use std::path::Path;

use bourbon_core::{OmarchyColors, Settings};
use gpui::App;

pub(crate) use watch::ModificationWatch;

pub(crate) fn apply(settings: &Settings, omarchy_colors_path: &Path, cx: &mut App) {
    let omarchy = std::fs::read_to_string(omarchy_colors_path)
        .ok()
        .and_then(|text| OmarchyColors::parse(&text));
    let (ui_tokens, ui_appearance) =
        tokens::tokens_for(settings.theme_source, settings.appearance, omarchy.as_ref());
    bourbon_ui::Theme::install_tokens(ui_tokens, ui_appearance, cx);
}
