use crate::token::{HighlightSpan, TokenClass};
use std::ops::Range;

pub struct Canvas<'a> {
    source: &'a str,
    classes: Vec<Option<TokenClass>>,
}

impl<'a> Canvas<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            classes: vec![None; source.len()],
        }
    }

    pub fn paint(&mut self, range: Range<usize>, class: TokenClass) {
        let end = range.end.min(self.source.len());
        let on_boundaries =
            self.source.is_char_boundary(range.start) && self.source.is_char_boundary(end);
        if let Some(slice) = self
            .classes
            .get_mut(range.start..end)
            .filter(|_| on_boundaries)
        {
            slice.fill(Some(class));
        }
    }

    pub fn into_spans(self) -> Vec<HighlightSpan> {
        let mut spans: Vec<HighlightSpan> = Vec::new();
        for (offset, class) in self.classes.iter().enumerate() {
            let Some(class) = *class else { continue };
            match spans.last_mut() {
                Some(last) if last.range.end == offset && last.class == class => {
                    last.range.end += 1;
                }
                _ => spans.push(HighlightSpan {
                    range: offset..offset + 1,
                    class,
                }),
            }
        }
        spans
    }
}

#[cfg(test)]
mod tests {
    use super::Canvas;
    use crate::token::TokenClass::{Emphasis, Strong};

    #[test]
    fn later_paint_wins_and_equal_neighbours_merge() {
        let mut canvas = Canvas::new("abcdef");
        canvas.paint(0..4, Emphasis);
        canvas.paint(2..3, Strong);
        canvas.paint(4..6, Emphasis);
        let spans: Vec<_> = canvas
            .into_spans()
            .into_iter()
            .map(|span| (span.range, span.class))
            .collect();
        assert_eq!(spans, [(0..2, Emphasis), (2..3, Strong), (3..6, Emphasis)]);
    }

    #[test]
    fn out_of_bounds_empty_and_mid_character_ranges_are_ignored() {
        let mut canvas = Canvas::new("aé");
        canvas.paint(0..1, Emphasis);
        canvas.paint(2..3, Strong);
        canvas.paint(1..1, Strong);
        canvas.paint(9..12, Strong);
        canvas.paint(1..2, Strong);
        let spans: Vec<_> = canvas
            .into_spans()
            .into_iter()
            .map(|span| (span.range, span.class))
            .collect();
        assert_eq!(spans, [(0..1, Emphasis)]);
    }

    #[test]
    fn range_past_the_end_is_clamped() {
        let mut canvas = Canvas::new("abc");
        canvas.paint(1..99, Strong);
        let spans: Vec<_> = canvas
            .into_spans()
            .into_iter()
            .map(|span| (span.range, span.class))
            .collect();
        assert_eq!(spans, [(1..3, Strong)]);
    }
}
