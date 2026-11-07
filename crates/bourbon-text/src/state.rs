use crate::change::ChangeSet;
use crate::selection::Selection;
use crate::text::Text;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionKind {
    Input,
    Delete,
    Paste,
    Command,
    Undo,
    Redo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub changes: ChangeSet,
    pub selection_after: Selection,
    pub kind: TransactionKind,
    pub isolate: bool,
}

impl Transaction {
    pub fn new(changes: ChangeSet, selection_after: Selection, kind: TransactionKind) -> Self {
        Self {
            changes,
            selection_after,
            kind,
            isolate: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct EditorState {
    pub text: Text,
    pub selection: Selection,
}

impl EditorState {
    pub fn new(text: Text, selection: Selection) -> Self {
        Self { text, selection }
    }

    pub fn apply(&self, transaction: &Transaction) -> EditorState {
        EditorState {
            text: transaction.changes.apply(&self.text),
            selection: transaction.selection_after.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change::Change;

    #[test]
    fn apply_edits_text_and_replaces_selection() {
        let state = EditorState::new(Text::new("abc"), Selection::cursor(1));
        let transaction = Transaction::new(
            ChangeSet::new(vec![Change::insert(1, "XY")]),
            Selection::cursor(3),
            TransactionKind::Input,
        );
        let next = state.apply(&transaction);
        assert_eq!(next.text.to_string(), "aXYbc");
        assert_eq!(next.selection, Selection::cursor(3));
        assert_eq!(state.text.to_string(), "abc");
    }

    #[test]
    fn new_transactions_are_not_isolated() {
        let transaction = Transaction::new(
            ChangeSet::default(),
            Selection::cursor(0),
            TransactionKind::Command,
        );
        assert!(!transaction.isolate);
    }
}
