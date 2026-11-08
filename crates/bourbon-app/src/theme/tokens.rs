use bourbon_core::{Appearance, OmarchyColors, Rgb, ThemeSource};
use bourbon_ui::Tokens;
use gpui_kit::Hsla;

const SIDEBAR_MIX: f32 = 0.05;
const NOTE_LIST_MIX: f32 = 0.03;
const MENU_MIX: f32 = 0.07;
const SELECTION_TINT: f32 = 0.2;
const NOTE_LIST_BAR_TINT: f32 = 0.4;

fn hsla(color: Rgb) -> Hsla {
    Hsla::from(gpui_kit::rgb(color.as_hex()))
}

pub(super) fn tokens_for(
    source: ThemeSource,
    appearance: Appearance,
    omarchy: Option<&OmarchyColors>,
) -> (Tokens, bourbon_ui::Appearance) {
    match (source, omarchy) {
        (ThemeSource::Omarchy, Some(colors)) => (
            omarchy_tokens(colors),
            ui_appearance(if colors.is_dark() {
                Appearance::Dark
            } else {
                Appearance::Light
            }),
        ),
        _ => (
            Tokens::for_appearance(ui_appearance(appearance)),
            ui_appearance(appearance),
        ),
    }
}

fn ui_appearance(appearance: Appearance) -> bourbon_ui::Appearance {
    match appearance {
        Appearance::Dark => bourbon_ui::Appearance::Dark,
        Appearance::Light => bourbon_ui::Appearance::Light,
    }
}

pub(crate) fn omarchy_tokens(scheme: &OmarchyColors) -> Tokens {
    let mut tokens = if scheme.is_dark() {
        Tokens::dark()
    } else {
        Tokens::light()
    };
    let colors = &mut tokens.colors;
    let background = scheme.background;
    let accent = hsla(scheme.accent);
    colors.page_background = hsla(background);
    colors.editor_background = hsla(background);
    colors.sidebar_background = hsla(background.mix(scheme.foreground, SIDEBAR_MIX));
    colors.note_list_background = hsla(background.mix(scheme.foreground, NOTE_LIST_MIX));
    colors.menu_background = hsla(background.mix(scheme.foreground, MENU_MIX));
    colors.text = hsla(scheme.foreground);
    colors.sidebar_item_text = hsla(scheme.foreground);
    colors.sidebar_item_active_text = hsla(scheme.foreground);
    colors.muted_text = hsla(scheme.muted);
    colors.primary = accent;
    colors.primary_hover = accent;
    colors.primary_active = accent;
    colors.link = accent;
    colors.note_list_date = accent;
    colors.negative = hsla(scheme.danger);
    colors.selection_inactive = hsla(scheme.selection).alpha(SELECTION_TINT);
    colors.selection_active_sidebar = accent;
    colors.selection_active_note_list = accent.alpha(NOTE_LIST_BAR_TINT);
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scheme(background: &str) -> OmarchyColors {
        OmarchyColors::parse(&format!(
            "background = \"{background}\"\nforeground = \"#c0c0c0\"\naccent = \"#268bd2\"\nselection = \"#6c7c81\"\ncolor1 = \"#d8322f\"\ncolor8 = \"#5f6f74\"\n"
        ))
        .unwrap()
    }

    #[test]
    fn the_bourbon_source_uses_the_inkdrop_tokens_of_the_chosen_appearance() {
        let (tokens, appearance) = tokens_for(
            ThemeSource::Bourbon,
            Appearance::Light,
            Some(&scheme("#001c2b")),
        );
        assert_eq!(appearance, bourbon_ui::Appearance::Light);
        assert_eq!(
            tokens.colors.page_background,
            Tokens::light().colors.page_background
        );
    }

    #[test]
    fn the_omarchy_source_follows_the_scheme_background_and_ignores_the_chosen_appearance() {
        let (tokens, appearance) = tokens_for(
            ThemeSource::Omarchy,
            Appearance::Light,
            Some(&scheme("#001c2b")),
        );
        assert_eq!(appearance, bourbon_ui::Appearance::Dark);
        assert_eq!(
            tokens.colors.page_background,
            hsla(scheme("#001c2b").background)
        );
        let (_, light) = tokens_for(
            ThemeSource::Omarchy,
            Appearance::Dark,
            Some(&scheme("#fdf6e3")),
        );
        assert_eq!(light, bourbon_ui::Appearance::Light);
    }

    #[test]
    fn omarchy_without_a_scheme_falls_back_to_the_bourbon_tokens() {
        let (tokens, appearance) = tokens_for(ThemeSource::Omarchy, Appearance::Dark, None);
        assert_eq!(appearance, bourbon_ui::Appearance::Dark);
        assert_eq!(
            tokens.colors.page_background,
            Tokens::dark().colors.page_background
        );
    }

    #[test]
    fn the_scheme_accent_drives_the_primary_colour_and_the_selection_bars() {
        let colors = scheme("#001c2b");
        let tokens = omarchy_tokens(&colors);
        assert_eq!(tokens.colors.primary, hsla(colors.accent));
        assert_eq!(tokens.colors.selection_active_sidebar, hsla(colors.accent));
        assert_eq!(tokens.colors.negative, hsla(colors.danger));
        assert_eq!(tokens.colors.muted_text, hsla(colors.muted));
    }

    #[test]
    fn the_sidebar_and_the_note_list_are_tinted_towards_the_foreground() {
        let colors = scheme("#001c2b");
        let tokens = omarchy_tokens(&colors);
        assert_eq!(
            tokens.colors.sidebar_background,
            hsla(colors.background.mix(colors.foreground, SIDEBAR_MIX))
        );
        assert_ne!(
            tokens.colors.sidebar_background,
            tokens.colors.note_list_background
        );
    }

    #[test]
    fn the_selection_tint_is_translucent_so_text_stays_readable() {
        let tokens = omarchy_tokens(&scheme("#001c2b"));
        assert!(tokens.colors.selection_inactive.a < 0.5);
    }
}
