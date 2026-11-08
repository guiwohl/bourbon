use bourbon_markdown::{AlertKind, Alignment, BlockKind};
use std::ops::Range;

use gpui::{FontStyle, FontWeight, HighlightStyle, Pixels, StrikethroughStyle, TextAlign, px};

use crate::inline::{BOLD, CODE, FlatText, ITALIC, LINK, STRIKE};
use crate::preview_theme::PreviewTheme;

const MARKERS: [&str; 3] = ["\u{2022}", "\u{25E6}", "\u{25AA}"];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeadingStyle {
    pub size: Pixels,
    pub muted: bool,
    pub rule: Option<Pixels>,
    pub padding_bottom: Pixels,
}

pub fn heading_style(level: u8, theme: &PreviewTheme) -> HeadingStyle {
    let size = theme.heading_base_size * theme.heading_scale(level);
    let rule = match level {
        1 => Some(px(2.0)),
        2 => Some(px(1.0)),
        _ => None,
    };
    HeadingStyle {
        size,
        muted: level == 6,
        rule,
        padding_bottom: if rule.is_some() { size * 0.3 } else { px(0.0) },
    }
}

pub fn list_indent(theme: &PreviewTheme) -> Pixels {
    theme.font_size * theme.list_indent_em
}

pub fn item_gap(index: usize, theme: &PreviewTheme) -> Pixels {
    if index == 0 {
        px(0.0)
    } else {
        theme.font_size * 0.3
    }
}

pub fn child_gap(index: usize, tight: bool, theme: &PreviewTheme) -> Pixels {
    if index == 0 || tight {
        px(0.0)
    } else {
        theme.block_gap
    }
}

pub fn stack_gap(blocks: &[&BlockKind], index: usize, loose: bool, theme: &PreviewTheme) -> Pixels {
    match index.checked_sub(1) {
        Some(previous) if loose => {
            crate::preview::gap_between(blocks[previous], blocks[index], theme)
        }
        _ => px(0.0),
    }
}

pub fn list_marker(ordered: bool, number: u64, depth: usize) -> String {
    if ordered {
        format!("{number}.")
    } else {
        MARKERS[depth.min(MARKERS.len() - 1)].to_owned()
    }
}

pub fn checkbox_size(theme: &PreviewTheme) -> Pixels {
    theme.font_size * 0.875
}

pub fn checkbox_top_margin(theme: &PreviewTheme) -> Pixels {
    theme.font_size * 0.2
}

pub fn checkbox_tick_size(theme: &PreviewTheme) -> Pixels {
    checkbox_size(theme) * 0.8
}

pub fn quote_bar_width(theme: &PreviewTheme) -> Pixels {
    theme.font_size * theme.quote_bar_em
}

pub fn alert_vertical_padding(theme: &PreviewTheme) -> Pixels {
    theme.font_size * 0.5
}

pub fn alert_title(kind: AlertKind) -> &'static str {
    match kind {
        AlertKind::Note => "Note",
        AlertKind::Tip => "Tip",
        AlertKind::Important => "Important",
        AlertKind::Warning => "Warning",
        AlertKind::Caution => "Caution",
    }
}

pub fn rule_height(theme: &PreviewTheme) -> Pixels {
    theme.font_size * 0.25
}

pub fn footnote_size(theme: &PreviewTheme) -> Pixels {
    theme.font_size * 0.75
}

pub fn row_is_striped(index: usize) -> bool {
    index.is_multiple_of(2)
}

pub fn cell_alignment(alignment: Option<&Alignment>) -> TextAlign {
    match alignment {
        Some(Alignment::Right) => TextAlign::Right,
        Some(Alignment::Center) => TextAlign::Center,
        _ => TextAlign::Left,
    }
}

pub fn item_number(start: u64, index: usize) -> u64 {
    start + u64::try_from(index).unwrap_or(u64::MAX)
}

pub fn item_is_struck(task: Option<bool>) -> bool {
    task == Some(true)
}

pub fn highlight_ranges(
    flat: &FlatText,
    theme: &PreviewTheme,
) -> Vec<(Range<usize>, HighlightStyle)> {
    let mut offset = 0;
    let mut ranges = Vec::new();
    for (len, flags) in flat.flag_runs() {
        if flags != 0 {
            ranges.push((offset..offset + len, highlight_style(flags, theme)));
        }
        offset += len;
    }
    ranges
}

pub fn highlight_style(flags: u8, theme: &PreviewTheme) -> HighlightStyle {
    HighlightStyle {
        color: (flags & LINK != 0).then_some(theme.link),
        font_weight: (flags & BOLD != 0).then_some(FontWeight::SEMIBOLD),
        font_style: (flags & ITALIC != 0).then_some(FontStyle::Italic),
        background_color: (flags & CODE != 0).then_some(theme.inline_code_background),
        strikethrough: (flags & STRIKE != 0).then_some(StrikethroughStyle {
            thickness: px(1.0),
            color: None,
        }),
        underline: None,
        fade_out: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bourbon_markdown::PreviewDoc;

    fn theme() -> PreviewTheme {
        PreviewTheme {
            font_size: px(16.0),
            ..PreviewTheme::light()
        }
    }

    #[test]
    fn h1_and_h2_get_a_rule_and_padding_the_others_do_not() {
        let theme = theme();
        let h1 = heading_style(1, &theme);
        assert_eq!(h1.size, px(32.0));
        assert_eq!(h1.rule, Some(px(2.0)));
        assert_eq!(h1.padding_bottom, px(32.0) * 0.3);
        assert!(!h1.muted);
        let h2 = heading_style(2, &theme);
        assert_eq!(h2.size, px(24.0));
        assert_eq!(h2.rule, Some(px(1.0)));
        assert_eq!(h2.padding_bottom, px(24.0) * 0.3);
        let h3 = heading_style(3, &theme);
        assert_eq!(h3.size, px(20.0));
        assert_eq!(h3.rule, None);
        assert_eq!(h3.padding_bottom, px(0.0));
    }

    #[test]
    fn headings_scale_from_their_own_base_not_the_body_size() {
        let theme = PreviewTheme {
            font_size: px(15.0),
            heading_base_size: px(16.0),
            ..PreviewTheme::light()
        };
        assert_eq!(heading_style(2, &theme).size, px(24.0));
    }

    #[test]
    fn only_h6_is_muted() {
        let theme = theme();
        let muted: Vec<bool> = (1..=6)
            .map(|level| heading_style(level, &theme).muted)
            .collect();
        assert_eq!(muted, [false, false, false, false, false, true]);
        assert_eq!(heading_style(6, &theme).size, px(13.6));
    }

    #[test]
    fn lists_indent_two_em_and_space_items_three_tenths_of_an_em() {
        let theme = theme();
        assert_eq!(list_indent(&theme), px(32.0));
        assert_eq!(item_gap(0, &theme), px(0.0));
        assert_eq!(item_gap(1, &theme), px(4.8));
    }

    #[test]
    fn loose_lists_separate_item_children_and_tight_ones_do_not() {
        let theme = theme();
        assert_eq!(child_gap(0, false, &theme), px(0.0));
        assert_eq!(child_gap(1, false, &theme), px(16.0));
        assert_eq!(child_gap(1, true, &theme), px(0.0));
    }

    #[test]
    fn stacks_use_the_kind_of_the_previous_block() {
        let theme = theme();
        let doc = PreviewDoc::parse("a\n\n---\n\nb\n");
        let kinds: Vec<&BlockKind> = doc.blocks.iter().map(|block| &block.kind).collect();
        assert_eq!(stack_gap(&kinds, 0, true, &theme), px(0.0));
        assert_eq!(stack_gap(&kinds, 1, true, &theme), px(24.0));
        assert_eq!(stack_gap(&kinds, 2, true, &theme), px(24.0));
        assert_eq!(stack_gap(&kinds, 2, false, &theme), px(0.0));
        let plain = PreviewDoc::parse("a\n\nb\n\n# c\n");
        let kinds: Vec<&BlockKind> = plain.blocks.iter().map(|block| &block.kind).collect();
        assert_eq!(stack_gap(&kinds, 1, true, &theme), px(16.0));
        assert_eq!(stack_gap(&kinds, 2, true, &theme), px(24.0));
    }

    #[test]
    fn markers_depend_on_order_and_depth() {
        assert_eq!(list_marker(true, 7, 0), "7.");
        assert_eq!(list_marker(false, 1, 0), "\u{2022}");
        assert_eq!(list_marker(false, 1, 1), "\u{25E6}");
        assert_eq!(list_marker(false, 1, 2), "\u{25AA}");
        assert_eq!(list_marker(false, 1, 9), "\u{25AA}");
    }

    #[test]
    fn checkbox_metrics_scale_with_the_font() {
        let theme = theme();
        assert_eq!(checkbox_size(&theme), px(14.0));
        assert_eq!(checkbox_top_margin(&theme), px(3.2));
        assert_eq!(checkbox_tick_size(&theme), px(11.2));
    }

    #[test]
    fn quote_alert_rule_and_footnote_metrics() {
        let theme = theme();
        assert_eq!(quote_bar_width(&theme), px(4.0));
        assert_eq!(alert_vertical_padding(&theme), px(8.0));
        assert_eq!(rule_height(&theme), px(4.0));
        assert_eq!(footnote_size(&theme), px(12.0));
    }

    #[test]
    fn every_alert_kind_has_a_title() {
        let titles = [
            alert_title(AlertKind::Note),
            alert_title(AlertKind::Tip),
            alert_title(AlertKind::Important),
            alert_title(AlertKind::Warning),
            alert_title(AlertKind::Caution),
        ];
        assert_eq!(titles, ["Note", "Tip", "Important", "Warning", "Caution"]);
    }

    #[test]
    fn even_body_rows_are_striped() {
        assert!(row_is_striped(0));
        assert!(!row_is_striped(1));
        assert!(row_is_striped(2));
        assert!(!row_is_striped(3));
    }

    #[test]
    fn column_alignment_maps_to_text_alignment() {
        assert_eq!(cell_alignment(Some(&Alignment::Right)), TextAlign::Right);
        assert_eq!(cell_alignment(Some(&Alignment::Center)), TextAlign::Center);
        assert_eq!(cell_alignment(Some(&Alignment::Left)), TextAlign::Left);
        assert_eq!(cell_alignment(Some(&Alignment::None)), TextAlign::Left);
        assert_eq!(cell_alignment(None), TextAlign::Left);
    }

    #[test]
    fn items_are_numbered_from_the_start() {
        assert_eq!(item_number(1, 0), 1);
        assert_eq!(item_number(4, 3), 7);
    }

    #[test]
    fn only_checked_tasks_are_struck() {
        assert!(item_is_struck(Some(true)));
        assert!(!item_is_struck(Some(false)));
        assert!(!item_is_struck(None));
    }

    #[test]
    fn highlight_ranges_skip_plain_text_and_follow_offsets() {
        use bourbon_markdown::Inline;
        let theme = theme();
        let flat = crate::inline::flatten_inlines(&[
            Inline::Text("ab".to_owned()),
            Inline::Strong(vec![Inline::Text("cde".to_owned())]),
            Inline::Text("f".to_owned()),
            Inline::Code("gh".to_owned()),
        ]);
        let ranges = highlight_ranges(&flat, &theme);
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0].0, 2..5);
        assert_eq!(ranges[0].1, highlight_style(BOLD, &theme));
        assert_eq!(ranges[1].0, 6..8);
        assert_eq!(ranges[1].1, highlight_style(CODE, &theme));
    }

    #[test]
    fn each_flag_sets_only_its_own_style() {
        let theme = theme();
        let none = highlight_style(0, &theme);
        assert_eq!(none, HighlightStyle::default());
        let link = highlight_style(LINK, &theme);
        assert_eq!(link.color, Some(theme.link));
        assert_eq!(link.font_weight, None);
        let bold = highlight_style(BOLD, &theme);
        assert_eq!(bold.font_weight, Some(FontWeight::SEMIBOLD));
        assert_eq!(bold.color, None);
        let italic = highlight_style(ITALIC, &theme);
        assert_eq!(italic.font_style, Some(FontStyle::Italic));
        assert_eq!(italic.font_weight, None);
        let code = highlight_style(CODE, &theme);
        assert_eq!(code.background_color, Some(theme.inline_code_background));
        assert_eq!(code.font_style, None);
        let strike = highlight_style(STRIKE, &theme);
        assert!(strike.strikethrough.is_some());
        assert_eq!(strike.background_color, None);
        let all = highlight_style(LINK | BOLD | ITALIC | CODE | STRIKE, &theme);
        assert!(all.color.is_some() && all.font_weight.is_some() && all.font_style.is_some());
        assert!(all.background_color.is_some() && all.strikethrough.is_some());
    }
}
