use bourbon_ui::{IconButton, IconButtonSize, IconName, SectionHeader};
use gpui::{Context, InteractiveElement as _, IntoElement, ParentElement as _, div};

use super::state::{Section, SidebarState};
use crate::test_id::TestSupportExt as _;
use crate::workspace::Workspace;

pub(super) struct Spec {
    pub(super) label: &'static str,
    pub(super) section: Section,
    pub(super) searchable: bool,
    pub(super) addable: bool,
    pub(super) collapsible: bool,
}

pub(super) fn render(
    spec: &Spec,
    state: &SidebarState,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let section = spec.section;
    let slug = spec.label.to_lowercase();
    let mut header = SectionHeader::new(format!("{slug}-header-row"), spec.label);
    if spec.searchable {
        header = header.accessory(
            IconButton::new(format!("{slug}-search"), IconName::Search)
                .size(IconButtonSize::Accessory)
                .on_click(cx.listener(|workspace, _, window, cx| {
                    workspace.open_note_search(window, cx);
                })),
        );
    }
    if spec.addable {
        header = header.accessory(
            div().id("new-notebook").test_support().child(
                IconButton::new("new-notebook-button", IconName::Plus)
                    .size(IconButtonSize::Accessory)
                    .on_click(cx.listener(|workspace, _, window, cx| {
                        workspace.begin_create_notebook(None, window, cx);
                    })),
            ),
        );
    }
    if spec.collapsible {
        let view = cx.entity();
        header = header.collapsible(state.is_collapsed(section), move |_, _, cx| {
            view.update(cx, |workspace, cx| workspace.toggle_section(section, cx));
        });
    }
    div()
        .id(format!("{slug}-header"))
        .test_support()
        .child(header)
}
