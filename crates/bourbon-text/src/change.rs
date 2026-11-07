use std::ops::Range;

use crate::compose;
use crate::text::Text;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Assoc {
    Before,
    After,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub range: Range<usize>,
    pub text: String,
}

impl Change {
    pub fn replace(range: Range<usize>, text: impl Into<String>) -> Self {
        Self {
            range,
            text: text.into(),
        }
    }

    pub fn insert(at: usize, text: impl Into<String>) -> Self {
        Self::replace(at..at, text)
    }

    pub fn delete(range: Range<usize>) -> Self {
        Self::replace(range, "")
    }

    pub fn removed_len(&self) -> usize {
        self.range.end - self.range.start
    }

    pub fn is_noop(&self) -> bool {
        self.range.is_empty() && self.text.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[must_use]
pub struct ChangeSet {
    changes: Vec<Change>,
}

impl ChangeSet {
    pub fn new(mut changes: Vec<Change>) -> Self {
        changes.sort_by_key(|change| (change.range.start, change.range.end));
        debug_assert!(
            changes
                .windows(2)
                .all(|pair| pair[0].range.end <= pair[1].range.start)
        );
        Self { changes }
    }

    pub fn changes(&self) -> &[Change] {
        &self.changes
    }

    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    pub fn apply(&self, text: &Text) -> Text {
        let mut result = text.clone();
        for change in self.changes.iter().rev() {
            result.splice(change.range.clone(), &change.text);
        }
        result
    }

    pub fn invert(&self, text: &Text) -> ChangeSet {
        let mut inserted = 0;
        let mut removed = 0;
        let changes = self
            .changes
            .iter()
            .map(|change| {
                let start = change.range.start + inserted - removed;
                inserted += change.text.len();
                removed += change.removed_len();
                Change::replace(
                    start..start + change.text.len(),
                    text.slice(change.range.clone()),
                )
            })
            .collect();
        ChangeSet { changes }
    }

    pub fn compose(&self, later: &ChangeSet) -> ChangeSet {
        compose::compose(self, later)
    }

    pub fn map_position(&self, position: usize, assoc: Assoc) -> usize {
        let mut inserted = 0;
        let mut removed = 0;
        for change in &self.changes {
            let Range { start, end } = change.range;
            if position < start {
                break;
            }
            if start == end && position == start {
                if assoc == Assoc::Before {
                    break;
                }
            } else if position < end {
                let mapped_start = start + inserted - removed;
                return if assoc == Assoc::After && position > start {
                    mapped_start + change.text.len()
                } else {
                    mapped_start
                };
            }
            inserted += change.text.len();
            removed += change.removed_len();
        }
        position + inserted - removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(changes: Vec<Change>) -> ChangeSet {
        ChangeSet::new(changes)
    }

    #[test]
    fn apply_replaces_every_change_against_the_original_offsets() {
        let changes = set(vec![Change::replace(0..1, "XX"), Change::delete(3..5)]);
        assert_eq!(changes.apply(&Text::new("abcdef")).to_string(), "XXbcf");
    }

    #[test]
    fn apply_sorts_changes_given_out_of_order() {
        let changes = set(vec![Change::insert(3, "-"), Change::insert(1, "+")]);
        assert_eq!(changes.apply(&Text::new("abcd")).to_string(), "a+bc-d");
    }

    #[test]
    fn insertions_at_the_same_offset_keep_their_order() {
        let changes = set(vec![Change::insert(1, "x"), Change::insert(1, "y")]);
        assert_eq!(changes.apply(&Text::new("ab")).to_string(), "axyb");
    }

    #[test]
    fn insertion_sorts_before_a_replacement_starting_at_the_same_offset() {
        let changes = set(vec![Change::replace(1..3, "Z"), Change::insert(1, "i")]);
        assert_eq!(changes.apply(&Text::new("abcd")).to_string(), "aiZd");
    }

    #[test]
    fn empty_set_is_identity() {
        let text = Text::new("abc");
        assert!(ChangeSet::default().is_empty());
        assert_eq!(ChangeSet::default().apply(&text), text);
    }

    #[test]
    fn noop_change_is_detected() {
        assert!(Change::insert(2, "").is_noop());
        assert!(!Change::insert(2, "a").is_noop());
        assert!(!Change::delete(1..2).is_noop());
    }

    #[test]
    fn invert_restores_the_original_text() {
        let text = Text::new("héllo wörld");
        let changes = set(vec![
            Change::replace(0..3, "J"),
            Change::insert(5, "!!"),
            Change::delete(7..10),
        ]);
        let edited = changes.apply(&text);
        assert_eq!(changes.invert(&text).apply(&edited), text);
    }

    #[test]
    fn invert_uses_offsets_of_the_edited_text() {
        let text = Text::new("abcdef");
        let changes = set(vec![Change::replace(0..1, "XXX"), Change::delete(4..5)]);
        let inverse = changes.invert(&text);
        assert_eq!(
            inverse.changes(),
            &[Change::replace(0..3, "a"), Change::insert(6, "e")]
        );
    }

    #[test]
    fn map_position_before_a_change_is_unchanged() {
        let changes = set(vec![Change::insert(5, "xx")]);
        assert_eq!(changes.map_position(2, Assoc::After), 2);
    }

    #[test]
    fn map_position_after_a_change_shifts() {
        let changes = set(vec![Change::insert(1, "xx"), Change::delete(3..5)]);
        assert_eq!(changes.map_position(9, Assoc::Before), 9);
        let changes = set(vec![Change::insert(1, "xx")]);
        assert_eq!(changes.map_position(4, Assoc::Before), 6);
        let changes = set(vec![Change::delete(1..3)]);
        assert_eq!(changes.map_position(4, Assoc::Before), 2);
    }

    #[test]
    fn map_position_at_an_insertion_follows_assoc() {
        let changes = set(vec![Change::insert(3, "xyz")]);
        assert_eq!(changes.map_position(3, Assoc::Before), 3);
        assert_eq!(changes.map_position(3, Assoc::After), 6);
    }

    #[test]
    fn map_position_passes_every_insertion_at_the_same_offset_when_after() {
        let changes = set(vec![Change::insert(3, "ab"), Change::insert(3, "c")]);
        assert_eq!(changes.map_position(3, Assoc::After), 6);
        assert_eq!(changes.map_position(3, Assoc::Before), 3);
    }

    #[test]
    fn map_position_inside_a_replacement_goes_to_an_edge() {
        let changes = set(vec![Change::replace(2..6, "XY")]);
        assert_eq!(changes.map_position(4, Assoc::Before), 2);
        assert_eq!(changes.map_position(4, Assoc::After), 4);
    }

    #[test]
    fn map_position_at_replacement_edges_stays_outside() {
        let changes = set(vec![Change::replace(2..6, "XY")]);
        assert_eq!(changes.map_position(2, Assoc::After), 2);
        assert_eq!(changes.map_position(2, Assoc::Before), 2);
        assert_eq!(changes.map_position(6, Assoc::Before), 4);
        assert_eq!(changes.map_position(6, Assoc::After), 4);
    }

    #[test]
    fn map_position_after_end_of_replacement_meets_following_insertion() {
        let changes = set(vec![Change::delete(2..4), Change::insert(4, "Q")]);
        assert_eq!(changes.map_position(4, Assoc::Before), 2);
        assert_eq!(changes.map_position(4, Assoc::After), 3);
    }

    #[test]
    fn compose_of_two_insertions_in_sequence() {
        let first = set(vec![Change::insert(1, "ab")]);
        let second = set(vec![Change::insert(3, "Z")]);
        let text = Text::new("xy");
        let composed = first.compose(&second);
        assert_eq!(composed.apply(&text).to_string(), "xabZy");
        assert_eq!(composed.changes(), &[Change::insert(1, "abZ")]);
    }

    #[test]
    fn compose_cancels_an_insertion_deleted_later() {
        let first = set(vec![Change::insert(1, "abc")]);
        let second = set(vec![Change::delete(1..4)]);
        assert!(first.compose(&second).is_empty());
    }

    #[test]
    fn compose_partially_deletes_an_insertion() {
        let first = set(vec![Change::insert(1, "abc")]);
        let second = set(vec![Change::delete(2..3)]);
        assert_eq!(first.compose(&second).changes(), &[Change::insert(1, "ac")]);
    }

    #[test]
    fn compose_merges_delete_with_later_edit_inside_untouched_text() {
        let text = Text::new("abcdefgh");
        let first = set(vec![Change::delete(1..3)]);
        let second = set(vec![Change::replace(2..3, "Q")]);
        let composed = first.compose(&second);
        assert_eq!(composed.apply(&text).to_string(), "adQfgh");
        assert_eq!(second.apply(&first.apply(&text)), composed.apply(&text));
    }

    #[test]
    fn compose_with_empty_sets_is_identity() {
        let changes = set(vec![Change::replace(1..2, "X")]);
        assert_eq!(changes.compose(&ChangeSet::default()), changes);
        assert_eq!(ChangeSet::default().compose(&changes), changes);
    }

    #[test]
    fn compose_keeps_multibyte_insertions_intact() {
        let first = set(vec![Change::insert(0, "日本")]);
        let second = set(vec![Change::delete(0..3)]);
        assert_eq!(first.compose(&second).changes(), &[Change::insert(0, "本")]);
    }
}
