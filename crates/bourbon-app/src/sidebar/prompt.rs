use bourbon_ui::{Button, ButtonVariant, ThemeExt as _};
use gpui_kit::component::input::Input;
use gpui_kit::{
    Context, InteractiveElement as _, IntoElement, MouseButton, ParentElement as _, Styled as _,
    TestSupportExt as _, div, px,
};

use super::state::Prompt;
use crate::workspace::Workspace;

const PANEL_MARGIN: f32 = 6.;
const PANEL_PADDING: f32 = 8.;
const PANEL_GAP: f32 = 8.;

pub(super) fn render(prompt: &Prompt, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let tokens = cx.theme().tokens.clone();
    let panel = div()
        .id("sidebar-prompt")
        .test_support()
        .flex()
        .flex_col()
        .gap(px(PANEL_GAP))
        .m(px(PANEL_MARGIN))
        .p(px(PANEL_PADDING))
        .rounded(tokens.scale.radius_default)
        .border_1()
        .border_color(tokens.colors.internal_border)
        .bg(tokens.colors.input_background)
        .text_color(tokens.colors.text)
        .text_size(tokens.scale.medium)
        .font_family(tokens.fonts.ui_family.clone());
    match prompt {
        Prompt::Name { target, input, .. } => panel
            .child(target.title())
            .child(Input::new(input).id("sidebar-prompt-name"))
            .child(buttons("Save", ButtonVariant::Primary, cx)),
        Prompt::Deletion { question, .. } => {
            panel
                .child(question.clone())
                .child(buttons("Delete", ButtonVariant::Primary, cx))
        }
    }
}

fn buttons(
    confirm_label: &'static str,
    variant: ButtonVariant,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    div()
        .flex()
        .gap(px(PANEL_GAP))
        .child(
            div()
                .id("sidebar-prompt-confirm")
                .test_support()
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .child(
                    Button::new("sidebar-prompt-confirm-button")
                        .label(confirm_label)
                        .variant(variant)
                        .on_click(cx.listener(|workspace, _, _, cx| workspace.confirm_prompt(cx))),
                ),
        )
        .child(
            div()
                .id("sidebar-prompt-cancel")
                .test_support()
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .child(
                    Button::new("sidebar-prompt-cancel-button")
                        .label("Cancel")
                        .on_click(cx.listener(|workspace, _, _, cx| workspace.cancel_prompt(cx))),
                ),
        )
}
