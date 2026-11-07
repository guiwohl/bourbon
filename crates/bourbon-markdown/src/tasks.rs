use crate::lines::LineIndex;
use crate::parse;
use pulldown_cmark::Event;

#[must_use]
pub fn task_lines(source: &str) -> Vec<(usize, bool)> {
    let index = LineIndex::new(source);
    parse::events(source)
        .into_iter()
        .filter_map(|(event, range)| match event {
            Event::TaskListMarker(checked) => Some((index.line_of(range.start), checked)),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::task_lines;

    #[test]
    fn lists_task_markers_with_their_source_line() {
        let source = "# Plan\n\n- [ ] one\n- [x] two\n- plain\n  - [X] nested\n";
        assert_eq!(task_lines(source), vec![(2, false), (3, true), (5, true)]);
    }

    #[test]
    fn ignores_brackets_that_are_not_tasks() {
        assert_eq!(
            task_lines("- [ ]no space\n\n`- [ ] code`\n\n[ ] loose\n"),
            vec![]
        );
    }

    #[test]
    fn empty_source_has_no_tasks() {
        assert_eq!(task_lines(""), vec![]);
    }
}
