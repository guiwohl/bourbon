use bourbon_ui::{Icon, IconName, Modal, Placement, ThemeExt as _, Tokens};
use gpui::prelude::FluentBuilder as _;
use gpui::{
    AnyElement, Context, InteractiveElement as _, IntoElement, ParentElement as _, Pixels,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use super::{ControlPane, HighlightNext, HighlightPrevious, KEY_CONTEXT, Mode, Results};
use crate::overlay::{layer, tagged};
use crate::presentation::display_title;
use crate::test_id::TestSupportExt as _;
use crate::workspace::Workspace;

const PANEL_WIDTH: f32 = 600.;
const PANEL_TOP: f32 = 56.;
const PANEL_BOTTOM_MARGIN: f32 = 28.;
const INPUT_ROW_HEIGHT: f32 = 48.;
const INPUT_PADDING_LEFT: f32 = 11.2;
const INPUT_PADDING_RIGHT: f32 = 8.4;
const INPUT_ICON_SIZE: f32 = 20.;
const INPUT_ICON_OPACITY: f32 = 0.5;
const INPUT_GAP: f32 = 10.;
const LIST_PADDING_VERTICAL: f32 = 4.;
const ROW_MARGIN_VERTICAL: f32 = 2.;
const ROW_MARGIN_HORIZONTAL: f32 = 8.;
const ROW_PADDING_VERTICAL: f32 = 8.;
const ROW_PADDING_HORIZONTAL: f32 = 10.;
const ROW_ICON_SLOT: f32 = 20.;
const ROW_ICON_SIZE: f32 = 16.;
const ROW_ICON_GAP: f32 = 12.;
const ROW_ACCENT_WIDTH: f32 = 3.;
const DETAIL_GAP: f32 = 2.;
const EMPTY_PADDING_VERTICAL: f32 = 40.;
const EMPTY_PADDING_HORIZONTAL: f32 = 20.;
const FOOTER_PADDING_VERTICAL: f32 = 8.;
const FOOTER_PADDING_HORIZONTAL: f32 = 20.;
const FOOTER_GAP: f32 = 16.;

struct Row {
    icon: IconName,
    label: SharedString,
    detail: Option<SharedString>,
    accessory: Option<SharedString>,
}

pub(crate) fn footer_hints(mode: Mode) -> [&'static str; 3] {
    let enter = match mode {
        Mode::Notes => "\u{21b5} to open",
        Mode::Actions => "\u{21b5} to run",
    };
    ["\u{2191}\u{2193} to navigate", enter, "esc to close"]
}

pub(crate) fn panel_max_height(viewport_height: Pixels) -> Pixels {
    viewport_height - px(PANEL_TOP + PANEL_BOTTOM_MARGIN)
}

fn mode_of(results: &Results) -> Mode {
    match results {
        Results::Notes(_) => Mode::Notes,
        Results::Actions(_) => Mode::Actions,
    }
}

fn rows_of(results: &Results) -> Vec<Row> {
    match results {
        Results::Notes(hits) => hits
            .iter()
            .map(|hit| Row {
                icon: IconName::FileText,
                label: display_title(&hit.summary.title).to_owned().into(),
                detail: Some(hit.snippet.clone().into()),
                accessory: None,
            })
            .collect(),
        Results::Actions(actions) => actions
            .iter()
            .map(|action| Row {
                icon: IconName::ChevronRight,
                label: action.name().into(),
                detail: None,
                accessory: action.shortcut().map(SharedString::from),
            })
            .collect(),
    }
}

pub(crate) fn render(
    pane: &ControlPane,
    window: &Window,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let tokens = cx.theme().tokens.clone();
    let colors = tokens.colors;
    let max_height = panel_max_height(window.viewport_size().height);
    let rows: Vec<AnyElement> = rows_of(pane.results())
        .into_iter()
        .enumerate()
        .map(|(index, row)| render_row(index, row, pane.highlighted_row() == index, &tokens, cx))
        .collect();
    let list = if rows.is_empty() {
        div()
            .py(px(EMPTY_PADDING_VERTICAL))
            .px(px(EMPTY_PADDING_HORIZONTAL))
            .flex()
            .justify_center()
            .text_color(colors.muted_text)
            .child("No results found")
            .into_any_element()
    } else {
        div()
            .id("pane-rows")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .py(px(LIST_PADDING_VERTICAL))
            .flex()
            .flex_col()
            .children(rows)
            .into_any_element()
    };
    let panel = div()
        .id("pane-panel")
        .test_support()
        .key_context(KEY_CONTEXT)
        .on_action(cx.listener(|workspace, _: &HighlightNext, _, cx| {
            workspace.highlight_next_pane_row(cx);
        }))
        .on_action(cx.listener(|workspace, _: &HighlightPrevious, _, cx| {
            workspace.highlight_previous_pane_row(cx);
        }))
        .w(px(PANEL_WIDTH))
        .max_h(max_height)
        .flex()
        .flex_col()
        .overflow_hidden()
        .rounded(tokens.scale.radius_control)
        .border_1()
        .border_color(colors.border)
        .bg(colors.page_background)
        .text_color(colors.text)
        .font_family(tokens.fonts.ui_family.clone())
        .text_size(tokens.scale.medium)
        .shadow(tokens.shadows.palette.clone())
        .child(input_row(pane, &tokens))
        .child(list)
        .child(footer(mode_of(pane.results()), &tokens));
    let view = cx.entity();
    layer(
        "pane-backdrop",
        Modal::new("pane-modal", panel)
            .placement(Placement::Top(px(PANEL_TOP)))
            .on_dismiss(move |window, cx| {
                view.update(cx, |workspace, cx| workspace.close_control_pane(window, cx));
            }),
    )
}

fn input_row(pane: &ControlPane, tokens: &Tokens) -> impl IntoElement + use<> {
    div()
        .flex_none()
        .flex()
        .items_center()
        .gap(px(INPUT_GAP))
        .h(px(INPUT_ROW_HEIGHT))
        .pl(px(INPUT_PADDING_LEFT))
        .pr(px(INPUT_PADDING_RIGHT))
        .border_b_1()
        .border_color(tokens.colors.border)
        .child(
            div().flex_none().opacity(INPUT_ICON_OPACITY).child(
                Icon::new(IconName::Search)
                    .size(px(INPUT_ICON_SIZE))
                    .color(tokens.colors.text),
            ),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(tagged("pane-input", pane.input().clone())),
        )
}

fn footer(mode: Mode, tokens: &Tokens) -> impl IntoElement + use<> {
    div()
        .flex_none()
        .flex()
        .gap(px(FOOTER_GAP))
        .py(px(FOOTER_PADDING_VERTICAL))
        .px(px(FOOTER_PADDING_HORIZONTAL))
        .border_t_1()
        .border_color(tokens.colors.internal_border)
        .text_size(tokens.scale.tiny)
        .text_color(tokens.colors.muted_text)
        .children(footer_hints(mode))
}

fn render_row(
    index: usize,
    row: Row,
    is_highlighted: bool,
    tokens: &Tokens,
    cx: &mut Context<Workspace>,
) -> AnyElement {
    let colors = tokens.colors;
    let text = div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(DETAIL_GAP))
        .child(
            div()
                .truncate()
                .font_weight(tokens.scale.weight_medium)
                .child(row.label),
        )
        .when_some(row.detail, |text, detail| {
            text.child(
                div()
                    .truncate()
                    .text_size(tokens.scale.tiny)
                    .text_color(colors.muted_text)
                    .child(detail),
            )
        });
    div()
        .id(SharedString::from(format!("pane-row-{index}")))
        .test_support()
        .relative()
        .flex_none()
        .flex()
        .items_center()
        .gap(px(ROW_ICON_GAP))
        .my(px(ROW_MARGIN_VERTICAL))
        .mx(px(ROW_MARGIN_HORIZONTAL))
        .py(px(ROW_PADDING_VERTICAL))
        .px(px(ROW_PADDING_HORIZONTAL))
        .rounded(tokens.scale.radius_default)
        .overflow_hidden()
        .cursor_pointer()
        .aria_selected(is_highlighted)
        .hover(move |style| style.bg(colors.selection_list_hover))
        .when(is_highlighted, |row| {
            row.bg(colors.selection_list_active).child(
                div()
                    .debug_selector(|| format!("pane-row-{index}-highlight"))
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left_0()
                    .w(px(ROW_ACCENT_WIDTH))
                    .bg(colors.info),
            )
        })
        .on_mouse_move(cx.listener(move |workspace, _, _, cx| {
            workspace.highlight_pane_row(index, cx);
        }))
        .on_click(cx.listener(move |workspace, _, window, cx| {
            workspace.activate_pane_row(index, window, cx);
        }))
        .child(
            div()
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .size(px(ROW_ICON_SLOT))
                .child(
                    Icon::new(row.icon)
                        .size(px(ROW_ICON_SIZE))
                        .color(colors.muted_text),
                ),
        )
        .child(text)
        .when_some(row.accessory, |line, accessory| {
            line.child(
                div()
                    .flex_none()
                    .text_size(tokens.scale.tiny)
                    .text_color(colors.muted_text)
                    .child(accessory),
            )
        })
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panel_leaves_56_px_above_and_28_px_below() {
        assert_eq!(panel_max_height(px(900.)), px(816.));
    }

    #[test]
    fn the_footer_names_the_enter_verb_of_each_mode() {
        assert_eq!(footer_hints(Mode::Notes)[1], "\u{21b5} to open");
        assert_eq!(footer_hints(Mode::Actions)[1], "\u{21b5} to run");
        assert_eq!(footer_hints(Mode::Notes)[0], footer_hints(Mode::Actions)[0]);
        assert_eq!(footer_hints(Mode::Notes)[2], "esc to close");
    }
}
