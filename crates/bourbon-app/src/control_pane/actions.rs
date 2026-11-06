use bourbon_core::NoteStatus;

use crate::presentation::status_label;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaneAction {
    NewNote,
    NewTemplate,
    ToggleSidebar,
    OpenSettings,
    GoToAllNotes,
    GoToTemplates,
    GoToTrash,
    SortByUpdated,
    SortByCreated,
    SortByTitle,
    SetStatus(NoteStatus),
    MoveToNotebook,
    DuplicateNote,
    TogglePin,
    CopyCliCommand,
    MoveToTrash,
    EmptyTrash,
    HistoryBack,
    HistoryForward,
    CycleViewMode,
    Find,
    Replace,
    ToggleFocusMode,
}

const ALL: [PaneAction; 27] = [
    PaneAction::NewNote,
    PaneAction::NewTemplate,
    PaneAction::ToggleSidebar,
    PaneAction::OpenSettings,
    PaneAction::GoToAllNotes,
    PaneAction::GoToTemplates,
    PaneAction::GoToTrash,
    PaneAction::SortByUpdated,
    PaneAction::SortByCreated,
    PaneAction::SortByTitle,
    PaneAction::SetStatus(NoteStatus::None),
    PaneAction::SetStatus(NoteStatus::Active),
    PaneAction::SetStatus(NoteStatus::OnHold),
    PaneAction::SetStatus(NoteStatus::Completed),
    PaneAction::SetStatus(NoteStatus::Dropped),
    PaneAction::MoveToNotebook,
    PaneAction::DuplicateNote,
    PaneAction::TogglePin,
    PaneAction::CopyCliCommand,
    PaneAction::MoveToTrash,
    PaneAction::EmptyTrash,
    PaneAction::HistoryBack,
    PaneAction::HistoryForward,
    PaneAction::CycleViewMode,
    PaneAction::Find,
    PaneAction::Replace,
    PaneAction::ToggleFocusMode,
];

impl PaneAction {
    pub(crate) fn name(self) -> String {
        match self {
            Self::NewNote => "New note".to_owned(),
            Self::NewTemplate => "New template".to_owned(),
            Self::ToggleSidebar => "Toggle sidebar".to_owned(),
            Self::OpenSettings => "Open settings".to_owned(),
            Self::GoToAllNotes => "Go to All Notes".to_owned(),
            Self::GoToTemplates => "Go to Templates".to_owned(),
            Self::GoToTrash => "Go to Trash".to_owned(),
            Self::SortByUpdated => "Sort by updated".to_owned(),
            Self::SortByCreated => "Sort by created".to_owned(),
            Self::SortByTitle => "Sort by title".to_owned(),
            Self::SetStatus(status) => format!("Set status {}", status_label(status)),
            Self::MoveToNotebook => "Move to notebook\u{2026}".to_owned(),
            Self::DuplicateNote => "Duplicate note".to_owned(),
            Self::TogglePin => "Pin/Unpin note".to_owned(),
            Self::CopyCliCommand => "Copy CLI command".to_owned(),
            Self::MoveToTrash => "Move to Trash".to_owned(),
            Self::EmptyTrash => "Empty Trash".to_owned(),
            Self::HistoryBack => "History back".to_owned(),
            Self::HistoryForward => "History forward".to_owned(),
            Self::CycleViewMode => "Cycle view mode".to_owned(),
            Self::Find => "Find".to_owned(),
            Self::Replace => "Replace".to_owned(),
            Self::ToggleFocusMode => "Toggle focus mode".to_owned(),
        }
    }

    pub(crate) fn shortcut(self) -> Option<&'static str> {
        match self {
            Self::NewNote => Some("Ctrl+N"),
            Self::ToggleSidebar => Some("Ctrl+/"),
            Self::OpenSettings => Some("Ctrl+,"),
            Self::SetStatus(NoteStatus::None) => Some("Shift+Alt+1"),
            Self::SetStatus(NoteStatus::Active) => Some("Shift+Alt+2"),
            Self::SetStatus(NoteStatus::OnHold) => Some("Shift+Alt+3"),
            Self::SetStatus(NoteStatus::Completed) => Some("Shift+Alt+4"),
            Self::SetStatus(NoteStatus::Dropped) => Some("Shift+Alt+5"),
            Self::MoveToNotebook => Some("Ctrl+Alt+M"),
            Self::HistoryBack => Some("Alt+Left"),
            Self::HistoryForward => Some("Alt+Right"),
            Self::CycleViewMode => Some("Ctrl+E"),
            Self::Find => Some("Ctrl+F"),
            Self::Replace => Some("Ctrl+H"),
            Self::ToggleFocusMode => Some("Ctrl+Shift+F"),
            _ => None,
        }
    }

    pub(crate) fn needs_selected_note(self) -> bool {
        matches!(
            self,
            Self::SetStatus(_)
                | Self::MoveToNotebook
                | Self::DuplicateNote
                | Self::TogglePin
                | Self::CopyCliCommand
                | Self::MoveToTrash
                | Self::CycleViewMode
                | Self::Find
                | Self::Replace
                | Self::ToggleFocusMode
        )
    }
}

pub(crate) fn matching(text: &str, has_selected_note: bool) -> Vec<PaneAction> {
    let wanted: Vec<char> = lowercase_characters(text)
        .into_iter()
        .filter(|character| !character.is_whitespace())
        .collect();
    let mut ranked: Vec<(MatchQuality, PaneAction)> = ALL
        .into_iter()
        .filter(|action| has_selected_note || !action.needs_selected_note())
        .filter_map(|action| {
            best_match(&lowercase_characters(&action.name()), &wanted)
                .map(|quality| (quality, action))
        })
        .collect();
    ranked.sort_by_key(|(quality, _)| *quality);
    ranked.into_iter().map(|(_, action)| action).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MatchQuality {
    span: usize,
    start: usize,
}

fn lowercase_characters(text: &str) -> Vec<char> {
    text.to_lowercase().chars().collect()
}

fn best_match(name: &[char], wanted: &[char]) -> Option<MatchQuality> {
    let Some(first) = wanted.first() else {
        return Some(MatchQuality { span: 0, start: 0 });
    };
    name.iter()
        .enumerate()
        .filter(|(_, character)| *character == first)
        .filter_map(|(start, _)| {
            span_from(name, start, wanted).map(|span| MatchQuality { span, start })
        })
        .min()
}

fn span_from(name: &[char], start: usize, wanted: &[char]) -> Option<usize> {
    let mut position = start;
    for character in wanted {
        position += name[position..]
            .iter()
            .position(|candidate| candidate == character)?;
        position += 1;
    }
    Some(position - start)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(text: &str, has_selected_note: bool) -> Vec<String> {
        matching(text, has_selected_note)
            .into_iter()
            .map(PaneAction::name)
            .collect()
    }

    #[test]
    fn open_settings_shows_its_shortcut() {
        assert_eq!(PaneAction::OpenSettings.shortcut(), Some("Ctrl+,"));
        assert_eq!(PaneAction::OpenSettings.name(), "Open settings");
    }

    #[test]
    fn empty_text_offers_every_action_in_declared_order() {
        assert_eq!(matching("", true), ALL);
    }

    #[test]
    fn there_is_one_action_per_command_and_every_name_is_unique() {
        let mut all_names: Vec<String> = ALL.into_iter().map(PaneAction::name).collect();
        all_names.sort();
        all_names.dedup();
        assert_eq!(all_names.len(), ALL.len());
    }

    #[test]
    fn matching_is_a_case_insensitive_subsequence() {
        assert_eq!(
            names("TRASH", true),
            ["Go to Trash", "Empty Trash", "Move to Trash"]
        );
        assert_eq!(
            names("gtt", true),
            ["Go to Templates", "Go to Trash", "Go to All Notes"]
        );
    }

    #[test]
    fn text_that_is_not_a_subsequence_matches_nothing() {
        assert_eq!(matching("zzz", true), []);
        assert_eq!(matching("ssn", false), []);
    }

    #[test]
    fn spaces_in_the_text_are_ignored() {
        assert_eq!(names("new note", true), ["New note"]);
        assert_eq!(names("newnote", true), ["New note"]);
    }

    #[test]
    fn a_contiguous_match_beats_a_scattered_one() {
        let ranked = names("tem", true);
        assert_eq!(ranked.first().map(String::as_str), Some("New template"));
        assert_eq!(ranked.get(1).map(String::as_str), Some("Go to Templates"));
    }

    #[test]
    fn an_earlier_match_beats_a_later_one_of_the_same_span() {
        assert_eq!(
            names("note", true),
            [
                "New note",
                "Move to notebook\u{2026}",
                "Go to All Notes",
                "Duplicate note",
                "Pin/Unpin note"
            ]
        );
    }

    #[test]
    fn ties_keep_the_declared_order() {
        assert_eq!(
            names("sort by", true),
            ["Sort by updated", "Sort by created", "Sort by title"]
        );
    }

    #[test]
    fn the_best_window_is_used_not_the_first_occurrence_of_the_first_letter() {
        assert_eq!(
            best_match(&chars("a xxx ab"), &chars("ab")),
            Some(MatchQuality { span: 2, start: 6 })
        );
    }

    fn chars(text: &str) -> Vec<char> {
        text.chars().collect()
    }

    #[test]
    fn actions_needing_a_note_are_hidden_without_one() {
        let without = matching("", false);
        assert!(!without.contains(&PaneAction::DuplicateNote));
        assert!(!without.contains(&PaneAction::TogglePin));
        assert!(!without.contains(&PaneAction::MoveToTrash));
        assert!(!without.contains(&PaneAction::MoveToNotebook));
        assert!(!without.contains(&PaneAction::SetStatus(NoteStatus::Active)));
        assert!(without.contains(&PaneAction::NewNote));
        assert!(without.contains(&PaneAction::EmptyTrash));
        assert_eq!(without.len(), ALL.len() - 14);
    }

    #[test]
    fn every_status_has_a_set_status_action() {
        for status in NoteStatus::ALL {
            assert!(ALL.contains(&PaneAction::SetStatus(status)));
        }
    }

    #[test]
    fn editor_actions_show_their_shortcuts() {
        assert_eq!(PaneAction::CycleViewMode.shortcut(), Some("Ctrl+E"));
        assert_eq!(PaneAction::Find.shortcut(), Some("Ctrl+F"));
        assert_eq!(PaneAction::Replace.shortcut(), Some("Ctrl+H"));
        assert_eq!(PaneAction::ToggleFocusMode.shortcut(), Some("Ctrl+Shift+F"));
    }

    #[test]
    fn shortcuts_match_the_bound_keys() {
        assert_eq!(PaneAction::NewNote.shortcut(), Some("Ctrl+N"));
        assert_eq!(PaneAction::ToggleSidebar.shortcut(), Some("Ctrl+/"));
        assert_eq!(PaneAction::MoveToNotebook.shortcut(), Some("Ctrl+Alt+M"));
        assert_eq!(PaneAction::HistoryBack.shortcut(), Some("Alt+Left"));
        assert_eq!(PaneAction::HistoryForward.shortcut(), Some("Alt+Right"));
        assert_eq!(PaneAction::GoToTrash.shortcut(), None);
        let statuses: Vec<_> = NoteStatus::ALL
            .into_iter()
            .map(|status| PaneAction::SetStatus(status).shortcut())
            .collect();
        assert_eq!(
            statuses,
            [
                Some("Shift+Alt+1"),
                Some("Shift+Alt+2"),
                Some("Shift+Alt+3"),
                Some("Shift+Alt+4"),
                Some("Shift+Alt+5")
            ]
        );
    }

    #[test]
    fn set_status_names_use_the_status_labels() {
        assert_eq!(
            PaneAction::SetStatus(NoteStatus::OnHold).name(),
            "Set status On Hold"
        );
    }
}
