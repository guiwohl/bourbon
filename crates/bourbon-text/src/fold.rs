use std::ops::Range;

use crate::change::{Assoc, ChangeSet};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Folds {
    hidden: Vec<Range<usize>>,
}

impl Folds {
    pub fn ranges(&self) -> &[Range<usize>] {
        &self.hidden
    }

    pub fn fold(&mut self, hidden: Range<usize>) {
        if !self.hidden.contains(&hidden) {
            self.hidden.push(hidden);
            self.hidden.sort_by_key(|range| (range.start, range.end));
        }
    }

    pub fn unfold(&mut self, start: usize) -> bool {
        let before = self.hidden.len();
        self.hidden.retain(|range| range.start != start);
        self.hidden.len() != before
    }

    pub fn unfold_hiding(&mut self, offset: usize) -> Vec<Range<usize>> {
        let (revealed, kept) = self
            .hidden
            .drain(..)
            .partition(|range| range.start < offset && offset <= range.end);
        self.hidden = kept;
        revealed
    }

    pub fn is_folded_at(&self, start: usize) -> bool {
        self.hidden.iter().any(|range| range.start == start)
    }

    pub fn hides(&self, offset: usize) -> bool {
        self.hidden
            .iter()
            .any(|range| range.start < offset && offset <= range.end)
    }

    pub fn clear(&mut self) {
        self.hidden.clear();
    }

    #[must_use]
    pub fn map(&self, changes: &ChangeSet) -> Self {
        let hidden =
            self.hidden
                .iter()
                .filter(|range| {
                    !changes.changes().iter().any(|change| {
                        change.range.end > range.start && change.range.start <= range.end
                    })
                })
                .map(|range| {
                    changes.map_position(range.start, Assoc::After)
                        ..changes.map_position(range.end, Assoc::Before)
                })
                .collect();
        Self { hidden }
    }

    pub fn visible_position(&self, offset: usize, forward: bool, text_len: usize) -> usize {
        let mut position = offset;
        for _ in 0..=self.hidden.len() {
            let Some(range) = self
                .hidden
                .iter()
                .find(|range| range.start < position && position <= range.end)
            else {
                break;
            };
            position = if forward && range.end < text_len {
                range.end + 1
            } else {
                range.start
            };
        }
        position
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change::Change;

    fn folds(ranges: &[(usize, usize)]) -> Folds {
        let mut folds = Folds::default();
        for (start, end) in ranges {
            folds.fold(*start..*end);
        }
        folds
    }

    fn pairs(folds: &Folds) -> Vec<(usize, usize)> {
        folds
            .ranges()
            .iter()
            .map(|range| (range.start, range.end))
            .collect()
    }

    #[test]
    fn folding_twice_keeps_one_range_in_order() {
        let folds = folds(&[(10, 20), (2, 5), (10, 20)]);
        assert_eq!(pairs(&folds), [(2, 5), (10, 20)]);
    }

    #[test]
    fn unfolding_removes_the_fold_starting_there() {
        let mut folds = folds(&[(2, 5), (10, 20)]);
        assert!(folds.is_folded_at(10));
        assert!(folds.unfold(10));
        assert!(!folds.is_folded_at(10));
        assert!(!folds.unfold(10));
        assert_eq!(pairs(&folds), [(2, 5)]);
    }

    #[test]
    fn unfolding_what_hides_an_offset_returns_every_enclosing_fold() {
        let mut folds = folds(&[(2, 5), (10, 40), (15, 20)]);
        let revealed: Vec<(usize, usize)> = folds
            .unfold_hiding(17)
            .iter()
            .map(|range| (range.start, range.end))
            .collect();
        assert_eq!(revealed, [(10, 40), (15, 20)]);
        assert_eq!(pairs(&folds), [(2, 5)]);
        assert_eq!(folds.unfold_hiding(2), Vec::<Range<usize>>::new());
        assert_eq!(pairs(&folds), [(2, 5)]);
    }

    #[test]
    fn the_start_stays_visible_and_the_end_is_hidden() {
        let folds = folds(&[(2, 5)]);
        assert!(!folds.hides(2));
        assert!(folds.hides(3));
        assert!(folds.hides(5));
        assert!(!folds.hides(6));
    }

    #[test]
    fn clearing_drops_every_fold() {
        let mut folds = folds(&[(2, 5)]);
        folds.clear();
        assert_eq!(pairs(&folds), []);
    }

    #[test]
    fn edits_outside_a_fold_shift_it() {
        let folds = folds(&[(10, 20)]);
        let before = ChangeSet::new(vec![Change::insert(3, "abc")]);
        assert_eq!(pairs(&folds.map(&before)), [(13, 23)]);
        let after = ChangeSet::new(vec![Change::insert(21, "x")]);
        assert_eq!(pairs(&folds.map(&after)), [(10, 20)]);
    }

    #[test]
    fn typing_at_the_end_of_the_visible_line_keeps_the_fold_after_it() {
        let folds = folds(&[(10, 20)]);
        let typed = ChangeSet::new(vec![Change::insert(10, "ab")]);
        assert_eq!(pairs(&folds.map(&typed)), [(12, 22)]);
        let deleted = ChangeSet::new(vec![Change::delete(8..10)]);
        assert_eq!(pairs(&folds.map(&deleted)), [(8, 18)]);
    }

    #[test]
    fn an_edit_touching_the_hidden_text_drops_the_fold() {
        let folds = folds(&[(10, 20), (30, 40)]);
        for change in [
            Change::delete(10..11),
            Change::insert(15, "x"),
            Change::insert(20, "x"),
            Change::delete(20..21),
            Change::replace(5..12, "y"),
        ] {
            let mapped = folds.map(&ChangeSet::new(vec![change.clone()]));
            assert_eq!(mapped.ranges().len(), 1, "{change:?}");
        }
    }

    #[test]
    fn a_hidden_position_moves_past_the_fold_or_back_to_its_start() {
        let folds = folds(&[(10, 20)]);
        assert_eq!(folds.visible_position(15, true, 50), 21);
        assert_eq!(folds.visible_position(15, false, 50), 10);
        assert_eq!(folds.visible_position(20, true, 20), 10);
        assert_eq!(folds.visible_position(7, true, 50), 7);
        assert_eq!(folds.visible_position(10, true, 50), 10);
    }

    #[test]
    fn nested_folds_are_left_through_the_outermost() {
        let folds = folds(&[(10, 40), (15, 20)]);
        assert_eq!(folds.visible_position(17, true, 50), 41);
        assert_eq!(folds.visible_position(17, false, 50), 10);
    }

    #[test]
    fn overlapping_folds_are_left_in_a_bounded_number_of_steps() {
        let folds = folds(&[(10, 20), (18, 30)]);
        assert_eq!(folds.visible_position(15, true, 50), 31);
        assert_eq!(folds.visible_position(25, false, 50), 10);
    }
}
