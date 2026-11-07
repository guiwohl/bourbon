use crate::change::Change;
use crate::movement::{next_grapheme_boundary, previous_grapheme_boundary, previous_word_boundary};
use crate::selection::SelectionRange;
use crate::state::{EditorState, Transaction, TransactionKind};

use super::pair::closing_of;
use super::{RangeEdit, edit_each};

pub fn delete_backward(state: &EditorState) -> Option<Transaction> {
    edit_each(state, TransactionKind::Delete, |text, range| {
        deletion(range, || {
            let position = range.head;
            let pair = text
                .char_before(position)
                .zip(text.char_after(position))
                .filter(|(before, after)| closing_of(*before) == Some(*after));
            match pair {
                Some((before, after)) => position - before.len_utf8()..position + after.len_utf8(),
                None => previous_grapheme_boundary(text, position)..position,
            }
        })
    })
}

pub fn delete_forward(state: &EditorState) -> Option<Transaction> {
    edit_each(state, TransactionKind::Delete, |text, range| {
        deletion(range, || {
            range.head..next_grapheme_boundary(text, range.head)
        })
    })
}

pub fn delete_word_backward(state: &EditorState) -> Option<Transaction> {
    edit_each(state, TransactionKind::Delete, |text, range| {
        deletion(range, || {
            previous_word_boundary(text, range.head)..range.head
        })
    })
}

fn deletion(
    range: &SelectionRange,
    cursor_deletion: impl FnOnce() -> std::ops::Range<usize>,
) -> Option<RangeEdit> {
    let deleted = if range.is_empty() {
        cursor_deletion()
    } else {
        range.from()..range.to()
    };
    (!deleted.is_empty()).then(|| RangeEdit::collapsed(Change::delete(deleted)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{expect, expect_nothing};

    #[test]
    fn backspace_deletes_the_previous_grapheme() {
        expect("ab|c", delete_backward, "a|c");
        expect("ae\u{301}|c", delete_backward, "a|c");
        expect("a日|", delete_backward, "a|");
    }

    #[test]
    fn backspace_joins_lines() {
        expect("a\n|b", delete_backward, "a|b");
    }

    #[test]
    fn backspace_deletes_the_selection() {
        expect("a«bc»d", delete_backward, "a|d");
    }

    #[test]
    fn backspace_at_the_start_does_nothing() {
        expect_nothing("|ab", delete_backward);
    }

    #[test]
    fn backspace_between_a_pair_removes_both() {
        expect("a(|)b", delete_backward, "a|b");
        expect("[|]", delete_backward, "|");
        expect("{|}", delete_backward, "|");
        expect("\"|\"", delete_backward, "|");
        expect("`|`", delete_backward, "|");
    }

    #[test]
    fn backspace_between_mismatched_chars_deletes_one() {
        expect("(|]", delete_backward, "|]");
        expect("(a|)", delete_backward, "(|)");
    }

    #[test]
    fn backspace_applies_to_every_cursor_and_skips_blocked_ones() {
        expect("|ab|c", delete_backward, "|a|c");
    }

    #[test]
    fn delete_forward_removes_the_next_grapheme() {
        expect("a|bc", delete_forward, "a|c");
        expect("|e\u{301}x", delete_forward, "|x");
        expect("a|\nb", delete_forward, "a|b");
    }

    #[test]
    fn delete_forward_at_the_end_does_nothing() {
        expect_nothing("ab|", delete_forward);
    }

    #[test]
    fn delete_forward_deletes_the_selection() {
        expect("a«bc»d", delete_forward, "a|d");
    }

    #[test]
    fn delete_forward_with_adjacent_cursors_removes_both_neighbours() {
        expect("|a|bc", delete_forward, "|c");
    }

    #[test]
    fn delete_word_backward_removes_the_word_before_the_cursor() {
        expect("one two|", delete_word_backward, "one |");
        expect("one tw|o", delete_word_backward, "one |o");
        expect("one  |", delete_word_backward, "|");
    }

    #[test]
    fn delete_word_backward_with_a_selection_deletes_it() {
        expect("one «two»", delete_word_backward, "one |");
    }

    #[test]
    fn delete_word_backward_at_the_start_does_nothing() {
        expect_nothing("|one", delete_word_backward);
    }

    #[test]
    fn delete_word_backward_with_overlapping_words_does_not_overlap() {
        expect("ab|cd|", delete_word_backward, "|");
    }

    #[test]
    fn deletions_have_the_delete_kind() {
        let state = crate::testing::state("a|b");
        assert_eq!(
            delete_backward(&state).map(|t| t.kind),
            Some(TransactionKind::Delete)
        );
    }
}
