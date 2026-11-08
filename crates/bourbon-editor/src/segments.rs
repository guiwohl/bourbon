use std::ops::Range;

use bourbon_markdown::{HighlightSpan, TokenClass};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Segment {
    pub len: usize,
    pub class: Option<TokenClass>,
    pub marked: bool,
}

pub fn line_segments(
    line: Range<usize>,
    spans: &[HighlightSpan],
    marked: Option<&Range<usize>>,
) -> Vec<Segment> {
    let classed = classed_pieces(&line, spans);
    match marked {
        Some(marked) => split_by_marked(&classed, line.start, marked),
        None => classed
            .into_iter()
            .map(|(range, class)| plain_segment(range.len(), class, false))
            .collect(),
    }
}

fn plain_segment(len: usize, class: Option<TokenClass>, marked: bool) -> Segment {
    Segment { len, class, marked }
}

fn classed_pieces(
    line: &Range<usize>,
    spans: &[HighlightSpan],
) -> Vec<(Range<usize>, Option<TokenClass>)> {
    let first = spans.partition_point(|span| span.range.end <= line.start);
    let mut pieces = Vec::new();
    let mut cursor = line.start;
    for span in spans[first..]
        .iter()
        .take_while(|span| span.range.start < line.end)
    {
        let start = span.range.start.max(cursor);
        let end = span.range.end.min(line.end);
        if start > cursor {
            pieces.push((cursor..start, None));
        }
        pieces.push((start..end, Some(span.class)));
        cursor = end;
    }
    if cursor < line.end {
        pieces.push((cursor..line.end, None));
    }
    pieces
}

fn split_by_marked(
    pieces: &[(Range<usize>, Option<TokenClass>)],
    line_start: usize,
    marked: &Range<usize>,
) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut start = line_start;
    for (range, class) in pieces {
        let end = start + range.len();
        let inside_start = start.max(marked.start).min(end);
        let inside_end = end.min(marked.end).max(inside_start);
        for (from, to, is_marked) in [
            (start, inside_start, false),
            (inside_start, inside_end, true),
            (inside_end, end, false),
        ] {
            if to > from {
                segments.push(plain_segment(to - from, *class, is_marked));
            }
        }
        start = end;
    }
    segments
}

pub fn slice_segments(segments: &[Segment], range: &Range<usize>) -> Vec<Segment> {
    let mut sliced = Vec::new();
    let mut start = 0;
    for segment in segments {
        let end = start + segment.len;
        let len = end.min(range.end).saturating_sub(start.max(range.start));
        if len > 0 {
            sliced.push(Segment { len, ..*segment });
        }
        start = end;
    }
    sliced
}

pub fn heading_level(line: Range<usize>, spans: &[HighlightSpan]) -> Option<u8> {
    let first = spans.partition_point(|span| span.range.end <= line.start);
    spans[first..]
        .iter()
        .take_while(|span| span.range.start < line.end)
        .find_map(|span| match span.class {
            TokenClass::Heading(level) => Some(level),
            _ => None,
        })
}

pub fn inline_code_ranges(segments: &[Segment]) -> Vec<Range<usize>> {
    let is_mark = |index: usize| {
        segments
            .get(index)
            .is_some_and(|segment| segment.class == Some(TokenClass::CodeMark))
    };
    let mut starts = Vec::with_capacity(segments.len() + 1);
    let mut offset = 0;
    for segment in segments {
        starts.push(offset);
        offset += segment.len;
    }
    starts.push(offset);
    segments
        .iter()
        .enumerate()
        .filter(|(_, segment)| segment.class == Some(TokenClass::InlineCode))
        .map(|(index, _)| {
            let first = if index > 0 && is_mark(index - 1) {
                index - 1
            } else {
                index
            };
            let last = if is_mark(index + 1) { index + 1 } else { index };
            starts[first]..starts[last + 1]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(range: Range<usize>, class: TokenClass) -> HighlightSpan {
        HighlightSpan { range, class }
    }

    fn segment(len: usize, class: Option<TokenClass>) -> Segment {
        Segment {
            len,
            class,
            marked: false,
        }
    }

    #[test]
    fn inline_code_boxes_cover_the_code_and_its_backticks() {
        let segments = [
            segment(2, None),
            segment(1, Some(TokenClass::CodeMark)),
            segment(4, Some(TokenClass::InlineCode)),
            segment(1, Some(TokenClass::CodeMark)),
            segment(3, None),
            segment(2, Some(TokenClass::CodeMark)),
            segment(1, Some(TokenClass::InlineCode)),
            segment(2, Some(TokenClass::CodeMark)),
        ];
        assert_eq!(inline_code_ranges(&segments), vec![2..8, 11..16]);
    }

    #[test]
    fn fence_marks_and_code_cut_by_the_line_are_boxed_alone() {
        assert_eq!(
            inline_code_ranges(&[segment(3, Some(TokenClass::CodeMark))]),
            Vec::<Range<usize>>::new()
        );
        assert_eq!(
            inline_code_ranges(&[
                segment(4, Some(TokenClass::InlineCode)),
                segment(1, Some(TokenClass::CodeMark)),
            ]),
            vec![0..5]
        );
        assert_eq!(
            inline_code_ranges(&[
                segment(1, Some(TokenClass::CodeMark)),
                segment(4, Some(TokenClass::InlineCode)),
            ]),
            vec![0..5]
        );
    }

    #[test]
    fn a_line_without_spans_is_one_plain_segment() {
        assert_eq!(line_segments(0..5, &[], None), vec![segment(5, None)]);
    }

    #[test]
    fn an_empty_line_has_no_segments() {
        assert_eq!(line_segments(4..4, &[], None), vec![]);
    }

    #[test]
    fn gaps_between_spans_are_plain() {
        let spans = [
            span(0..1, TokenClass::HeaderMark),
            span(2..5, TokenClass::Heading(1)),
        ];
        assert_eq!(
            line_segments(0..7, &spans, None),
            vec![
                segment(1, Some(TokenClass::HeaderMark)),
                segment(1, None),
                segment(3, Some(TokenClass::Heading(1))),
                segment(2, None),
            ]
        );
    }

    #[test]
    fn spans_are_clipped_to_the_line() {
        let spans = [
            span(0..8, TokenClass::CodeBlockText),
            span(8..20, TokenClass::Strong),
        ];
        assert_eq!(
            line_segments(5..12, &spans, None),
            vec![
                segment(3, Some(TokenClass::CodeBlockText)),
                segment(4, Some(TokenClass::Strong)),
            ]
        );
    }

    #[test]
    fn spans_outside_the_line_are_ignored() {
        let spans = [
            span(0..3, TokenClass::Strong),
            span(10..12, TokenClass::Emphasis),
        ];
        assert_eq!(line_segments(4..9, &spans, None), vec![segment(5, None)]);
    }

    #[test]
    fn the_marked_range_splits_segments_and_flags_them() {
        let spans = [span(0..6, TokenClass::Strong)];
        let segments = line_segments(0..6, &spans, Some(&(2..4)));
        assert_eq!(
            segments,
            vec![
                Segment {
                    len: 2,
                    class: Some(TokenClass::Strong),
                    marked: false
                },
                Segment {
                    len: 2,
                    class: Some(TokenClass::Strong),
                    marked: true
                },
                Segment {
                    len: 2,
                    class: Some(TokenClass::Strong),
                    marked: false
                },
            ]
        );
    }

    #[test]
    fn segment_lengths_always_add_up_to_the_line() {
        let spans = [
            span(1..3, TokenClass::Emphasis),
            span(4..9, TokenClass::Url),
        ];
        let total: usize = line_segments(0..10, &spans, Some(&(2..6)))
            .iter()
            .map(|segment| segment.len)
            .sum();
        assert_eq!(total, 10);
    }

    #[test]
    fn slicing_keeps_the_segments_inside_the_range() {
        let segments = [segment(2, None), segment(3, Some(TokenClass::Strong))];
        assert_eq!(slice_segments(&segments, &(0..2)), vec![segment(2, None)]);
        assert_eq!(
            slice_segments(&segments, &(1..4)),
            vec![segment(1, None), segment(2, Some(TokenClass::Strong))]
        );
        assert_eq!(
            slice_segments(&segments, &(2..9)),
            vec![segment(3, Some(TokenClass::Strong))]
        );
        assert_eq!(slice_segments(&segments, &(5..5)), vec![]);
    }

    #[test]
    fn heading_level_comes_from_a_heading_span_on_the_line() {
        let spans = [
            span(0..2, TokenClass::HeaderMark),
            span(2..7, TokenClass::Heading(3)),
        ];
        assert_eq!(heading_level(0..7, &spans), Some(3));
        assert_eq!(heading_level(8..12, &spans), None);
    }

    #[test]
    fn heading_level_ignores_other_classes() {
        let spans = [span(0..4, TokenClass::Strong)];
        assert_eq!(heading_level(0..4, &spans), None);
    }

    #[test]
    fn a_span_starting_at_the_line_end_adds_no_empty_segment() {
        let spans = [
            span(0..3, TokenClass::Strong),
            span(5..8, TokenClass::Emphasis),
        ];
        assert_eq!(
            line_segments(0..5, &spans, None),
            vec![segment(3, Some(TokenClass::Strong)), segment(2, None)]
        );
    }

    #[test]
    fn marked_ranges_on_piece_edges_add_no_empty_segments() {
        let spans = [span(0..2, TokenClass::Strong)];
        let segments = line_segments(0..4, &spans, Some(&(2..4)));
        assert!(segments.iter().all(|segment| segment.len > 0));
        assert_eq!(segments.len(), 2);
    }

    #[test]
    fn heading_level_ignores_a_heading_that_starts_at_the_line_end() {
        let spans = [span(4..8, TokenClass::Heading(2))];
        assert_eq!(heading_level(0..4, &spans), None);
    }
}
