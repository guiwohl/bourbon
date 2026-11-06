use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::Input;
use gpui_kit::component::label::Label;
use gpui_kit::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _,
    TestSupportExt as _, div,
};

use super::state::Prompt;
use crate::workspace::Workspace;

pub(super) fn render(prompt: &Prompt, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let panel = div()
        .id("sidebar-prompt")
        .test_support()
        .flex()
        .flex_col()
        .gap_2()
        .p_2()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().sidebar_border)
        .bg(cx.theme().sidebar_accent);
    match prompt {
        Prompt::Name { target, input, .. } => panel
            .child(target.title())
            .child(Input::new(input).id("sidebar-prompt-name"))
            .child(buttons(
                Button::new("sidebar-prompt-confirm").label("Save"),
                cx,
            )),
        Prompt::Deletion { question, .. } => {
            panel.child(Label::new(question.clone())).child(buttons(
                Button::new("sidebar-prompt-confirm")
                    .label("Delete")
                    .danger(),
                cx,
            ))
        }
    }
}

fn buttons(confirm: Button, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    div()
        .flex()
        .flex_row()
        .gap_2()
        .child(confirm.on_click(cx.listener(|workspace, _, _, cx| workspace.confirm_prompt(cx))))
        .child(
            Button::new("sidebar-prompt-cancel")
                .label("Cancel")
                .on_click(cx.listener(|workspace, _, _, cx| workspace.cancel_prompt(cx))),
        )
}
