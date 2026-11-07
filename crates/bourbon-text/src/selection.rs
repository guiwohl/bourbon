use crate::change::{Assoc, ChangeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub struct SelectionRange {
    pub anchor: usize,
    pub head: usize,
}

impl SelectionRange {
    pub fn new(anchor: usize, head: usize) -> Self {
        Self { anchor, head }
    }

    pub fn cursor(position: usize) -> Self {
        Self::new(position, position)
    }

    pub fn from(&self) -> usize {
        self.anchor.min(self.head)
    }

    pub fn to(&self) -> usize {
        self.anchor.max(self.head)
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }

    pub fn is_reversed(&self) -> bool {
        self.head < self.anchor
    }

    pub fn map(&self, changes: &ChangeSet) -> Self {
        if self.is_empty() {
            return Self::cursor(changes.map_position(self.head, Assoc::After));
        }
        let from = changes.map_position(self.from(), Assoc::After);
        let to = changes.map_position(self.to(), Assoc::Before).max(from);
        if self.is_reversed() {
            Self::new(to, from)
        } else {
            Self::new(from, to)
        }
    }

    fn touches(&self, previous: &Self) -> bool {
        self.from() < previous.to()
            || (self.from() == previous.to() && (self.is_empty() || previous.is_empty()))
    }

    fn union(&self, other: &Self) -> Self {
        let from = self.from().min(other.from());
        let to = self.to().max(other.to());
        if self.is_reversed() {
            Self::new(to, from)
        } else {
            Self::new(from, to)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct Selection {
    ranges: Vec<SelectionRange>,
    main: usize,
}

impl Selection {
    pub fn new(ranges: Vec<SelectionRange>, main: usize) -> Self {
        assert!(main < ranges.len(), "main index must point at a range");
        let mut indexed: Vec<(usize, SelectionRange)> = ranges.into_iter().enumerate().collect();
        indexed.sort_by_key(|(_, range)| (range.from(), range.to()));
        let mut merged: Vec<SelectionRange> = Vec::new();
        let mut merged_main = 0;
        for (index, range) in indexed {
            match merged.last_mut() {
                Some(previous) if range.touches(previous) => *previous = previous.union(&range),
                _ => merged.push(range),
            }
            if index == main {
                merged_main = merged.len() - 1;
            }
        }
        Self {
            ranges: merged,
            main: merged_main,
        }
    }

    pub fn single(range: SelectionRange) -> Self {
        Self::new(vec![range], 0)
    }

    pub fn cursor(position: usize) -> Self {
        Self::single(SelectionRange::cursor(position))
    }

    pub fn ranges(&self) -> &[SelectionRange] {
        &self.ranges
    }

    pub fn main_index(&self) -> usize {
        self.main
    }

    pub fn main(&self) -> SelectionRange {
        self.ranges[self.main]
    }

    pub fn is_empty(&self) -> bool {
        self.ranges.iter().all(SelectionRange::is_empty)
    }

    pub fn map(&self, changes: &ChangeSet) -> Self {
        Self::new(
            self.ranges.iter().map(|range| range.map(changes)).collect(),
            self.main,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change::Change;

    fn range(anchor: usize, head: usize) -> SelectionRange {
        SelectionRange::new(anchor, head)
    }

    #[test]
    fn range_helpers_ignore_direction() {
        let forward = range(2, 5);
        let backward = range(5, 2);
        assert_eq!((forward.from(), forward.to()), (2, 5));
        assert_eq!((backward.from(), backward.to()), (2, 5));
        assert!(!forward.is_reversed());
        assert!(backward.is_reversed());
        assert!(!forward.is_empty());
        assert!(SelectionRange::cursor(3).is_empty());
        assert!(!SelectionRange::cursor(3).is_reversed());
    }

    #[test]
    fn new_sorts_ranges_and_tracks_main() {
        let selection = Selection::new(vec![range(8, 9), range(1, 2), range(4, 5)], 0);
        assert_eq!(selection.ranges(), &[range(1, 2), range(4, 5), range(8, 9)]);
        assert_eq!(selection.main(), range(8, 9));
        assert_eq!(selection.main_index(), 2);
    }

    #[test]
    fn overlapping_ranges_merge() {
        let selection = Selection::new(vec![range(1, 5), range(3, 8)], 1);
        assert_eq!(selection.ranges(), &[range(1, 8)]);
        assert_eq!(selection.main_index(), 0);
    }

    #[test]
    fn merge_keeps_the_direction_of_the_first_range() {
        let selection = Selection::new(vec![range(5, 1), range(3, 8)], 0);
        assert_eq!(selection.ranges(), &[range(8, 1)]);
    }

    #[test]
    fn contained_range_is_absorbed() {
        let selection = Selection::new(vec![range(0, 10), range(2, 3)], 0);
        assert_eq!(selection.ranges(), &[range(0, 10)]);
    }

    #[test]
    fn touching_non_empty_ranges_stay_separate() {
        let selection = Selection::new(vec![range(1, 3), range(3, 5)], 0);
        assert_eq!(selection.ranges().len(), 2);
    }

    #[test]
    fn cursor_touching_a_range_merges() {
        let selection = Selection::new(vec![range(1, 3), range(3, 3)], 0);
        assert_eq!(selection.ranges(), &[range(1, 3)]);
        let selection = Selection::new(vec![range(3, 3), range(3, 5)], 0);
        assert_eq!(selection.ranges(), &[range(3, 5)]);
    }

    #[test]
    fn equal_cursors_merge() {
        let selection = Selection::new(vec![range(2, 2), range(2, 2)], 1);
        assert_eq!(selection.ranges(), &[range(2, 2)]);
        assert_eq!(selection.main_index(), 0);
    }

    #[test]
    fn is_empty_requires_every_range_to_be_a_cursor() {
        assert!(Selection::new(vec![range(1, 1), range(4, 4)], 0).is_empty());
        assert!(!Selection::new(vec![range(1, 1), range(4, 6)], 0).is_empty());
    }

    #[test]
    fn single_and_cursor_build_one_range() {
        assert_eq!(Selection::single(range(1, 4)).ranges(), &[range(1, 4)]);
        assert_eq!(Selection::cursor(7).main(), range(7, 7));
    }

    #[test]
    fn cursor_moves_after_text_inserted_at_it() {
        let changes = ChangeSet::new(vec![Change::insert(3, "ab")]);
        assert_eq!(Selection::cursor(3).map(&changes), Selection::cursor(5));
    }

    #[test]
    fn range_does_not_grow_over_insertions_at_its_edges() {
        let changes = ChangeSet::new(vec![Change::insert(2, "x"), Change::insert(5, "y")]);
        assert_eq!(
            Selection::single(range(2, 5)).map(&changes),
            Selection::single(range(3, 6))
        );
    }

    #[test]
    fn mapping_preserves_direction() {
        let changes = ChangeSet::new(vec![Change::insert(0, "ab")]);
        assert_eq!(
            Selection::single(range(5, 2)).map(&changes),
            Selection::single(range(7, 4))
        );
    }

    #[test]
    fn range_inside_a_deleted_region_collapses() {
        let changes = ChangeSet::new(vec![Change::delete(1..9)]);
        assert_eq!(
            Selection::single(range(3, 5)).map(&changes),
            Selection::cursor(1)
        );
    }

    #[test]
    fn mapping_merges_ranges_that_collide() {
        let changes = ChangeSet::new(vec![Change::delete(3..5)]);
        let selection = Selection::new(vec![range(1, 3), range(4, 4)], 1).map(&changes);
        assert_eq!(selection.ranges(), &[range(1, 3)]);
        assert_eq!(selection.main_index(), 0);
    }
}
