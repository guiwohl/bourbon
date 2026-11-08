use bourbon_core::ViewMode;
use bourbon_ui::{IconButton, IconName, Segment, SegmentedControl, ThemeExt as _};
use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _, div, px,
};

use super::actions::{HistoryBack, HistoryForward, ToggleFocusMode};
use super::menu_anchor::MenuAnchor;
use crate::test_id::TestSupportExt as _;
use crate::workspace::Workspace;

const LEFT_PADDING: f32 = 4.;

pub(crate) struct ToolbarData {
    pub(crate) mode: ViewMode,
    pub(crate) focus_mode: bool,
    pub(crate) can_go_back: bool,
    pub(crate) can_go_forward: bool,
}

pub(crate) fn mode_index(mode: ViewMode) -> usize {
    ViewMode::ALL
        .iter()
        .position(|candidate| *candidate == mode)
        .unwrap_or(0)
}

fn mode_at(index: usize) -> ViewMode {
    ViewMode::ALL.get(index).copied().unwrap_or(ViewMode::Edit)
}

fn focus_mode_icon(focus_mode: bool) -> IconName {
    if focus_mode {
        IconName::Minimize2
    } else {
        IconName::Maximize2
    }
}

fn segments() -> Vec<Segment> {
    vec![
        Segment::new(IconName::Pencil, "Show Editor"),
        Segment::new(IconName::Columns2, "View Side by Side"),
        Segment::new(IconName::Eye, "Show Preview"),
    ]
}

pub(crate) fn render(
    data: &ToolbarData,
    more_anchor: &MenuAnchor,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let tokens = cx.theme().tokens.clone();
    let focus = div().id("toggle-focus-mode").test_support().child(
        IconButton::new("toggle-focus-mode-button", focus_mode_icon(data.focus_mode))
            .tooltip_action("Toggle distraction free", &ToggleFocusMode)
            .on_click(cx.listener(|workspace, _, window, cx| {
                workspace.toggle_focus_mode(window, cx);
            })),
    );
    let back = div().id("history-back").test_support().child(
        IconButton::new("history-back-button", IconName::ArrowLeft)
            .disabled(!data.can_go_back)
            .tooltip_action("Go back", &HistoryBack)
            .on_click(cx.listener(|workspace, _, _, cx| {
                workspace.history_back(cx);
            })),
    );
    let forward = div().id("history-forward").test_support().child(
        IconButton::new("history-forward-button", IconName::ArrowRight)
            .disabled(!data.can_go_forward)
            .tooltip_action("Go forward", &HistoryForward)
            .on_click(cx.listener(|workspace, _, _, cx| {
                workspace.history_forward(cx);
            })),
    );
    let owner = cx.entity();
    let modes = div().id("view-mode").test_support().child(
        SegmentedControl::new("view-mode-switch", segments())
            .selected(mode_index(data.mode))
            .on_change(move |index, window, cx| {
                owner.update(cx, |workspace, cx| {
                    workspace.set_view_mode(mode_at(index), window, cx);
                });
            }),
    );
    let more = more_anchor.wrap(
        div().id("note-actions").test_support().child(
            IconButton::new("note-actions-button", IconName::EllipsisVertical)
                .tooltip("More actions")
                .on_click(cx.listener(|workspace, _, window, cx| {
                    workspace.open_note_actions(window, cx);
                })),
        ),
    );
    div()
        .flex()
        .flex_none()
        .items_center()
        .min_h(tokens.scale.header_height)
        .pl(px(LEFT_PADDING))
        .child(focus)
        .child(back)
        .child(forward)
        .child(div().flex_1())
        .child(modes)
        .child(div().flex_1())
        .child(more)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mode_has_its_own_segment_in_order() {
        for (index, mode) in ViewMode::ALL.into_iter().enumerate() {
            assert_eq!(mode_index(mode), index);
            assert_eq!(mode_at(index), mode);
        }
        assert_eq!(segments().len(), ViewMode::ALL.len());
    }

    #[test]
    fn an_unknown_segment_falls_back_to_the_editor() {
        assert_eq!(mode_at(7), ViewMode::Edit);
    }

    #[test]
    fn the_focus_button_shows_shrink_while_focused() {
        assert_eq!(focus_mode_icon(true), IconName::Minimize2);
        assert_eq!(focus_mode_icon(false), IconName::Maximize2);
    }
}
