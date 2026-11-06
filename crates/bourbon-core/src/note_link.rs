use crate::NoteId;

const PREFIX: &str = "bourbon://note/";

#[must_use]
pub fn format_note_link(title: &str, id: &NoteId) -> String {
    let mut label = String::with_capacity(title.len());
    for character in title.chars() {
        match character {
            '\\' | '[' | ']' => {
                label.push('\\');
                label.push(character);
            }
            '\n' | '\r' => label.push(' '),
            other => label.push(other),
        }
    }
    format!("[{label}]({PREFIX}{id})")
}

#[must_use]
pub fn parse_note_link(url: &str) -> Option<NoteId> {
    url.strip_prefix(PREFIX)
        .and_then(|identifier| NoteId::parse(identifier).ok())
}

#[cfg(test)]
mod tests {
    use super::{format_note_link, parse_note_link};
    use crate::NoteId;

    #[test]
    fn a_link_is_standard_markdown_pointing_at_the_note_scheme() {
        let id = NoteId::parse("abcdefghij12").unwrap();
        assert_eq!(
            format_note_link("Groceries", &id),
            "[Groceries](bourbon://note/abcdefghij12)"
        );
    }

    #[test]
    fn brackets_backslashes_and_line_breaks_in_the_title_cannot_break_the_link() {
        let id = NoteId::parse("abcdefghij12").unwrap();
        assert_eq!(
            format_note_link("a [b] \\ c\nd", &id),
            "[a \\[b\\] \\\\ c d](bourbon://note/abcdefghij12)"
        );
    }

    #[test]
    fn a_note_url_parses_back_to_its_id() {
        let id = NoteId::generate();
        let url = format!("bourbon://note/{id}");
        assert_eq!(parse_note_link(&url), Some(id));
    }

    #[test]
    fn other_urls_and_malformed_ids_are_not_note_links() {
        assert_eq!(parse_note_link("https://example.com"), None);
        assert_eq!(parse_note_link("bourbon://note/short"), None);
        assert_eq!(parse_note_link("bourbon://note/"), None);
        assert_eq!(parse_note_link("bourbon://note/abcdefghij12/extra"), None);
        assert_eq!(parse_note_link("xbourbon://note/abcdefghij12"), None);
    }
}
