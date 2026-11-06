#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LinkQuery<'a> {
    pub(crate) start: usize,
    pub(crate) text: &'a str,
}

const OPENER: &str = "[[";

pub(crate) fn link_query_before_cursor(text_before_cursor: &str) -> Option<LinkQuery<'_>> {
    let line_start = text_before_cursor
        .rfind('\n')
        .map_or(0, |newline| newline + 1);
    let line = &text_before_cursor[line_start..];
    let opener = line.rfind(OPENER)?;
    let text = &line[opener + OPENER.len()..];
    if text.contains(['[', ']']) {
        return None;
    }
    Some(LinkQuery {
        start: line_start + opener,
        text,
    })
}

pub(crate) fn closing_brackets_after(text_after_cursor: &str) -> usize {
    text_after_cursor
        .chars()
        .take(OPENER.len())
        .take_while(|&character| character == ']')
        .count()
}

#[cfg(test)]
mod tests {
    use super::{LinkQuery, closing_brackets_after, link_query_before_cursor};

    #[test]
    fn the_brackets_the_editor_closed_for_us_are_up_to_two_right_after_the_cursor() {
        assert_eq!(closing_brackets_after("]] rest"), 2);
        assert_eq!(closing_brackets_after("]"), 1);
        assert_eq!(closing_brackets_after("]]]"), 2);
        assert_eq!(closing_brackets_after(" ]]"), 0);
        assert_eq!(closing_brackets_after(""), 0);
    }

    #[test]
    fn two_opening_brackets_start_an_empty_query() {
        assert_eq!(
            link_query_before_cursor("see [["),
            Some(LinkQuery { start: 4, text: "" })
        );
    }

    #[test]
    fn what_follows_the_brackets_is_the_query() {
        assert_eq!(
            link_query_before_cursor("see [[gro ce"),
            Some(LinkQuery {
                start: 4,
                text: "gro ce"
            })
        );
    }

    #[test]
    fn a_single_bracket_is_not_a_trigger() {
        assert_eq!(link_query_before_cursor("see [gro"), None);
    }

    #[test]
    fn a_closed_link_or_a_new_line_ends_the_query() {
        assert_eq!(link_query_before_cursor("[[done]] and more"), None);
        assert_eq!(link_query_before_cursor("[[abc\ndef"), None);
    }

    #[test]
    fn the_start_is_a_byte_offset_into_the_whole_text() {
        assert_eq!(
            link_query_before_cursor("a\u{e7}\n[[x"),
            Some(LinkQuery {
                start: 4,
                text: "x"
            })
        );
    }

    #[test]
    fn the_nearest_opener_wins() {
        assert_eq!(
            link_query_before_cursor("[[a [[b"),
            Some(LinkQuery {
                start: 4,
                text: "b"
            })
        );
    }
}
