use bourbon_core::ViewMode;
use bourbon_editor::{Editor, PreviewView, SearchPanel};
use bourbon_ui::ThemeExt as _;
use gpui::prelude::FluentBuilder as _;
use gpui::{
    Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _, div,
};

use super::link_picker::{self, LinkPicker};
use crate::test_id::TestSupportExt as _;
use crate::workspace::Workspace;

pub(super) struct BodyPanes<'a> {
    pub(super) mode: ViewMode,
    pub(super) editor: &'a Entity<Editor>,
    pub(super) search: &'a Entity<SearchPanel>,
    pub(super) preview: &'a Entity<PreviewView>,
    pub(super) link_picker: Option<&'a LinkPicker>,
}

pub(super) fn render(panes: &BodyPanes, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let tokens = cx.theme().tokens.clone();
    let picker = panes
        .link_picker
        .map(|picker| link_picker::render(picker, cx));
    div()
        .flex_1()
        .min_h_0()
        .flex()
        .when(panes.mode.shows_editor(), |row| {
            row.child(
                div()
                    .id("body-editor")
                    .test_support()
                    .when(picker.is_some(), |wrapper| {
                        wrapper.key_context("LinkPicker")
                    })
                    .relative()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .flex()
                    .flex_col()
                    .child(panes.search.clone())
                    .child(div().flex_1().min_h_0().child(panes.editor.clone()))
                    .children(picker),
            )
        })
        .when(panes.mode == ViewMode::SideBySide, |row| {
            row.child(
                div()
                    .w(tokens.scale.border)
                    .h_full()
                    .flex_none()
                    .bg(tokens.colors.internal_border),
            )
        })
        .when(panes.mode.shows_preview(), |row| {
            row.child(
                div()
                    .id("preview")
                    .test_support()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(panes.preview.clone()),
            )
        })
}
