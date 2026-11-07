use crate::change::Change;
use crate::markup::{LinePrefix, MarkerKind};
use crate::selection::SelectionRange;
use crate::state::{EditorState, Transaction, TransactionKind};
use crate::text::Text;

use super::{RangeEdit, edit_each};

pub fn insert_newline_continue_markup(state: &EditorState) -> Option<Transaction> {
    edit_each(state, TransactionKind::Input, |text, range| {
        Some(newline_edit(text, range))
    })
}

fn newline_edit(text: &Text, range: &SelectionRange) -> RangeEdit {
    let line = text.line_of(range.from());
    let line_start = text.line_start(line);
    let line_text = text.line_text(line);
    let prefix = LinePrefix::parse(&line_text);
    let column = range.from() - line_start;
    let has_markup = prefix.marker.is_some() || prefix.has_quote();

    if !has_markup || column < prefix.content_start {
        let indent = &line_text[..prefix.indent_end.min(column)];
        return RangeEdit::collapsed(Change::replace(
            range.from()..range.to(),
            format!("\n{indent}"),
        ));
    }
    let is_empty_item = range.is_empty()
        && column == line_text.len()
        && line_text[prefix.content_start..].trim().is_empty();
    if is_empty_item {
        let remaining = if prefix.marker.is_some() && prefix.has_quote() {
            &line_text[..prefix.quote_end]
        } else {
            ""
        };
        return RangeEdit::collapsed(Change::replace(
            line_start..line_start + line_text.len(),
            remaining,
        ));
    }
    RangeEdit::collapsed(Change::replace(
        range.from()..range.to(),
        format!("\n{}", continuation(&line_text, &prefix)),
    ))
}

fn continuation(line: &str, prefix: &LinePrefix) -> String {
    let mut continued = line[..prefix.quote_end].to_owned();
    if let Some(marker) = prefix.marker {
        match marker.kind {
            MarkerKind::Bullet => continued.push_str(&line[marker.start..marker.end]),
            MarkerKind::Ordered { number, digits } => {
                continued.push_str(&(number + 1).to_string());
                continued.push_str(&line[marker.start + digits..marker.end]);
            }
        }
    }
    if prefix.task.is_some() {
        continued.push_str("[ ] ");
    }
    continued
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::expect;

    fn enter(state: &EditorState) -> Option<Transaction> {
        insert_newline_continue_markup(state)
    }

    #[test]
    fn plain_line_breaks_without_indentation() {
        expect("abc|", enter, "abc\n|");
        expect("ab|c", enter, "ab\n|c");
    }

    #[test]
    fn plain_line_keeps_its_indentation() {
        expect("  abc|", enter, "  abc\n  |");
        expect(" | abc", enter, " \n | abc");
    }

    #[test]
    fn bullets_continue_with_the_same_marker() {
        expect("- a|", enter, "- a\n- |");
        expect("* a|", enter, "* a\n* |");
        expect("+ a|", enter, "+ a\n+ |");
    }

    #[test]
    fn continuation_keeps_indentation_and_spacing() {
        expect("  -   a|", enter, "  -   a\n  -   |");
    }

    #[test]
    fn ordered_items_increment_and_keep_the_delimiter() {
        expect("1. a|", enter, "1. a\n2. |");
        expect("9. a|", enter, "9. a\n10. |");
        expect("3) a|", enter, "3) a\n4) |");
    }

    #[test]
    fn tasks_continue_unchecked() {
        expect("- [ ] a|", enter, "- [ ] a\n- [ ] |");
        expect("- [x] a|", enter, "- [x] a\n- [ ] |");
        expect("1. [x] a|", enter, "1. [x] a\n2. [ ] |");
    }

    #[test]
    fn quotes_continue() {
        expect("> a|", enter, "> a\n> |");
        expect("> > a|", enter, "> > a\n> > |");
        expect("> - a|", enter, "> - a\n> - |");
    }

    #[test]
    fn splitting_inside_an_item_moves_the_tail_into_a_new_item() {
        expect("- ab|cd", enter, "- ab\n- |cd");
    }

    #[test]
    fn cursor_inside_the_marker_just_breaks_the_line() {
        expect("-| a", enter, "-\n| a");
        expect("|- a", enter, "\n|- a");
    }

    #[test]
    fn enter_on_an_empty_item_removes_the_marker() {
        expect("- |", enter, "|");
        expect("1. |", enter, "|");
        expect("  - |", enter, "|");
        expect("- [ ] |", enter, "|");
        expect("a\n- |", enter, "a\n|");
    }

    #[test]
    fn enter_on_an_empty_quote_removes_it() {
        expect("> |", enter, "|");
    }

    #[test]
    fn enter_on_an_empty_item_in_a_quote_keeps_the_quote() {
        expect("> - |", enter, "> |");
    }

    #[test]
    fn selection_is_replaced_by_the_continuation() {
        expect("- a«bc»d", enter, "- a\n- |d");
    }

    #[test]
    fn every_cursor_continues_its_own_line() {
        expect("- a|\n1. b|", enter, "- a\n- |\n1. b\n2. |");
    }
}
