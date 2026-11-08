use std::collections::HashSet;

use bourbon_core::{NoteId, NoteSort, NoteStore, StoreError, Timestamp};
use gpui::{Context, Window};

use crate::note_list::{Step, adjacent_note};
use crate::workspace::Workspace;

pub(crate) fn empty_trash_prompt(count: usize) -> String {
    let noun = if count == 1 { "note" } else { "notes" };
    format!("Permanently delete {count} {noun}?")
}

const FILTER_RESULT_LIMIT: usize = 100_000;

impl Workspace {
    pub(crate) fn filter_changed(&mut self, cx: &mut Context<Self>) {
        self.reload_list(cx);
        cx.notify();
    }

    pub(crate) fn clear_filter(&mut self, cx: &mut Context<Self>) {
        let input = self.filter_input.clone();
        input.update(cx, |input, cx| input.set_text("", cx));
        self.filter_changed(cx);
    }

    pub(crate) fn focus_note_list(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.note_list_focus, cx);
    }

    pub(super) fn filter_matches(&mut self, cx: &mut Context<Self>) -> Option<HashSet<NoteId>> {
        let text = self.filter_input.read(cx).text();
        if text.trim().is_empty() {
            return None;
        }
        match self.store.search_notes(&text, FILTER_RESULT_LIMIT) {
            Ok(hits) => Some(hits.into_iter().map(|hit| hit.summary.id).collect()),
            Err(error) => {
                self.report(&error, cx);
                None
            }
        }
    }

    pub(crate) fn select_adjacent_note(
        &mut self,
        step: Step,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(target) = adjacent_note(&self.notes, self.selected.as_ref(), step) {
            self.select(&target, window, cx);
        }
    }

    pub(crate) fn sort_by(&mut self, sort: NoteSort, cx: &mut Context<Self>) {
        self.listing.sort = sort;
        self.reload_list(cx);
        cx.notify();
    }

    pub(crate) fn toggle_pin(&mut self, id: &NoteId, cx: &mut Context<Self>) {
        self.change_notes(cx, |store| store.toggle_pin(id));
    }

    pub(crate) fn duplicate(&mut self, id: &NoteId, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(copy) =
            self.change_notes(cx, |store| store.duplicate_note(id, Timestamp::now()))
        {
            self.show(copy, window, cx);
        }
    }

    pub(crate) fn move_to_trash(
        &mut self,
        id: &NoteId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(successor) = self.flush_then(cx, |this, _| this.neighbour_of(id)) else {
            return;
        };
        if self
            .change_notes(cx, |store| store.trash_note(id, Timestamp::now()))
            .is_none()
        {
            return;
        }
        if self.selected.as_ref() != Some(id) {
            return;
        }
        self.selected = None;
        if let Some(next) = successor {
            self.select(&next, window, cx);
        }
        cx.notify();
    }

    pub(crate) fn restore(&mut self, id: &NoteId, cx: &mut Context<Self>) {
        self.change_notes(cx, |store| store.restore_note(id));
    }

    pub(crate) fn empty_trash(&mut self, cx: &mut Context<Self>) {
        self.change_notes(cx, NoteStore::empty_trash);
        self.clear_unavailable_selection();
        cx.notify();
    }

    fn change_notes<T>(
        &mut self,
        cx: &mut Context<Self>,
        change: impl FnOnce(&NoteStore) -> Result<T, StoreError>,
    ) -> Option<T> {
        self.flush_then(cx, |this, cx| {
            let outcome = change(&this.store);
            this.refresh_listing(cx);
            match outcome {
                Ok(value) => Some(value),
                Err(error) => {
                    this.report(&error, cx);
                    None
                }
            }
        })
        .flatten()
    }

    pub(super) fn clear_unavailable_selection(&mut self) {
        let is_unavailable = self.selected.as_ref().is_some_and(|id| {
            self.store
                .find_note(id)
                .is_ok_and(|found| found.is_none_or(|note| note.trashed_at.is_some()))
        });
        if is_unavailable {
            self.selected = None;
        }
    }

    fn neighbour_of(&self, id: &NoteId) -> Option<NoteId> {
        let position = self.notes.iter().position(|note| note.id == *id)?;
        self.notes
            .get(position + 1)
            .or_else(|| {
                position
                    .checked_sub(1)
                    .and_then(|before| self.notes.get(before))
            })
            .map(|note| note.id.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_prompt_counts_the_notes() {
        assert_eq!(empty_trash_prompt(3), "Permanently delete 3 notes?");
    }

    #[test]
    fn the_prompt_is_singular_for_one_note() {
        assert_eq!(empty_trash_prompt(1), "Permanently delete 1 note?");
    }
}
