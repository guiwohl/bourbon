use unicode_segmentation::UnicodeSegmentation;

use crate::text::Text;

fn line_with_break(text: &Text, line: usize) -> (usize, String) {
    let start = text.line_start(line);
    let end = text.line_start(line + 1);
    (start, text.slice(start..end))
}

fn is_word_like(segment: &str) -> bool {
    segment.chars().any(|c| !c.is_whitespace())
}

pub fn next_grapheme_boundary(text: &Text, position: usize) -> usize {
    if position >= text.len() {
        return text.len();
    }
    let (start, line) = line_with_break(text, text.line_of(position));
    line.grapheme_indices(true)
        .map(|(index, grapheme)| start + index + grapheme.len())
        .find(|end| *end > position)
        .unwrap_or_else(|| text.len())
}

pub fn previous_grapheme_boundary(text: &Text, position: usize) -> usize {
    if position == 0 {
        return 0;
    }
    let mut line = text.line_of(position);
    if position == text.line_start(line) {
        line -= 1;
    }
    let (start, content) = line_with_break(text, line);
    content
        .grapheme_indices(true)
        .map(|(index, _)| start + index)
        .rfind(|grapheme_start| *grapheme_start < position)
        .unwrap_or(0)
}

pub fn next_word_boundary(text: &Text, position: usize) -> usize {
    if position >= text.len() {
        return text.len();
    }
    let line = text.line_of(position);
    let (start, content) = line_with_break(text, line);
    let content_end = text.line_end(line) - start;
    let relative = position - start;
    if relative >= content_end {
        return start + content.len();
    }
    content
        .split_word_bound_indices()
        .find(|(index, segment)| index + segment.len() > relative && is_word_like(segment))
        .map_or(start + content_end, |(index, segment)| {
            start + index + segment.len()
        })
}

pub fn previous_word_boundary(text: &Text, position: usize) -> usize {
    if position == 0 {
        return 0;
    }
    let line = text.line_of(position);
    let start = text.line_start(line);
    if position == start {
        return text.line_end(line - 1);
    }
    let content = text.line_text(line);
    let relative = position - start;
    content
        .split_word_bound_indices()
        .rfind(|(index, segment)| *index < relative && is_word_like(segment))
        .map_or(start, |(index, _)| start + index)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(content: &str) -> Text {
        Text::new(content)
    }

    #[test]
    fn next_grapheme_steps_over_ascii_and_multibyte() {
        let text = text("aé日");
        assert_eq!(next_grapheme_boundary(&text, 0), 1);
        assert_eq!(next_grapheme_boundary(&text, 1), 3);
        assert_eq!(next_grapheme_boundary(&text, 3), 6);
        assert_eq!(next_grapheme_boundary(&text, 6), 6);
    }

    #[test]
    fn next_grapheme_treats_combining_marks_and_emoji_sequences_as_one() {
        let combining = text("e\u{301}x");
        assert_eq!(next_grapheme_boundary(&combining, 0), 3);
        let family = text("👨‍👩‍👧x");
        assert_eq!(next_grapheme_boundary(&family, 0), family.len() - 1);
    }

    #[test]
    fn next_grapheme_crosses_line_breaks_one_at_a_time() {
        let text = text("a\nb");
        assert_eq!(next_grapheme_boundary(&text, 1), 2);
        assert_eq!(next_grapheme_boundary(&text, 2), 3);
    }

    #[test]
    fn next_grapheme_from_inside_a_grapheme_moves_to_its_end() {
        let text = text("é");
        assert_eq!(next_grapheme_boundary(&text, 1), 2);
    }

    #[test]
    fn previous_grapheme_steps_back() {
        let text = text("aé日");
        assert_eq!(previous_grapheme_boundary(&text, 6), 3);
        assert_eq!(previous_grapheme_boundary(&text, 3), 1);
        assert_eq!(previous_grapheme_boundary(&text, 1), 0);
        assert_eq!(previous_grapheme_boundary(&text, 0), 0);
    }

    #[test]
    fn previous_grapheme_keeps_clusters_together() {
        let combining = text("xe\u{301}");
        assert_eq!(previous_grapheme_boundary(&combining, 4), 1);
        let family = text("x👨‍👩‍👧");
        assert_eq!(previous_grapheme_boundary(&family, family.len()), 1);
    }

    #[test]
    fn previous_grapheme_crosses_line_starts() {
        let text = text("ab\ncd");
        assert_eq!(previous_grapheme_boundary(&text, 3), 2);
        assert_eq!(previous_grapheme_boundary(&text, 2), 1);
        let blank = self::text("\n\nx");
        assert_eq!(previous_grapheme_boundary(&blank, 1), 0);
        assert_eq!(previous_grapheme_boundary(&blank, 2), 1);
    }

    #[test]
    fn next_word_skips_to_the_end_of_the_word() {
        let text = text("hello world");
        assert_eq!(next_word_boundary(&text, 0), 5);
        assert_eq!(next_word_boundary(&text, 2), 5);
        assert_eq!(next_word_boundary(&text, 5), 11);
        assert_eq!(next_word_boundary(&text, 11), 11);
    }

    #[test]
    fn next_word_stops_at_punctuation_segments() {
        let text = text("foo-bar");
        assert_eq!(next_word_boundary(&text, 0), 3);
        assert_eq!(next_word_boundary(&text, 3), 4);
        assert_eq!(next_word_boundary(&text, 4), 7);
    }

    #[test]
    fn next_word_stops_at_line_end_then_crosses_the_break() {
        let text = text("ab  \ncd");
        assert_eq!(next_word_boundary(&text, 2), 4);
        assert_eq!(next_word_boundary(&text, 4), 5);
        assert_eq!(next_word_boundary(&text, 5), 7);
    }

    #[test]
    fn next_word_works_on_later_lines() {
        let text = text("x\nab\ncd");
        assert_eq!(next_word_boundary(&text, 4), 5);
        let text = self::text("x\nhello world");
        assert_eq!(next_word_boundary(&text, 4), 7);
    }

    #[test]
    fn previous_word_goes_to_the_start_of_the_word() {
        let text = text("hello world");
        assert_eq!(previous_word_boundary(&text, 11), 6);
        assert_eq!(previous_word_boundary(&text, 8), 6);
        assert_eq!(previous_word_boundary(&text, 6), 0);
        assert_eq!(previous_word_boundary(&text, 0), 0);
    }

    #[test]
    fn previous_word_skips_trailing_whitespace_and_stops_at_punctuation() {
        let text = text("foo-bar  ");
        assert_eq!(previous_word_boundary(&text, 9), 4);
        assert_eq!(previous_word_boundary(&text, 4), 3);
        assert_eq!(previous_word_boundary(&text, 3), 0);
    }

    #[test]
    fn previous_word_at_line_start_goes_to_previous_line_end() {
        let text = text("ab  \ncd");
        assert_eq!(previous_word_boundary(&text, 5), 4);
    }

    #[test]
    fn previous_word_with_only_whitespace_goes_to_line_start() {
        let text = text("a\n   x");
        assert_eq!(previous_word_boundary(&text, 4), 2);
    }

    #[test]
    fn word_boundaries_handle_unicode_words() {
        let text = text("naïve déjà");
        assert_eq!(next_word_boundary(&text, 0), 6);
        assert_eq!(previous_word_boundary(&text, text.len()), 7);
    }
}
