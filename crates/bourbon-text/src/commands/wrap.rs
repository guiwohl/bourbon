use crate::change::Change;
use crate::selection::SelectionRange;
use crate::state::{EditorState, Transaction, TransactionKind};
use crate::text::Text;

use super::{RangeEdit, edit_each};

pub fn toggle_bold(state: &EditorState) -> Option<Transaction> {
    toggle_wrap(state, "**")
}

pub fn toggle_italic(state: &EditorState) -> Option<Transaction> {
    toggle_wrap(state, "_")
}

pub fn toggle_strikethrough(state: &EditorState) -> Option<Transaction> {
    toggle_wrap(state, "~~")
}

pub fn toggle_inline_code(state: &EditorState) -> Option<Transaction> {
    toggle_wrap(state, "`")
}

pub fn toggle_wrap(state: &EditorState, mark: &str) -> Option<Transaction> {
    edit_each(state, TransactionKind::Command, |text, range| {
        Some(wrap_edit(text, range, mark))
    })
}

fn wrap_edit(text: &Text, range: &SelectionRange, mark: &str) -> RangeEdit {
    let (from, to) = (range.from(), range.to());
    let mark_len = mark.len();
    let inner = text.slice(from..to);
    let reversed = range.is_reversed();
    if text.has_before(from, mark) && text.has_at(to, mark) {
        return RangeEdit::selecting(
            Change::replace(from - mark_len..to + mark_len, inner.as_str()),
            0..inner.len(),
            reversed,
        );
    }
    if inner.len() >= 2 * mark_len && inner.starts_with(mark) && inner.ends_with(mark) {
        let unwrapped = &inner[mark_len..inner.len() - mark_len];
        return RangeEdit::selecting(
            Change::replace(from..to, unwrapped),
            0..unwrapped.len(),
            reversed,
        );
    }
    RangeEdit::selecting(
        Change::replace(from..to, format!("{mark}{inner}{mark}")),
        mark_len..mark_len + inner.len(),
        reversed,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::expect;

    #[test]
    fn selection_gets_wrapped_and_stays_on_the_inner_text() {
        expect("a«bc»d", toggle_bold, "a**«bc»**d");
        expect("a«bc»d", toggle_italic, "a_«bc»_d");
        expect("a«bc»d", toggle_strikethrough, "a~~«bc»~~d");
        expect("a«bc»d", toggle_inline_code, "a`«bc»`d");
    }

    #[test]
    fn toggling_again_removes_the_surrounding_marks() {
        expect("a**«bc»**d", toggle_bold, "a«bc»d");
        expect("a_«bc»_d", toggle_italic, "a«bc»d");
        expect("a~~«bc»~~d", toggle_strikethrough, "a«bc»d");
        expect("a`«bc»`d", toggle_inline_code, "a«bc»d");
    }

    #[test]
    fn selection_that_includes_its_marks_is_unwrapped() {
        expect("a«**bc**»d", toggle_bold, "a«bc»d");
    }

    #[test]
    fn selection_holding_only_a_mark_pair_is_not_unwrapped_twice() {
        expect("«**»", toggle_bold, "**«**»**");
    }

    #[test]
    fn selection_of_exactly_two_marks_unwraps_to_nothing() {
        expect("«``»", toggle_inline_code, "|");
    }

    #[test]
    fn empty_selection_inserts_a_pair_with_the_caret_inside() {
        expect("a|b", toggle_bold, "a**|**b");
        expect("a|b", toggle_inline_code, "a`|`b");
    }

    #[test]
    fn empty_selection_between_doubled_marks_removes_them() {
        expect("a**|**b", toggle_bold, "a|b");
        expect("a~~|~~b", toggle_strikethrough, "a|b");
    }

    #[test]
    fn marks_only_on_one_side_wrap_instead_of_unwrapping() {
        expect("**«bc»d", toggle_bold, "****«bc»**d");
    }

    #[test]
    fn reversed_selections_stay_reversed() {
        expect("a»bc«d", toggle_bold, "a**»bc«**d");
        expect("a**»bc«**d", toggle_bold, "a»bc«d");
    }

    #[test]
    fn every_selection_toggles_independently() {
        expect("«a» «b»", toggle_italic, "_«a»_ _«b»_");
    }

    #[test]
    fn multibyte_content_is_preserved() {
        expect("«é日»", toggle_bold, "**«é日»**");
        expect("**«é日»**", toggle_bold, "«é日»");
    }

    #[test]
    fn custom_marks_are_supported() {
        expect("«a»", |s| toggle_wrap(s, "=="), "==«a»==");
    }
}
