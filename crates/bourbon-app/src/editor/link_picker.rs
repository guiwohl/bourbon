use bourbon_core::NoteSummary;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, div,
};

use crate::presentation::display_title;
use crate::workspace::Workspace;

const MAX_SUGGESTIONS: usize = 8;

pub(crate) struct LinkPicker {
    pub(crate) start: usize,
    pub(crate) candidates: Vec<NoteSummary>,
    pub(crate) highlight: usize,
}

impl LinkPicker {
    pub(crate) fn matching(start: usize, query: &str, notes: &[NoteSummary]) -> Option<Self> {
        let needle = query.to_lowercase();
        let candidates: Vec<NoteSummary> = notes
            .iter()
            .filter(|note| display_title(&note.title).to_lowercase().contains(&needle))
            .take(MAX_SUGGESTIONS)
            .cloned()
            .collect();
        (!candidates.is_empty()).then_some(Self {
            start,
            candidates,
            highlight: 0,
        })
    }

    pub(crate) fn highlight_next(&mut self) {
        self.highlight = (self.highlight + 1).min(self.candidates.len() - 1);
    }

    pub(crate) fn highlight_previous(&mut self) {
        self.highlight = self.highlight.saturating_sub(1);
    }

    pub(crate) fn candidate(&self, index: usize) -> Option<&NoteSummary> {
        self.candidates.get(index)
    }
}

pub(super) fn render(picker: &LinkPicker, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let (background, foreground) = (cx.theme().accent, cx.theme().accent_foreground);
    let rows = picker.candidates.iter().enumerate().map(|(index, note)| {
        let is_highlighted = index == picker.highlight;
        div()
            .id(format!("link-suggestion-{}", note.id))
            .aria_selected(is_highlighted)
            .test_support()
            .px_2()
            .py_1()
            .rounded_md()
            .cursor_pointer()
            .child(display_title(&note.title).to_owned())
            .when(is_highlighted, |row| {
                row.bg(background).text_color(foreground)
            })
            .on_click(cx.listener(move |workspace, _, window, cx| {
                workspace.pick_link_suggestion(index, window, cx);
            }))
    });
    div()
        .id("link-suggestions")
        .test_support()
        .flex()
        .flex_col()
        .p_1()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().popover)
        .children(rows)
}

#[cfg(test)]
mod tests {
    use bourbon_core::{NoteId, NoteStatus, NoteSummary, Timestamp};

    use super::LinkPicker;

    fn note(title: &str) -> NoteSummary {
        NoteSummary {
            id: NoteId::generate(),
            title: title.to_owned(),
            excerpt: String::new(),
            updated_at: Timestamp::from_millis(0),
            created_at: Timestamp::from_millis(0),
            pinned: false,
            status: NoteStatus::None,
            tags: Vec::new(),
        }
    }

    #[test]
    fn matching_ignores_case_and_keeps_listing_order() {
        let notes = [note("Alpha"), note("beta"), note("ALPHABET")];
        let picker = LinkPicker::matching(3, "alph", &notes).unwrap();
        let titles: Vec<&str> = picker
            .candidates
            .iter()
            .map(|candidate| candidate.title.as_str())
            .collect();
        assert_eq!(titles, ["Alpha", "ALPHABET"]);
        assert_eq!(picker.start, 3);
        assert_eq!(picker.highlight, 0);
    }

    #[test]
    fn an_empty_query_matches_everything_up_to_the_limit() {
        let notes: Vec<NoteSummary> = (0..20).map(|n| note(&format!("n{n}"))).collect();
        assert_eq!(
            LinkPicker::matching(0, "", &notes)
                .unwrap()
                .candidates
                .len(),
            8
        );
    }

    #[test]
    fn untitled_notes_match_by_their_displayed_name() {
        let notes = [note("")];
        assert!(LinkPicker::matching(0, "untit", &notes).is_some());
    }

    #[test]
    fn nothing_matching_gives_no_picker() {
        assert!(LinkPicker::matching(0, "zzz", &[note("a")]).is_none());
    }

    #[test]
    fn the_highlight_stays_inside_the_candidates() {
        let mut picker = LinkPicker::matching(0, "", &[note("a"), note("b")]).unwrap();
        picker.highlight_previous();
        assert_eq!(picker.highlight, 0);
        picker.highlight_next();
        picker.highlight_next();
        assert_eq!(picker.highlight, 1);
        picker.highlight_previous();
        assert_eq!(picker.highlight, 0);
    }
}
