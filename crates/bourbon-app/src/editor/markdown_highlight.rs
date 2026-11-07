use gpui_kit::Hsla;
use gpui_kit::component::Theme;
use gpui_kit::component::highlighter::HighlightTheme;
use serde_json::{Value, json};

const ACTIVE_LINE_TINT: f32 = 0.1;
const LINE_NUMBER_TINT: f32 = 0.6;
const BLUE_HUE: f32 = 0.6;
const MAGENTA_HUE: f32 = 0.92;
const BOLD: u16 = 700;

fn colored(color: Hsla) -> Value {
    json!({ "color": color })
}

fn bold(color: Hsla) -> Value {
    json!({ "color": color, "font_weight": BOLD })
}

fn restyled(theme: &Theme) -> Option<HighlightTheme> {
    let colors = &theme.colors;
    let base = colors.blue;
    let blue = gpui_kit::hsla(BLUE_HUE, base.s.max(0.6), base.l, base.a);
    let magenta = gpui_kit::hsla(MAGENTA_HUE, base.s.max(0.6), base.l, base.a);
    let mut value = serde_json::to_value(&*theme.highlight_theme).ok()?;
    let style = value.get_mut("style")?;
    style["editor.active_line.background"] = json!(colors.primary.opacity(ACTIVE_LINE_TINT));
    style["editor.line_number"] = json!(colors.muted_foreground.opacity(LINE_NUMBER_TINT));
    style["editor.active_line_number"] = json!(colors.muted_foreground);
    style["editor.background"] = json!(colors.background);
    style["editor.foreground"] = json!(colors.foreground);
    let syntax = &mut style["syntax"];
    syntax["title"] = bold(magenta);
    syntax["punctuation.special"] = bold(blue);
    syntax["punctuation.list_marker"] = colored(blue);
    syntax["punctuation.delimiter"] = colored(blue);
    syntax["text.literal"] = colored(colors.foreground);
    syntax["text.code.span"] = colored(colors.muted_foreground);
    syntax["emphasis.strong"] = bold(colors.foreground);
    syntax["link_text"] = colored(blue);
    serde_json::from_value(value).ok()
}

pub(crate) fn paint_markdown_highlight(theme: &mut Theme) {
    if let Some(highlight) = restyled(theme) {
        theme.highlight_theme = std::sync::Arc::new(highlight);
    }
}
