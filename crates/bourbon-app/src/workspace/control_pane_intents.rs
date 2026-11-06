use bourbon_core::{NoteId, NoteSort};
use gpui_kit::{Context, Window};

use super::Workspace;
use crate::cli_command::copy_cli_command;
use crate::control_pane::{self, ACTIONS_QUERY, ControlPane, PaneAction, Results};
use crate::navigation::Navigation;

enum ActivatedRow {
    Note(NoteId),
    Action(PaneAction),
}

impl Workspace {
    pub(crate) fn open_note_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open_control_pane("", window, cx);
    }

    pub(crate) fn open_action_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open_control_pane(ACTIONS_QUERY, window, cx);
    }

    fn open_control_pane(&mut self, query: &str, window: &mut Window, cx: &mut Context<Self>) {
        match ControlPane::open(query, &self.store, self.selected.is_some(), window, cx) {
            Ok(pane) => self.control_pane = Some(pane),
            Err(error) => self.report(&error, cx),
        }
        cx.notify();
    }

    pub(crate) fn close_control_pane(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.control_pane.take().is_some() {
            window.focus(&self.focus_handle, cx);
            cx.notify();
        }
    }

    pub(crate) fn control_pane_query_changed(&mut self, cx: &mut Context<Self>) {
        let Some(pane) = self.control_pane.as_mut() else {
            return;
        };
        let query = pane.query(cx);
        match control_pane::search(&query, &self.store, self.selected.is_some()) {
            Ok(results) => pane.show(results),
            Err(error) => self.report(&error, cx),
        }
        cx.notify();
    }

    pub(crate) fn highlight_next_pane_row(&mut self, cx: &mut Context<Self>) {
        if let Some(pane) = self.control_pane.as_mut() {
            pane.highlight_next();
            cx.notify();
        }
    }

    pub(crate) fn highlight_previous_pane_row(&mut self, cx: &mut Context<Self>) {
        if let Some(pane) = self.control_pane.as_mut() {
            pane.highlight_previous();
            cx.notify();
        }
    }

    pub(crate) fn activate_highlighted_row(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(index) = self.control_pane.as_ref().map(ControlPane::highlighted_row) {
            self.activate_pane_row(index, window, cx);
        }
    }

    pub(crate) fn activate_pane_row(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(row) = self
            .control_pane
            .as_ref()
            .and_then(|pane| activated(pane, index))
        else {
            return;
        };
        self.close_control_pane(window, cx);
        match row {
            ActivatedRow::Note(id) => self.open_found_note(&id, window, cx),
            ActivatedRow::Action(action) => self.run_pane_action(action, window, cx),
        }
    }

    fn open_found_note(&mut self, id: &NoteId, window: &mut Window, cx: &mut Context<Self>) {
        self.flush_then(cx, |this, cx| {
            if !this.notes.iter().any(|note| note.id == *id) {
                this.navigate(Navigation::AllNotes, cx);
            }
            this.select(id, window, cx);
        });
    }

    pub(crate) fn run_pane_action(
        &mut self,
        action: PaneAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let selected = self.selected.clone();
        match (action, selected) {
            (PaneAction::NewNote, _) => self.create_note(window, cx),
            (PaneAction::NewTemplate, _) => self.create_template(window, cx),
            (PaneAction::ToggleSidebar, _) => self.toggle_sidebar(cx),
            (PaneAction::OpenSettings, _) => self.open_settings(cx),
            (PaneAction::BackUpNow, _) => self.back_up_now(cx),
            (PaneAction::RestoreFromBackup, _) => self.begin_restore(cx),
            (PaneAction::ExportToMarkdown, _) => self.export_to_markdown(cx),
            (PaneAction::GoToAllNotes, _) => self.navigate(Navigation::AllNotes, cx),
            (PaneAction::GoToTemplates, _) => self.navigate(Navigation::Templates, cx),
            (PaneAction::GoToTrash, _) => self.navigate(Navigation::Trash, cx),
            (PaneAction::SortByUpdated, _) => self.sort_by(NoteSort::UpdatedNewestFirst, cx),
            (PaneAction::SortByCreated, _) => self.sort_by(NoteSort::CreatedNewestFirst, cx),
            (PaneAction::SortByTitle, _) => self.sort_by(NoteSort::TitleAlphabetical, cx),
            (PaneAction::EmptyTrash, _) => {
                self.navigate(Navigation::Trash, cx);
                self.confirm_empty_trash(window, cx);
            }
            (PaneAction::HistoryBack, _) => self.history_back(window, cx),
            (PaneAction::HistoryForward, _) => self.history_forward(window, cx),
            (PaneAction::CycleViewMode, _) => self.cycle_view_mode(window, cx),
            (PaneAction::Find, _) => self.find_in_note(false, window, cx),
            (PaneAction::Replace, _) => self.find_in_note(true, window, cx),
            (PaneAction::ToggleFocusMode, _) => self.toggle_focus_mode(window, cx),
            (PaneAction::SetStatus(status), Some(_)) => self.set_selected_status(status, cx),
            (PaneAction::MoveToNotebook, Some(_)) => self.open_notebook_selector(cx),
            (PaneAction::DuplicateNote, Some(id)) => self.duplicate(&id, window, cx),
            (PaneAction::TogglePin, Some(id)) => self.toggle_pin(&id, cx),
            (PaneAction::CopyCliCommand, Some(id)) => copy_cli_command(&id, cx),
            (PaneAction::MoveToTrash, Some(id)) => self.move_to_trash(&id, window, cx),
            (_, None) => {}
        }
    }
}

fn activated(pane: &ControlPane, index: usize) -> Option<ActivatedRow> {
    match pane.results() {
        Results::Notes(hits) => hits
            .get(index)
            .map(|hit| ActivatedRow::Note(hit.summary.id.clone())),
        Results::Actions(actions) => actions.get(index).copied().map(ActivatedRow::Action),
    }
}
