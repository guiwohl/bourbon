use std::time::Duration;

use crate::change::ChangeSet;
use crate::selection::Selection;
use crate::state::{EditorState, Transaction, TransactionKind};

const DEFAULT_GROUP_WINDOW: Duration = Duration::from_millis(500);

#[derive(Debug, Clone)]
struct Entry {
    inverse: ChangeSet,
    selection_before: Selection,
}

#[derive(Debug, Clone, Copy)]
struct LastEdit {
    kind: TransactionKind,
    at: Duration,
}

#[derive(Debug, Clone)]
pub struct History {
    undo_stack: Vec<Entry>,
    redo_stack: Vec<Entry>,
    last_edit: Option<LastEdit>,
    group_window: Duration,
}

impl Default for History {
    fn default() -> Self {
        Self::with_group_window(DEFAULT_GROUP_WINDOW)
    }
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_group_window(group_window: Duration) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_edit: None,
            group_window,
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn record(&mut self, before: &EditorState, transaction: &Transaction, now: Duration) {
        if transaction.changes.is_empty()
            || matches!(
                transaction.kind,
                TransactionKind::Undo | TransactionKind::Redo
            )
        {
            return;
        }
        self.redo_stack.clear();
        let inverse = transaction.changes.invert(&before.text);
        let grouped = self.groups_with_previous(transaction, now);
        match self.undo_stack.last_mut() {
            Some(top) if grouped => top.inverse = inverse.compose(&top.inverse),
            _ => self.undo_stack.push(Entry {
                inverse,
                selection_before: before.selection.clone(),
            }),
        }
        self.last_edit = (!transaction.isolate).then_some(LastEdit {
            kind: transaction.kind,
            at: now,
        });
    }

    pub fn undo(&mut self, state: &EditorState) -> Option<Transaction> {
        self.last_edit = None;
        step(
            &mut self.undo_stack,
            &mut self.redo_stack,
            state,
            TransactionKind::Undo,
        )
    }

    pub fn redo(&mut self, state: &EditorState) -> Option<Transaction> {
        self.last_edit = None;
        step(
            &mut self.redo_stack,
            &mut self.undo_stack,
            state,
            TransactionKind::Redo,
        )
    }

    fn groups_with_previous(&self, transaction: &Transaction, now: Duration) -> bool {
        !transaction.isolate
            && matches!(
                transaction.kind,
                TransactionKind::Input | TransactionKind::Delete
            )
            && self.last_edit.is_some_and(|last| {
                last.kind == transaction.kind && now.saturating_sub(last.at) <= self.group_window
            })
    }
}

fn step(
    from: &mut Vec<Entry>,
    to: &mut Vec<Entry>,
    state: &EditorState,
    kind: TransactionKind,
) -> Option<Transaction> {
    let entry = from.pop()?;
    to.push(Entry {
        inverse: entry.inverse.invert(&state.text),
        selection_before: state.selection.clone(),
    });
    Some(Transaction {
        changes: entry.inverse,
        selection_after: entry.selection_before,
        kind,
        isolate: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change::Change;
    use crate::text::Text;

    fn at(milliseconds: u64) -> Duration {
        Duration::from_millis(milliseconds)
    }

    fn typing(state: &EditorState, text: &str, kind: TransactionKind) -> Transaction {
        let position = state.selection.main().head;
        Transaction::new(
            ChangeSet::new(vec![Change::insert(position, text)]),
            Selection::cursor(position + text.len()),
            kind,
        )
    }

    fn type_and_record(
        history: &mut History,
        state: &mut EditorState,
        text: &str,
        kind: TransactionKind,
        now: u64,
    ) {
        let transaction = typing(state, text, kind);
        history.record(state, &transaction, at(now));
        *state = state.apply(&transaction);
    }

    fn start(text: &str, cursor: usize) -> EditorState {
        EditorState::new(Text::new(text), Selection::cursor(cursor))
    }

    fn undo(history: &mut History, state: &mut EditorState) -> bool {
        match history.undo(state) {
            Some(transaction) => {
                *state = state.apply(&transaction);
                true
            }
            None => false,
        }
    }

    fn redo(history: &mut History, state: &mut EditorState) -> bool {
        match history.redo(state) {
            Some(transaction) => {
                *state = state.apply(&transaction);
                true
            }
            None => false,
        }
    }

    #[test]
    fn nothing_to_undo_or_redo_initially() {
        let mut history = History::new();
        let state = start("a", 0);
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert!(history.undo(&state).is_none());
        assert!(history.redo(&state).is_none());
    }

    #[test]
    fn undo_restores_text_and_selection() {
        let mut history = History::new();
        let original = start("ab", 1);
        let mut state = original.clone();
        type_and_record(&mut history, &mut state, "X", TransactionKind::Input, 0);
        assert!(undo(&mut history, &mut state));
        assert_eq!(state, original);
    }

    #[test]
    fn typing_within_the_window_is_one_undo_step() {
        let mut history = History::new();
        let original = start("", 0);
        let mut state = original.clone();
        for (index, letter) in ["a", "b", "c"].into_iter().enumerate() {
            type_and_record(
                &mut history,
                &mut state,
                letter,
                TransactionKind::Input,
                index as u64 * 400,
            );
        }
        assert_eq!(state.text.to_string(), "abc");
        assert!(undo(&mut history, &mut state));
        assert_eq!(state, original);
        assert!(!history.can_undo());
    }

    #[test]
    fn the_window_slides_with_each_edit() {
        let mut history = History::new();
        let mut state = start("", 0);
        for (index, letter) in ["a", "b", "c"].into_iter().enumerate() {
            type_and_record(
                &mut history,
                &mut state,
                letter,
                TransactionKind::Input,
                index as u64 * 500,
            );
        }
        assert!(undo(&mut history, &mut state));
        assert_eq!(state.text.to_string(), "");
    }

    #[test]
    fn a_pause_longer_than_the_window_starts_a_new_step() {
        let mut history = History::new();
        let mut state = start("", 0);
        type_and_record(&mut history, &mut state, "a", TransactionKind::Input, 0);
        type_and_record(&mut history, &mut state, "b", TransactionKind::Input, 501);
        assert!(undo(&mut history, &mut state));
        assert_eq!(state.text.to_string(), "a");
    }

    #[test]
    fn custom_window_is_honoured() {
        let mut history = History::with_group_window(at(50));
        let mut state = start("", 0);
        type_and_record(&mut history, &mut state, "a", TransactionKind::Input, 0);
        type_and_record(&mut history, &mut state, "b", TransactionKind::Input, 51);
        assert!(undo(&mut history, &mut state));
        assert_eq!(state.text.to_string(), "a");
    }

    #[test]
    fn different_kinds_do_not_group() {
        let mut history = History::new();
        let mut state = start("", 0);
        type_and_record(&mut history, &mut state, "a", TransactionKind::Input, 0);
        type_and_record(&mut history, &mut state, "b", TransactionKind::Delete, 10);
        assert!(undo(&mut history, &mut state));
        assert_eq!(state.text.to_string(), "a");
    }

    #[test]
    fn paste_and_command_never_group() {
        for kind in [TransactionKind::Paste, TransactionKind::Command] {
            let mut history = History::new();
            let mut state = start("", 0);
            type_and_record(&mut history, &mut state, "a", kind, 0);
            type_and_record(&mut history, &mut state, "b", kind, 10);
            assert!(undo(&mut history, &mut state));
            assert_eq!(state.text.to_string(), "a");
        }
    }

    #[test]
    fn delete_kind_groups_with_delete() {
        let mut history = History::new();
        let original = start("abc", 3);
        let mut state = original.clone();
        for now in [0, 100] {
            let end = state.selection.main().head;
            let transaction = Transaction::new(
                ChangeSet::new(vec![Change::delete(end - 1..end)]),
                Selection::cursor(end - 1),
                TransactionKind::Delete,
            );
            history.record(&state, &transaction, at(now));
            state = state.apply(&transaction);
        }
        assert!(undo(&mut history, &mut state));
        assert_eq!(state, original);
    }

    #[test]
    fn isolate_breaks_grouping_on_both_sides() {
        let mut history = History::new();
        let mut state = start("", 0);
        type_and_record(&mut history, &mut state, "a", TransactionKind::Input, 0);
        let mut isolated = typing(&state, "b", TransactionKind::Input);
        isolated.isolate = true;
        history.record(&state, &isolated, at(10));
        state = state.apply(&isolated);
        type_and_record(&mut history, &mut state, "c", TransactionKind::Input, 20);
        assert!(undo(&mut history, &mut state));
        assert_eq!(state.text.to_string(), "ab");
        assert!(undo(&mut history, &mut state));
        assert_eq!(state.text.to_string(), "a");
        assert!(undo(&mut history, &mut state));
        assert_eq!(state.text.to_string(), "");
    }

    #[test]
    fn redo_reapplies_with_the_selection_after_the_edit() {
        let mut history = History::new();
        let mut state = start("ab", 1);
        type_and_record(&mut history, &mut state, "X", TransactionKind::Input, 0);
        let edited = state.clone();
        assert!(undo(&mut history, &mut state));
        assert!(history.can_redo());
        assert!(redo(&mut history, &mut state));
        assert_eq!(state, edited);
        assert!(!history.can_redo());
        assert!(history.can_undo());
    }

    #[test]
    fn a_new_edit_clears_the_redo_stack() {
        let mut history = History::new();
        let mut state = start("", 0);
        type_and_record(&mut history, &mut state, "a", TransactionKind::Input, 0);
        assert!(undo(&mut history, &mut state));
        type_and_record(&mut history, &mut state, "b", TransactionKind::Input, 10);
        assert!(!history.can_redo());
    }

    #[test]
    fn typing_after_an_undo_starts_a_new_step() {
        let mut history = History::new();
        let mut state = start("", 0);
        type_and_record(&mut history, &mut state, "a", TransactionKind::Input, 0);
        type_and_record(&mut history, &mut state, "b", TransactionKind::Input, 10);
        type_and_record(&mut history, &mut state, "c", TransactionKind::Paste, 20);
        assert!(undo(&mut history, &mut state));
        type_and_record(&mut history, &mut state, "d", TransactionKind::Input, 30);
        assert!(undo(&mut history, &mut state));
        assert_eq!(state.text.to_string(), "ab");
    }

    #[test]
    fn undo_and_redo_transactions_are_not_recorded() {
        let mut history = History::new();
        let mut state = start("", 0);
        type_and_record(&mut history, &mut state, "a", TransactionKind::Input, 0);
        let transaction = history.undo(&state).expect("undo available");
        assert_eq!(transaction.kind, TransactionKind::Undo);
        assert!(transaction.isolate);
        history.record(&state, &transaction, at(5));
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn selection_only_transactions_are_ignored() {
        let mut history = History::new();
        let state = start("abc", 0);
        let transaction = Transaction::new(
            ChangeSet::default(),
            Selection::cursor(2),
            TransactionKind::Command,
        );
        history.record(&state, &transaction, at(0));
        assert!(!history.can_undo());
    }
}
