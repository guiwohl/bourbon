use crate::numeric::signed;
use bourbon_text::movement::{
    next_grapheme_boundary, next_word_boundary, previous_grapheme_boundary, previous_word_boundary,
};
use bourbon_text::{SelectionRange, Text};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    Left,
    Right,
    WordLeft,
    WordRight,
    LineStart,
    LineEnd,
    DocumentStart,
    DocumentEnd,
}

pub fn move_range(
    text: &Text,
    range: SelectionRange,
    motion: Motion,
    extend: bool,
) -> SelectionRange {
    let collapsed = (!extend && !range.is_empty()).then(|| match motion {
        Motion::Left => range.from(),
        _ => range.to(),
    });
    let target = match (motion, collapsed) {
        (Motion::Left | Motion::Right, Some(position)) => position,
        (Motion::Left, None) => previous_grapheme_boundary(text, range.head),
        (Motion::Right, None) => next_grapheme_boundary(text, range.head),
        (Motion::WordLeft, _) => previous_word_boundary(text, range.head),
        (Motion::WordRight, _) => next_word_boundary(text, range.head),
        (Motion::LineStart, _) => text.line_start(text.line_of(range.head)),
        (Motion::LineEnd, _) => text.line_end(text.line_of(range.head)),
        (Motion::DocumentStart, _) => 0,
        (Motion::DocumentEnd, _) => text.len(),
    };
    if extend {
        extend_to(range, target)
    } else {
        SelectionRange::cursor(target)
    }
}

pub fn word_range_at(text: &Text, offset: usize) -> SelectionRange {
    let end = next_word_boundary(text, offset.min(text.len()));
    SelectionRange::new(previous_word_boundary(text, end), end)
}

pub fn line_range_at(text: &Text, offset: usize) -> SelectionRange {
    let line = text.line_of(offset.min(text.len()));
    SelectionRange::new(text.line_start(line), text.line_start(line + 1))
}

pub fn extend_to(range: SelectionRange, head: usize) -> SelectionRange {
    SelectionRange::new(range.anchor, head)
}

pub fn walk_rows(
    line: usize,
    row: usize,
    delta: i64,
    line_count: usize,
    mut rows_in: impl FnMut(usize) -> usize,
) -> (usize, usize) {
    let mut line = line;
    let mut row = signed(row) + delta;
    for _ in 0..=line_count {
        let rows_here = signed(rows_in(line));
        if row < 0 {
            if line == 0 {
                return (0, 0);
            }
            line -= 1;
            row += signed(rows_in(line));
        } else if row >= rows_here {
            if line + 1 >= line_count {
                return (line, usize::try_from(rows_here - 1).unwrap_or(0));
            }
            row -= rows_here;
            line += 1;
        } else {
            break;
        }
    }
    (line, usize::try_from(row).unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text() -> Text {
        Text::new("hello world\nsecond line")
    }

    fn range(anchor: usize, head: usize) -> SelectionRange {
        SelectionRange::new(anchor, head)
    }

    #[test]
    fn left_and_right_move_one_grapheme() {
        let text = text();
        assert_eq!(
            move_range(&text, range(3, 3), Motion::Left, false),
            range(2, 2)
        );
        assert_eq!(
            move_range(&text, range(3, 3), Motion::Right, false),
            range(4, 4)
        );
    }

    #[test]
    fn left_and_right_respect_multibyte_characters() {
        let text = Text::new("a🙂b");
        assert_eq!(
            move_range(&text, range(5, 5), Motion::Left, false),
            range(1, 1)
        );
        assert_eq!(
            move_range(&text, range(1, 1), Motion::Right, false),
            range(5, 5)
        );
    }

    #[test]
    fn left_with_a_selection_collapses_to_its_start() {
        let text = text();
        assert_eq!(
            move_range(&text, range(2, 6), Motion::Left, false),
            range(2, 2)
        );
        assert_eq!(
            move_range(&text, range(6, 2), Motion::Left, false),
            range(2, 2)
        );
    }

    #[test]
    fn right_with_a_selection_collapses_to_its_end() {
        let text = text();
        assert_eq!(
            move_range(&text, range(2, 6), Motion::Right, false),
            range(6, 6)
        );
        assert_eq!(
            move_range(&text, range(6, 2), Motion::Right, false),
            range(6, 6)
        );
    }

    #[test]
    fn extending_keeps_the_anchor() {
        let text = text();
        assert_eq!(
            move_range(&text, range(2, 2), Motion::Right, true),
            range(2, 3)
        );
        assert_eq!(
            move_range(&text, range(5, 2), Motion::Left, true),
            range(5, 1)
        );
    }

    #[test]
    fn edges_of_the_document_do_not_move() {
        let text = text();
        assert_eq!(
            move_range(&text, range(0, 0), Motion::Left, false),
            range(0, 0)
        );
        let end = text.len();
        assert_eq!(
            move_range(&text, range(end, end), Motion::Right, false),
            range(end, end)
        );
    }

    #[test]
    fn word_motions_use_word_boundaries() {
        let text = text();
        assert_eq!(
            move_range(&text, range(0, 0), Motion::WordRight, false),
            range(5, 5)
        );
        assert_eq!(
            move_range(&text, range(11, 11), Motion::WordLeft, false),
            range(6, 6)
        );
    }

    #[test]
    fn line_start_and_end_stay_on_the_logical_line() {
        let text = text();
        assert_eq!(
            move_range(&text, range(15, 15), Motion::LineStart, false),
            range(12, 12)
        );
        assert_eq!(
            move_range(&text, range(3, 3), Motion::LineEnd, false),
            range(11, 11)
        );
    }

    #[test]
    fn document_start_and_end_jump_to_the_extremes() {
        let text = text();
        assert_eq!(
            move_range(&text, range(5, 5), Motion::DocumentStart, false),
            range(0, 0)
        );
        assert_eq!(
            move_range(&text, range(5, 5), Motion::DocumentEnd, true),
            range(5, text.len())
        );
    }

    #[test]
    fn double_click_selects_the_word() {
        let text = text();
        assert_eq!(word_range_at(&text, 7), range(6, 11));
        assert_eq!(word_range_at(&text, 6), range(6, 11));
    }

    #[test]
    fn double_click_at_the_end_of_text_selects_the_last_word() {
        let text = Text::new("one two");
        assert_eq!(word_range_at(&text, 7), range(4, 7));
    }

    #[test]
    fn triple_click_selects_the_line_with_its_break() {
        let text = text();
        assert_eq!(line_range_at(&text, 3), range(0, 12));
    }

    #[test]
    fn triple_click_on_the_last_line_stops_at_the_end() {
        let text = text();
        assert_eq!(line_range_at(&text, 14), range(12, text.len()));
    }

    #[test]
    fn extend_to_moves_only_the_head() {
        assert_eq!(extend_to(range(4, 8), 2), range(4, 2));
    }

    fn rows_of(rows: &'static [usize]) -> impl Fn(usize) -> usize {
        move |line| rows[line]
    }

    #[test]
    fn walking_inside_a_wrapped_line_stays_on_it() {
        assert_eq!(walk_rows(0, 0, 1, 3, rows_of(&[3, 1, 1])), (0, 1));
    }

    #[test]
    fn walking_down_crosses_into_the_next_line() {
        assert_eq!(walk_rows(0, 2, 1, 3, rows_of(&[3, 2, 1])), (1, 0));
        assert_eq!(walk_rows(0, 1, 3, 3, rows_of(&[3, 2, 1])), (1, 1));
    }

    #[test]
    fn walking_up_crosses_into_the_previous_line() {
        assert_eq!(walk_rows(1, 0, -1, 3, rows_of(&[3, 2, 1])), (0, 2));
        assert_eq!(walk_rows(2, 0, -3, 3, rows_of(&[3, 2, 1])), (0, 2));
    }

    #[test]
    fn walking_past_the_start_stops_at_the_first_row() {
        assert_eq!(walk_rows(1, 1, -50, 3, rows_of(&[3, 2, 1])), (0, 0));
    }

    #[test]
    fn walking_past_the_end_stops_at_the_last_row() {
        assert_eq!(walk_rows(0, 0, 50, 3, rows_of(&[3, 2, 1])), (2, 0));
        assert_eq!(walk_rows(0, 0, 50, 2, rows_of(&[1, 4])), (1, 3));
    }

    #[test]
    fn walking_zero_rows_stays_put() {
        assert_eq!(walk_rows(1, 1, 0, 3, rows_of(&[3, 2, 1])), (1, 1));
    }
}
