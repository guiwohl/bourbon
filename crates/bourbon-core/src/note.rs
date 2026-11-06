use crate::identifier::{NoteId, NotebookId};
use crate::status::NoteStatus;
use crate::tag::Tag;
use crate::timestamp::Timestamp;

const EXCERPT_LENGTH: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub id: NoteId,
    pub title: String,
    pub body: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub notebook: NotebookId,
    pub status: NoteStatus,
    pub pinned: bool,
    pub is_template: bool,
    pub tags: Vec<Tag>,
    pub trashed_at: Option<Timestamp>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteSummary {
    pub id: NoteId,
    pub title: String,
    pub excerpt: String,
    pub updated_at: Timestamp,
    pub created_at: Timestamp,
    pub pinned: bool,
    pub status: NoteStatus,
    pub tags: Vec<Tag>,
}

pub(crate) fn excerpt_of(body: &str) -> String {
    body.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|line| line.chars().take(EXCERPT_LENGTH).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excerpt_is_the_first_line() {
        assert_eq!(excerpt_of("first\nsecond"), "first");
    }

    #[test]
    fn excerpt_skips_blank_lines_and_trims() {
        assert_eq!(excerpt_of("\n  \n\t  hello world \t\nnext"), "hello world");
    }

    #[test]
    fn excerpt_of_blank_body_is_empty() {
        assert_eq!(excerpt_of(""), "");
        assert_eq!(excerpt_of("  \n\r\n \t"), "");
    }

    #[test]
    fn excerpt_handles_windows_line_endings() {
        assert_eq!(excerpt_of("one\r\ntwo"), "one");
    }

    #[test]
    fn excerpt_is_cut_at_one_hundred_characters() {
        let body = "a".repeat(250);
        assert_eq!(excerpt_of(&body), "a".repeat(100));
    }

    #[test]
    fn excerpt_of_exactly_one_hundred_characters_is_kept_whole() {
        let body = "b".repeat(100);
        assert_eq!(excerpt_of(&body), body);
    }

    #[test]
    fn excerpt_cuts_on_character_boundaries() {
        let body = "ç".repeat(150);
        let excerpt = excerpt_of(&body);
        assert_eq!(excerpt.chars().count(), 100);
        assert!(excerpt.chars().all(|character| character == 'ç'));
    }
}
