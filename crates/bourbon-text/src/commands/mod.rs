mod delete;
mod insert;
mod list;
mod newline;
mod pair;
mod task;
mod wrap;

use std::collections::BTreeSet;
use std::ops::Range;

use crate::change::{Change, ChangeSet};
use crate::selection::{Selection, SelectionRange};
use crate::state::{EditorState, Transaction, TransactionKind};
use crate::text::Text;

pub use delete::{delete_backward, delete_forward, delete_word_backward};
pub use insert::{insert_text, type_character};
pub use list::{indent_list_item, outdent_list_item};
pub use newline::insert_newline_continue_markup;
pub use task::toggle_task;
pub use wrap::{toggle_bold, toggle_inline_code, toggle_italic, toggle_strikethrough, toggle_wrap};

struct RangeEdit {
    change: Change,
    anchor: usize,
    head: usize,
}

impl RangeEdit {
    fn collapsed(change: Change) -> Self {
        let end = change.text.len();
        Self {
            change,
            anchor: end,
            head: end,
        }
    }

    fn selecting(change: Change, selected: Range<usize>, reversed: bool) -> Self {
        let (anchor, head) = if reversed {
            (selected.end, selected.start)
        } else {
            (selected.start, selected.end)
        };
        Self {
            change,
            anchor,
            head,
        }
    }
}

fn edit_each(
    state: &EditorState,
    kind: TransactionKind,
    edit: impl Fn(&Text, &SelectionRange) -> Option<RangeEdit>,
) -> Option<Transaction> {
    let mut changes = Vec::new();
    let mut ranges = Vec::new();
    let mut inserted = 0;
    let mut removed = 0;
    let mut covered_end = 0;
    for range in state.selection.ranges() {
        let Some(edit) = edit(&state.text, range) else {
            let shift = |position: usize| (position + inserted).saturating_sub(removed);
            ranges.push(SelectionRange::new(shift(range.anchor), shift(range.head)));
            continue;
        };
        let start = edit.change.range.start.max(covered_end);
        let end = edit.change.range.end.max(start);
        let new_start = start + inserted - removed;
        ranges.push(SelectionRange::new(
            new_start + edit.anchor,
            new_start + edit.head,
        ));
        inserted += edit.change.text.len();
        removed += end - start;
        covered_end = end;
        let change = Change::replace(start..end, edit.change.text);
        if !change.is_noop() {
            changes.push(change);
        }
    }
    let selection = Selection::new(ranges, state.selection.main_index());
    if changes.is_empty() && selection == state.selection {
        return None;
    }
    Some(Transaction::new(ChangeSet::new(changes), selection, kind))
}

fn selected_lines(text: &Text, selection: &Selection) -> Vec<usize> {
    let mut lines = BTreeSet::new();
    for range in selection.ranges() {
        let first = text.line_of(range.from());
        let mut last = text.line_of(range.to());
        if !range.is_empty() && range.to() == text.line_start(last) {
            last -= 1;
        }
        lines.extend(first..=last);
    }
    lines.into_iter().collect()
}

fn line_change(line_start: usize, old: &str, new: &str) -> Option<Change> {
    if old == new {
        return None;
    }
    let prefix: usize = old
        .chars()
        .zip(new.chars())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a.len_utf8())
        .sum();
    let suffix: usize = old[prefix..]
        .chars()
        .rev()
        .zip(new[prefix..].chars().rev())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a.len_utf8())
        .sum();
    Some(Change::replace(
        line_start + prefix..line_start + old.len() - suffix,
        &new[prefix..new.len() - suffix],
    ))
}

fn line_edits_transaction(state: &EditorState, changes: Vec<Change>) -> Option<Transaction> {
    if changes.is_empty() {
        return None;
    }
    let changes = ChangeSet::new(changes);
    let selection = state.selection.map(&changes);
    Some(Transaction::new(
        changes,
        selection,
        TransactionKind::Command,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{render, state};

    #[test]
    fn line_change_isolates_the_differing_middle() {
        assert_eq!(
            line_change(10, "- a", "-  a"),
            Some(Change::insert(12, " "))
        );
        assert_eq!(
            line_change(0, "9. x", "10. x"),
            Some(Change::replace(0..1, "10"))
        );
        assert_eq!(line_change(5, "same", "same"), None);
    }

    #[test]
    fn line_change_respects_multibyte_chars() {
        assert_eq!(
            line_change(0, "é1é", "é22é"),
            Some(Change::replace(2..3, "22"))
        );
    }

    #[test]
    fn selected_lines_cover_ranges_and_dedupe() {
        let state = state("a|a\nb«b\ncc»\ndd|");
        assert_eq!(
            selected_lines(&state.text, &state.selection),
            vec![0, 1, 2, 3]
        );
        let state = self::state("a|a|\nb");
        assert_eq!(selected_lines(&state.text, &state.selection), vec![0]);
    }

    #[test]
    fn selected_lines_exclude_a_line_selected_only_at_its_start() {
        let state = state("«aa\n»bb");
        assert_eq!(selected_lines(&state.text, &state.selection), vec![0]);
        let reversed = self::state("»aa\n«bb");
        assert_eq!(selected_lines(&reversed.text, &reversed.selection), vec![0]);
    }

    #[test]
    fn edit_each_keeps_unedited_ranges_shifted() {
        let state = state("|a|b");
        let transaction = edit_each(&state, TransactionKind::Input, |_, range| {
            (range.head == 0).then(|| RangeEdit::collapsed(Change::insert(0, "xyz")))
        })
        .expect("edit");
        assert_eq!(render(&state.apply(&transaction)), "xyz|a|b");
    }

    #[test]
    fn edit_each_clamps_overlapping_deletions() {
        let state = state("abc|d|ef");
        let transaction = edit_each(&state, TransactionKind::Delete, |_, range| {
            Some(RangeEdit::collapsed(Change::delete(0..range.head)))
        })
        .expect("edit");
        assert_eq!(render(&state.apply(&transaction)), "|ef");
    }

    #[test]
    fn edit_each_reports_nothing_when_nothing_changes() {
        let state = state("a|b");
        assert!(edit_each(&state, TransactionKind::Input, |_, _| None).is_none());
    }
}
