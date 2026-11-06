#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Notes,
    Actions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ParsedQuery<'a> {
    pub(crate) mode: Mode,
    pub(crate) text: &'a str,
}

const ACTIONS_PREFIX: char = '>';

pub(crate) fn parse(query: &str) -> ParsedQuery<'_> {
    match query.strip_prefix(ACTIONS_PREFIX) {
        Some(text) => ParsedQuery {
            mode: Mode::Actions,
            text: text.trim(),
        },
        None => ParsedQuery {
            mode: Mode::Notes,
            text: query,
        },
    }
}

pub(crate) const ACTIONS_QUERY: &str = ">";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_is_a_notes_search() {
        let parsed = parse("meeting notes");
        assert_eq!(parsed.mode, Mode::Notes);
        assert_eq!(parsed.text, "meeting notes");
    }

    #[test]
    fn empty_text_is_a_notes_search() {
        assert_eq!(parse("").mode, Mode::Notes);
    }

    #[test]
    fn a_leading_angle_bracket_is_actions_mode_and_is_not_part_of_the_text() {
        let parsed = parse(">new");
        assert_eq!(parsed.mode, Mode::Actions);
        assert_eq!(parsed.text, "new");
    }

    #[test]
    fn the_actions_text_is_trimmed() {
        assert_eq!(parse(">  go to  ").text, "go to");
    }

    #[test]
    fn a_lone_angle_bracket_is_actions_mode_with_no_text() {
        let parsed = parse(ACTIONS_QUERY);
        assert_eq!(parsed.mode, Mode::Actions);
        assert_eq!(parsed.text, "");
    }

    #[test]
    fn an_angle_bracket_anywhere_else_stays_in_notes_mode() {
        assert_eq!(parse("a>b").mode, Mode::Notes);
        assert_eq!(parse(" >a").mode, Mode::Notes);
    }
}
