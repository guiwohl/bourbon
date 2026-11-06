#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WordCount {
    words: usize,
    characters: usize,
}

impl WordCount {
    pub(crate) fn of(body: &str) -> Self {
        Self {
            words: body.split_whitespace().count(),
            characters: body.chars().count(),
        }
    }

    pub(crate) fn label(self) -> String {
        format!("{} words \u{b7} {} characters", self.words, self.characters)
    }
}

#[cfg(test)]
mod tests {
    use super::WordCount;

    #[test]
    fn an_empty_body_has_nothing() {
        assert_eq!(WordCount::of("").label(), "0 words \u{b7} 0 characters");
    }

    #[test]
    fn words_are_runs_separated_by_any_unicode_whitespace() {
        let body = "one  two\nthree\t four\u{a0}five\u{3000}six";
        assert_eq!(WordCount::of(body).words, 6);
    }

    #[test]
    fn characters_count_every_char_including_whitespace_not_bytes() {
        assert_eq!(WordCount::of("a b\n").characters, 4);
        assert_eq!(WordCount::of("a\u{e7}\u{e3}o").characters, 4);
    }

    #[test]
    fn the_label_reads_words_then_characters() {
        assert_eq!(
            WordCount::of("hello big world").label(),
            "3 words \u{b7} 15 characters"
        );
    }
}
