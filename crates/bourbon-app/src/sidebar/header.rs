use gpui_kit::assets::IconName;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::Sizable as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _,
    TestSupportExt as _, div, px,
};

use super::state::{Section, SidebarState};
use crate::icons::glyph;
use crate::workspace::Workspace;

const HEADER_HEIGHT: f32 = 28.;

pub(super) struct Spec {
    pub(super) label: &'static str,
    pub(super) section: Section,
    pub(super) searchable: bool,
    pub(super) addable: bool,
}

pub(super) fn render(
    spec: &Spec,
    state: &SidebarState,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let section = spec.section;
    let slug = spec.label.to_lowercase();
    let search = spec.searchable.then(|| {
        Button::new(format!("{slug}-search"))
            .ghost()
            .xsmall()
            .icon(glyph(IconName::Search))
            .on_click(cx.listener(|workspace, _, window, cx| {
                workspace.open_note_search(window, cx);
            }))
    });
    let add = spec.addable.then(|| {
        Button::new("new-notebook")
            .ghost()
            .xsmall()
            .icon(glyph(IconName::Plus))
            .on_click(cx.listener(|workspace, _, window, cx| {
                workspace.begin_create_notebook(None, window, cx);
            }))
    });
    let chevron = (!spec.addable).then(|| {
        let icon = if state.is_collapsed(section) {
            IconName::ChevronDown
        } else {
            IconName::ChevronUp
        };
        Button::new(format!("{slug}-collapse"))
            .ghost()
            .xsmall()
            .icon(glyph(icon))
            .on_click(cx.listener(move |workspace, _, _, cx| workspace.toggle_section(section, cx)))
    });
    div()
        .id(format!("{slug}-header"))
        .test_support()
        .h(px(HEADER_HEIGHT))
        .flex_none()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .px_2()
        .pt_2()
        .text_size(px(12.))
        .text_color(cx.theme().muted_foreground)
        .child(spec.label)
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .children(search)
                .children(add)
                .children(chevron),
        )
}
