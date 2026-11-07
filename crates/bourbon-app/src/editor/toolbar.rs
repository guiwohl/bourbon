use bourbon_core::ViewMode;
use gpui_kit::assets::IconName as EditorIconName;
use gpui_kit::base::Disableable as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{ActiveTheme as _, Icon, IconName, Selectable as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, div, px,
};

use super::header::{self, NoteHeader};
use super::icons::editor_icon;
use crate::workspace::Workspace;

const TOOLBAR_HEIGHT: f32 = 40.;
const EDGE_PADDING: f32 = 12.;
const LEFT_GAP: f32 = 14.;
const SEGMENT_WIDTH: f32 = 44.;
const SEGMENT_HEIGHT: f32 = 30.;
const SEGMENT_RADIUS: f32 = 6.;
const TRAILING_SPACE: f32 = 290.;

pub(super) struct ToolbarView<'a> {
    pub(super) mode: ViewMode,
    pub(super) header: Option<&'a NoteHeader>,
    pub(super) can_go_back: bool,
    pub(super) can_go_forward: bool,
}

fn button_id(mode: ViewMode) -> &'static str {
    match mode {
        ViewMode::Edit => "view-mode-edit",
        ViewMode::SideBySide => "view-mode-side-by-side",
        ViewMode::Preview => "view-mode-preview",
    }
}

fn mode_icon(mode: ViewMode) -> Icon {
    match mode {
        ViewMode::Edit => editor_icon(EditorIconName::Pencil),
        ViewMode::SideBySide => editor_icon(EditorIconName::Columns2),
        ViewMode::Preview => Icon::new(IconName::Eye),
    }
}

pub(super) fn render(view: &ToolbarView, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let muted = cx.theme().muted_foreground;
    let border = cx.theme().border;
    let actions = view.header.map(|header| header::actions_button(header, cx));
    div()
        .flex()
        .flex_none()
        .items_center()
        .h(px(TOOLBAR_HEIGHT))
        .px(px(EDGE_PADDING))
        .text_color(muted)
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(LEFT_GAP))
                .child(
                    Button::new("focus-toggle")
                        .ghost()
                        .small()
                        .icon(editor_icon(EditorIconName::Expand))
                        .accessibility_label("Focus mode")
                        .on_click(cx.listener(|workspace, _, window, cx| {
                            workspace.toggle_focus_mode(window, cx);
                        })),
                )
                .child(
                    Button::new("history-back")
                        .ghost()
                        .small()
                        .icon(IconName::ArrowLeft)
                        .accessibility_label("Back")
                        .disabled(!view.can_go_back)
                        .on_click(cx.listener(|workspace, _, window, cx| {
                            workspace.history_back(window, cx);
                        })),
                )
                .child(
                    Button::new("history-forward")
                        .ghost()
                        .small()
                        .icon(IconName::ArrowRight)
                        .accessibility_label("Forward")
                        .disabled(!view.can_go_forward)
                        .on_click(cx.listener(|workspace, _, window, cx| {
                            workspace.history_forward(window, cx);
                        })),
                ),
        )
        .child(div().flex_1())
        .child(
            div()
                .flex()
                .flex_none()
                .overflow_hidden()
                .border_1()
                .border_color(border)
                .rounded(px(SEGMENT_RADIUS))
                .children(ViewMode::ALL.map(|mode| {
                    let is_current = mode == view.mode;
                    div()
                        .id(format!("{}-state", button_id(mode)))
                        .aria_selected(is_current)
                        .test_support()
                        .when(mode != ViewMode::Preview, |segment| {
                            segment.border_r_1().border_color(border)
                        })
                        .child(
                            Button::new(button_id(mode))
                                .ghost()
                                .selected(is_current)
                                .w(px(SEGMENT_WIDTH))
                                .h(px(SEGMENT_HEIGHT))
                                .rounded_none()
                                .icon(mode_icon(mode))
                                .accessibility_label(mode.label())
                                .on_click(cx.listener(move |workspace, _, window, cx| {
                                    workspace.set_view_mode(mode, window, cx);
                                })),
                        )
                })),
        )
        .child(div().flex_shrink(1.).min_w_0().w(px(TRAILING_SPACE)))
        .children(actions)
}
