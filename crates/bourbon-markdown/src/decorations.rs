use crate::lines::LineIndex;
use crate::parse;
use pulldown_cmark::{Event, Tag};
use std::ops::RangeInclusive;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecorationKind {
    Card,
    QuoteBar,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineDecoration {
    pub lines: RangeInclusive<usize>,
    pub kind: DecorationKind,
}

#[must_use]
pub fn line_decorations(source: &str) -> Vec<LineDecoration> {
    let index = LineIndex::new(source);
    let mut decorations: Vec<LineDecoration> = Vec::new();
    for (event, range) in parse::events(source) {
        let kind = match event {
            Event::Start(Tag::CodeBlock(_) | Tag::Table(_)) => DecorationKind::Card,
            Event::Start(Tag::BlockQuote(_)) => DecorationKind::QuoteBar,
            _ => continue,
        };
        let last_byte = source[..range.end]
            .trim_end_matches(['\n', '\r'])
            .len()
            .max(range.start + 1)
            - 1;
        let lines = index.line_of(range.start)..=index.line_of(last_byte);
        let nested = decorations
            .iter()
            .any(|outer| outer.kind == kind && outer.lines.contains(lines.start()));
        if !nested {
            decorations.push(LineDecoration { lines, kind });
        }
    }
    decorations
}

#[cfg(test)]
mod tests {
    use super::{DecorationKind::*, LineDecoration, line_decorations};

    fn decoration(
        lines: std::ops::RangeInclusive<usize>,
        kind: super::DecorationKind,
    ) -> LineDecoration {
        LineDecoration { lines, kind }
    }

    #[test]
    fn a_fenced_block_is_one_card_from_fence_to_fence() {
        let source = "text\n\n```rust\nlet a = 1;\n```\n\nafter\n";
        assert_eq!(line_decorations(source), vec![decoration(2..=4, Card)]);
    }

    #[test]
    fn an_unclosed_fence_runs_to_the_last_line() {
        assert_eq!(line_decorations("```\na\nb"), vec![decoration(0..=2, Card)]);
    }

    #[test]
    fn a_table_is_a_card() {
        let source = "| a | b |\n|---|---|\n| 1 | 2 |\n";
        assert_eq!(line_decorations(source), vec![decoration(0..=2, Card)]);
    }

    #[test]
    fn a_quote_gets_a_bar_and_a_nested_quote_does_not_repeat_it() {
        let source = "> one\n> > two\n> three\n\nplain\n";
        assert_eq!(line_decorations(source), vec![decoration(0..=2, QuoteBar)]);
    }

    #[test]
    fn code_inside_a_quote_keeps_both_decorations() {
        let source = "> quote\n>\n>     code\n";
        assert_eq!(
            line_decorations(source),
            vec![decoration(0..=2, QuoteBar), decoration(2..=2, Card)]
        );
    }

    #[test]
    fn plain_text_has_no_decorations() {
        assert_eq!(line_decorations("# Title\n\ntext\n"), vec![]);
        assert_eq!(line_decorations(""), vec![]);
    }
}
