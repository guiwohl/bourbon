#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ViewMode {
    #[default]
    Edit,
    SideBySide,
    Preview,
}

impl ViewMode {
    pub(crate) const ALL: [Self; 3] = [Self::Edit, Self::SideBySide, Self::Preview];

    pub(crate) fn next(self) -> Self {
        match self {
            Self::Edit => Self::SideBySide,
            Self::SideBySide => Self::Preview,
            Self::Preview => Self::Edit,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Edit => "Edit",
            Self::SideBySide => "Side-by-side",
            Self::Preview => "Preview",
        }
    }

    pub(crate) fn shows_editor(self) -> bool {
        self != Self::Preview
    }

    pub(crate) fn shows_preview(self) -> bool {
        self != Self::Edit
    }
}

#[cfg(test)]
mod tests {
    use super::ViewMode;

    #[test]
    fn the_default_is_edit() {
        assert_eq!(ViewMode::default(), ViewMode::Edit);
    }

    #[test]
    fn cycling_visits_edit_side_by_side_preview_and_wraps() {
        assert_eq!(ViewMode::Edit.next(), ViewMode::SideBySide);
        assert_eq!(ViewMode::SideBySide.next(), ViewMode::Preview);
        assert_eq!(ViewMode::Preview.next(), ViewMode::Edit);
    }

    #[test]
    fn each_mode_shows_the_panes_it_names() {
        assert!(ViewMode::Edit.shows_editor() && !ViewMode::Edit.shows_preview());
        assert!(ViewMode::SideBySide.shows_editor() && ViewMode::SideBySide.shows_preview());
        assert!(!ViewMode::Preview.shows_editor() && ViewMode::Preview.shows_preview());
    }
}
