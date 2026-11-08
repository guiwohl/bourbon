use bourbon_ui::{Button, ButtonVariant, Dialog};
use gpui::{AnyElement, Context, FocusHandle};

use super::state::{DeletionTarget, Prompt};
use crate::overlay::{layer, tagged};
use crate::workspace::Workspace;

pub(super) fn deletion_title(target: &DeletionTarget) -> &'static str {
    match target {
        DeletionTarget::Notebook(_) => "Delete notebook",
        DeletionTarget::Tag(_) => "Delete tag",
    }
}

pub(crate) fn prompt_dialog(
    prompt: &Prompt,
    focus: &FocusHandle,
    cx: &mut Context<Workspace>,
) -> AnyElement {
    let (submitting, dismissing) = (cx.entity(), cx.entity());
    let dialog = match prompt {
        Prompt::Name { target, input, .. } => Dialog::new("sidebar-prompt-dialog", target.title())
            .child(tagged("sidebar-prompt-name", input.clone()))
            .action(cancel_button(cx))
            .action(confirm_button("Save", ButtonVariant::Primary, cx)),
        Prompt::Deletion { target, question } => {
            Dialog::new("sidebar-prompt-dialog", deletion_title(target))
                .track_focus(focus)
                .child(question.clone())
                .action(cancel_button(cx))
                .action(confirm_button("Delete", ButtonVariant::Danger, cx))
        }
    }
    .on_submit(move |_, cx| submitting.update(cx, Workspace::confirm_prompt))
    .on_dismiss(move |_, cx| dismissing.update(cx, Workspace::cancel_prompt));
    layer("sidebar-prompt", dialog)
}

fn confirm_button(
    label: &'static str,
    variant: ButtonVariant,
    cx: &mut Context<Workspace>,
) -> AnyElement {
    tagged(
        "sidebar-prompt-confirm",
        Button::new("sidebar-prompt-confirm-button")
            .label(label)
            .variant(variant)
            .on_click(cx.listener(|workspace, _, _, cx| workspace.confirm_prompt(cx))),
    )
}

fn cancel_button(cx: &mut Context<Workspace>) -> AnyElement {
    tagged(
        "sidebar-prompt-cancel",
        Button::new("sidebar-prompt-cancel-button")
            .label("Cancel")
            .on_click(cx.listener(|workspace, _, _, cx| workspace.cancel_prompt(cx))),
    )
}

#[cfg(test)]
mod tests {
    use bourbon_core::{NotebookId, TagId};

    use super::*;

    #[test]
    fn the_deletion_title_names_what_is_deleted() {
        assert_eq!(
            deletion_title(&DeletionTarget::Notebook(NotebookId::generate())),
            "Delete notebook"
        );
        assert_eq!(
            deletion_title(&DeletionTarget::Tag(TagId::generate())),
            "Delete tag"
        );
    }
}
