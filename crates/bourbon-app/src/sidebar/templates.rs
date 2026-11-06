use gpui_kit::component::{ActiveTheme as _, Icon, IconName};
use gpui_kit::{
    Context, IntoElement, ParentElement as _, StatefulInteractiveElement as _, Styled as _,
    TestSupportExt as _,
};

use super::nav_row;
use crate::workspace::Workspace;

pub(super) fn new_row(cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    nav_row("new-template".into(), 0, false, cx)
        .test_support()
        .text_color(cx.theme().muted_foreground)
        .child(Icon::new(IconName::Plus))
        .child("New template")
        .on_click(cx.listener(|workspace, _, window, cx| {
            workspace.create_template(window, cx);
        }))
}
