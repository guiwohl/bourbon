use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Selectable as _, Sizable as _};
use gpui_kit::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, div,
};

use crate::workspace::Workspace;
use bourbon_core::ViewMode;

fn button_id(mode: ViewMode) -> &'static str {
    match mode {
        ViewMode::Edit => "view-mode-edit",
        ViewMode::SideBySide => "view-mode-side-by-side",
        ViewMode::Preview => "view-mode-preview",
    }
}

pub(super) fn render(current: ViewMode, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    div().flex().gap_1().children(ViewMode::ALL.map(|mode| {
        let is_current = mode == current;
        div()
            .id(format!("{}-state", button_id(mode)))
            .aria_selected(is_current)
            .test_support()
            .child(
                Button::new(button_id(mode))
                    .ghost()
                    .small()
                    .selected(is_current)
                    .label(mode.label())
                    .on_click(cx.listener(move |workspace, _, window, cx| {
                        workspace.set_view_mode(mode, window, cx);
                    })),
            )
    }))
}
