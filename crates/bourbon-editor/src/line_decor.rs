use bourbon_markdown::{DecorationKind, LineDecoration};
use gpui::{Bounds, Corners, Edges, Pixels, point, px, size};

use crate::layout::LineBox;
use crate::theme::EditorTheme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardEdges {
    pub first: bool,
    pub last: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LineDecor {
    pub card: Option<CardEdges>,
    pub quote: bool,
}

pub fn decor_of(line: usize, decorations: &[LineDecoration]) -> LineDecor {
    let mut decor = LineDecor::default();
    for decoration in decorations
        .iter()
        .filter(|decoration| decoration.lines.contains(&line))
    {
        match decoration.kind {
            DecorationKind::Card => {
                decor.card = Some(CardEdges {
                    first: line == *decoration.lines.start(),
                    last: line == *decoration.lines.end(),
                });
            }
            DecorationKind::QuoteBar => decor.quote = true,
        }
    }
    decor
}

pub fn line_box(decor: LineDecor, theme: &EditorTheme) -> LineBox {
    let Some(card) = decor.card else {
        return LineBox::default();
    };
    let padding = |edge: bool| if edge { theme.card_padding_y } else { px(0.0) };
    LineBox {
        inset: theme.block_padding - theme.line_padding,
        padding_top: padding(card.first),
        padding_bottom: padding(card.last),
    }
}

pub struct CardShape {
    pub bounds: Bounds<Pixels>,
    pub borders: Edges<Pixels>,
    pub radii: Corners<Pixels>,
}

pub fn card_shape(line_box: Bounds<Pixels>, card: CardEdges, theme: &EditorTheme) -> CardShape {
    let inset = theme.card_inset;
    let radius = theme.font_size * theme.card_radius_em;
    let edge = |on: bool, value: Pixels| if on { value } else { px(0.0) };
    CardShape {
        bounds: Bounds::new(
            point(line_box.left() + inset, line_box.top()),
            size(line_box.size.width - inset * 2.0, line_box.size.height),
        ),
        borders: Edges {
            top: edge(card.first, px(1.0)),
            right: px(1.0),
            bottom: edge(card.last, px(1.0)),
            left: px(1.0),
        },
        radii: Corners {
            top_left: edge(card.first, radius),
            top_right: edge(card.first, radius),
            bottom_right: edge(card.last, radius),
            bottom_left: edge(card.last, radius),
        },
    }
}

pub fn quote_bar(line_box: Bounds<Pixels>, theme: &EditorTheme) -> Bounds<Pixels> {
    Bounds::new(
        line_box.origin,
        size(theme.font_size * theme.quote_bar_em, line_box.size.height),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use bourbon_markdown::line_decorations;

    fn decors(source: &str) -> Vec<LineDecor> {
        let decorations = line_decorations(source);
        (0..source.lines().count())
            .map(|line| decor_of(line, &decorations))
            .collect()
    }

    fn card(first: bool, last: bool) -> LineDecor {
        LineDecor {
            card: Some(CardEdges { first, last }),
            quote: false,
        }
    }

    #[test]
    fn fence_lines_know_their_place_in_the_card() {
        assert_eq!(
            decors("a\n```\nx\n```\nb\n"),
            vec![
                LineDecor::default(),
                card(true, false),
                card(false, false),
                card(false, true),
                LineDecor::default(),
            ]
        );
    }

    #[test]
    fn a_one_line_table_card_is_both_first_and_last() {
        let decorations = vec![LineDecoration {
            lines: 3..=3,
            kind: DecorationKind::Card,
        }];
        assert_eq!(decor_of(3, &decorations), card(true, true));
    }

    #[test]
    fn quoted_lines_get_the_bar() {
        let quoted = decors("> a\n> b\n\nc\n");
        assert!(quoted[0].quote && quoted[1].quote);
        assert!(!quoted[3].quote);
        assert_eq!(quoted[0].card, None);
    }

    #[test]
    fn card_lines_are_padded_at_the_ends_and_inset_to_the_block_padding() {
        let theme = EditorTheme::light();
        assert_eq!(line_box(LineDecor::default(), &theme), LineBox::default());
        let first = line_box(card(true, false), &theme);
        assert_eq!(first.inset, px(2.0));
        assert_eq!(first.padding_top, px(4.0));
        assert_eq!(first.padding_bottom, px(0.0));
        let last = line_box(card(false, true), &theme);
        assert_eq!(last.padding_top, px(0.0));
        assert_eq!(last.padding_bottom, px(4.0));
    }

    fn line_bounds() -> Bounds<Pixels> {
        Bounds::new(point(px(47.0), px(100.0)), size(px(371.0), px(20.0)))
    }

    #[test]
    fn the_card_is_inset_and_only_its_ends_are_closed_and_rounded() {
        let theme = EditorTheme::light();
        let middle = card_shape(
            line_bounds(),
            CardEdges {
                first: false,
                last: false,
            },
            &theme,
        );
        assert_eq!(middle.bounds.origin, point(px(50.0), px(100.0)));
        assert_eq!(middle.bounds.size, size(px(365.0), px(20.0)));
        assert_eq!(middle.borders.top, px(0.0));
        assert_eq!(middle.borders.bottom, px(0.0));
        assert_eq!(middle.borders.left, px(1.0));
        assert_eq!(middle.borders.right, px(1.0));
        assert_eq!(middle.radii.top_left, px(0.0));
        let single = card_shape(
            line_bounds(),
            CardEdges {
                first: true,
                last: true,
            },
            &theme,
        );
        assert_eq!(single.borders.top, px(1.0));
        assert_eq!(single.borders.bottom, px(1.0));
        assert_eq!(single.radii.top_left, px(14.5 * 0.4));
        assert_eq!(single.radii.top_right, px(14.5 * 0.4));
        assert_eq!(single.radii.bottom_left, px(14.5 * 0.4));
        assert_eq!(single.radii.bottom_right, px(14.5 * 0.4));
    }

    #[test]
    fn the_quote_bar_hugs_the_left_of_the_line_box() {
        let theme = EditorTheme::light();
        let bar = quote_bar(line_bounds(), &theme);
        assert_eq!(bar.origin, point(px(47.0), px(100.0)));
        assert_eq!(bar.size, size(px(14.5 * 0.3), px(20.0)));
    }
}
