use bourbon_ui::{Button, ButtonVariant, Dialog};
use gpui::{AnyElement, Context, FocusHandle, SharedString, Window};

use super::Workspace;
use super::note_list_intents::empty_trash_prompt;
use crate::overlay::{layer, tagged};
use crate::sidebar;

type Answer = fn(&mut Workspace, &mut Context<Workspace>);

pub(crate) struct DialogState {
    pub(super) focus: FocusHandle,
    pub(super) empty_trash: bool,
}

impl DialogState {
    pub(super) fn new(focus: FocusHandle) -> Self {
        Self {
            focus,
            empty_trash: false,
        }
    }
}

pub(super) struct Confirmation {
    pub(super) id: &'static str,
    pub(super) title: &'static str,
    pub(super) question: SharedString,
    pub(super) confirm_label: &'static str,
    pub(super) confirm: Answer,
    pub(super) cancel: Answer,
}

impl Workspace {
    pub(crate) fn confirm_empty_trash(&mut self, cx: &mut Context<Self>) {
        self.dialogs.empty_trash = true;
        cx.notify();
    }

    fn answer_empty_trash(&mut self, cx: &mut Context<Self>) {
        self.dialogs.empty_trash = false;
        self.empty_trash(cx);
    }

    fn keep_trash(&mut self, cx: &mut Context<Self>) {
        self.dialogs.empty_trash = false;
        cx.notify();
    }

    fn confirmation(&self) -> Option<Confirmation> {
        if let Some(question) = self.restore_question() {
            return Some(Confirmation {
                id: "restore",
                title: "Restore from backup",
                question: question.into(),
                confirm_label: "Restore",
                confirm: Workspace::confirm_restore,
                cancel: Workspace::cancel_restore,
            });
        }
        self.dialogs.empty_trash.then(|| Confirmation {
            id: "empty-trash",
            title: "Empty trash",
            question: empty_trash_prompt(self.notes.len()).into(),
            confirm_label: "Empty trash",
            confirm: Workspace::answer_empty_trash,
            cancel: Workspace::keep_trash,
        })
    }

    pub(super) fn dialog(&self, window: &mut Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let shown = self.shown_dialog(window, cx);
        if shown.is_none() {
            self.take_focus_back_from_dialog(window, cx);
        }
        shown
    }

    fn shown_dialog(&self, window: &mut Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        if let Some(prompt) = &self.sidebar.prompt {
            if !prompt.has_input() {
                self.focus_dialog(window, cx);
            }
            return Some(sidebar::prompt_dialog(prompt, &self.dialogs.focus, cx));
        }
        let confirmation = self.confirmation()?;
        self.focus_dialog(window, cx);
        Some(render_confirmation(confirmation, &self.dialogs.focus, cx))
    }

    fn take_focus_back_from_dialog(&self, window: &mut Window, cx: &mut Context<Self>) {
        if self.dialogs.focus.is_focused(window) || window.focused(cx).is_none() {
            window.focus(&self.focus_handle, cx);
        }
    }

    fn focus_dialog(&self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.dialogs.focus.is_focused(window) {
            window.focus(&self.dialogs.focus, cx);
        }
    }
}

fn render_confirmation(
    confirmation: Confirmation,
    focus: &FocusHandle,
    cx: &mut Context<Workspace>,
) -> AnyElement {
    let Confirmation {
        id,
        title,
        question,
        confirm_label,
        confirm,
        cancel,
    } = confirmation;
    let (submitting, dismissing) = (cx.entity(), cx.entity());
    let dialog = Dialog::new(SharedString::from(format!("{id}-dialog")), title)
        .track_focus(focus)
        .child(question)
        .action(tagged(
            format!("{id}-cancel"),
            Button::new(SharedString::from(format!("{id}-cancel-button")))
                .label("Cancel")
                .on_click(cx.listener(move |workspace, _, _, cx| cancel(workspace, cx))),
        ))
        .action(tagged(
            format!("{id}-confirm"),
            Button::new(SharedString::from(format!("{id}-confirm-button")))
                .label(confirm_label)
                .variant(ButtonVariant::Danger)
                .on_click(cx.listener(move |workspace, _, _, cx| confirm(workspace, cx))),
        ))
        .on_submit(move |_, cx| submitting.update(cx, confirm))
        .on_dismiss(move |_, cx| dismissing.update(cx, cancel));
    layer(format!("{id}-confirmation"), dialog)
}
