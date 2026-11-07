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
    prefix: Option<ShapedLine>,
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
        let prefix = (request.prefix_len > 0).then(|| {
            window.text_system().shape_line(
                prefix_text.to_owned().into(),
                request.font_size,
                &text_runs(&prefix_segments, request.theme, request.bold),
                None,
            )
        });
        let indent = prefix.as_ref().map_or(px(0.0), |prefix| prefix.width);
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
        if let Some(prefix) = self.prefix.as_ref()
            && index < self.prefix_len
        {
            return prefix.x_for_index(index);
        }
        let body_index = index - self.prefix_len;
        if row > 0 && index == rows[row].start {
            return self.indent;
        }
        self.indent
            + self
                .body
                .position_for_index(body_index, self.line_height)
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
        if let Some(prefix) = self.prefix.as_ref()
            && row == 0
            && position.x < self.indent
        {
            return prefix.closest_index_for_x(position.x).min(self.prefix_len);
        }
        let body_point = point(position.x - self.indent, self.line_height * count_f32(row));
        let body_index = self
            .body
            .closest_index_for_position(body_point, self.line_height)
            .unwrap_or_else(|edge| edge);
        self.prefix_len + body_index
    }

    pub fn glyph_index_at(&self, position: Point<Pixels>) -> Option<usize> {
        let row_count = self.row_count();
        let row = row_from_ratio(position.y / self.line_height).min(row_count - 1);
        if let Some(prefix) = self.prefix.as_ref()
            && row == 0
            && position.x < self.indent
        {
            return prefix.index_for_x(position.x);
        }
        let body_point = point(position.x - self.indent, self.line_height * count_f32(row));
        self.body
            .index_for_position(body_point, self.line_height)
            .ok()
            .map(|index| self.prefix_len + index)
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
        if let Some(prefix) = self.prefix.as_ref() {
            prefix
                .paint_background(origin, self.line_height, TextAlign::Left, None, window, cx)
                .ok();
            prefix
                .paint(origin, self.line_height, TextAlign::Left, None, window, cx)
                .ok();
        }
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
