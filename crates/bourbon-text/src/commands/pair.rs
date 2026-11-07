use crate::text::Text;

const PAIRS: [(char, char); 5] = [('(', ')'), ('[', ']'), ('{', '}'), ('"', '"'), ('`', '`')];

pub fn closing_of(opening: char) -> Option<char> {
    PAIRS
        .iter()
        .find(|(open, _)| *open == opening)
        .map(|(_, close)| *close)
}

pub fn is_closing(character: char) -> bool {
    PAIRS.iter().any(|(_, close)| *close == character)
}

pub fn should_auto_pair(text: &Text, position: usize, opening: char, closing: char) -> bool {
    let next_allows = text
        .char_after(position)
        .is_none_or(|next| next.is_whitespace() || matches!(next, ')' | ']' | '}'));
    let symmetric_after_word = opening == closing
        && text
            .char_before(position)
            .is_some_and(char::is_alphanumeric);
    next_allows && !symmetric_after_word
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closing_characters_are_known_for_every_opener() {
        assert_eq!(closing_of('('), Some(')'));
        assert_eq!(closing_of('['), Some(']'));
        assert_eq!(closing_of('{'), Some('}'));
        assert_eq!(closing_of('"'), Some('"'));
        assert_eq!(closing_of('`'), Some('`'));
        assert_eq!(closing_of('<'), None);
        assert_eq!(closing_of(')'), None);
    }

    #[test]
    fn closers_are_recognised() {
        for closer in [')', ']', '}', '"', '`'] {
            assert!(is_closing(closer));
        }
        assert!(!is_closing('('));
        assert!(!is_closing('a'));
    }

    #[test]
    fn pairing_requires_a_friendly_next_character() {
        let text = Text::new("a b)c");
        assert!(should_auto_pair(&text, 1, '(', ')'));
        assert!(should_auto_pair(&text, 3, '(', ')'));
        assert!(!should_auto_pair(&text, 4, '(', ')'));
        assert!(should_auto_pair(&text, 5, '(', ')'));
        assert!(should_auto_pair(&Text::new(""), 0, '(', ')'));
    }

    #[test]
    fn quotes_do_not_pair_after_a_word_character() {
        let text = Text::new("it ");
        assert!(!should_auto_pair(&Text::new("it"), 2, '"', '"'));
        assert!(should_auto_pair(&text, 3, '"', '"'));
        assert!(should_auto_pair(&Text::new("it"), 2, '(', ')'));
    }
}
