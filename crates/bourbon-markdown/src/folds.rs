use crate::lines::LineIndex;
use crate::parse;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Tag};
use std::ops::RangeInclusive;

struct Heading {
    line: usize,
    level: HeadingLevel,
}

#[must_use]
pub fn fold_ranges(source: &str) -> Vec<RangeInclusive<usize>> {
    let index = LineIndex::new(source);
    let lines: Vec<&str> = source.split('\n').collect();
    let mut headings = Vec::new();
    let mut fences = Vec::new();
    for (event, range) in parse::events(source) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => headings.push(Heading {
                line: index.line_of(range.start),
                level,
            }),
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_))) => {
                let content_end = source[..range.end].trim_end_matches(['\n', '\r']).len();
                fences.push(index.line_of(range.start)..=index.line_of(content_end));
            }
            _ => {}
        }
    }
    let mut ranges: Vec<RangeInclusive<usize>> = headings
        .iter()
        .enumerate()
        .map(|(position, heading)| {
            let next = headings[position + 1..]
                .iter()
                .find(|later| later.level <= heading.level)
                .map_or(lines.len(), |later| later.line);
            heading.line..=last_non_blank_line(&lines, heading.line, next)
        })
        .chain(fences)
        .filter(|range| range.end() > range.start())
        .collect();
    ranges.sort_by_key(|range| (*range.start(), std::cmp::Reverse(*range.end())));
    ranges
}

fn last_non_blank_line(lines: &[&str], first: usize, end: usize) -> usize {
    (first..end)
        .rev()
        .find(|line| !lines[*line].trim().is_empty())
        .unwrap_or(first)
}

#[cfg(test)]
mod tests {
    use super::fold_ranges;

    #[test]
    fn a_heading_folds_until_the_next_heading_of_the_same_level() {
        let source = "## A\ntext\n\n## B\nmore\n";
        assert_eq!(fold_ranges(source), vec![0..=1, 3..=4]);
    }

    #[test]
    fn a_heading_section_contains_its_deeper_headings() {
        let source = "# Top\n## Sub\nbody\n# Next\nx";
        assert_eq!(fold_ranges(source), vec![0..=2, 1..=2, 3..=4]);
    }

    #[test]
    fn a_heading_section_stops_at_a_higher_heading() {
        let source = "## Sub\nbody\n# Top\nx";
        assert_eq!(fold_ranges(source), vec![0..=1, 2..=3]);
    }

    #[test]
    fn a_heading_without_content_cannot_fold() {
        assert_eq!(fold_ranges("## A\n\n## B\n"), Vec::new());
        assert_eq!(fold_ranges("# Only"), Vec::new());
    }

    #[test]
    fn a_fenced_block_folds_from_fence_to_fence() {
        let source = "text\n\n```rust\nlet a = 1;\n```\n\nafter\n";
        assert_eq!(fold_ranges(source), vec![2..=4]);
    }

    #[test]
    fn indented_code_and_plain_text_do_not_fold() {
        assert_eq!(fold_ranges("para\n\n    code\n    more\n"), Vec::new());
        assert_eq!(fold_ranges(""), Vec::new());
    }

    #[test]
    fn a_fence_inside_a_section_folds_on_its_own() {
        let source = "## A\n```\nx\n```\n";
        assert_eq!(fold_ranges(source), vec![0..=3, 1..=3]);
    }
}
