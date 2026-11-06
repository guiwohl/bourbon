use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Selectable as _, Sizable as _};
use gpui_kit::{Context, IntoElement, ParentElement as _, Styled as _, div};

use super::view_mode::ViewMode;
use crate::workspace::Workspace;

fn button_id(mode: ViewMode) -> &'static str {
    match mode {
        ViewMode::Edit => "view-mode-edit",
        ViewMode::SideBySide => "view-mode-side-by-side",
        ViewMode::Preview => "view-mode-preview",
    }
}

pub(super) fn render(current: ViewMode, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    div().flex().gap_1().children(ViewMode::ALL.map(|mode| {
        Button::new(button_id(mode))
            .ghost()
            .small()
            .selected(mode == current)
            .label(mode.label())
            .on_click(cx.listener(move |workspace, _, window, cx| {
                workspace.set_view_mode(mode, window, cx);
            }))
    }))
}
