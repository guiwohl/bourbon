use std::collections::HashSet;

use bourbon_core::{NoteId, NoteSummary};
use bourbon_editor::TextField;
use bourbon_ui::ThemeExt as _;
use gpui_kit::{
    Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _,
    TestSupportExt as _, div, px,
};

use crate::workspace::Workspace;

const SIDE_MARGIN: f32 = 12.;
const VERTICAL_MARGIN: f32 = 2.;
const GAP_BELOW: f32 = 4.;

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
    field: &Entity<TextField>,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let tokens = cx.theme().tokens.clone();
    div()
        .h(tokens.scale.search_bar_height)
        .mb(px(GAP_BELOW))
        .flex_none()
        .flex()
        .items_center()
        .child(
            div()
                .id("note-filter-input")
                .test_support()
                .flex_1()
                .min_w_0()
                .mx(px(SIDE_MARGIN))
                .my(px(VERTICAL_MARGIN))
                .child(field.clone()),
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
