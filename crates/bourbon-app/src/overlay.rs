use gpui::{
    AnyElement, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    Styled as _, div,
};

use crate::test_id::TestSupportExt as _;

pub(crate) fn tagged(id: impl Into<SharedString>, content: impl IntoElement) -> AnyElement {
    div()
        .id(ElementId::Name(id.into()))
        .test_support()
        .flex()
        .flex_col()
        .child(content)
        .into_any_element()
}

pub(crate) fn layer(id: impl Into<SharedString>, dialog: impl IntoElement) -> AnyElement {
    div()
        .id(ElementId::Name(id.into()))
        .test_support()
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .child(dialog)
        .into_any_element()
}
