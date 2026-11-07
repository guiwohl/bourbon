use bourbon_core::ViewMode;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::text::{TextView, TextViewStyle};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, StyleRefinement, Styled as _, TestSupportExt as _, div, px,
    relative,
};

use super::preview_blocks::{PreviewBlock, preview_blocks};
use crate::workspace::Workspace;

const INDENT_STEP: f32 = 6.;
const PREVIEW_PADDING: f32 = 20.;
const PREVIEW_TEXT_SIZE: f32 = 16.;
const PREVIEW_LINE_HEIGHT: f32 = 1.6;
const PREVIEW_MEASURE: f32 = 760.;
const CODE_BLOCK_PADDING: f32 = 12.;

fn heading_size(level: u8, base: gpui_kit::Pixels) -> gpui_kit::Pixels {
    match level {
        1 => base * 2.,
        2 => base * 1.5,
        3 => base * 1.25,
        _ => base,
    }
}

fn markdown_style(cx: &gpui_kit::App) -> TextViewStyle {
    let theme = cx.theme();
    let code_block = StyleRefinement::default()
        .rounded_md()
        .p(px(CODE_BLOCK_PADDING))
        .bg(theme.muted)
        .text_size(px(14.));
    let mut style = TextViewStyle::default()
        .heading_font_size(heading_size)
        .code_block(code_block);
    style.heading_base_font_size = px(PREVIEW_TEXT_SIZE);
    style.highlight_theme = theme.highlight_theme.clone();
    style
}

pub(super) fn render(
    body: &str,
    mode: ViewMode,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    let blocks = preview_blocks(body)
        .into_iter()
        .enumerate()
        .map(|(index, block)| match block {
            PreviewBlock::Markdown(text) => {
                markdown(format!("preview-markdown-{index}"), text, cx).into_any_element()
            }
            PreviewBlock::Task {
                line,
                indent,
                checked,
                label,
            } => task(line, indent, checked, label, cx).into_any_element(),
        })
        .collect::<Vec<_>>();
    div()
        .id("preview")
        .test_support()
        .flex_1()
        .min_w_0()
        .h_full()
        .overflow_y_scroll()
        .child(
            div()
                .p(px(PREVIEW_PADDING))
                .flex()
                .flex_col()
                .mx_auto()
                .w_full()
                .when(mode == ViewMode::Preview, |column| {
                    column.max_w(px(PREVIEW_MEASURE))
                })
                .text_size(px(PREVIEW_TEXT_SIZE))
                .line_height(relative(PREVIEW_LINE_HEIGHT))
                .text_color(cx.theme().foreground)
                .children(blocks),
        )
}

fn markdown(id: String, text: String, cx: &mut Context<Workspace>) -> TextView {
    let owner = cx.entity().downgrade();
    TextView::markdown(id, text)
        .style(markdown_style(cx))
        .selectable(true)
        .on_link_click(move |url, _, window, cx| {
            owner
                .update(cx, |workspace, cx| workspace.open_link(url, window, cx))
                .ok();
        })
}

fn task(
    line: usize,
    indent: usize,
    checked: bool,
    label: String,
    cx: &mut Context<Workspace>,
) -> impl IntoElement + use<> {
    div()
        .flex()
        .items_start()
        .gap_2()
        .ml(px(INDENT_STEP) * f32::from(u16::try_from(indent).unwrap_or(0)))
        .child(
            Checkbox::new(format!("preview-task-{line}"))
                .checked(checked)
                .on_click(cx.listener(move |workspace, _, window, cx| {
                    workspace.toggle_task_on_line(line, window, cx);
                })),
        )
        .child(markdown(format!("preview-task-label-{line}"), label, cx))
}
