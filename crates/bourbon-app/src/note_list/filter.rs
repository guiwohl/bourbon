use std::collections::HashSet;

use bourbon_core::{NoteId, NoteSummary};
use bourbon_ui::{Icon, IconName, ThemeExt as _};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::{
    Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _,
    TestSupportExt as _, div, px,
};

use crate::workspace::Workspace;

gpui_kit::actions!(bourbon, [ClearFilter]);

const SIDE_MARGIN: f32 = 12.;
const VERTICAL_MARGIN: f32 = 2.;
const HORIZONTAL_PADDING: f32 = 8.;
const FIELD_HEIGHT: f32 = 30.;
const GAP_BELOW: f32 = 4.;
const ICON_OPACITY: f32 = 0.3;
const ICON_SIZE: f32 = 16.;

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
    let tokens = cx.theme().tokens.clone();
    let colors = tokens.colors;
    div()
        .h(tokens.scale.search_bar_height)
        .mb(px(GAP_BELOW))
        .flex_none()
        .flex()
        .items_center()
        .child(
            div()
                .id("note-filter")
                .test_support()
                .key_context("NoteFilter")
                .on_action(cx.listener(|workspace, _: &ClearFilter, window, cx| {
                    workspace.clear_filter(window, cx);
                }))
                .flex_1()
                .min_w_0()
                .mx(px(SIDE_MARGIN))
                .my(px(VERTICAL_MARGIN))
                .px(px(HORIZONTAL_PADDING))
                .h(px(FIELD_HEIGHT))
                .overflow_hidden()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(HORIZONTAL_PADDING))
                .rounded(tokens.scale.radius_default)
                .border_1()
                .border_color(colors.internal_border)
                .text_color(colors.text)
                .child(
                    div().flex_none().opacity(ICON_OPACITY).child(
                        Icon::new(IconName::Search)
                            .size(px(ICON_SIZE))
                            .color(colors.text),
                    ),
                )
                .child(
                    div().flex_1().child(
                        Input::new(input)
                            .id("note-filter-input")
                            .appearance(false)
                            .bordered(false),
                    ),
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
