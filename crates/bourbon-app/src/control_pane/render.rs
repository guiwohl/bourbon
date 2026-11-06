use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::input::{Escape, Input, MoveDown, MoveUp};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, Context, InteractiveElement as _, IntoElement, MouseButton, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, div, px,
};

use super::{ControlPane, Results};
use crate::presentation::display_title;
use crate::workspace::Workspace;

const PANEL_WIDTH: f32 = 560.0;
const PANEL_TOP: f32 = 80.0;
const PANEL_MAX_HEIGHT: f32 = 420.0;

pub(crate) fn render(pane: &ControlPane, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
    let rows = rows(pane, cx);
    div()
        .id("pane-backdrop")
        .test_support()
        .absolute()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .pt(px(PANEL_TOP))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|workspace, _, window, cx| workspace.close_control_pane(window, cx)),
        )
        .capture_action(cx.listener(|workspace, _: &MoveDown, _, cx| {
            workspace.highlight_next_pane_row(cx);
            cx.stop_propagation();
        }))
        .capture_action(cx.listener(|workspace, _: &MoveUp, _, cx| {
            workspace.highlight_previous_pane_row(cx);
            cx.stop_propagation();
        }))
        .capture_action(cx.listener(|workspace, _: &Escape, window, cx| {
            workspace.close_control_pane(window, cx);
            cx.stop_propagation();
        }))
        .child(
            div()
                .id("pane-panel")
                .test_support()
                .w(px(PANEL_WIDTH))
                .max_h(px(PANEL_MAX_HEIGHT))
                .flex()
                .flex_col()
                .gap_2()
                .p_2()
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().popover)
                .text_color(cx.theme().popover_foreground)
                .shadow_lg()
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .child(Input::new(&pane.input).id("pane-input"))
                .child(
                    div()
                        .id("pane-rows")
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .flex()
                        .flex_col()
                        .children(rows),
                ),
        )
}

fn rows(pane: &ControlPane, cx: &mut Context<Workspace>) -> Vec<AnyElement> {
    match pane.results() {
        Results::Notes(hits) => hits
            .iter()
            .enumerate()
            .map(|(index, hit)| {
                let content = div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .truncate()
                            .child(display_title(&hit.summary.title).to_owned()),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(hit.snippet.clone()),
                    );
                row(index, pane.highlighted_row() == index, content, cx)
            })
            .collect(),
        Results::Actions(actions) => actions
            .iter()
            .enumerate()
            .map(|(index, action)| {
                let content = div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(div().flex_1().truncate().child(action.name()))
                    .when_some(action.shortcut(), |line, shortcut| {
                        line.child(
                            div()
                                .flex_none()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(shortcut),
                        )
                    });
                row(index, pane.highlighted_row() == index, content, cx)
            })
            .collect(),
    }
}

fn row(
    index: usize,
    is_highlighted: bool,
    content: impl IntoElement,
    cx: &mut Context<Workspace>,
) -> AnyElement {
    div()
        .id(SharedString::from(format!("pane-row-{index}")))
        .test_support()
        .px_2()
        .py_1()
        .rounded_md()
        .cursor_pointer()
        .aria_selected(is_highlighted)
        .when(is_highlighted, |row| {
            row.bg(cx.theme().accent)
                .text_color(cx.theme().accent_foreground)
        })
        .on_click(cx.listener(move |workspace, _, window, cx| {
            workspace.activate_pane_row(index, window, cx);
        }))
        .child(content)
        .into_any_element()
}
