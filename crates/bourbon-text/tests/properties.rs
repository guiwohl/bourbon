use std::time::Duration;

use bourbon_text::{
    Assoc, Change, ChangeSet, EditorState, History, Selection, Text, Transaction, TransactionKind,
};
use proptest::prelude::*;

fn content() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        prop_oneof![
            Just('a'),
            Just('b'),
            Just(' '),
            Just('\n'),
            Just('é'),
            Just('日'),
            Just('-')
        ],
        0..40,
    )
    .prop_map(|characters| characters.into_iter().collect())
}

fn byte_offset(content: &str, char_index: usize) -> usize {
    content
        .char_indices()
        .nth(char_index)
        .map_or(content.len(), |(offset, _)| offset)
}

fn change_set(content: &str, shape: &[(usize, usize, String)]) -> ChangeSet {
    let chars = content.chars().count();
    let mut cursor = 0;
    let mut changes = Vec::new();
    for (gap, length, insert) in shape {
        let start = (cursor + gap).min(chars);
        let end = (start + length).min(chars);
        changes.push(Change::replace(
            byte_offset(content, start)..byte_offset(content, end),
            insert.as_str(),
        ));
        cursor = end;
    }
    ChangeSet::new(changes)
}

fn shape() -> impl Strategy<Value = Vec<(usize, usize, String)>> {
    proptest::collection::vec((0usize..6, 0usize..4, "[a-z日é\n]{0,4}"), 0..5)
}

proptest! {
    #[test]
    fn apply_then_invert_is_identity(original in content(), shape in shape()) {
        let text = Text::new(&original);
        let changes = change_set(&original, &shape);
        let edited = changes.apply(&text);
        prop_assert_eq!(changes.invert(&text).apply(&edited), text);
    }

    #[test]
    fn composed_changes_equal_applying_in_sequence(
        original in content(),
        first_shape in shape(),
        second_shape in shape(),
    ) {
        let text = Text::new(&original);
        let first = change_set(&original, &first_shape);
        let middle = first.apply(&text);
        let second = change_set(&middle.to_string(), &second_shape);
        prop_assert_eq!(first.compose(&second).apply(&text), second.apply(&middle));
    }

    #[test]
    fn mapped_positions_stay_in_bounds_and_keep_order(
        original in content(),
        shape in shape(),
        first in 0usize..45,
        second in 0usize..45,
    ) {
        let text = Text::new(&original);
        let changes = change_set(&original, &shape);
        let new_len = changes.apply(&text).len();
        let low = byte_offset(&original, first.min(second));
        let high = byte_offset(&original, first.max(second));
        for assoc in [Assoc::Before, Assoc::After] {
            let mapped_low = changes.map_position(low, assoc);
            let mapped_high = changes.map_position(high, assoc);
            prop_assert!(mapped_low <= mapped_high);
            prop_assert!(mapped_high <= new_len);
        }
    }

    #[test]
    fn undoing_every_edit_restores_text_and_selection(
        original in content(),
        edits in proptest::collection::vec(
            (shape(), 0u64..1200, 0usize..4, any::<bool>()),
            0..8,
        ),
    ) {
        let initial = EditorState::new(Text::new(&original), Selection::cursor(0));
        let mut state = initial.clone();
        let mut history = History::new();
        let mut clock = 0;
        let mut states = vec![state.clone()];
        for (edit_shape, pause, kind_index, isolate) in edits {
            clock += pause;
            let kind = [
                TransactionKind::Input,
                TransactionKind::Delete,
                TransactionKind::Paste,
                TransactionKind::Command,
            ][kind_index];
            let changes = change_set(&state.text.to_string(), &edit_shape);
            let position = changes.changes().last().map_or(0, |change| {
                change.range.start + change.text.len()
            });
            let transaction = Transaction {
                selection_after: Selection::cursor(position.min(changes.apply(&state.text).len())),
                changes,
                kind,
                isolate,
            };
            history.record(&state, &transaction, Duration::from_millis(clock));
            state = state.apply(&transaction);
            states.push(state.clone());
        }
        let edited = state.clone();
        while let Some(undo) = history.undo(&state) {
            state = state.apply(&undo);
        }
        prop_assert_eq!(&state, &initial);
        while let Some(redo) = history.redo(&state) {
            state = state.apply(&redo);
        }
        prop_assert_eq!(&state, &edited);
    }
}
