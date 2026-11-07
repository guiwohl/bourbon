use std::fmt;
use std::ops::Range;

use ropey::Rope;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Text {
    rope: Rope,
}

impl Text {
    pub fn new(content: &str) -> Self {
        Self {
            rope: Rope::from_str(content),
        }
    }

    pub fn len(&self) -> usize {
        self.rope.len_bytes()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn line_of(&self, offset: usize) -> usize {
        self.rope.byte_to_line(offset)
    }

    pub fn line_start(&self, line: usize) -> usize {
        self.rope.line_to_byte(line)
    }

    pub fn line_end(&self, line: usize) -> usize {
        if line + 1 < self.line_count() {
            self.line_start(line + 1) - 1
        } else {
            self.len()
        }
    }

    pub fn line_text(&self, line: usize) -> String {
        self.slice(self.line_start(line)..self.line_end(line))
    }

    pub fn column(&self, offset: usize) -> usize {
        let line_start = self.line_start(self.line_of(offset));
        self.rope.byte_to_char(offset) - self.rope.byte_to_char(line_start)
    }

    pub fn slice(&self, range: Range<usize>) -> String {
        self.rope.byte_slice(range).to_string()
    }

    pub fn char_after(&self, offset: usize) -> Option<char> {
        (offset < self.len()).then(|| self.rope.char(self.rope.byte_to_char(offset)))
    }

    pub fn char_before(&self, offset: usize) -> Option<char> {
        (offset > 0).then(|| self.rope.char(self.rope.byte_to_char(offset) - 1))
    }

    pub fn has_at(&self, offset: usize, expected: &str) -> bool {
        let mut actual = self.rope.byte_slice(offset..).chars();
        expected.chars().all(|c| actual.next() == Some(c))
    }

    pub fn has_before(&self, offset: usize, expected: &str) -> bool {
        let mut position = offset;
        expected.chars().rev().all(|expected_char| {
            let matches = self.char_before(position) == Some(expected_char);
            position = position.saturating_sub(expected_char.len_utf8());
            matches
        })
    }

    pub(crate) fn splice(&mut self, range: Range<usize>, replacement: &str) {
        let start = self.rope.byte_to_char(range.start);
        let end = self.rope.byte_to_char(range.end);
        self.rope.remove(start..end);
        self.rope.insert(start, replacement);
    }
}

impl fmt::Display for Text {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.rope
            .chunks()
            .try_for_each(|chunk| formatter.write_str(chunk))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_has_one_empty_line() {
        let text = Text::new("");
        assert!(text.is_empty());
        assert_eq!(text.len(), 0);
        assert_eq!(text.line_count(), 1);
        assert_eq!(text.line_end(0), 0);
    }

    #[test]
    fn is_empty_is_false_for_content() {
        assert!(!Text::new("a").is_empty());
    }

    #[test]
    fn len_counts_bytes_not_chars() {
        assert_eq!(Text::new("aé日").len(), 6);
    }

    #[test]
    fn lines_are_split_on_line_feed_only() {
        let text = Text::new("ab\ncd\n");
        assert_eq!(text.line_count(), 3);
        assert_eq!(text.line_start(1), 3);
        assert_eq!(text.line_start(2), 6);
        assert_eq!(text.line_end(0), 2);
        assert_eq!(text.line_end(1), 5);
        assert_eq!(text.line_end(2), 6);
    }

    #[test]
    fn carriage_return_stays_inside_the_line() {
        let text = Text::new("a\rb\nc");
        assert_eq!(text.line_count(), 2);
        assert_eq!(text.line_text(0), "a\rb");
    }

    #[test]
    fn line_of_maps_offsets_including_line_break_and_end() {
        let text = Text::new("ab\ncd");
        assert_eq!(text.line_of(0), 0);
        assert_eq!(text.line_of(2), 0);
        assert_eq!(text.line_of(3), 1);
        assert_eq!(text.line_of(5), 1);
    }

    #[test]
    fn column_counts_chars_from_line_start() {
        let text = Text::new("é日x\nzé");
        assert_eq!(text.column(0), 0);
        assert_eq!(text.column(2), 1);
        assert_eq!(text.column(5), 2);
        assert_eq!(text.column(6), 3);
        assert_eq!(text.column(7), 0);
        assert_eq!(text.column(8), 1);
        assert_eq!(text.column(10), 2);
    }

    #[test]
    fn slice_and_line_text_return_content() {
        let text = Text::new("héllo\nworld");
        assert_eq!(text.slice(0..3), "hé");
        assert_eq!(text.line_text(0), "héllo");
        assert_eq!(text.line_text(1), "world");
    }

    #[test]
    fn display_round_trips() {
        assert_eq!(Text::new("a\nb日").to_string(), "a\nb日");
    }

    #[test]
    fn neighbouring_chars_respect_utf8_widths() {
        let text = Text::new("a日b");
        assert_eq!(text.char_after(0), Some('a'));
        assert_eq!(text.char_after(1), Some('日'));
        assert_eq!(text.char_after(4), Some('b'));
        assert_eq!(text.char_after(5), None);
        assert_eq!(text.char_before(0), None);
        assert_eq!(text.char_before(1), Some('a'));
        assert_eq!(text.char_before(4), Some('日'));
        assert_eq!(text.char_before(5), Some('b'));
    }

    #[test]
    fn has_at_and_has_before_compare_around_an_offset() {
        let text = Text::new("a**b**c");
        assert!(text.has_at(1, "**"));
        assert!(!text.has_at(0, "**"));
        assert!(!text.has_at(6, "cc"));
        assert!(text.has_before(3, "**"));
        assert!(!text.has_before(1, "**"));
        assert!(text.has_before(0, ""));
    }

    #[test]
    fn splice_replaces_a_byte_range() {
        let mut text = Text::new("aéb");
        text.splice(1..3, "日日");
        assert_eq!(text.to_string(), "a日日b");
    }
}
