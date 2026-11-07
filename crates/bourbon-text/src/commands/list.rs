use std::ops::Range;

use crate::markup::{LinePrefix, MarkerKind};
use crate::state::{EditorState, Transaction};
use crate::text::Text;

use super::{line_change, line_edits_transaction, selected_lines};

#[derive(Clone, Copy)]
enum Direction {
    Indent,
    Outdent,
}

pub fn indent_list_item(state: &EditorState) -> Option<Transaction> {
    shift_list_items(state, Direction::Indent)
}

pub fn outdent_list_item(state: &EditorState) -> Option<Transaction> {
    shift_list_items(state, Direction::Outdent)
}

struct Line {
    text: String,
    prefix: LinePrefix,
}

impl Line {
    fn new(text: String) -> Self {
        let prefix = LinePrefix::parse(&text);
        Self { text, prefix }
    }

    fn indent(&self) -> usize {
        self.prefix.indent_end
    }

    fn is_item(&self) -> bool {
        self.prefix.is_list_item()
    }

    fn is_blank(&self) -> bool {
        self.text.trim().is_empty()
    }

    fn continues_list(&self) -> bool {
        !self.is_blank() && (self.is_item() || self.indent() > 0)
    }
}

fn shift_list_items(state: &EditorState, direction: Direction) -> Option<Transaction> {
    let selected = selected_lines(&state.text, &state.selection);
    let window = list_block(&state.text, &selected)?;
    let lines: Vec<Line> = window
        .clone()
        .map(|line| Line::new(state.text.line_text(line)))
        .collect();
    let selected: Vec<usize> = selected
        .iter()
        .filter(|line| window.contains(line))
        .map(|line| line - window.start)
        .collect();
    let first_item = selected
        .iter()
        .copied()
        .find(|index| lines[*index].is_item())?;
    let amount = match direction {
        Direction::Indent => indent_amount(&lines, first_item)?,
        Direction::Outdent => outdent_amount(&lines, first_item)?,
    };
    let shifted = shifted_lines(&lines, &selected);
    let mut rewritten: Vec<String> = lines
        .iter()
        .zip(&shifted)
        .map(|(line, shifted)| match (shifted, direction) {
            (false, _) => line.text.clone(),
            (true, Direction::Indent) => format!("{}{}", " ".repeat(amount), line.text),
            (true, Direction::Outdent) => line.text[amount.min(line.indent())..].to_owned(),
        })
        .collect();
    renumber(&mut rewritten, &shifted);
    let changes = lines
        .iter()
        .zip(&rewritten)
        .enumerate()
        .filter_map(|(index, (old, new))| {
            let start = state.text.line_start(window.start + index);
            line_change(start, &old.text, new)
        })
        .collect();
    line_edits_transaction(state, changes)
}

fn list_block(text: &Text, selected: &[usize]) -> Option<Range<usize>> {
    let continues = |line: usize| Line::new(text.line_text(line)).continues_list();
    let mut blocks = Vec::new();
    let mut open = None;
    for line in 0..text.line_count() {
        match (continues(line), open) {
            (true, None) => open = Some(line),
            (false, Some(start)) => {
                blocks.push(start..line);
                open = None;
            }
            _ => {}
        }
    }
    blocks.extend(open.map(|start| start..text.line_count()));
    blocks
        .into_iter()
        .find(|block| selected.iter().any(|line| block.contains(line)))
}

fn indent_amount(lines: &[Line], index: usize) -> Option<usize> {
    let current = lines[index].indent();
    let previous = lines[..index]
        .iter()
        .rev()
        .find(|line| line.indent() <= current)?;
    let marker = previous.prefix.marker.filter(|_| previous.is_item())?;
    (previous.indent() == current).then(|| marker.end - current)
}

fn outdent_amount(lines: &[Line], index: usize) -> Option<usize> {
    let current = lines[index].indent();
    if current == 0 {
        return None;
    }
    let target = lines[..index]
        .iter()
        .rev()
        .find(|line| line.is_item() && line.indent() < current)
        .map_or(0, Line::indent);
    Some(current - target)
}

fn shifted_lines(lines: &[Line], selected: &[usize]) -> Vec<bool> {
    let mut shifted = vec![false; lines.len()];
    for &index in selected.iter().filter(|index| lines[**index].is_item()) {
        shifted[index] = true;
        let indent = lines[index].indent();
        for (offset, child) in lines[index + 1..].iter().enumerate() {
            if child.is_blank() || child.indent() <= indent {
                break;
            }
            shifted[index + 1 + offset] = true;
        }
    }
    shifted
}

struct Numbering {
    start: usize,
    first_original: usize,
    same: bool,
    count: usize,
}

impl Numbering {
    fn first(original: usize, shifted: bool) -> Self {
        Self {
            start: if shifted { 1 } else { original },
            first_original: original,
            same: false,
            count: 0,
        }
    }

    fn next(&mut self, original: usize, shifted: bool) -> usize {
        self.count += 1;
        if self.count == 1 && !shifted {
            self.same = original == self.first_original;
        }
        if self.same {
            self.start
        } else {
            self.start + self.count
        }
    }
}

struct Level {
    indent: usize,
    numbering: Option<Numbering>,
}

impl Level {
    fn assign(&mut self, original: usize, shifted: bool) -> usize {
        if let Some(numbering) = self.numbering.as_mut() {
            return numbering.next(original, shifted);
        }
        let numbering = Numbering::first(original, shifted);
        let start = numbering.start;
        self.numbering = Some(numbering);
        start
    }
}

fn renumber(lines: &mut [String], shifted: &[bool]) {
    let mut levels: Vec<Level> = Vec::new();
    for (index, line) in lines.iter_mut().enumerate() {
        let prefix = LinePrefix::parse(line);
        let Some(marker) = prefix.marker.filter(|_| prefix.is_list_item()) else {
            continue;
        };
        let indent = prefix.indent_end;
        while levels.last().is_some_and(|level| level.indent > indent) {
            levels.pop();
        }
        let original = match marker.kind {
            MarkerKind::Ordered { number, .. } => Some(number),
            MarkerKind::Bullet => None,
        };
        let assigned = match (levels.last_mut().filter(|l| l.indent == indent), original) {
            (Some(level), Some(original)) => Some(level.assign(original, shifted[index])),
            (Some(level), None) => {
                level.numbering = None;
                None
            }
            (None, _) => {
                let numbering = original.map(|original| Numbering::first(original, shifted[index]));
                let start = numbering.as_ref().map(|numbering| numbering.start);
                levels.push(Level { indent, numbering });
                start
            }
        };
        if let (Some(assigned), MarkerKind::Ordered { number, digits }) = (assigned, marker.kind)
            && assigned != number
        {
            *line = format!(
                "{}{assigned}{}",
                &line[..marker.start],
                &line[marker.start + digits..]
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{expect, expect_nothing};

    #[test]
    fn tab_nests_a_bullet_under_its_previous_sibling() {
        expect("- a\n- b|", indent_list_item, "- a\n  - b|");
    }

    #[test]
    fn tab_uses_the_content_column_of_ordered_lists() {
        expect("1. a\n2. b|", indent_list_item, "1. a\n   1. b|");
        expect("10. a\n11. b|", indent_list_item, "10. a\n    1. b|");
    }

    #[test]
    fn tab_keeps_a_caret_at_the_line_start_after_the_new_indent() {
        expect("- a\n|- b", indent_list_item, "- a\n  |- b");
    }

    #[test]
    fn tab_on_the_first_item_does_nothing() {
        expect_nothing("- a|", indent_list_item);
        expect_nothing("text\n- a|", indent_list_item);
    }

    #[test]
    fn tab_on_a_non_list_line_does_nothing() {
        expect_nothing("abc|", indent_list_item);
        expect_nothing("- a\nabc|", indent_list_item);
    }

    #[test]
    fn tab_after_a_deeper_child_still_finds_the_sibling() {
        expect(
            "- a\n  - child\n- b|",
            indent_list_item,
            "- a\n  - child\n  - b|",
        );
    }

    #[test]
    fn tab_moves_the_item_together_with_its_children() {
        expect(
            "- a\n- b|\n  - c\n    - d\n- e",
            indent_list_item,
            "- a\n  - b|\n    - c\n      - d\n- e",
        );
    }

    #[test]
    fn tab_moves_every_selected_item() {
        expect("- a\n«- b\n- c»", indent_list_item, "- a\n  «- b\n  - c»");
    }

    #[test]
    fn tab_does_not_shift_a_line_twice_when_a_child_is_also_selected() {
        expect(
            "- a\n«- b\n  - c»",
            indent_list_item,
            "- a\n  «- b\n    - c»",
        );
    }

    #[test]
    fn tab_renumbers_the_source_and_target_lists() {
        expect(
            "1. a\n2. b|\n3. c",
            indent_list_item,
            "1. a\n   1. b|\n2. c",
        );
    }

    #[test]
    fn tab_joins_an_existing_nested_ordered_list() {
        expect(
            "1. a\n   1. x\n2. b|",
            indent_list_item,
            "1. a\n   1. x\n   2. b|",
        );
    }

    #[test]
    fn equal_numbers_are_preserved() {
        expect(
            "1. a\n1. b|\n1. c\n1. d",
            indent_list_item,
            "1. a\n   1. b|\n1. c\n1. d",
        );
    }

    #[test]
    fn shift_tab_moves_an_item_out_to_its_parent_level() {
        expect("- a\n  - b|", outdent_list_item, "- a\n- b|");
        expect(
            "- a\n  - b\n    - c|",
            outdent_list_item,
            "- a\n  - b\n  - c|",
        );
    }

    #[test]
    fn shift_tab_on_a_top_level_item_does_nothing() {
        expect_nothing("- a|", outdent_list_item);
        expect_nothing("abc|", outdent_list_item);
    }

    #[test]
    fn shift_tab_without_a_parent_goes_to_column_zero() {
        expect("  - a|", outdent_list_item, "- a|");
        expect("    - a|", outdent_list_item, "- a|");
    }

    #[test]
    fn shift_tab_renumbers_into_the_parent_list() {
        expect(
            "1. a\n   1. b|\n2. c",
            outdent_list_item,
            "1. a\n2. b|\n3. c",
        );
    }

    #[test]
    fn shift_tab_moves_children_along() {
        expect(
            "- a\n  - b|\n    - c",
            outdent_list_item,
            "- a\n- b|\n  - c",
        );
    }

    #[test]
    fn shift_tab_never_removes_more_than_the_indentation() {
        expect(
            "- a\n    - b|\n  - c",
            outdent_list_item,
            "- a\n- b|\n  - c",
        );
    }

    #[test]
    fn blank_lines_end_the_renumbered_window() {
        expect(
            "1. a\n2. b|\n\n5. z",
            indent_list_item,
            "1. a\n   1. b|\n\n5. z",
        );
    }

    #[test]
    fn bullets_break_ordered_numbering_at_the_same_level() {
        expect(
            "1. a\n- b\n- c|\n7. d",
            indent_list_item,
            "1. a\n- b\n  - c|\n7. d",
        );
    }

    #[test]
    fn ordered_item_after_a_bullet_keeps_its_number() {
        expect("- a\n7. b\n8. c|", indent_list_item, "- a\n7. b\n   1. c|");
    }

    #[test]
    fn quoted_list_items_are_not_list_items_here() {
        expect_nothing("> - a\n> - b|", indent_list_item);
    }

    #[test]
    fn tab_nests_under_a_sibling_that_is_itself_indented() {
        expect(
            "- a\n  - b\n  - c|",
            indent_list_item,
            "- a\n  - b\n    - c|",
        );
    }

    #[test]
    fn shift_tab_goes_to_the_nearest_shallower_item() {
        expect("- a\n  - b\n  - c|", outdent_list_item, "- a\n  - b\n- c|");
    }

    #[test]
    fn shift_tab_ignores_shallower_text_that_is_not_an_item() {
        expect("  cont\n    - c|", outdent_list_item, "  cont\n- c|");
    }

    #[test]
    fn whitespace_only_lines_end_the_list() {
        expect(
            "1. a\n2. b|\n  \n  7. x\n  9. y",
            indent_list_item,
            "1. a\n   1. b|\n  \n  7. x\n  9. y",
        );
    }

    #[test]
    fn whitespace_only_lines_end_the_children_of_a_moved_item() {
        expect(
            "- a\n- b|\n   \n  - c",
            indent_list_item,
            "- a\n  - b|\n   \n  - c",
        );
    }

    #[test]
    fn plain_text_lines_end_the_list() {
        expect(
            "1. a\n2. b|\npara\n5. z",
            indent_list_item,
            "1. a\n   1. b|\npara\n5. z",
        );
    }

    #[test]
    fn indented_text_belongs_to_the_list() {
        expect(
            "1. a\n   cont\n2. b|",
            indent_list_item,
            "1. a\n   cont\n   1. b|",
        );
    }

    #[test]
    fn a_list_at_the_end_of_the_document_is_found() {
        expect("x\n\n- a\n- b|", indent_list_item, "x\n\n- a\n  - b|");
    }

    #[test]
    fn transaction_is_a_command() {
        let state = crate::testing::state("- a\n- b|");
        assert_eq!(
            indent_list_item(&state).map(|t| t.kind),
            Some(crate::TransactionKind::Command)
        );
    }
}
