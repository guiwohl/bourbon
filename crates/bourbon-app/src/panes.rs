use gpui_kit::component::ActiveTheme as _;
use gpui_kit::{
    AppContext as _, Context, Empty, InteractiveElement as _, IntoElement,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, div, px,
};

use crate::workspace::Workspace;

const DIVIDER_WIDTH: f32 = 4.;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Divider {
    AfterSidebar,
    AfterNoteList,
}

pub(crate) fn divider(
    id: &'static str,
    which: Divider,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    div()
        .id(id)
        .w(px(DIVIDER_WIDTH))
        .h_full()
        .flex_none()
        .cursor_col_resize()
        .hover(|handle| handle.bg(cx.theme().border))
        .on_drag(which, |_, _, _, cx| cx.new(|_| Empty))
        .test_support()
}
