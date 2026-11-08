use bourbon_markdown::TokenClass;
use gpui::{Hsla, Pixels, hsla, px};

const HEADING_SCALES: [f32; 6] = [2.0, 1.5, 1.17, 1.0, 0.83, 0.67];

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct TokenStyle {
    pub color: Hsla,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
}

impl TokenStyle {
    fn plain(color: Hsla) -> Self {
        Self {
            color,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
        }
    }

    fn bold(color: Hsla) -> Self {
        Self {
            bold: true,
            ..Self::plain(color)
        }
    }

    fn italic(color: Hsla) -> Self {
        Self {
            italic: true,
            ..Self::plain(color)
        }
    }

    fn underlined(color: Hsla) -> Self {
        Self {
            underline: true,
            ..Self::plain(color)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EditorTheme {
    pub font_family: String,
    pub font_fallbacks: Vec<String>,
    pub font_size: Pixels,
    pub line_height: f32,
    pub heading_scales: [f32; 6],
    pub line_padding: Pixels,
    pub block_padding: Pixels,
    pub card_inset: Pixels,
    pub card_padding_y: Pixels,
    pub card_radius_em: f32,
    pub quote_bar_em: f32,
    pub caret_width: Pixels,
    pub gutter_font_scale: f32,
    pub gutter_min_width_em: f32,
    pub gutter_padding_em: f32,
    pub foreground: Hsla,
    pub background: Hsla,
    pub caret: Hsla,
    pub selection: Hsla,
    pub selection_unfocused: Hsla,
    pub active_line: Hsla,
    pub gutter_text: Hsla,
    pub gutter_border: Hsla,
    pub heading: Hsla,
    pub mark: Hsla,
    pub list_mark: Hsla,
    pub quote: Hsla,
    pub link: Hsla,
    pub comment: Hsla,
    pub code_background: Hsla,
    pub code_border: Hsla,
    pub quote_bar: Hsla,
}

fn hsl(hue: f32, saturation: f32, lightness: f32) -> Hsla {
    hsl_alpha(hue, saturation, lightness, 1.0)
}

fn hsl_alpha(hue: f32, saturation: f32, lightness: f32, alpha: f32) -> Hsla {
    hsla(hue / 360.0, saturation / 100.0, lightness / 100.0, alpha)
}

impl EditorTheme {
    pub fn light() -> Self {
        Self {
            foreground: hsl(215.0, 19.0, 35.0),
            background: hsl(0.0, 0.0, 100.0),
            caret: hsl(262.0, 83.0, 58.0),
            selection: hsl_alpha(262.0, 83.0, 58.0, 0.30),
            selection_unfocused: hsl_alpha(262.0, 83.0, 58.0, 0.13),
            active_line: hsl_alpha(212.0, 96.0, 78.0, 0.20),
            gutter_text: hsl(213.0, 27.0, 84.0),
            gutter_border: hsl(214.0, 32.0, 91.0),
            heading: hsl(333.0, 71.0, 51.0),
            mark: hsl(221.0, 83.0, 53.0),
            list_mark: hsl(239.0, 84.0, 67.0),
            quote: hsl(215.0, 19.0, 35.0),
            link: hsl(175.0, 84.0, 32.0),
            comment: hsl(215.0, 20.0, 65.0),
            code_background: hsl_alpha(214.0, 32.0, 91.0, 0.30),
            code_border: hsl_alpha(213.0, 27.0, 84.0, 0.40),
            quote_bar: hsl(215.0, 20.0, 65.0),
            ..Self::base()
        }
    }

    pub fn dark() -> Self {
        Self {
            foreground: hsl(216.0, 12.0, 84.0),
            background: hsl(0.0, 0.0, 4.0),
            caret: hsl(255.0, 92.0, 76.0),
            selection: hsl_alpha(255.0, 92.0, 76.0, 0.30),
            selection_unfocused: hsl_alpha(255.0, 92.0, 76.0, 0.13),
            active_line: hsl_alpha(215.0, 14.0, 34.0, 0.20),
            gutter_text: hsl(220.0, 9.0, 46.0),
            gutter_border: hsl(0.0, 0.0, 15.0),
            heading: hsl(329.0, 86.0, 70.0),
            mark: hsl(213.0, 94.0, 68.0),
            list_mark: hsl(239.0, 84.0, 67.0),
            quote: hsl(218.0, 11.0, 65.0),
            link: hsl(173.0, 80.0, 40.0),
            comment: hsl(218.0, 11.0, 65.0),
            code_background: hsl_alpha(0.0, 0.0, 25.0, 0.20),
            code_border: hsl_alpha(213.0, 97.0, 87.0, 0.10),
            quote_bar: hsl(0.0, 0.0, 45.0),
            ..Self::base()
        }
    }

    fn base() -> Self {
        let placeholder = hsl(0.0, 0.0, 50.0);
        Self {
            font_family: "JetBrains Mono".to_owned(),
            font_fallbacks: ["DejaVu Sans Mono", "Liberation Mono", "monospace"]
                .map(str::to_owned)
                .to_vec(),
            font_size: px(14.5),
            line_height: 1.16,
            heading_scales: HEADING_SCALES,
            line_padding: px(8.0),
            block_padding: px(10.0),
            card_inset: px(3.0),
            card_padding_y: px(4.0),
            card_radius_em: 0.4,
            quote_bar_em: 0.3,
            caret_width: px(2.0),
            gutter_font_scale: 0.7,
            gutter_min_width_em: 4.5,
            gutter_padding_em: 2.6,
            foreground: placeholder,
            background: placeholder,
            caret: placeholder,
            selection: placeholder,
            selection_unfocused: placeholder,
            active_line: placeholder,
            gutter_text: placeholder,
            gutter_border: placeholder,
            heading: placeholder,
            mark: placeholder,
            list_mark: placeholder,
            quote: placeholder,
            link: placeholder,
            comment: placeholder,
            code_background: placeholder,
            code_border: placeholder,
            quote_bar: placeholder,
        }
    }

    pub fn heading_scale(&self, level: u8) -> f32 {
        let index = usize::from(level.clamp(1, 6)) - 1;
        self.heading_scales[index]
    }

    pub fn token_style(&self, class: Option<TokenClass>) -> TokenStyle {
        let Some(class) = class else {
            return TokenStyle::plain(self.foreground);
        };
        match class {
            TokenClass::Heading(_) | TokenClass::HorizontalRule => TokenStyle::bold(self.heading),
            TokenClass::HeaderMark | TokenClass::TaskMarker { .. } => TokenStyle::bold(self.mark),
            TokenClass::EmphasisMark
            | TokenClass::StrongMark
            | TokenClass::StrikethroughMark
            | TokenClass::CodeMark
            | TokenClass::LinkMark
            | TokenClass::QuoteMark
            | TokenClass::TableDelimiter
            | TokenClass::AlertMarker
            | TokenClass::MathMark
            | TokenClass::HtmlTag => TokenStyle::plain(self.mark),
            TokenClass::ListMark => TokenStyle::plain(self.list_mark),
            TokenClass::Emphasis => TokenStyle::italic(self.foreground),
            TokenClass::Strong | TokenClass::TableHeader => TokenStyle::bold(self.foreground),
            TokenClass::Strikethrough => TokenStyle {
                strikethrough: true,
                ..TokenStyle::plain(self.foreground)
            },
            TokenClass::CodeBlockInfo | TokenClass::Frontmatter => TokenStyle::italic(self.comment),
            TokenClass::LinkText | TokenClass::NoteLink => TokenStyle::underlined(self.link),
            TokenClass::Url | TokenClass::Image => TokenStyle::plain(self.link),
            TokenClass::QuoteText => TokenStyle::italic(self.quote),
            TokenClass::CodeBlockText | TokenClass::InlineCode | TokenClass::Math => {
                TokenStyle::plain(self.foreground)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_follow_the_measured_reference() {
        let theme = EditorTheme::light();
        assert_eq!(theme.font_size, px(14.5));
        assert_eq!(theme.line_height, 1.16);
        assert_eq!(theme.caret_width, px(2.0));
        assert_eq!(theme.line_padding, px(8.0));
        assert_eq!(theme.block_padding, px(10.0));
        assert_eq!(theme.card_inset, px(3.0));
        assert_eq!(theme.card_padding_y, px(4.0));
        assert_ne!(theme.font_fallbacks, [] as [std::string::String; 0]);
    }

    #[test]
    fn heading_scales_follow_the_research() {
        let theme = EditorTheme::light();
        let scales: Vec<f32> = (1..=6).map(|level| theme.heading_scale(level)).collect();
        assert_eq!(scales, [2.0, 1.5, 1.17, 1.0, 0.83, 0.67]);
    }

    #[test]
    fn heading_scale_clamps_unknown_levels() {
        let theme = EditorTheme::light();
        assert_eq!(theme.heading_scale(0), 2.0);
        assert_eq!(theme.heading_scale(9), 0.67);
    }

    #[test]
    fn headings_are_bold_and_pink_and_marks_are_blue() {
        let theme = EditorTheme::light();
        let heading = theme.token_style(Some(TokenClass::Heading(2)));
        assert!(heading.bold);
        assert_eq!(heading.color, theme.heading);
        assert_eq!(
            theme.token_style(Some(TokenClass::CodeMark)).color,
            theme.mark
        );
        assert_eq!(
            theme.token_style(Some(TokenClass::LinkMark)).color,
            theme.mark
        );
    }

    #[test]
    fn emphasis_strong_and_strikethrough_carry_their_decorations() {
        let theme = EditorTheme::light();
        assert!(theme.token_style(Some(TokenClass::Emphasis)).italic);
        assert!(theme.token_style(Some(TokenClass::Strong)).bold);
        assert!(
            theme
                .token_style(Some(TokenClass::Strikethrough))
                .strikethrough
        );
        assert!(theme.token_style(Some(TokenClass::LinkText)).underline);
    }

    #[test]
    fn code_and_math_text_keep_the_foreground_and_their_marks_are_blue() {
        let theme = EditorTheme::light();
        for class in [
            TokenClass::InlineCode,
            TokenClass::Math,
            TokenClass::CodeBlockText,
        ] {
            let style = theme.token_style(Some(class));
            assert_eq!(style, TokenStyle::plain(theme.foreground));
        }
        assert_eq!(
            theme.token_style(Some(TokenClass::MathMark)).color,
            theme.mark
        );
        assert_eq!(theme.token_style(None).color, theme.foreground);
    }

    #[test]
    fn light_and_dark_differ() {
        let light = EditorTheme::light();
        let dark = EditorTheme::dark();
        assert_ne!(light.background, dark.background);
        assert_ne!(light.foreground, dark.foreground);
        assert_ne!(light.heading, dark.heading);
    }

    #[test]
    fn selection_is_stronger_when_focused() {
        let theme = EditorTheme::dark();
        assert!(theme.selection.a > theme.selection_unfocused.a);
    }

    fn colours(theme: &EditorTheme) -> Vec<Hsla> {
        vec![
            theme.foreground,
            theme.background,
            theme.caret,
            theme.selection,
            theme.selection_unfocused,
            theme.active_line,
            theme.gutter_text,
            theme.gutter_border,
            theme.heading,
            theme.mark,
            theme.list_mark,
            theme.quote,
            theme.link,
            theme.comment,
            theme.code_background,
            theme.code_border,
            theme.quote_bar,
        ]
    }

    #[test]
    fn every_colour_of_both_themes_is_set() {
        let unset = hsla(0.0, 0.0, 0.5, 1.0);
        for theme in [EditorTheme::light(), EditorTheme::dark()] {
            assert!(colours(&theme).iter().all(|colour| *colour != unset));
        }
    }

    #[test]
    fn colours_are_converted_from_hsl_degrees_and_percent() {
        let light = EditorTheme::light();
        assert_eq!(light.foreground, hsla(215.0 / 360.0, 0.19, 0.35, 1.0));
        assert_eq!(light.selection, hsla(262.0 / 360.0, 0.83, 0.58, 0.30));
    }

    #[test]
    fn dark_colours_differ_from_light_ones_field_by_field() {
        let light = colours(&EditorTheme::light());
        let dark = colours(&EditorTheme::dark());
        let same = light.iter().zip(&dark).filter(|(a, b)| a == b).count();
        assert!(same <= 1);
    }
}
