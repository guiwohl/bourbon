use bourbon_core::{NoteId, NoteSort, NoteStore, StoreError, TagId, Timestamp};
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::ButtonVariant;
use gpui_kit::{Context, Window};

use crate::navigation::Navigation;
use crate::workspace::Workspace;

pub(crate) fn empty_trash_prompt(count: usize) -> String {
    let noun = if count == 1 { "note" } else { "notes" };
    format!("Permanently delete {count} {noun}?")
}

impl Workspace {
    pub(crate) fn sort_by(&mut self, sort: NoteSort, cx: &mut Context<Self>) {
        self.listing.sort = sort;
        self.reload_list(cx);
        cx.notify();
    }

    pub(crate) fn show_tag(&mut self, tag: &TagId, cx: &mut Context<Self>) {
        self.navigate(Navigation::Tags(vec![tag.clone()]), cx);
    }

    pub(crate) fn toggle_pin(&mut self, id: &NoteId, cx: &mut Context<Self>) {
        let pinned = self.notes.iter().any(|note| note.id == *id && note.pinned);
        self.change_notes(cx, |store| {
            if pinned {
                store.unpin_note(id)
            } else {
                store.pin_note(id)
            }
        });
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
        if let Err(error) = self.flush_pending_save(cx) {
            self.report(&error, cx);
            return;
        }
        let successor = self.neighbour_of(id);
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

    pub(crate) fn confirm_empty_trash(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let view = cx.entity();
        let prompt = empty_trash_prompt(self.notes.len());
        window.open_alert_dialog(cx, move |alert, _, _| {
            let view = view.clone();
            alert
                .title("Empty trash")
                .description(prompt.clone())
                .ok_text("Empty trash")
                .ok_variant(ButtonVariant::Danger)
                .show_cancel(true)
                .on_ok(move |_, _, cx| {
                    view.update(cx, Workspace::empty_trash);
                    true
                })
        });
    }

    pub(crate) fn empty_trash(&mut self, cx: &mut Context<Self>) {
        self.change_notes(cx, NoteStore::empty_trash);
        let selected_was_purged = self
            .selected
            .as_ref()
            .is_some_and(|id| matches!(self.store.find_note(id), Ok(None)));
        if selected_was_purged {
            self.selected = None;
        }
        cx.notify();
    }

    fn change_notes<T>(
        &mut self,
        cx: &mut Context<Self>,
        change: impl FnOnce(&NoteStore) -> Result<T, StoreError>,
    ) -> Option<T> {
        if let Err(error) = self.flush_pending_save(cx) {
            self.report(&error, cx);
            return None;
        }
        let outcome = change(&self.store);
        self.reload_list(cx);
        self.reload_sidebar(cx);
        cx.notify();
        match outcome {
            Ok(value) => Some(value),
            Err(error) => {
                self.report(&error, cx);
                None
            }
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
