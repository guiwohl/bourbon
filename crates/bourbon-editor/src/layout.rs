use std::ops::Range;

use gpui::{
    App, Bounds, Font, FontFallbacks, FontStyle, FontWeight, Pixels, Point, ShapedLine,
    StrikethroughStyle, TextAlign, TextRun, UnderlineStyle, Window, WrappedLine, point, px, size,
};

use crate::numeric::{count_f32, row_from_ratio};
use crate::segments::{Segment, split_segments};
use crate::theme::EditorTheme;

pub struct ShapeRequest<'a> {
    pub text: &'a str,
    pub segments: &'a [Segment],
    pub prefix_len: usize,
    pub font_size: Pixels,
    pub line_height: Pixels,
    pub wrap_width: Option<Pixels>,
    pub bold: bool,
    pub theme: &'a EditorTheme,
}

pub struct LineLayout {
    prefix: ShapedLine,
    prefix_len: usize,
    indent: Pixels,
    body: WrappedLine,
    line_height: Pixels,
    len: usize,
}

pub fn editor_font(theme: &EditorTheme, bold: bool, italic: bool) -> Font {
    Font {
        family: theme.font_family.clone().into(),
        features: gpui::FontFeatures::default(),
        fallbacks: Some(FontFallbacks::from_fonts(theme.font_fallbacks.clone())),
        weight: if bold {
            FontWeight::BOLD
        } else {
            FontWeight::NORMAL
        },
        style: if italic {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        },
    }
}

pub fn text_runs(segments: &[Segment], theme: &EditorTheme, force_bold: bool) -> Vec<TextRun> {
    segments
        .iter()
        .map(|segment| {
            let style = theme.token_style(segment.class);
            let underline = (style.underline || segment.marked).then_some(UnderlineStyle {
                thickness: px(1.0),
                color: Some(style.color),
                wavy: false,
            });
            let strikethrough = style.strikethrough.then_some(StrikethroughStyle {
                thickness: px(1.0),
                color: Some(style.color),
            });
            TextRun {
                len: segment.len,
                font: editor_font(theme, style.bold || force_bold, style.italic),
                color: style.color,
                background_color: style.background,
                underline,
                strikethrough,
            }
        })
        .collect()
}

impl LineLayout {
    pub fn shape(request: &ShapeRequest, window: &mut Window) -> Self {
        let (prefix_segments, body_segments) = split_segments(request.segments, request.prefix_len);
        let (prefix_text, body_text) = request.text.split_at(request.prefix_len);
        let prefix = window.text_system().shape_line(
            prefix_text.to_owned().into(),
            request.font_size,
            &text_runs(&prefix_segments, request.theme, request.bold),
            None,
        );
        let indent = prefix.width;
        let body_wrap = request
            .wrap_width
            .map(|width| (width - indent).max(request.font_size));
        let body = window
            .text_system()
            .shape_text(
                body_text.to_owned().into(),
                request.font_size,
                &text_runs(&body_segments, request.theme, request.bold),
                body_wrap,
                None,
            )
            .unwrap_or_default()
            .into_iter()
            .next()
            .unwrap_or_default();
        Self {
            prefix,
            prefix_len: request.prefix_len,
            indent,
            body,
            line_height: request.line_height,
            len: request.text.len(),
        }
    }

    pub fn line_height(&self) -> Pixels {
        self.line_height
    }

    pub fn row_count(&self) -> usize {
        self.body.wrap_boundaries().len() + 1
    }

    pub fn height(&self) -> Pixels {
        self.line_height * count_f32(self.row_count())
    }

    pub fn rows(&self) -> Vec<Range<usize>> {
        let runs = self.body.runs();
        let starts = std::iter::once(0).chain(
            self.body
                .wrap_boundaries()
                .iter()
                .map(|boundary| runs[boundary.run_ix].glyphs[boundary.glyph_ix].index),
        );
        let ends = self
            .body
            .wrap_boundaries()
            .iter()
            .map(|boundary| runs[boundary.run_ix].glyphs[boundary.glyph_ix].index)
            .chain(std::iter::once(self.body.len()));
        starts
            .zip(ends)
            .enumerate()
            .map(|(row, (start, end))| {
                let first = if row == 0 { 0 } else { self.prefix_len + start };
                first..self.prefix_len + end
            })
            .collect()
    }

    fn row_of(index: usize, rows: &[Range<usize>]) -> usize {
        rows.iter().rposition(|row| row.start <= index).unwrap_or(0)
    }

    fn x_in_row(&self, row: usize, index: usize, rows: &[Range<usize>]) -> Pixels {
        let index = index.min(self.len);
        if row > 0 && index == rows[row].start {
            return self.indent;
        }
        self.prefix.x_for_index(index.min(self.prefix_len))
            + self
                .body
                .position_for_index(index.saturating_sub(self.prefix_len), self.line_height)
                .map_or(px(0.0), |position| position.x)
    }

    pub fn position_for_index(&self, index: usize) -> Point<Pixels> {
        let rows = self.rows();
        let row = Self::row_of(index.min(self.len), &rows);
        point(
            self.x_in_row(row, index, &rows),
            self.line_height * count_f32(row),
        )
    }

    pub fn index_for_position(&self, position: Point<Pixels>) -> usize {
        let rows = self.rows();
        let row = row_from_ratio(position.y / self.line_height).min(rows.len() - 1);
        let prefix_index = if row == 0 {
            self.prefix
                .closest_index_for_x(position.x.min(self.indent))
                .min(self.prefix_len)
        } else {
            self.prefix_len
        };
        let body_point = point(
            (position.x - self.indent).max(px(0.0)),
            self.line_height * count_f32(row),
        );
        let body_index = self
            .body
            .closest_index_for_position(body_point, self.line_height)
            .unwrap_or_else(|edge| edge);
        prefix_index + body_index
    }

    pub fn glyph_index_at(&self, position: Point<Pixels>) -> Option<usize> {
        let row_count = self.row_count();
        let row = row_from_ratio(position.y / self.line_height).min(row_count - 1);
        let prefix_hit = (row == 0)
            .then(|| self.prefix.index_for_x(position.x))
            .flatten();
        prefix_hit.or_else(|| {
            let body_point = point(position.x - self.indent, self.line_height * count_f32(row));
            self.body
                .index_for_position(body_point, self.line_height)
                .ok()
                .map(|index| self.prefix_len + index)
        })
    }

    pub fn selection_rects(
        &self,
        selected: &Range<usize>,
        through_line_break: bool,
        break_width: Pixels,
    ) -> Vec<Bounds<Pixels>> {
        let rows = self.rows();
        let last = rows.len() - 1;
        let mut rects = Vec::new();
        for (row, range) in rows.iter().enumerate() {
            if selected.start > range.end || selected.end < range.start {
                continue;
            }
            let start = selected.start.clamp(range.start, range.end);
            let end = selected.end.clamp(range.start, range.end);
            let left = if start == range.start {
                if row == 0 { px(0.0) } else { self.indent }
            } else {
                self.x_in_row(row, start, &rows)
            };
            let mut right = self.x_in_row(row, end, &rows);
            if through_line_break && row == last {
                right += break_width;
            }
            if right > left {
                rects.push(Bounds::new(
                    point(left, self.line_height * count_f32(row)),
                    size(right - left, self.line_height),
                ));
            }
        }
        rects
    }

    pub fn paint(&self, origin: Point<Pixels>, window: &mut Window, cx: &mut App) {
        self.prefix
            .paint_background(origin, self.line_height, TextAlign::Left, None, window, cx)
            .ok();
        self.prefix
            .paint(origin, self.line_height, TextAlign::Left, None, window, cx)
            .ok();
        let body_origin = point(origin.x + self.indent, origin.y);
        self.body
            .paint_background(
                body_origin,
                self.line_height,
                TextAlign::Left,
                None,
                window,
                cx,
            )
            .ok();
        self.body
            .paint(
                body_origin,
                self.line_height,
                TextAlign::Left,
                None,
                window,
                cx,
            )
            .ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    const CHAR: f32 = 7.8;
    const LINE: f32 = 19.5;

    fn shape(cx: &mut TestAppContext, text: &str, wrap_chars: Option<f32>) -> LineLayout {
        let theme = EditorTheme::light();
        let segments = [Segment {
            len: text.len(),
            class: None,
            marked: false,
        }];
        let prefix_len = crate::segments::hanging_prefix_len(text);
        let window = cx.add_empty_window();
        window.update(|window, _| {
            LineLayout::shape(
                &ShapeRequest {
                    text,
                    segments: if text.is_empty() { &[] } else { &segments },
                    prefix_len,
                    font_size: px(13.0),
                    line_height: px(LINE),
                    wrap_width: wrap_chars.map(|chars| px(chars * CHAR + 0.5)),
                    bold: false,
                    theme: &theme,
                },
                window,
            )
        })
    }

    fn close(actual: Pixels, expected: f32) -> bool {
        (f32::from(actual) - expected).abs() < 0.05
    }

    #[gpui::test]
    fn an_unwrapped_line_is_one_row(cx: &mut TestAppContext) {
        let layout = shape(cx, "hello", None);
        assert_eq!(layout.row_count(), 1);
        assert_eq!(layout.rows(), vec![0..5]);
        assert!(close(layout.height(), LINE));
    }

    #[gpui::test]
    fn an_empty_line_has_one_empty_row(cx: &mut TestAppContext) {
        let layout = shape(cx, "", None);
        assert_eq!(layout.rows(), vec![0..0]);
        assert!(close(layout.height(), LINE));
    }

    #[gpui::test]
    fn wrapping_splits_rows_at_word_boundaries(cx: &mut TestAppContext) {
        let layout = shape(cx, "aaaa bbbb cccc dddd", Some(10.0));
        let rows = layout.rows();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].start, 0);
        assert_eq!(rows[0].end, rows[1].start);
        assert_eq!(rows[1].end, 19);
        assert!(close(layout.height(), 2.0 * LINE));
    }

    #[gpui::test]
    fn positions_follow_the_character_advance(cx: &mut TestAppContext) {
        let layout = shape(cx, "hello", None);
        let position = layout.position_for_index(3);
        assert!(close(position.x, 3.0 * CHAR));
        assert!(close(position.y, 0.0));
    }

    #[gpui::test]
    fn wrapped_rows_stack_vertically_and_restart_at_the_left(cx: &mut TestAppContext) {
        let layout = shape(cx, "aaaa bbbb cccc dddd", Some(10.0));
        let second_start = layout.rows()[1].start;
        let at_start = layout.position_for_index(second_start);
        assert!(close(at_start.x, 0.0));
        assert!(close(at_start.y, LINE));
        let inside = layout.position_for_index(second_start + 2);
        assert!(close(inside.x, 2.0 * CHAR));
        assert!(close(inside.y, LINE));
    }

    #[gpui::test]
    fn the_first_row_end_stays_on_the_first_row_for_selection_math(cx: &mut TestAppContext) {
        let layout = shape(cx, "aaaa bbbb cccc dddd", Some(10.0));
        let end = layout.rows()[0].end;
        let rects = layout.selection_rects(&(0..end), false, px(6.0));
        assert_eq!(rects.len(), 1);
        assert!(close(
            rects[0].size.width,
            crate::numeric::count_f32(end) * CHAR
        ));
    }

    #[gpui::test]
    fn list_lines_hang_their_continuation_rows_under_the_text(cx: &mut TestAppContext) {
        let layout = shape(cx, "- aaaa bbbb cccc dddd", Some(14.0));
        let rows = layout.rows();
        assert!(rows.len() >= 2);
        assert_eq!(rows[0].start, 0);
        assert_eq!(rows[0].end, 12);
        assert!(close(layout.position_for_index(0).x, 0.0));
        let continuation = layout.position_for_index(rows[1].start);
        assert!(close(continuation.x, 2.0 * CHAR));
        assert!(close(continuation.y, LINE));
        let after_marker = layout.position_for_index(2);
        assert!(close(after_marker.x, 2.0 * CHAR));
        assert!(close(layout.position_for_index(1).x, CHAR));
    }

    #[gpui::test]
    fn every_index_maps_back_from_its_own_position(cx: &mut TestAppContext) {
        for text in [
            "hello world",
            "- aaaa bbbb cccc dddd eeee",
            "aaaa bbbb cccc dddd",
        ] {
            let layout = shape(cx, text, Some(12.0));
            let skipped = [
                text.len() - 1,
                crate::segments::hanging_prefix_len(text).max(1) - 1,
            ];
            for index in (0..text.len()).filter(|index| !skipped.contains(index)) {
                let position = layout.position_for_index(index);
                let probe = point(position.x + px(1.0), position.y + px(5.0));
                assert_eq!(
                    layout.index_for_position(probe),
                    index,
                    "{text:?} at {index}"
                );
            }
        }
    }

    #[gpui::test]
    fn points_left_of_the_first_row_text_pick_prefix_indices(cx: &mut TestAppContext) {
        let layout = shape(cx, "- item", None);
        assert_eq!(layout.index_for_position(point(px(1.0), px(5.0))), 0);
        assert_eq!(
            layout.index_for_position(point(px(2.0 * CHAR + 1.0), px(5.0))),
            2
        );
    }

    #[gpui::test]
    fn points_beyond_the_line_clamp_to_its_ends(cx: &mut TestAppContext) {
        let layout = shape(cx, "hello", None);
        assert_eq!(layout.index_for_position(point(px(-20.0), px(5.0))), 0);
        assert_eq!(layout.index_for_position(point(px(500.0), px(5.0))), 5);
        assert_eq!(layout.index_for_position(point(px(500.0), px(500.0))), 5);
        assert_eq!(layout.index_for_position(point(px(1.0), px(-30.0))), 0);
    }

    #[gpui::test]
    fn glyph_lookup_finds_the_character_under_the_point(cx: &mut TestAppContext) {
        let layout = shape(cx, "- [ ] task", None);
        assert_eq!(
            layout.glyph_index_at(point(px(3.5 * CHAR), px(5.0))),
            Some(3)
        );
        assert_eq!(
            layout.glyph_index_at(point(px(0.5 * CHAR), px(5.0))),
            Some(0)
        );
        assert_eq!(
            layout.glyph_index_at(point(px(7.5 * CHAR), px(5.0))),
            Some(7)
        );
        assert_eq!(layout.glyph_index_at(point(px(500.0), px(5.0))), None);
        assert_eq!(
            layout.glyph_index_at(point(px(3.5 * CHAR), px(500.0))),
            Some(3)
        );
    }

    #[gpui::test]
    fn glyph_lookup_on_continuation_rows_uses_the_hanging_indent(cx: &mut TestAppContext) {
        let layout = shape(cx, "- aaaa bbbb cccc dddd", Some(14.0));
        let start = layout.rows()[1].start;
        let found = layout.glyph_index_at(point(px(2.0 * CHAR + 1.0), px(LINE + 5.0)));
        assert_eq!(found, Some(start));
    }

    #[gpui::test]
    fn a_selection_inside_a_row_is_one_rectangle(cx: &mut TestAppContext) {
        let layout = shape(cx, "hello world", None);
        let rects = layout.selection_rects(&(2..7), false, px(6.0));
        assert_eq!(rects.len(), 1);
        assert!(close(rects[0].origin.x, 2.0 * CHAR));
        assert!(close(rects[0].size.width, 5.0 * CHAR));
        assert!(close(rects[0].origin.y, 0.0));
        assert!(close(rects[0].size.height, LINE));
    }

    #[gpui::test]
    fn a_selection_through_the_line_break_is_widened(cx: &mut TestAppContext) {
        let layout = shape(cx, "hello", None);
        let rects = layout.selection_rects(&(2..5), true, px(6.0));
        assert!(close(rects[0].size.width, 3.0 * CHAR + 6.0));
        let plain = layout.selection_rects(&(2..5), false, px(6.0));
        assert!(close(plain[0].size.width, 3.0 * CHAR));
    }

    #[gpui::test]
    fn an_empty_selection_has_no_rectangle(cx: &mut TestAppContext) {
        let layout = shape(cx, "hello", None);
        assert_eq!(layout.selection_rects(&(2..2), false, px(6.0)), vec![]);
    }

    #[gpui::test]
    fn a_selection_covering_a_blank_line_shows_the_break(cx: &mut TestAppContext) {
        let layout = shape(cx, "", None);
        let rects = layout.selection_rects(&(0..0), true, px(6.0));
        assert_eq!(rects.len(), 1);
        assert!(close(rects[0].size.width, 6.0));
    }

    #[gpui::test]
    fn a_selection_spanning_rows_gets_a_rectangle_per_row(cx: &mut TestAppContext) {
        let layout = shape(cx, "aaaa bbbb cccc dddd", Some(10.0));
        let rows = layout.rows();
        let rects = layout.selection_rects(&(2..15), false, px(6.0));
        assert_eq!(rects.len(), 2);
        assert!(close(rects[0].origin.x, 2.0 * CHAR));
        assert!(close(
            rects[0].size.width,
            crate::numeric::count_f32(rows[0].end - 2) * CHAR
        ));
        assert!(close(rects[1].origin.x, 0.0));
        assert!(close(rects[1].origin.y, LINE));
        assert!(close(
            rects[1].size.width,
            crate::numeric::count_f32(15 - rows[1].start) * CHAR
        ));
    }

    #[gpui::test]
    fn a_selection_on_hanging_rows_starts_at_the_indent(cx: &mut TestAppContext) {
        let layout = shape(cx, "- aaaa bbbb cccc dddd", Some(14.0));
        let rows = layout.rows();
        let rects = layout.selection_rects(&(0..rows[1].end), false, px(6.0));
        assert!(close(rects[0].origin.x, 0.0));
        assert!(close(rects[1].origin.x, 2.0 * CHAR));
    }

    #[gpui::test]
    fn a_selection_outside_the_rows_adds_nothing(cx: &mut TestAppContext) {
        let layout = shape(cx, "aaaa bbbb cccc dddd", Some(10.0));
        let rows = layout.rows();
        let rects = layout.selection_rects(&(rows[1].start + 1..rows[1].start + 3), false, px(6.0));
        assert_eq!(rects.len(), 1);
        assert!(close(rects[0].origin.y, LINE));
    }

    #[gpui::test]
    fn runs_carry_the_token_style(cx: &mut TestAppContext) {
        let _ = cx;
        let theme = EditorTheme::light();
        let segments = [
            Segment {
                len: 2,
                class: Some(bourbon_markdown::TokenClass::Strong),
                marked: false,
            },
            Segment {
                len: 3,
                class: Some(bourbon_markdown::TokenClass::LinkText),
                marked: true,
            },
            Segment {
                len: 1,
                class: Some(bourbon_markdown::TokenClass::Strikethrough),
                marked: false,
            },
            Segment {
                len: 1,
                class: None,
                marked: false,
            },
        ];
        let runs = text_runs(&segments, &theme, false);
        assert_eq!(runs.len(), 4);
        assert_eq!(runs[0].len, 2);
        assert_eq!(runs[0].font.weight, FontWeight::BOLD);
        assert!(runs[0].underline.is_none());
        let marked_strong = text_runs(
            &[Segment {
                len: 1,
                class: Some(bourbon_markdown::TokenClass::Strong),
                marked: true,
            }],
            &theme,
            false,
        );
        assert!(marked_strong[0].underline.is_some());
        let plain_link = text_runs(
            &[Segment {
                len: 1,
                class: Some(bourbon_markdown::TokenClass::LinkText),
                marked: false,
            }],
            &theme,
            false,
        );
        assert!(plain_link[0].underline.is_some());
        assert!(runs[1].underline.is_some());
        assert_eq!(runs[1].font.weight, FontWeight::NORMAL);
        assert!(runs[2].strikethrough.is_some());
        assert!(runs[3].underline.is_none() && runs[3].strikethrough.is_none());
        assert_eq!(runs[3].color, theme.foreground);
        let forced = text_runs(&segments, &theme, true);
        assert!(forced.iter().all(|run| run.font.weight == FontWeight::BOLD));
    }

    #[test]
    fn the_editor_font_carries_family_weight_style_and_fallbacks() {
        let theme = EditorTheme::light();
        let font = editor_font(&theme, true, true);
        assert_eq!(font.family.as_ref(), theme.font_family.as_str());
        assert_eq!(font.weight, FontWeight::BOLD);
        assert_eq!(font.style, FontStyle::Italic);
        assert_eq!(
            font.fallbacks
                .map(|fallbacks| fallbacks.fallback_list().to_vec()),
            Some(theme.font_fallbacks.clone())
        );
        let plain = editor_font(&theme, false, false);
        assert_eq!(plain.weight, FontWeight::NORMAL);
        assert_eq!(plain.style, FontStyle::Normal);
    }

    #[gpui::test]
    fn a_selection_beyond_the_line_adds_nothing_even_through_the_break(cx: &mut TestAppContext) {
        let layout = shape(cx, "hello", None);
        assert_eq!(layout.selection_rects(&(10..12), true, px(6.0)), vec![]);
        assert_eq!(layout.selection_rects(&(0..0), true, px(6.0)).len(), 1);
    }
}
