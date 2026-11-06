use crate::identifier::TagId;
use crate::stored_text::stored_text_enum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TagColor {
    #[default]
    Default,
    Red,
    Orange,
    Yellow,
    Olive,
    Green,
    Teal,
    Blue,
    Violet,
    Purple,
    Pink,
    Brown,
    Grey,
    Black,
}

impl TagColor {
    pub const ALL: [Self; 14] = [
        Self::Default,
        Self::Red,
        Self::Orange,
        Self::Yellow,
        Self::Olive,
        Self::Green,
        Self::Teal,
        Self::Blue,
        Self::Violet,
        Self::Purple,
        Self::Pink,
        Self::Brown,
        Self::Grey,
        Self::Black,
    ];

    #[must_use]
    pub fn as_stored(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Red => "red",
            Self::Orange => "orange",
            Self::Yellow => "yellow",
            Self::Olive => "olive",
            Self::Green => "green",
            Self::Teal => "teal",
            Self::Blue => "blue",
            Self::Violet => "violet",
            Self::Purple => "purple",
            Self::Pink => "pink",
            Self::Brown => "brown",
            Self::Grey => "grey",
            Self::Black => "black",
        }
    }

    #[must_use]
    pub fn from_stored(text: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|color| color.as_stored() == text)
    }
}

stored_text_enum!(TagColor);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub id: TagId,
    pub name: String,
    pub color: TagColor,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_names_are_the_lowercase_variant_names() {
        for color in TagColor::ALL {
            assert_eq!(color.as_stored(), format!("{color:?}").to_lowercase());
        }
    }

    #[test]
    fn stored_names_parse_back() {
        for color in TagColor::ALL {
            assert_eq!(TagColor::from_stored(color.as_stored()), Some(color));
        }
        assert_eq!(TagColor::from_stored("magenta"), None);
        assert_eq!(TagColor::from_stored("Red"), None);
    }

    #[test]
    fn palette_has_fourteen_distinct_colors() {
        let mut names: Vec<_> = TagColor::ALL.into_iter().map(TagColor::as_stored).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 14);
    }

    #[test]
    fn color_round_trips_through_sqlite() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        let read: TagColor = connection
            .query_row("SELECT ?1", [TagColor::Violet], |row| row.get(0))
            .unwrap();
        assert_eq!(read, TagColor::Violet);
    }

    #[test]
    fn unknown_stored_color_fails_to_read() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        let result: rusqlite::Result<TagColor> =
            connection.query_row("SELECT 'magenta'", [], |row| row.get(0));
        assert!(result.is_err());
    }
}
