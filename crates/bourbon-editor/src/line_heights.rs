use bourbon_text::{ChangeSet, Text};

#[allow(clippy::cast_precision_loss)]
fn lossy(line: usize) -> f32 {
    line as f32
}

pub struct LineHeights {
    heights: Vec<f32>,
    measured: Vec<bool>,
    tops: Vec<f32>,
    estimate: f32,
}

impl LineHeights {
    pub fn new(line_count: usize, estimate: f32) -> Self {
        let mut heights = Self {
            heights: vec![estimate; line_count],
            measured: vec![false; line_count],
            tops: Vec::new(),
            estimate,
        };
        heights.refresh();
        heights
    }

    pub fn line_count(&self) -> usize {
        self.heights.len()
    }

    pub fn total(&self) -> f32 {
        self.tops.last().copied().unwrap_or(0.0)
    }

    pub fn top(&self, line: usize) -> f32 {
        self.tops.get(line).copied().unwrap_or_else(|| self.total())
    }

    pub fn is_measured(&self, line: usize) -> bool {
        self.measured.get(line).copied().unwrap_or(false)
    }

    pub fn set_measured(&mut self, line: usize, height: f32) -> bool {
        let Some(slot) = self.heights.get_mut(line) else {
            return false;
        };
        let changed = (*slot - height).abs() > f32::EPSILON || !self.measured[line];
        *slot = height;
        self.measured[line] = true;
        changed
    }

    pub fn refresh(&mut self) {
        self.tops.clear();
        let mut running = 0.0;
        self.tops.push(running);
        for height in &self.heights {
            running += height;
            self.tops.push(running);
        }
    }

    pub fn line_at(&self, y: f32) -> usize {
        let last = self.heights.len().saturating_sub(1);
        self.tops
            .partition_point(|top| *top <= y)
            .saturating_sub(1)
            .min(last)
    }

    pub fn line_position(&self, y: f32) -> f32 {
        let line = self.line_at(y);
        let height = self.top(line + 1) - self.top(line);
        let within = if height > 0.0 {
            ((y - self.top(line)) / height).clamp(0.0, 1.0)
        } else {
            0.0
        };
        lossy(line) + within
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn offset_of_line(&self, line: f32) -> f32 {
        let whole = line.max(0.0).floor();
        let index = whole as usize;
        let height = self.top(index + 1) - self.top(index);
        self.top(index) + (line.max(0.0) - whole) * height
    }

    pub fn replace_lines(&mut self, first: usize, removed: usize, inserted: usize) {
        let first = first.min(self.heights.len());
        let end = (first + removed).min(self.heights.len());
        self.heights
            .splice(first..end, std::iter::repeat_n(self.estimate, inserted));
        self.measured
            .splice(first..end, std::iter::repeat_n(false, inserted));
    }

    pub fn apply_changes(&mut self, before: &Text, changes: &ChangeSet) {
        for change in changes.changes().iter().rev() {
            let first = before.line_of(change.range.start);
            let removed = before.slice(change.range.clone()).matches('\n').count();
            let inserted = change.text.matches('\n').count();
            self.replace_lines(first, removed + 1, inserted + 1);
        }
    }

    pub fn reset(&mut self, estimate: f32) {
        self.estimate = estimate;
        self.heights.fill(estimate);
        self.measured.fill(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bourbon_text::Change;

    fn heights(count: usize) -> LineHeights {
        LineHeights::new(count, 20.0)
    }

    #[test]
    fn a_pixel_offset_becomes_a_fractional_line() {
        let mut heights = heights(3);
        heights.set_measured(1, 40.0);
        heights.refresh();
        assert!((heights.line_position(0.0) - 0.0).abs() < 1e-4);
        assert!((heights.line_position(10.0) - 0.5).abs() < 1e-4);
        assert!((heights.line_position(30.0) - 1.25).abs() < 1e-4);
        assert!((heights.line_position(70.0) - 2.5).abs() < 1e-4);
    }

    #[test]
    fn a_fractional_line_becomes_a_pixel_offset() {
        let mut heights = heights(3);
        heights.set_measured(1, 40.0);
        heights.refresh();
        assert!((heights.offset_of_line(1.25) - 30.0).abs() < 1e-4);
        assert!((heights.offset_of_line(2.5) - 70.0).abs() < 1e-4);
        assert!((heights.offset_of_line(-3.0) - 0.0).abs() < 1e-4);
    }

    #[test]
    fn past_the_last_line_the_offset_stays_inside_the_document() {
        let heights = heights(2);
        assert!((heights.line_position(500.0) - 2.0).abs() < 1e-4);
        assert!((heights.offset_of_line(1.5) - 30.0).abs() < 1e-4);
    }

    #[test]
    fn an_empty_document_maps_everything_to_the_start() {
        let heights = heights(0);
        assert!((heights.line_position(25.0) - 0.0).abs() < 1e-4);
        assert!((heights.offset_of_line(3.0) - 0.0).abs() < 1e-4);
    }

    #[test]
    fn starts_with_estimates() {
        let heights = heights(4);
        assert_eq!(heights.line_count(), 4);
        assert_eq!(heights.total(), 80.0);
        assert_eq!(heights.top(3), 60.0);
        assert!(!heights.is_measured(0));
    }

    #[test]
    fn measuring_shifts_later_lines_after_refresh() {
        let mut heights = heights(3);
        assert!(heights.set_measured(0, 50.0));
        heights.refresh();
        assert_eq!(heights.top(1), 50.0);
        assert_eq!(heights.total(), 90.0);
        assert!(heights.is_measured(0));
    }

    #[test]
    fn measuring_the_same_height_reports_no_change() {
        let mut heights = heights(2);
        assert!(heights.set_measured(1, 30.0));
        assert!(!heights.set_measured(1, 30.0));
    }

    #[test]
    fn measuring_outside_the_document_is_ignored() {
        let mut heights = heights(2);
        assert!(!heights.set_measured(9, 30.0));
        assert_eq!(heights.line_count(), 2);
    }

    #[test]
    fn line_at_finds_the_line_containing_the_offset() {
        let mut heights = heights(3);
        heights.set_measured(1, 40.0);
        heights.refresh();
        assert_eq!(heights.line_at(0.0), 0);
        assert_eq!(heights.line_at(19.9), 0);
        assert_eq!(heights.line_at(20.0), 1);
        assert_eq!(heights.line_at(59.9), 1);
        assert_eq!(heights.line_at(60.0), 2);
    }

    #[test]
    fn line_at_clamps_to_the_document() {
        let heights = heights(3);
        assert_eq!(heights.line_at(-5.0), 0);
        assert_eq!(heights.line_at(1000.0), 2);
    }

    #[test]
    fn replacing_lines_invalidates_only_the_edited_range() {
        let mut heights = heights(5);
        for line in 0..5 {
            heights.set_measured(line, 30.0);
        }
        heights.replace_lines(2, 1, 3);
        heights.refresh();
        assert_eq!(heights.line_count(), 7);
        assert!(heights.is_measured(0));
        assert!(heights.is_measured(1));
        assert!(!heights.is_measured(2));
        assert!(!heights.is_measured(4));
        assert!(heights.is_measured(5));
        assert_eq!(heights.top(6) - heights.top(5), 30.0);
        assert_eq!(heights.top(4) - heights.top(3), 20.0);
    }

    #[test]
    fn removing_lines_shrinks_the_document() {
        let mut heights = heights(5);
        heights.replace_lines(1, 3, 1);
        heights.refresh();
        assert_eq!(heights.line_count(), 3);
        assert_eq!(heights.total(), 60.0);
    }

    #[test]
    fn replacing_past_the_end_is_clamped() {
        let mut heights = heights(2);
        heights.replace_lines(1, 10, 1);
        heights.refresh();
        assert_eq!(heights.line_count(), 2);
    }

    #[test]
    fn reset_forgets_every_measurement() {
        let mut heights = heights(3);
        heights.set_measured(0, 50.0);
        heights.reset(10.0);
        heights.refresh();
        assert!(!heights.is_measured(0));
        assert_eq!(heights.total(), 30.0);
    }

    #[test]
    fn typing_inside_a_line_invalidates_only_that_line() {
        let before = Text::new("one\ntwo\nthree");
        let mut heights = heights(3);
        for line in 0..3 {
            heights.set_measured(line, 30.0);
        }
        heights.apply_changes(&before, &ChangeSet::new(vec![Change::insert(5, "x")]));
        assert_eq!(heights.line_count(), 3);
        assert!(heights.is_measured(0));
        assert!(!heights.is_measured(1));
        assert!(heights.is_measured(2));
    }

    #[test]
    fn inserting_a_line_break_adds_a_line() {
        let before = Text::new("one\ntwo");
        let mut heights = heights(2);
        for line in 0..2 {
            heights.set_measured(line, 30.0);
        }
        heights.apply_changes(&before, &ChangeSet::new(vec![Change::insert(1, "\n")]));
        assert_eq!(heights.line_count(), 3);
        assert!(!heights.is_measured(0));
        assert!(!heights.is_measured(1));
        assert!(heights.is_measured(2));
    }

    #[test]
    fn deleting_a_line_break_joins_two_lines() {
        let before = Text::new("one\ntwo\nthree");
        let mut heights = heights(3);
        for line in 0..3 {
            heights.set_measured(line, 30.0);
        }
        heights.apply_changes(&before, &ChangeSet::new(vec![Change::delete(3..4)]));
        assert_eq!(heights.line_count(), 2);
        assert!(!heights.is_measured(0));
        assert!(heights.is_measured(1));
    }

    #[test]
    fn several_changes_keep_later_lines_aligned() {
        let before = Text::new("a\nb\nc\nd");
        let mut heights = heights(4);
        for line in 0..4 {
            heights.set_measured(line, 30.0);
        }
        heights.apply_changes(
            &before,
            &ChangeSet::new(vec![Change::insert(0, "x\n"), Change::insert(6, "y")]),
        );
        assert_eq!(heights.line_count(), 5);
        assert!(!heights.is_measured(0));
        assert!(!heights.is_measured(1));
        assert!(heights.is_measured(2));
        assert!(heights.is_measured(3));
        assert!(!heights.is_measured(4));
    }

    #[test]
    fn the_first_measurement_counts_even_at_the_estimated_height() {
        let mut heights = heights(1);
        assert!(heights.set_measured(0, 20.0));
        assert!(heights.is_measured(0));
    }

    #[test]
    fn a_change_of_exactly_epsilon_is_not_a_change() {
        let mut heights = heights(1);
        heights.set_measured(0, 1.0);
        assert!(!heights.set_measured(0, 1.0 + f32::EPSILON));
    }
}
