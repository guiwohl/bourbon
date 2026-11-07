use std::ops::Range;

use bourbon_markdown::{HighlightSpan, TokenClass};
use bourbon_text::commands::toggle_task;
use bourbon_text::{EditorState, Selection, Transaction};

pub fn task_marker_at(spans: &[HighlightSpan], offset: usize) -> Option<Range<usize>> {
    let first = spans.partition_point(|span| span.range.end <= offset);
    spans
        .get(first)
        .filter(|span| {
            matches!(span.class, TokenClass::TaskMarker { .. }) && span.range.start <= offset
        })
        .map(|span| span.range.clone())
}

pub fn toggle_task_at(state: &EditorState, offset: usize) -> Option<Transaction> {
    let probe = EditorState::new(state.text.clone(), Selection::cursor(offset));
    let transaction = toggle_task(&probe)?;
    let selection_after = state.selection.map(&transaction.changes);
    Some(Transaction {
        selection_after,
        ..transaction
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bourbon_markdown::highlight;
    use bourbon_text::Text;

    fn state(source: &str, caret: usize) -> EditorState {
        EditorState::new(Text::new(source), Selection::cursor(caret))
    }

    #[test]
    fn finds_the_marker_under_the_offset() {
        let spans = highlight("- [ ] a\n");
        assert_eq!(task_marker_at(&spans, 2), Some(2..5));
        assert_eq!(task_marker_at(&spans, 4), Some(2..5));
    }

    #[test]
    fn offsets_outside_the_marker_find_nothing() {
        let spans = highlight("- [ ] a\n");
        assert_eq!(task_marker_at(&spans, 0), None);
        assert_eq!(task_marker_at(&spans, 5), None);
        assert_eq!(task_marker_at(&spans, 6), None);
    }

    #[test]
    fn toggling_checks_the_box_and_keeps_the_caret() {
        let state = state("- [ ] a\nsecond", 12);
        let transaction = toggle_task_at(&state, 3).expect("task toggled");
        let next = state.apply(&transaction);
        assert_eq!(next.text.to_string(), "- [x] a\nsecond");
        assert_eq!(next.selection, Selection::cursor(12));
    }

    #[test]
    fn toggling_unchecks_a_checked_box() {
        let state = state("- [x] a", 0);
        let next = state.apply(&toggle_task_at(&state, 3).expect("task toggled"));
        assert_eq!(next.text.to_string(), "- [ ] a");
    }

    #[test]
    fn toggling_only_touches_the_clicked_line() {
        let state = state("- [ ] a\n- [ ] b", 0);
        let next = state.apply(&toggle_task_at(&state, 11).expect("task toggled"));
        assert_eq!(next.text.to_string(), "- [ ] a\n- [x] b");
    }
}
