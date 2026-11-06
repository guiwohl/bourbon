use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::text::TextView;
use gpui_kit::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, div, px,
};

use super::preview_blocks::{PreviewBlock, preview_blocks};
use crate::workspace::Workspace;

const INDENT_STEP: f32 = 6.;

pub(super) fn render(body: &str, cx: &mut Context<Workspace>) -> impl IntoElement + use<> {
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
        .p_2()
        .flex()
        .flex_col()
        .text_color(cx.theme().foreground)
        .children(blocks)
}

fn markdown(id: String, text: String, cx: &mut Context<Workspace>) -> TextView {
    let owner = cx.entity().downgrade();
    TextView::markdown(id, text)
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
