use bourbon_core::{NoteSummary, SearchHit};
use bourbon_ui::ThemeExt as _;
use gpui::prelude::FluentBuilder as _;
use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, div, px,
};

use crate::presentation::display_title;
use crate::test_id::TestSupportExt as _;
use crate::workspace::Workspace;

pub(crate) const MAX_SUGGESTIONS: usize = 8;
const PICKER_OFFSET: f32 = 16.;

pub(crate) struct LinkPicker {
    pub(crate) start: usize,
    pub(crate) candidates: Vec<NoteSummary>,
    pub(crate) highlight: usize,
}

impl LinkPicker {
    pub(crate) fn from_hits(start: usize, hits: Vec<SearchHit>) -> Option<Self> {
        let candidates: Vec<NoteSummary> = hits.into_iter().map(|hit| hit.summary).collect();
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
    let tokens = cx.theme().tokens.clone();
    let rows = picker.candidates.iter().enumerate().map(|(index, note)| {
        let is_highlighted = index == picker.highlight;
        div()
            .id(format!("link-suggestion-{}", note.id))
            .relative()
            .aria_selected(is_highlighted)
            .test_support()
            .px(tokens.scale.menu_item_padding_horizontal)
            .py(tokens.scale.menu_item_padding_vertical)
            .cursor_pointer()
            .child(display_title(&note.title).to_owned())
            .when(is_highlighted, |row| {
                row.bg(tokens.colors.menu_hover)
                    .child(super::header::highlight_marker(format!(
                        "link-suggestion-{}-highlight",
                        note.id
                    )))
            })
            .on_click(cx.listener(move |workspace, _, window, cx| {
                workspace.pick_link_suggestion(index, window, cx);
            }))
    });
    div()
        .id("link-suggestions")
        .test_support()
        .occlude()
        .absolute()
        .top(px(PICKER_OFFSET))
        .left(px(PICKER_OFFSET))
        .min_w(tokens.scale.menu_min_width)
        .flex()
        .flex_col()
        .rounded(tokens.scale.radius_large)
        .border_1()
        .border_color(tokens.colors.border)
        .bg(tokens.colors.menu_background)
        .shadow(tokens.shadows.raised.clone())
        .font_family(tokens.fonts.ui_family.clone())
        .text_size(tokens.scale.medium)
        .text_color(tokens.colors.text)
        .children(rows)
}

#[cfg(test)]
mod tests {
    use bourbon_core::{NoteId, NoteStatus, NoteSummary, SearchHit, Timestamp};

    use super::LinkPicker;

    fn hit(title: &str) -> SearchHit {
        SearchHit {
            summary: NoteSummary {
                id: NoteId::generate(),
                title: title.to_owned(),
                excerpt: String::new(),
                updated_at: Timestamp::from_millis(0),
                created_at: Timestamp::from_millis(0),
                pinned: false,
                status: NoteStatus::None,
                tags: Vec::new(),
                task_progress: None,
            },
            snippet: String::new(),
        }
    }

    #[test]
    fn the_hits_become_candidates_in_order() {
        let picker = LinkPicker::from_hits(3, vec![hit("Alpha"), hit("beta")]).unwrap();
        let titles: Vec<&str> = picker
            .candidates
            .iter()
            .map(|candidate| candidate.title.as_str())
            .collect();
        assert_eq!(titles, ["Alpha", "beta"]);
        assert_eq!(picker.start, 3);
        assert_eq!(picker.highlight, 0);
    }

    #[test]
    fn no_hits_give_no_picker() {
        assert!(LinkPicker::from_hits(0, Vec::new()).is_none());
    }

    #[test]
    fn the_highlight_stays_inside_the_candidates() {
        let mut picker = LinkPicker::from_hits(0, vec![hit("a"), hit("b")]).unwrap();
        picker.highlight_previous();
        assert_eq!(picker.highlight, 0);
        picker.highlight_next();
        picker.highlight_next();
        assert_eq!(picker.highlight, 1);
        picker.highlight_previous();
        assert_eq!(picker.highlight, 0);
    }
}
