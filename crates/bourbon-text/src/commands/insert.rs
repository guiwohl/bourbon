use crate::change::Change;
use crate::selection::SelectionRange;
use crate::state::{EditorState, Transaction, TransactionKind};
use crate::text::Text;

use super::pair::{closing_of, is_closing, should_auto_pair};
use super::{RangeEdit, edit_each};

pub fn insert_text(state: &EditorState, text: &str) -> Option<Transaction> {
    edit_each(state, TransactionKind::Input, |_, range| {
        Some(RangeEdit::collapsed(Change::replace(
            range.from()..range.to(),
            text,
        )))
    })
}

pub fn type_character(state: &EditorState, character: char) -> Option<Transaction> {
    edit_each(state, TransactionKind::Input, |text, range| {
        Some(typed_character_edit(text, range, character))
    })
}

fn typed_character_edit(text: &Text, range: &SelectionRange, character: char) -> RangeEdit {
    let position = range.from();
    if range.is_empty() && is_closing(character) && text.char_after(position) == Some(character) {
        return RangeEdit {
            change: Change::insert(position, ""),
            anchor: character.len_utf8(),
            head: character.len_utf8(),
        };
    }
    let Some(closing) = closing_of(character) else {
        return plain(range, character);
    };
    if !range.is_empty() {
        let inner = text.slice(range.from()..range.to());
        let opening_len = character.len_utf8();
        return RangeEdit::selecting(
            Change::replace(
                range.from()..range.to(),
                format!("{character}{inner}{closing}"),
            ),
            opening_len..opening_len + inner.len(),
            range.is_reversed(),
        );
    }
    if should_auto_pair(text, position, character, closing) {
        return RangeEdit {
            change: Change::insert(position, format!("{character}{closing}")),
            anchor: character.len_utf8(),
            head: character.len_utf8(),
        };
    }
    plain(range, character)
}

fn plain(range: &SelectionRange, character: char) -> RangeEdit {
    RangeEdit::collapsed(Change::replace(
        range.from()..range.to(),
        character.to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{expect, expect_nothing};

    #[test]
    fn insert_text_at_a_cursor() {
        expect("ab|c", |s| insert_text(s, "XY"), "abXY|c");
    }

    #[test]
    fn insert_text_replaces_the_selection() {
        expect("a«bc»d", |s| insert_text(s, "X"), "aX|d");
        expect("a»bc«d", |s| insert_text(s, "X"), "aX|d");
    }

    #[test]
    fn insert_text_applies_to_every_selection() {
        expect("|a|b«c»", |s| insert_text(s, "-"), "-|a-|b-|");
    }

    #[test]
    fn insert_empty_text_at_a_cursor_does_nothing() {
        expect_nothing("a|b", |s| insert_text(s, ""));
    }

    #[test]
    fn insert_text_has_the_input_kind() {
        let state = crate::testing::state("|");
        assert_eq!(
            insert_text(&state, "a").map(|t| t.kind),
            Some(TransactionKind::Input)
        );
    }

    #[test]
    fn plain_characters_insert_normally() {
        expect("a|b", |s| type_character(s, 'x'), "ax|b");
    }

    #[test]
    fn every_opener_inserts_its_pair() {
        for (opener, expected) in [
            ('(', "(|)"),
            ('[', "[|]"),
            ('{', "{|}"),
            ('"', "\"|\""),
            ('`', "`|`"),
        ] {
            expect("|", |s| type_character(s, opener), expected);
        }
    }

    #[test]
    fn typing_the_closer_skips_over_it() {
        for closer in [')', ']', '}'] {
            let before = format!("a{}|{closer}b", closer_opening(closer));
            let after = format!("a{}{closer}|b", closer_opening(closer));
            expect(&before, |s| type_character(s, closer), &after);
        }
        expect("\"a|\"", |s| type_character(s, '"'), "\"a\"|");
        expect("`a|`", |s| type_character(s, '`'), "`a`|");
    }

    fn closer_opening(closer: char) -> char {
        match closer {
            ')' => '(',
            ']' => '[',
            _ => '{',
        }
    }

    #[test]
    fn typing_a_closer_without_a_match_inserts_it() {
        expect("a|b", |s| type_character(s, ')'), "a)|b");
    }

    #[test]
    fn a_selected_closer_is_replaced_not_skipped() {
        expect("«)»", |s| type_character(s, ')'), ")|");
    }

    #[test]
    fn no_pair_before_a_word_character() {
        expect("|a", |s| type_character(s, '('), "(|a");
    }

    #[test]
    fn pair_before_a_closing_bracket_or_whitespace() {
        expect("(|)", |s| type_character(s, '['), "([|])");
        expect("|  ", |s| type_character(s, '('), "(|)  ");
    }

    #[test]
    fn quote_after_a_word_does_not_pair() {
        expect("it|", |s| type_character(s, '"'), "it\"|");
        expect("it |", |s| type_character(s, '"'), "it \"|\"");
    }

    #[test]
    fn opener_over_a_selection_surrounds_it() {
        expect("a«bc»d", |s| type_character(s, '('), "a(«bc»)d");
        expect("a»bc«d", |s| type_character(s, '`'), "a`»bc«`d");
    }

    #[test]
    fn pairs_apply_to_each_cursor() {
        expect("|a |", |s| type_character(s, '('), "(|a (|)");
    }
}
