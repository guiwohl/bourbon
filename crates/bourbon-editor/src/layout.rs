use std::ops::Range;

use gpui::{
    App, Bounds, Font, FontFallbacks, FontStyle, FontWeight, Pixels, Point, ShapedLine,
    StrikethroughStyle, TextAlign, TextRun, UnderlineStyle, Window, point, px, size,
};

use crate::segments::{Segment, slice_segments};
use crate::theme::EditorTheme;

pub struct ShapeRequest<'a> {
    pub text: &'a str,
    pub segments: &'a [Segment],
    pub font_size: Pixels,
    pub line_height: Pixels,
    pub tall_ranges: &'a [Range<usize>],
    pub tall_row_height: Pixels,
    pub wrap_width: Option<Pixels>,
    pub bold: bool,
    pub box_model: LineBox,
    pub theme: &'a EditorTheme,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LineBox {
    pub inset: Pixels,
    pub padding_top: Pixels,
    pub padding_bottom: Pixels,
}

struct Row {
    range: Range<usize>,
    shaped: ShapedLine,
    top: Pixels,
    height: Pixels,
}

pub struct LineLayout {
    rows: Vec<Row>,
    len: usize,
    box_model: LineBox,
}

pub fn editor_font(theme: &EditorTheme, bold: bool, italic: bool) -> Font {
    Font {
        family: theme.font_family.clone().into(),
        features: crate::fonts::without_ligatures(),
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
                background_color: None,
                underline,
                strikethrough,
            }
        })
        .collect()
}

fn wrapped_row_ranges(request: &ShapeRequest, window: &mut Window) -> Vec<Range<usize>> {
    let wrap_width = request
        .wrap_width
        .map(|width| (width - request.box_model.inset * 2.0).max(request.font_size));
    let wrapped = window
        .text_system()
        .shape_text(
            request.text.to_owned().into(),
            request.font_size,
            &text_runs(request.segments, request.theme, request.bold),
            wrap_width,
            None,
        )
        .unwrap_or_default()
        .into_iter()
        .next()
        .unwrap_or_default();
    let runs = wrapped.runs();
    let boundaries: Vec<usize> = wrapped
        .wrap_boundaries()
        .iter()
        .map(|boundary| runs[boundary.run_ix].glyphs[boundary.glyph_ix].index)
        .collect();
    let starts = std::iter::once(0).chain(boundaries.iter().copied());
    let ends = boundaries
        .iter()
        .copied()
        .chain(std::iter::once(request.text.len()));
    starts.zip(ends).map(|(start, end)| start..end).collect()
}

fn holds_tall_text(row: &Range<usize>, tall_ranges: &[Range<usize>]) -> bool {
    tall_ranges
        .iter()
        .any(|tall| tall.start < row.end && tall.end > row.start)
}

impl LineLayout {
    pub fn shape(request: &ShapeRequest, window: &mut Window) -> Self {
        let mut top = px(0.0);
        let rows = wrapped_row_ranges(request, window)
            .into_iter()
            .map(|range| {
                let shaped = window.text_system().shape_line(
                    request.text[range.clone()].to_owned().into(),
                    request.font_size,
                    &text_runs(
                        &slice_segments(request.segments, &range),
                        request.theme,
                        request.bold,
                    ),
                    None,
                );
                let height = if holds_tall_text(&range, request.tall_ranges) {
                    request.line_height.max(request.tall_row_height)
                } else {
                    request.line_height
                };
                let row = Row {
                    range,
                    shaped,
                    top,
                    height,
                };
                top += height;
                row
            })
            .collect();
        Self {
            rows,
            len: request.text.len(),
            box_model: request.box_model,
        }
    }

    fn text_origin(&self) -> Point<Pixels> {
        point(self.box_model.inset, self.box_model.padding_top)
    }

    pub fn row_of_y(&self, y: Pixels) -> usize {
        let y = y - self.box_model.padding_top;
        self.rows.iter().rposition(|row| row.top <= y).unwrap_or(0)
    }

    pub fn row_middle(&self, row: usize) -> Pixels {
        let row = &self.rows[row];
        self.box_model.padding_top + row.top + row.height / 2.0
    }

    pub fn first_row_height(&self) -> Pixels {
        self.rows[0].height
    }

    pub fn row_height_at(&self, index: usize) -> Pixels {
        self.rows[self.row_of_index(index)].height
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn height(&self) -> Pixels {
        let last = &self.rows[self.rows.len() - 1];
        last.top + last.height + self.box_model.padding_top + self.box_model.padding_bottom
    }

    pub fn rows(&self) -> Vec<Range<usize>> {
        self.rows.iter().map(|row| row.range.clone()).collect()
    }

    fn row_of_index(&self, index: usize) -> usize {
        let index = index.min(self.len);
        self.rows
            .iter()
            .rposition(|row| row.range.start <= index)
            .unwrap_or(0)
    }

    fn x_in_row(&self, row: usize, index: usize) -> Pixels {
        let row = &self.rows[row];
        let index = index.clamp(row.range.start, row.range.end);
        row.shaped.x_for_index(index - row.range.start)
    }

    pub fn position_for_index(&self, index: usize) -> Point<Pixels> {
        let row = self.row_of_index(index);
        self.text_origin() + point(self.x_in_row(row, index), self.rows[row].top)
    }

    pub fn index_for_position(&self, position: Point<Pixels>) -> usize {
        let position = position - self.text_origin();
        let row = &self.rows[self.row_of_y(position.y + self.box_model.padding_top)];
        let within = row.shaped.closest_index_for_x(position.x.max(px(0.0)));
        let last_in_row = if row.range.end == self.len {
            row.range.end
        } else {
            row.range.end.saturating_sub(1).max(row.range.start)
        };
        (row.range.start + within).min(last_in_row)
    }

    pub fn glyph_index_at(&self, position: Point<Pixels>) -> Option<usize> {
        let position = position - self.text_origin();
        let row = &self.rows[self.row_of_y(position.y + self.box_model.padding_top)];
        row.shaped
            .index_for_x(position.x)
            .map(|index| row.range.start + index)
    }

    pub fn selection_rects(
        &self,
        selected: &Range<usize>,
        through_line_break: bool,
        break_width: Pixels,
    ) -> Vec<Bounds<Pixels>> {
        let last = self.rows.len() - 1;
        let mut rects = Vec::new();
        for (index, row) in self.rows.iter().enumerate() {
            let range = &row.range;
            if selected.start > range.end || selected.end < range.start {
                continue;
            }
            let left = self.x_in_row(index, selected.start);
            let mut right = self.x_in_row(index, selected.end);
            if through_line_break && index == last {
                right += break_width;
            }
            if right > left {
                rects.push(Bounds::new(
                    self.text_origin() + point(left, row.top),
                    size(right - left, row.height),
                ));
            }
        }
        rects
    }

    pub fn text_paint_origin(&self, line_origin: Point<Pixels>) -> Point<Pixels> {
        line_origin + self.text_origin()
    }

    pub fn paint(&self, origin: Point<Pixels>, window: &mut Window, cx: &mut App) {
        let origin = self.text_paint_origin(origin);
        for row in &self.rows {
            let row_origin = point(origin.x, origin.y + row.top);
            row.shaped
                .paint(row_origin, row.height, TextAlign::Left, None, window, cx)
                .ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    const CHAR: f32 = 7.8;
    const LINE: f32 = 19.5;
    const TALL: f32 = 25.0;

    fn shape(cx: &mut TestAppContext, text: &str, wrap_chars: Option<f32>) -> LineLayout {
        shape_boxed(cx, text, wrap_chars, LineBox::default())
    }

    fn shape_boxed(
        cx: &mut TestAppContext,
        text: &str,
        wrap_chars: Option<f32>,
        box_model: LineBox,
    ) -> LineLayout {
        shape_with_tall(cx, text, wrap_chars, box_model, &[])
    }

    fn shape_with_tall(
        cx: &mut TestAppContext,
        text: &str,
        wrap_chars: Option<f32>,
        box_model: LineBox,
        tall_ranges: &[Range<usize>],
    ) -> LineLayout {
        let theme = EditorTheme::light();
        let segments = [Segment {
            len: text.len(),
            class: None,
            marked: false,
        }];
        let window = cx.add_empty_window();
        window.update(|window, _| {
            LineLayout::shape(
                &ShapeRequest {
                    text,
                    segments: if text.is_empty() { &[] } else { &segments },
                    font_size: px(13.0),
                    line_height: px(LINE),
                    tall_ranges,
                    tall_row_height: px(TALL),
                    wrap_width: wrap_chars.map(|chars| px(chars * CHAR + 0.5)),
                    bold: false,
                    box_model,
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

    fn card_box() -> LineBox {
        LineBox {
            inset: px(2.0),
            padding_top: px(4.0),
            padding_bottom: px(3.0),
        }
    }

    #[gpui::test]
    fn a_boxed_line_adds_its_padding_and_shifts_its_text(cx: &mut TestAppContext) {
        let layout = shape_boxed(cx, "hello", None, card_box());
        assert!(close(layout.height(), LINE + 7.0));
        let position = layout.position_for_index(3);
        assert!(close(position.x, 2.0 + 3.0 * CHAR));
        assert!(close(position.y, 4.0));
        assert_eq!(
            layout.index_for_position(point(px(2.0 + 3.0 * CHAR + 1.0), px(9.0))),
            3
        );
        assert_eq!(
            layout.glyph_index_at(point(px(2.0 + 0.5 * CHAR), px(9.0))),
            Some(0)
        );
        let rects = layout.selection_rects(&(1..3), false, px(6.0));
        assert!(close(rects[0].origin.x, 2.0 + CHAR));
        assert!(close(rects[0].origin.y, 4.0));
    }

    #[gpui::test]
    fn a_boxed_line_wraps_inside_both_sides_of_its_inset(cx: &mut TestAppContext) {
        let room_for_one_inset = Some(9.0 + 2.0 / CHAR);
        let plain = shape(cx, "aaaa bbbb", room_for_one_inset);
        assert_eq!(plain.row_count(), 1);
        let boxed = shape_boxed(cx, "aaaa bbbb", room_for_one_inset, card_box());
        assert_eq!(boxed.row_count(), 2);
    }

    #[gpui::test]
    fn the_text_is_painted_at_the_box_offset(cx: &mut TestAppContext) {
        let layout = shape_boxed(cx, "x", None, card_box());
        assert_eq!(
            layout.text_paint_origin(point(px(10.0), px(20.0))),
            point(px(12.0), px(24.0))
        );
    }

    #[gpui::test]
    fn rows_are_found_below_the_top_padding(cx: &mut TestAppContext) {
        let layout = shape_boxed(cx, "aaaa bbbb cccc dddd", Some(10.0), card_box());
        assert_eq!(layout.row_of_y(px(0.0)), 0);
        assert_eq!(layout.row_of_y(px(4.0 + LINE - 0.5)), 0);
        assert_eq!(layout.row_of_y(px(4.0 + LINE + 0.5)), 1);
        assert_eq!(layout.row_of_y(px(500.0)), layout.row_count() - 1);
        assert!(close(layout.row_middle(1), 4.0 + 1.5 * LINE));
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
    fn list_continuation_rows_start_at_the_left_edge(cx: &mut TestAppContext) {
        let layout = shape(cx, "- aaaa bbbb cccc dddd", Some(14.0));
        let rows = layout.rows();
        assert!(rows.len() >= 2);
        let continuation = layout.position_for_index(rows[1].start);
        assert!(close(continuation.x, 0.0));
        assert!(close(continuation.y, LINE));
        assert!(close(layout.position_for_index(2).x, 2.0 * CHAR));
    }

    #[gpui::test]
    fn only_rows_holding_tall_text_get_the_tall_height(cx: &mut TestAppContext) {
        let layout = shape_with_tall(
            cx,
            "aaaa bbbb cccc dddd",
            Some(10.0),
            LineBox::default(),
            std::slice::from_ref(&(12..14)),
        );
        assert_eq!(layout.row_count(), 2);
        assert!(close(layout.height(), LINE + TALL));
        assert!(close(layout.first_row_height(), LINE));
        assert!(close(layout.row_height_at(15), TALL));
        assert!(close(layout.position_for_index(15).y, LINE));
        assert!(close(layout.row_middle(1), LINE + TALL / 2.0));
        assert_eq!(layout.row_of_y(px(LINE + TALL - 0.5)), 1);
        let rects = layout.selection_rects(&(0..15), false, px(6.0));
        assert!(close(rects[1].size.height, TALL));
        assert!(close(rects[1].origin.y, LINE));
    }

    #[gpui::test]
    fn a_tall_range_never_shrinks_a_row(cx: &mut TestAppContext) {
        let theme = EditorTheme::light();
        let segments = [Segment {
            len: 3,
            class: None,
            marked: false,
        }];
        let window = cx.add_empty_window();
        let layout = window.update(|window, _| {
            LineLayout::shape(
                &ShapeRequest {
                    text: "abc",
                    segments: &segments,
                    font_size: px(13.0),
                    line_height: px(LINE),
                    tall_ranges: std::slice::from_ref(&(0..1)),
                    tall_row_height: px(5.0),
                    wrap_width: None,
                    bold: false,
                    box_model: LineBox::default(),
                    theme: &theme,
                },
                window,
            )
        });
        assert!(close(layout.height(), LINE));
    }

    #[gpui::test]
    fn a_click_past_a_wrapped_row_stays_on_that_row(cx: &mut TestAppContext) {
        let layout = shape(cx, "aaaa bbbb cccc dddd", Some(10.0));
        let rows = layout.rows();
        let index = layout.index_for_position(point(px(500.0), px(5.0)));
        assert_eq!(index, rows[0].end - 1);
    }

    #[gpui::test]
    fn every_index_maps_back_from_its_own_position(cx: &mut TestAppContext) {
        for text in [
            "hello world",
            "- aaaa bbbb cccc dddd eeee",
            "aaaa bbbb cccc dddd",
        ] {
            let layout = shape(cx, text, Some(12.0));
            let skipped = [text.len() - 1];
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
    fn glyph_lookup_on_continuation_rows_starts_at_the_left_edge(cx: &mut TestAppContext) {
        let layout = shape(cx, "- aaaa bbbb cccc dddd", Some(14.0));
        let start = layout.rows()[1].start;
        let found = layout.glyph_index_at(point(px(1.0), px(LINE + 5.0)));
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
        assert_eq!(font.features, crate::fonts::without_ligatures());
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
