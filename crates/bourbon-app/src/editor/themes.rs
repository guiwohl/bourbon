use bourbon_editor::{EditorTheme, PreviewTheme};
use bourbon_ui::{Appearance, Theme};

pub(crate) struct PaneThemes {
    pub(crate) editor: EditorTheme,
    pub(crate) preview: PreviewTheme,
}

pub(crate) fn pane_themes(theme: &Theme) -> PaneThemes {
    let (mut editor, mut preview) = match theme.appearance {
        Appearance::Light => (EditorTheme::light(), PreviewTheme::light()),
        Appearance::Dark => (EditorTheme::dark(), PreviewTheme::dark()),
    };
    let background = theme.tokens.colors.editor_background;
    editor.background = background;
    preview.background = background;
    PaneThemes { editor, preview }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_appearance_picks_the_light_or_dark_palettes() {
        let light = pane_themes(&Theme::new(Appearance::Light));
        let dark = pane_themes(&Theme::new(Appearance::Dark));
        assert_eq!(light.editor.foreground, EditorTheme::light().foreground);
        assert_eq!(light.preview.text, PreviewTheme::light().text);
        assert_eq!(dark.editor.foreground, EditorTheme::dark().foreground);
        assert_eq!(dark.preview.text, PreviewTheme::dark().text);
    }

    #[test]
    fn both_panes_paint_the_editor_background_of_the_tokens() {
        let mut theme = Theme::new(Appearance::Dark);
        theme.tokens.colors.editor_background = gpui::red();
        let themes = pane_themes(&theme);
        assert_eq!(themes.editor.background, gpui::red());
        assert_eq!(themes.preview.background, gpui::red());
    }
}
