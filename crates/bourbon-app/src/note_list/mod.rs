mod date;
mod filter;
mod header;
mod row;
mod title;

use std::rc::Rc;

use bourbon_core::{NoteId, NoteSort, NoteSummary, NotebookNode, TagSummary, Timestamp};
use bourbon_ui::ThemeExt as _;
use bourbon_ui::list_row::accent;
use gpui_kit::{
    AnyElement, Context, Entity, FocusHandle, InteractiveElement as _, IntoElement, ListAlignment,
    ListOffset, ListState, ParentElement as _, Styled as _, TestSupportExt as _, div, list, px,
};
use jiff::tz::TimeZone;

use crate::navigation::Navigation;
use crate::workspace::Workspace;

pub(crate) use filter::narrow;
use header::ListView;

gpui_kit::actions!(bourbon, [SelectPreviousNote, SelectNextNote]);

const OVERDRAW: f32 = 400.;
const EMPTY_OPACITY: f32 = 0.5;
const EMPTY_MESSAGE_SIZE: f32 = 20.;
const EMPTY_BOTTOM_PADDING: f32 = 80.;

pub(crate) struct NoteListData<'a> {
    pub(crate) navigation: &'a Navigation,
    pub(crate) sort: NoteSort,
    pub(crate) notebooks: &'a [NotebookNode],
    pub(crate) tags: &'a [TagSummary],
    pub(crate) notes: &'a Rc<[NoteSummary]>,
    pub(crate) templates: &'a [NoteSummary],
    pub(crate) selected: Option<&'a NoteId>,
    pub(crate) filter: &'a Entity<bourbon_editor::TextField>,
    pub(crate) width: f32,
    pub(crate) is_focused: bool,
    pub(crate) focus: &'a FocusHandle,
    pub(crate) state: &'a ListState,
}

pub(crate) fn new_list_state() -> ListState {
    ListState::new(0, ListAlignment::Top, px(OVERDRAW))
}

pub(crate) fn reveal(state: &ListState, index: usize) {
    let viewport = state.viewport_bounds();
    let is_visible = state.bounds_for_item(index).is_some_and(|bounds| {
        bounds.top() >= viewport.top() && bounds.bottom() <= viewport.bottom()
    });
    if !is_visible {
        state.scroll_to(ListOffset {
            item_ix: index,
            offset_in_item: px(0.),
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    Previous,
    Next,
}

fn is_selected(selected: Option<&NoteId>, id: &NoteId) -> bool {
    selected == Some(id)
}

pub(crate) fn adjacent_note(
    notes: &[NoteSummary],
    selected: Option<&NoteId>,
    step: Step,
) -> Option<NoteId> {
    let current = selected.and_then(|id| notes.iter().position(|note| note.id == *id));
    let target = match (current, step) {
        (None, _) => 0,
        (Some(position), Step::Next) => position + 1,
        (Some(position), Step::Previous) => position.checked_sub(1)?,
    };
    notes.get(target).map(|note| note.id.clone())
}

pub(crate) fn render(data: &NoteListData, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let tokens = cx.theme().tokens.clone();
    let in_trash = *data.navigation == Navigation::Trash;
    let view_kind = ListView::of(data.navigation);
    let header = header::render(
        header::HeaderData {
            title: title::view_title(data.navigation, data.notebooks, data.tags).into(),
            sort: data.sort,
            view: view_kind,
            note_count: data.notes.len(),
            templates: data
                .templates
                .iter()
                .map(|template| (template.id.clone(), template.title.clone()))
                .collect(),
        },
        cx,
    );
    let body: AnyElement = if data.notes.is_empty() {
        empty_state(cx).into_any_element()
    } else {
        rows(data, in_trash, view_kind.shows_dates(), cx)
    };
    div()
        .w(px(data.width))
        .h_full()
        .flex_none()
        .flex()
        .flex_col()
        .bg(tokens.colors.note_list_background)
        .child(header)
        .child(filter::render(data.filter, cx))
        .child(
            div()
                .id("note-list")
                .key_context("NoteList")
                .track_focus(data.focus)
                .on_action(
                    cx.listener(|workspace, _: &SelectPreviousNote, window, cx| {
                        workspace.select_adjacent_note(Step::Previous, window, cx);
                    }),
                )
                .on_action(cx.listener(|workspace, _: &SelectNextNote, window, cx| {
                    workspace.select_adjacent_note(Step::Next, window, cx);
                }))
                .flex_1()
                .min_h_0()
                .child(body),
        )
}

fn rows(
    data: &NoteListData,
    in_trash: bool,
    shows_date: bool,
    cx: &mut Context<Workspace>,
) -> AnyElement {
    let notes = Rc::clone(data.notes);
    let selected = data.selected.cloned();
    let (is_focused, sort) = (data.is_focused, data.sort);
    let view = cx.entity();
    let zone = TimeZone::system();
    let now = Timestamp::now();
    list(data.state.clone(), move |index, _, cx| {
        let summary = &notes[index];
        row::render(
            &row::RowData {
                summary,
                shown: accent(is_selected(selected.as_ref(), &summary.id), is_focused),
                in_trash,
                shows_date,
                sort,
                now,
                zone: &zone,
            },
            &view,
            cx,
        )
    })
    .size_full()
    .into_any_element()
}

fn empty_state(cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let tokens = cx.theme().tokens.clone();
    div()
        .id("no-notes")
        .test_support()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .pb(px(EMPTY_BOTTOM_PADDING))
        .opacity(EMPTY_OPACITY)
        .text_size(px(EMPTY_MESSAGE_SIZE))
        .font_weight(gpui_kit::FontWeight::LIGHT)
        .font_family(tokens.fonts.ui_family.clone())
        .text_color(tokens.colors.text)
        .child("No notes")
}

#[cfg(test)]
mod tests {
    use bourbon_core::NoteStatus;

    use super::*;

    fn notes(count: usize) -> Vec<NoteSummary> {
        (0..count)
            .map(|_| NoteSummary {
                id: NoteId::generate(),
                title: String::new(),
                excerpt: String::new(),
                updated_at: Timestamp::from_millis(0),
                created_at: Timestamp::from_millis(0),
                pinned: false,
                status: NoteStatus::None,
                tags: Vec::new(),
                task_progress: None,
            })
            .collect()
    }

    #[test]
    fn only_the_selected_note_is_highlighted() {
        let selected = NoteId::generate();
        assert!(is_selected(Some(&selected), &selected));
        assert!(!is_selected(Some(&selected), &NoteId::generate()));
        assert!(!is_selected(None, &selected));
    }

    #[test]
    fn down_moves_to_the_next_note_and_up_to_the_previous_one() {
        let notes = notes(3);
        assert_eq!(
            adjacent_note(&notes, Some(&notes[1].id), Step::Next),
            Some(notes[2].id.clone())
        );
        assert_eq!(
            adjacent_note(&notes, Some(&notes[1].id), Step::Previous),
            Some(notes[0].id.clone())
        );
    }

    #[test]
    fn the_ends_of_the_list_do_not_wrap() {
        let notes = notes(2);
        assert_eq!(adjacent_note(&notes, Some(&notes[1].id), Step::Next), None);
        assert_eq!(
            adjacent_note(&notes, Some(&notes[0].id), Step::Previous),
            None
        );
    }

    #[test]
    fn without_a_selection_either_arrow_opens_the_first_note() {
        let notes = notes(2);
        assert_eq!(
            adjacent_note(&notes, None, Step::Next),
            Some(notes[0].id.clone())
        );
        assert_eq!(
            adjacent_note(&notes, None, Step::Previous),
            Some(notes[0].id.clone())
        );
        assert_eq!(adjacent_note(&[], None, Step::Next), None);
    }

    #[test]
    fn a_selection_outside_the_list_is_treated_as_no_selection() {
        let notes = notes(2);
        assert_eq!(
            adjacent_note(&notes, Some(&NoteId::generate()), Step::Next),
            Some(notes[0].id.clone())
        );
    }
}
