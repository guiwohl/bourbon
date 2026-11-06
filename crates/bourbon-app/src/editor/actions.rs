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
        CycleViewMode,
        ToggleTaskAtCursor,
        FindInNote,
        ReplaceInNote,
        ToggleFocusMode,
        ExitFocusMode,
        LinkPickNext,
        LinkPickPrevious,
        LinkPickConfirm,
        LinkPickCancel,
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

pub(crate) fn register_writing(root: Div, cx: &mut Context<Workspace>) -> Div {
    root.on_action(cx.listener(|workspace, _: &CycleViewMode, window, cx| {
        workspace.cycle_view_mode(window, cx);
    }))
    .on_action(
        cx.listener(|workspace, _: &ToggleTaskAtCursor, window, cx| {
            workspace.toggle_task_at_cursor(window, cx);
        }),
    )
    .on_action(cx.listener(|workspace, _: &FindInNote, window, cx| {
        workspace.find_in_note(false, window, cx);
    }))
    .on_action(cx.listener(|workspace, _: &ReplaceInNote, window, cx| {
        workspace.find_in_note(true, window, cx);
    }))
    .on_action(cx.listener(|workspace, _: &ToggleFocusMode, window, cx| {
        workspace.toggle_focus_mode(window, cx);
    }))
    .on_action(cx.listener(|workspace, _: &ExitFocusMode, window, cx| {
        if !workspace.exit_focus_mode(window, cx) {
            cx.propagate();
        }
    }))
    .on_action(cx.listener(|workspace, _: &LinkPickNext, _, cx| {
        workspace.highlight_next_link(cx);
    }))
    .on_action(cx.listener(|workspace, _: &LinkPickPrevious, _, cx| {
        workspace.highlight_previous_link(cx);
    }))
    .on_action(cx.listener(|workspace, _: &LinkPickConfirm, window, cx| {
        workspace.confirm_link(window, cx);
    }))
    .on_action(cx.listener(|workspace, _: &LinkPickCancel, _, cx| {
        workspace.cancel_link_picker(cx);
    }))
}
