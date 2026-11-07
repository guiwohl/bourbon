use std::collections::HashSet;

use bourbon_core::{NoteId, NoteSummary};
use gpui_kit::assets::IconName;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::{
    Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _,
    TestSupportExt as _, div, px,
};

use crate::icons::glyph;
use crate::workspace::Workspace;

gpui_kit::actions!(bourbon, [ClearFilter]);

const FILTER_HEIGHT: f32 = 32.;
const SIDE_MARGIN: f32 = 12.;

pub(crate) fn narrow(
    notes: Vec<NoteSummary>,
    matching: Option<&HashSet<NoteId>>,
) -> Vec<NoteSummary> {
    match matching {
        Some(ids) => notes
            .into_iter()
            .filter(|note| ids.contains(&note.id))
            .collect(),
        None => notes,
    }
}

pub(crate) fn render(
    input: &Entity<InputState>,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let theme = cx.theme();
    div()
        .id("note-filter")
        .test_support()
        .key_context("NoteFilter")
        .on_action(cx.listener(|workspace, _: &ClearFilter, window, cx| {
            workspace.clear_filter(window, cx);
        }))
        .mx(px(SIDE_MARGIN))
        .mb_2()
        .h(px(FILTER_HEIGHT))
        .flex_none()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .px_2()
        .rounded(px(6.))
        .border_1()
        .border_color(theme.border)
        .bg(theme.background)
        .text_color(theme.muted_foreground)
        .child(glyph(IconName::Funnel))
        .child(
            div().flex_1().child(
                Input::new(input)
                    .id("note-filter-input")
                    .appearance(false)
                    .bordered(false),
            ),
        )
}

#[cfg(test)]
mod tests {
    use bourbon_core::{NoteStatus, Timestamp};

    use super::*;

    fn note() -> NoteSummary {
        NoteSummary {
            id: NoteId::generate(),
            title: String::new(),
            excerpt: String::new(),
            updated_at: Timestamp::from_millis(0),
            created_at: Timestamp::from_millis(0),
            pinned: false,
            status: NoteStatus::None,
            tags: Vec::new(),
            task_progress: None,
        }
    }

    #[test]
    fn no_filter_keeps_every_note() {
        let notes = vec![note(), note()];
        assert_eq!(narrow(notes.clone(), None), notes);
    }

    #[test]
    fn a_filter_keeps_only_matching_notes_in_their_order() {
        let notes = vec![note(), note(), note()];
        let matching = HashSet::from([notes[2].id.clone(), notes[0].id.clone()]);
        let kept = narrow(notes.clone(), Some(&matching));
        assert_eq!(kept, vec![notes[0].clone(), notes[2].clone()]);
    }

    #[test]
    fn a_filter_matching_nothing_empties_the_list() {
        assert_eq!(narrow(vec![note()], Some(&HashSet::new())).len(), 0);
    }
}
