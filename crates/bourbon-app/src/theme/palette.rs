use bourbon_core::{Appearance, OmarchyColors, Rgb, ThemeSource};
use gpui_kit::Hsla;
use gpui_kit::component::ThemeColor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Palette {
    pub(crate) background: Rgb,
    pub(crate) sidebar: Rgb,
    pub(crate) foreground: Rgb,
    pub(crate) border: Rgb,
    pub(crate) muted: Rgb,
    pub(crate) muted_foreground: Rgb,
    pub(crate) accent: Rgb,
    pub(crate) accent_foreground: Rgb,
    pub(crate) selection: Rgb,
    pub(crate) sidebar_selection: Rgb,
    pub(crate) danger: Rgb,
}

const fn rgb(hex: u32) -> Rgb {
    let [_, red, green, blue] = hex.to_be_bytes();
    Rgb { red, green, blue }
}

pub(crate) const BOURBON_DARK: Palette = Palette {
    background: rgb(0x0026_2a31),
    sidebar: rgb(0x001f_2228),
    foreground: rgb(0x00d7_dae0),
    border: rgb(0x0034_3944),
    muted: rgb(0x002d_323b),
    muted_foreground: rgb(0x008a_909c),
    accent: rgb(0x005b_9bf0),
    accent_foreground: rgb(0x0010_131a),
    selection: rgb(0x002f_3d52),
    sidebar_selection: rgb(0x002c_313b),
    danger: rgb(0x00e5_534b),
};

pub(crate) const BOURBON_LIGHT: Palette = Palette {
    background: rgb(0x00f7_f7f8),
    sidebar: rgb(0x00eb_ebee),
    foreground: rgb(0x001f_2024),
    border: rgb(0x00de_dee2),
    muted: rgb(0x00ee_eef1),
    muted_foreground: rgb(0x006f_7078),
    accent: rgb(0x003d_7be0),
    accent_foreground: rgb(0x00ff_ffff),
    selection: rgb(0x00d6_dfeb),
    sidebar_selection: rgb(0x00d8_d8de),
    danger: rgb(0x00d1_4b4b),
};

const WHITE: Rgb = rgb(0x00ff_ffff);
const BLACK: Rgb = rgb(0x0000_0000);

impl Palette {
    pub(crate) fn bourbon(appearance: Appearance) -> Self {
        match appearance {
            Appearance::Dark => BOURBON_DARK,
            Appearance::Light => BOURBON_LIGHT,
        }
    }

    pub(crate) fn from_omarchy(colors: &OmarchyColors) -> Self {
        let background = colors.background;
        let foreground = colors.foreground;
        Self {
            background,
            sidebar: background.mix(foreground, 0.05),
            foreground,
            border: background.mix(foreground, 0.18),
            muted: background.mix(foreground, 0.09),
            muted_foreground: colors.muted,
            accent: colors.accent,
            accent_foreground: if colors.accent.is_dark() {
                WHITE
            } else {
                BLACK
            },
            selection: background.mix(colors.selection, 0.4),
            sidebar_selection: background.mix(colors.selection, 0.4),
            danger: colors.danger,
        }
    }

    pub(crate) fn paint(&self, colors: &mut ThemeColor) {
        let hsla = |color: Rgb| Hsla::from(gpui_kit::rgb(color.as_hex()));
        colors.background = hsla(self.background);
        colors.foreground = hsla(self.foreground);
        colors.popover = hsla(self.sidebar);
        colors.popover_foreground = hsla(self.foreground);
        colors.title_bar = hsla(self.sidebar);
        colors.sidebar = hsla(self.sidebar);
        colors.sidebar_foreground = hsla(self.foreground);
        colors.sidebar_border = hsla(self.border);
        colors.sidebar_accent = hsla(self.sidebar_selection);
        colors.sidebar_accent_foreground = hsla(self.foreground);
        colors.border = hsla(self.border);
        colors.input = hsla(self.border);
        colors.muted = hsla(self.muted);
        colors.muted_foreground = hsla(self.muted_foreground);
        colors.secondary = hsla(self.muted);
        colors.secondary_foreground = hsla(self.foreground);
        colors.accent = hsla(self.selection);
        colors.accent_foreground = hsla(self.foreground);
        colors.list_hover = hsla(self.muted);
        colors.list_active = hsla(self.selection);
        colors.selection = hsla(self.selection);
        colors.primary = hsla(self.accent);
        colors.primary_foreground = hsla(self.accent_foreground);
        colors.ring = hsla(self.accent);
        colors.link = hsla(self.accent);
        colors.danger = hsla(self.danger);
    }
}

pub(crate) fn resolve(
    source: ThemeSource,
    appearance: Appearance,
    omarchy: Option<&OmarchyColors>,
) -> (Palette, Appearance) {
    match (source, omarchy) {
        (ThemeSource::Omarchy, Some(colors)) => {
            let own_appearance = if colors.is_dark() {
                Appearance::Dark
            } else {
                Appearance::Light
            };
            (Palette::from_omarchy(colors), own_appearance)
        }
        _ => (Palette::bourbon(appearance), appearance),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn omarchy(background: &str) -> OmarchyColors {
        OmarchyColors::parse(&format!(
            "background = \"{background}\"\nforeground = \"#c0c0c0\"\naccent = \"#268bd2\"\nselection = \"#6c7c81\"\ncolor1 = \"#d8322f\"\ncolor8 = \"#5f6f74\"\n"
        ))
        .unwrap()
    }

    #[test]
    fn bourbon_source_uses_its_own_palette_for_the_chosen_appearance() {
        assert_eq!(
            resolve(ThemeSource::Bourbon, Appearance::Light, None),
            (BOURBON_LIGHT, Appearance::Light)
        );
        assert_eq!(
            resolve(
                ThemeSource::Bourbon,
                Appearance::Dark,
                Some(&omarchy("#000000"))
            ),
            (BOURBON_DARK, Appearance::Dark)
        );
    }

    #[test]
    fn omarchy_source_ignores_the_chosen_appearance_and_follows_the_background() {
        let (palette, appearance) = resolve(
            ThemeSource::Omarchy,
            Appearance::Light,
            Some(&omarchy("#001c2b")),
        );
        assert_eq!(appearance, Appearance::Dark);
        assert_eq!(palette.background, Rgb::from_hex("#001c2b").unwrap());
        let (_, appearance) = resolve(
            ThemeSource::Omarchy,
            Appearance::Dark,
            Some(&omarchy("#fdf6e3")),
        );
        assert_eq!(appearance, Appearance::Light);
    }

    #[test]
    fn omarchy_without_colors_falls_back_to_bourbon() {
        assert_eq!(
            resolve(ThemeSource::Omarchy, Appearance::Light, None),
            (BOURBON_LIGHT, Appearance::Light)
        );
    }

    #[test]
    fn the_omarchy_mapping_takes_accent_muted_and_danger_straight_from_the_scheme() {
        let colors = omarchy("#001c2b");
        let palette = Palette::from_omarchy(&colors);
        assert_eq!(palette.accent, colors.accent);
        assert_eq!(palette.muted_foreground, colors.muted);
        assert_eq!(palette.danger, colors.danger);
        assert_eq!(palette.foreground, colors.foreground);
    }

    #[test]
    fn the_omarchy_mapping_derives_surfaces_between_background_and_foreground() {
        let colors = omarchy("#001c2b");
        let palette = Palette::from_omarchy(&colors);
        assert_eq!(
            palette.sidebar,
            colors.background.mix(colors.foreground, 0.05)
        );
        assert_eq!(
            palette.border,
            colors.background.mix(colors.foreground, 0.18)
        );
        assert_eq!(
            palette.muted,
            colors.background.mix(colors.foreground, 0.09)
        );
        assert_eq!(
            palette.selection,
            colors.background.mix(colors.selection, 0.4)
        );
    }

    #[test]
    fn text_on_the_accent_contrasts_with_it() {
        let dark_accent = omarchy("#001c2b");
        assert_eq!(Palette::from_omarchy(&dark_accent).accent_foreground, WHITE);
        let mut light_accent = omarchy("#001c2b");
        light_accent.accent = Rgb::from_hex("#f0f0a0").unwrap();
        assert_eq!(
            Palette::from_omarchy(&light_accent).accent_foreground,
            BLACK
        );
    }

    #[test]
    fn the_bourbon_palettes_follow_the_inkdrop_reference() {
        assert_eq!(BOURBON_LIGHT.sidebar, Rgb::from_hex("#ebebee").unwrap());
        assert_eq!(BOURBON_LIGHT.background, Rgb::from_hex("#f7f7f8").unwrap());
        assert_eq!(BOURBON_LIGHT.accent, Rgb::from_hex("#3d7be0").unwrap());
        assert_eq!(BOURBON_DARK.sidebar, Rgb::from_hex("#1f2228").unwrap());
        assert_eq!(BOURBON_DARK.background, Rgb::from_hex("#262a31").unwrap());
        assert_eq!(BOURBON_DARK.accent, Rgb::from_hex("#5b9bf0").unwrap());
    }

    #[test]
    fn the_sidebar_selection_differs_from_the_note_list_selection() {
        assert_ne!(BOURBON_LIGHT.sidebar_selection, BOURBON_LIGHT.selection);
        assert_ne!(BOURBON_DARK.sidebar_selection, BOURBON_DARK.selection);
    }
}
