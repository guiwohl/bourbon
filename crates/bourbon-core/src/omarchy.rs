use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Rgb {
    #[must_use]
    pub fn from_hex(text: &str) -> Option<Self> {
        let digits = text.strip_prefix('#')?;
        if digits.len() != 6 || !digits.chars().all(|digit| digit.is_ascii_hexdigit()) {
            return None;
        }
        let channel = |start: usize| u8::from_str_radix(&digits[start..start + 2], 16).ok();
        Some(Self {
            red: channel(0)?,
            green: channel(2)?,
            blue: channel(4)?,
        })
    }

    #[must_use]
    pub fn as_hex(self) -> u32 {
        (u32::from(self.red) << 16) | (u32::from(self.green) << 8) | u32::from(self.blue)
    }

    #[must_use]
    pub fn mix(self, other: Self, weight: f32) -> Self {
        let blend = |from: u8, to: u8| {
            let value = f32::from(from) + (f32::from(to) - f32::from(from)) * weight;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let channel = value.round().clamp(0., 255.) as u8;
            channel
        };
        Self {
            red: blend(self.red, other.red),
            green: blend(self.green, other.green),
            blue: blend(self.blue, other.blue),
        }
    }

    #[must_use]
    pub fn luminance(self) -> f32 {
        (0.2126 * f32::from(self.red)
            + 0.7152 * f32::from(self.green)
            + 0.0722 * f32::from(self.blue))
            / 255.
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OmarchyColors {
    pub background: Rgb,
    pub foreground: Rgb,
    pub accent: Rgb,
    pub selection: Rgb,
    pub danger: Rgb,
    pub muted: Rgb,
}

impl OmarchyColors {
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let table = text.parse::<toml::Table>().ok()?;
        let color = |key: &str| Rgb::from_hex(table.get(key)?.as_str()?);
        Some(Self {
            background: color("background")?,
            foreground: color("foreground")?,
            accent: color("accent")?,
            selection: color("selection")?,
            danger: color("color1")?,
            muted: color("color8")?,
        })
    }

    #[must_use]
    pub fn is_dark(&self) -> bool {
        self.background.luminance() < 0.5
    }
}

#[must_use]
pub fn default_omarchy_colors_path() -> PathBuf {
    resolve_omarchy_colors_path(
        std::env::var("XDG_STATE_HOME").ok().as_deref(),
        std::env::var("HOME").ok().as_deref(),
    )
}

fn resolve_omarchy_colors_path(xdg_state_home: Option<&str>, home: Option<&str>) -> PathBuf {
    let state_directory = match (xdg_state_home, home) {
        (Some(xdg), _) if !xdg.is_empty() => PathBuf::from(xdg),
        (_, Some(home)) => PathBuf::from(home).join(".local/state"),
        _ => PathBuf::from(".local/state"),
    };
    state_directory.join("omarchy/current/theme/colors.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOLARIZED: &str = r##"
accent = "#268bd2"
selection = "#6c7c81"
background = "#001c2b"
foreground = "#6c7c81"
color1 = "#d8322f"
color8 = "#5f6f74"
"##;

    fn rgb(red: u8, green: u8, blue: u8) -> Rgb {
        Rgb { red, green, blue }
    }

    #[test]
    fn hex_colors_parse_into_channels() {
        assert_eq!(Rgb::from_hex("#268bd2"), Some(rgb(0x26, 0x8b, 0xd2)));
        assert_eq!(Rgb::from_hex("#FFFFFF"), Some(rgb(255, 255, 255)));
    }

    #[test]
    fn malformed_hex_colors_are_rejected() {
        for text in ["268bd2", "#268bd", "#268bd2f", "#26gbd2", "", "#"] {
            assert_eq!(Rgb::from_hex(text), None, "{text}");
        }
    }

    #[test]
    fn a_color_converts_to_its_packed_hex_value() {
        assert_eq!(rgb(0x26, 0x8b, 0xd2).as_hex(), 0x0026_8bd2);
    }

    #[test]
    fn mixing_interpolates_each_channel() {
        let black = rgb(0, 0, 0);
        let white = rgb(255, 255, 255);
        assert_eq!(black.mix(white, 0.), black);
        assert_eq!(black.mix(white, 1.), white);
        assert_eq!(black.mix(white, 0.5), rgb(128, 128, 128));
        assert_eq!(
            rgb(10, 100, 200).mix(rgb(20, 0, 200), 0.5),
            rgb(15, 50, 200)
        );
    }

    #[test]
    fn luminance_separates_black_from_white() {
        assert!(rgb(0, 0, 0).luminance() < 0.01);
        assert!(rgb(255, 255, 255).luminance() > 0.99);
        assert!(rgb(0, 255, 0).luminance() > rgb(255, 0, 0).luminance());
        assert!(rgb(255, 0, 0).luminance() > rgb(0, 0, 255).luminance());
    }

    #[test]
    fn omarchy_colors_are_read_from_their_keys() {
        let colors = OmarchyColors::parse(SOLARIZED).unwrap();
        assert_eq!(colors.background, rgb(0x00, 0x1c, 0x2b));
        assert_eq!(colors.foreground, rgb(0x6c, 0x7c, 0x81));
        assert_eq!(colors.accent, rgb(0x26, 0x8b, 0xd2));
        assert_eq!(colors.selection, rgb(0x6c, 0x7c, 0x81));
        assert_eq!(colors.danger, rgb(0xd8, 0x32, 0x2f));
        assert_eq!(colors.muted, rgb(0x5f, 0x6f, 0x74));
    }

    #[test]
    fn a_missing_key_or_bad_value_or_bad_toml_yields_nothing() {
        let without_accent = SOLARIZED.replace("accent", "other");
        assert_eq!(OmarchyColors::parse(&without_accent), None);
        let bad_value = SOLARIZED.replace("#268bd2", "blue");
        assert_eq!(OmarchyColors::parse(&bad_value), None);
        assert_eq!(OmarchyColors::parse("= = ="), None);
        assert_eq!(OmarchyColors::parse(""), None);
    }

    #[test]
    fn dark_and_light_backgrounds_are_told_apart() {
        let dark = OmarchyColors::parse(SOLARIZED).unwrap();
        assert!(dark.is_dark());
        let light = OmarchyColors::parse(&SOLARIZED.replace("#001c2b", "#fdf6e3")).unwrap();
        assert!(!light.is_dark());
    }

    #[test]
    fn xdg_state_home_wins() {
        assert_eq!(
            resolve_omarchy_colors_path(Some("/x/state"), Some("/home/u")),
            PathBuf::from("/x/state/omarchy/current/theme/colors.toml")
        );
    }

    #[test]
    fn falls_back_to_home_local_state() {
        assert_eq!(
            resolve_omarchy_colors_path(Some(""), Some("/home/u")),
            PathBuf::from("/home/u/.local/state/omarchy/current/theme/colors.toml")
        );
        assert_eq!(
            resolve_omarchy_colors_path(None, None),
            PathBuf::from(".local/state/omarchy/current/theme/colors.toml")
        );
    }

    #[test]
    fn default_path_ends_with_colors_toml() {
        assert!(default_omarchy_colors_path().ends_with("theme/colors.toml"));
    }
}
