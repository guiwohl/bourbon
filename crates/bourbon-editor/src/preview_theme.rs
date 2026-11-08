use bourbon_markdown::AlertKind;
use gpui::{Hsla, Pixels, hsla, px};

#[derive(Clone, Debug, PartialEq)]
pub struct PreviewTheme {
    pub font_family: String,
    pub font_fallbacks: Vec<String>,
    pub mono_family: String,
    pub mono_fallbacks: Vec<String>,
    pub font_size: Pixels,
    pub code_font_size: Pixels,
    pub line_height: f32,
    pub heading_scales: [f32; 6],
    pub heading_line_height: f32,
    pub code_line_height: f32,
    pub block_gap: Pixels,
    pub heading_top_gap: Pixels,
    pub rule_gap: Pixels,
    pub list_indent_em: f32,
    pub quote_bar_em: f32,
    pub background: Hsla,
    pub text: Hsla,
    pub muted: Hsla,
    pub border: Hsla,
    pub solid_border: Hsla,
    pub link: Hsla,
    pub code_background: Hsla,
    pub code_border: Hsla,
    pub inline_code_background: Hsla,
    pub table_header_background: Hsla,
    pub table_stripe_background: Hsla,
    pub alert_note: Hsla,
    pub alert_tip: Hsla,
    pub alert_important: Hsla,
    pub alert_warning: Hsla,
    pub alert_caution: Hsla,
}

fn hsl(hue: f32, saturation: f32, lightness: f32, alpha: f32) -> Hsla {
    hsla(hue / 360.0, saturation / 100.0, lightness / 100.0, alpha)
}

impl PreviewTheme {
    pub fn light() -> Self {
        Self {
            background: hsl(0.0, 0.0, 100.0, 1.0),
            text: hsl(0.0, 0.0, 0.0, 0.87),
            muted: hsl(0.0, 0.0, 0.0, 0.60),
            border: hsl(0.0, 0.0, 9.0, 0.15),
            solid_border: hsl(240.0, 5.0, 84.0, 1.0),
            code_background: hsl(214.0, 32.0, 91.0, 0.10),
            code_border: hsl(213.0, 27.0, 84.0, 0.50),
            inline_code_background: hsl(0.0, 0.0, 0.0, 0.05),
            table_header_background: hsl(0.0, 0.0, 0.0, 0.10),
            table_stripe_background: hsl(0.0, 0.0, 0.0, 0.05),
            ..Self::base()
        }
    }

    pub fn dark() -> Self {
        Self {
            background: hsl(0.0, 0.0, 4.0, 1.0),
            text: hsl(0.0, 0.0, 98.0, 0.87),
            muted: hsl(0.0, 0.0, 98.0, 0.60),
            border: hsl(0.0, 0.0, 96.0, 0.10),
            solid_border: hsl(240.0, 5.0, 26.0, 1.0),
            code_background: hsl(0.0, 0.0, 25.0, 0.20),
            code_border: hsl(213.0, 97.0, 87.0, 0.10),
            inline_code_background: hsl(0.0, 0.0, 100.0, 0.08),
            table_header_background: hsl(0.0, 0.0, 100.0, 0.15),
            table_stripe_background: hsl(0.0, 0.0, 100.0, 0.08),
            ..Self::base()
        }
    }

    fn base() -> Self {
        let unset = hsl(0.0, 0.0, 50.0, 1.0);
        Self {
            font_family: "Inter".to_owned(),
            font_fallbacks: ["Helvetica Neue", "Arial", "Liberation Sans", "sans-serif"]
                .map(str::to_owned)
                .to_vec(),
            mono_family: "JetBrains Mono".to_owned(),
            mono_fallbacks: ["DejaVu Sans Mono", "Liberation Mono", "monospace"]
                .map(str::to_owned)
                .to_vec(),
            font_size: px(16.0),
            code_font_size: px(12.0),
            line_height: 1.6,
            heading_scales: [2.0, 1.5, 1.25, 1.0, 0.875, 0.85],
            heading_line_height: 1.25,
            code_line_height: 1.45,
            block_gap: px(16.0),
            heading_top_gap: px(24.0),
            rule_gap: px(24.0),
            list_indent_em: 2.0,
            quote_bar_em: 0.25,
            background: unset,
            text: unset,
            muted: unset,
            border: unset,
            solid_border: unset,
            link: hsl(217.0, 91.0, 60.0, 1.0),
            code_background: unset,
            code_border: unset,
            inline_code_background: unset,
            table_header_background: unset,
            table_stripe_background: unset,
            alert_note: hsl(217.0, 91.0, 60.0, 1.0),
            alert_tip: hsl(142.0, 71.0, 45.0, 1.0),
            alert_important: hsl(271.0, 91.0, 65.0, 1.0),
            alert_warning: hsl(25.0, 95.0, 53.0, 1.0),
            alert_caution: hsl(0.0, 84.0, 60.0, 1.0),
        }
    }

    pub fn heading_scale(&self, level: u8) -> f32 {
        self.heading_scales[usize::from(level.clamp(1, 6)) - 1]
    }

    pub fn alert_color(&self, kind: AlertKind) -> Hsla {
        match kind {
            AlertKind::Note => self.alert_note,
            AlertKind::Tip => self.alert_tip,
            AlertKind::Important => self.alert_important,
            AlertKind::Warning => self.alert_warning,
            AlertKind::Caution => self.alert_caution,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_follow_the_researched_typography() {
        let theme = PreviewTheme::light();
        assert_eq!(theme.font_size, px(16.0));
        assert_eq!(theme.line_height, 1.6);
        assert_eq!(theme.block_gap, px(16.0));
        assert_eq!(theme.heading_top_gap, px(24.0));
        assert_eq!(theme.list_indent_em, 2.0);
        assert_eq!(theme.font_family, "Inter");
    }

    #[test]
    fn colours_are_converted_from_hsl_degrees_and_percent() {
        let theme = PreviewTheme::light();
        assert_eq!(theme.text, hsla(0.0, 0.0, 0.0, 0.87));
        assert_eq!(theme.solid_border, hsla(240.0 / 360.0, 0.05, 0.84, 1.0));
        assert_eq!(theme.code_border, hsla(213.0 / 360.0, 0.27, 0.84, 0.50));
    }

    #[test]
    fn heading_scales_follow_the_research() {
        let theme = PreviewTheme::light();
        let scales: Vec<f32> = (1..=6).map(|level| theme.heading_scale(level)).collect();
        assert_eq!(scales, [2.0, 1.5, 1.25, 1.0, 0.875, 0.85]);
    }

    #[test]
    fn each_alert_kind_has_its_own_colour() {
        let theme = PreviewTheme::light();
        let kinds = [
            AlertKind::Note,
            AlertKind::Tip,
            AlertKind::Important,
            AlertKind::Warning,
            AlertKind::Caution,
        ];
        let colours: Vec<Hsla> = kinds.iter().map(|kind| theme.alert_color(*kind)).collect();
        for (index, colour) in colours.iter().enumerate() {
            assert!(!colours[..index].contains(colour));
        }
    }

    #[test]
    fn light_and_dark_differ() {
        assert_ne!(PreviewTheme::light().text, PreviewTheme::dark().text);
        assert_ne!(
            PreviewTheme::light().background,
            PreviewTheme::dark().background
        );
    }

    fn colours(theme: &PreviewTheme) -> Vec<Hsla> {
        vec![
            theme.background,
            theme.text,
            theme.muted,
            theme.border,
            theme.solid_border,
            theme.code_background,
            theme.code_border,
            theme.inline_code_background,
            theme.table_header_background,
            theme.table_stripe_background,
        ]
    }

    #[test]
    fn every_colour_of_both_themes_is_set() {
        let unset = hsl(0.0, 0.0, 50.0, 1.0);
        for theme in [PreviewTheme::light(), PreviewTheme::dark()] {
            assert!(colours(&theme).iter().all(|colour| *colour != unset));
        }
    }

    #[test]
    fn dark_colours_differ_from_light_ones_field_by_field() {
        let light = colours(&PreviewTheme::light());
        let dark = colours(&PreviewTheme::dark());
        assert!(light.iter().zip(&dark).all(|(a, b)| a != b));
    }
}
