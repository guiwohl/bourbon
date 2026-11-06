use std::fmt::Write;

use crate::note::Note;
use crate::timestamp::Timestamp;

pub fn render(note: &Note) -> String {
    let tags = if note.tags.is_empty() {
        "tags: []\n".to_owned()
    } else {
        let mut list = String::from("tags:\n");
        for tag in &note.tags {
            let _ = writeln!(list, "  - {}", quoted(&tag.name));
        }
        list
    };
    format!(
        "---\nid: {}\ntitle: {}\n{tags}created: {}\nupdated: {}\n---\n\n{}",
        note.id,
        quoted(&note.title),
        rfc3339(note.created_at),
        rfc3339(note.updated_at),
        note.body
    )
}

fn quoted(text: &str) -> String {
    let mut quoted = String::from('"');
    for character in text.chars() {
        match character {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            other if other.is_control() || matches!(other, '\u{2028}' | '\u{2029}') => {
                let _ = write!(quoted, "\\u{:04x}", u32::from(other));
            }
            other => quoted.push(other),
        }
    }
    quoted.push('"');
    quoted
}

fn rfc3339(timestamp: Timestamp) -> String {
    jiff::Timestamp::from_millisecond(timestamp.as_millis())
        .unwrap_or(jiff::Timestamp::UNIX_EPOCH)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NoteId, NoteStatus, NotebookId, Tag, TagColor, TagId};

    fn note(title: &str, body: &str, tag_names: &[&str]) -> Note {
        Note {
            id: NoteId::parse("abcdefghijkl").unwrap(),
            title: title.to_owned(),
            body: body.to_owned(),
            created_at: Timestamp::from_millis(1_700_000_000_000),
            updated_at: Timestamp::from_millis(1_700_000_123_456),
            notebook: NotebookId::generate(),
            status: NoteStatus::None,
            pinned: false,
            is_template: false,
            tags: tag_names
                .iter()
                .map(|name| Tag {
                    id: TagId::generate(),
                    name: (*name).to_owned(),
                    color: TagColor::Default,
                })
                .collect(),
            trashed_at: None,
        }
    }

    #[test]
    fn renders_frontmatter_blank_line_and_body() {
        let rendered = render(&note("Hello", "# Hi\n\ntext\n", &["work", "idea"]));
        assert_eq!(
            rendered,
            "---\nid: abcdefghijkl\ntitle: \"Hello\"\ntags:\n  - \"work\"\n  - \"idea\"\ncreated: 2023-11-14T22:13:20Z\nupdated: 2023-11-14T22:15:23.456Z\n---\n\n# Hi\n\ntext\n"
        );
    }

    #[test]
    fn a_note_without_tags_has_an_empty_tag_list() {
        let rendered = render(&note("T", "", &[]));
        assert!(rendered.contains("\ntags: []\n"));
        assert!(rendered.ends_with("---\n\n"));
    }

    #[test]
    fn quotes_escape_backslashes_and_line_breaks() {
        let rendered = render(&note("say \"hi\" \\ now\nnext\ttab\r", "", &[]));
        assert!(rendered.contains("title: \"say \\\"hi\\\" \\\\ now\\nnext\\ttab\\r\"\n"));
    }

    #[test]
    fn other_control_and_line_separator_characters_are_escaped() {
        let rendered = render(&note("a\u{1b}b\u{2028}c\u{85}d", "", &[]));
        assert!(rendered.contains("title: \"a\\u001bb\\u2028c\\u0085d\"\n"));
    }

    #[test]
    fn yaml_looking_titles_stay_plain_strings() {
        let rendered = render(&note("true: [1, 2] # not a comment", "", &["null"]));
        assert!(rendered.contains("title: \"true: [1, 2] # not a comment\"\n"));
        assert!(rendered.contains("  - \"null\"\n"));
    }

    #[test]
    fn out_of_range_timestamps_fall_back_to_the_epoch() {
        let mut broken = note("T", "", &[]);
        broken.created_at = Timestamp::from_millis(i64::MAX);
        assert!(render(&broken).contains("created: 1970-01-01T00:00:00Z\n"));
    }
}
