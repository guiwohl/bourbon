use bourbon_core::NoteStatus;
use gpui_kit::{Context, Div, InteractiveElement as _};

use crate::workspace::Workspace;

gpui_kit::actions!(
    bourbon,
    [
        SetStatusNone,
        SetStatusActive,
        SetStatusOnHold,
        SetStatusCompleted,
        SetStatusDropped,
        OpenNotebookSelector,
        HistoryBack,
        HistoryForward,
    ]
);

pub(crate) fn register(root: Div, cx: &mut Context<Workspace>) -> Div {
    root.on_action(cx.listener(|workspace, _: &SetStatusNone, _, cx| {
        workspace.set_selected_status(NoteStatus::None, cx);
    }))
    .on_action(cx.listener(|workspace, _: &SetStatusActive, _, cx| {
        workspace.set_selected_status(NoteStatus::Active, cx);
    }))
    .on_action(cx.listener(|workspace, _: &SetStatusOnHold, _, cx| {
        workspace.set_selected_status(NoteStatus::OnHold, cx);
    }))
    .on_action(cx.listener(|workspace, _: &SetStatusCompleted, _, cx| {
        workspace.set_selected_status(NoteStatus::Completed, cx);
    }))
    .on_action(cx.listener(|workspace, _: &SetStatusDropped, _, cx| {
        workspace.set_selected_status(NoteStatus::Dropped, cx);
    }))
    .on_action(cx.listener(|workspace, _: &OpenNotebookSelector, _, cx| {
        workspace.open_notebook_selector(cx);
    }))
    .on_action(cx.listener(|workspace, _: &HistoryBack, window, cx| {
        workspace.history_back(window, cx);
    }))
    .on_action(cx.listener(|workspace, _: &HistoryForward, window, cx| {
        workspace.history_forward(window, cx);
    }))
}
