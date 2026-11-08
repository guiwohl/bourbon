use std::cell::Cell;
use std::rc::Rc;

use bourbon_markdown::{
    AlertKind, Alignment, Block, BlockKind, Inline, ListItem, PreviewDoc, SourceLines,
};
use gpui::{
    AnyElement, Bounds, Context, CursorStyle, Entity, EventEmitter, FontWeight, InteractiveText,
    IntoElement, ParentElement, Pixels, Render, StatefulInteractiveElement, Styled, StyledText,
    Window, div, prelude::*, px, relative,
};

use crate::inline::{LinkTarget, flatten_inlines};
use crate::preview_style::{
    alert_title, alert_vertical_padding, cell_alignment, checkbox_size, checkbox_tick_size,
    checkbox_top_margin, child_gap, footnote_size, heading_style, highlight_style, item_gap,
    list_indent, list_marker, quote_bar_width, row_is_striped, rule_height, stack_gap,
};
use crate::preview_theme::PreviewTheme;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreviewEvent {
    ToggleTask { source_line: usize },
    OpenLink(String),
    OpenNote(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockLayout {
    pub source_lines: SourceLines,
    pub top: Pixels,
    pub height: Pixels,
}

pub struct PreviewView {
    doc: PreviewDoc,
    theme: Rc<PreviewTheme>,
    rendered_lines: Vec<SourceLines>,
    layouts: Vec<BlockLayout>,
}

impl EventEmitter<PreviewEvent> for PreviewView {}

pub fn gap_between(previous: &BlockKind, next: &BlockKind, theme: &PreviewTheme) -> Pixels {
    let after_previous = match previous {
        BlockKind::ThematicBreak => theme.rule_gap,
        _ => theme.block_gap,
    };
    let before_next = match next {
        BlockKind::Heading { .. } => theme.heading_top_gap,
        BlockKind::ThematicBreak => theme.rule_gap,
        _ => px(0.0),
    };
    after_previous.max(before_next)
}

pub fn block_layouts(lines: &[SourceLines], bounds: &[Bounds<Pixels>]) -> Vec<BlockLayout> {
    let Some(first) = bounds.first() else {
        return Vec::new();
    };
    lines
        .iter()
        .zip(bounds)
        .map(|(source_lines, bounds)| BlockLayout {
            source_lines: *source_lines,
            top: bounds.origin.y - first.origin.y,
            height: bounds.size.height,
        })
        .collect()
}

impl PreviewView {
    pub fn new(source: &str, theme: PreviewTheme, _cx: &mut Context<Self>) -> Self {
        Self {
            doc: PreviewDoc::parse(source),
            theme: Rc::new(theme),
            rendered_lines: Vec::new(),
            layouts: Vec::new(),
        }
    }

    pub fn set_source(&mut self, source: &str, cx: &mut Context<Self>) {
        self.doc = PreviewDoc::parse(source);
        cx.notify();
    }

    pub fn set_theme(&mut self, theme: PreviewTheme, cx: &mut Context<Self>) {
        self.theme = Rc::new(theme);
        cx.notify();
    }

    pub fn theme(&self) -> &PreviewTheme {
        &self.theme
    }

    pub fn block_layouts(&self) -> &[BlockLayout] {
        &self.layouts
    }

    fn record_layouts(&mut self, bounds: &[Bounds<Pixels>]) {
        self.layouts = block_layouts(&self.rendered_lines, bounds);
    }
}

struct Renderer {
    theme: Rc<PreviewTheme>,
    view: Entity<PreviewView>,
    next_text: Cell<usize>,
}

impl Render for PreviewView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let renderer = Renderer {
            theme: self.theme.clone(),
            view: cx.entity(),
            next_text: Cell::new(0),
        };
        let blocks: Vec<&Block> = self
            .doc
            .blocks
            .iter()
            .filter(|block| !matches!(block.kind, BlockKind::Frontmatter(_)))
            .collect();
        self.rendered_lines = blocks.iter().map(|block| block.source_lines).collect();
        let children = renderer.stack(&blocks, true);
        let view = cx.entity();
        let theme = self.theme.clone();
        div()
            .id("bourbon-preview")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.background)
            .text_color(theme.text)
            .font_family(theme.font_family.clone())
            .text_size(theme.font_size)
            .line_height(relative(theme.line_height))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .p(px(10.0))
                    .on_children_prepainted(move |bounds, _, cx| {
                        view.update(cx, |preview, _| preview.record_layouts(&bounds));
                    })
                    .children(children),
            )
    }
}

impl Renderer {
    fn stack(&self, blocks: &[&Block], loose: bool) -> Vec<AnyElement> {
        let kinds: Vec<&BlockKind> = blocks.iter().map(|block| &block.kind).collect();
        blocks
            .iter()
            .enumerate()
            .map(|(index, block)| {
                let gap = stack_gap(&kinds, index, loose, &self.theme);
                div().mt(gap).child(self.block(block, 0)).into_any_element()
            })
            .collect()
    }

    fn owned_stack(&self, blocks: &[Block], loose: bool) -> Vec<AnyElement> {
        let references: Vec<&Block> = blocks.iter().collect();
        self.stack(&references, loose)
    }

    fn block(&self, block: &Block, depth: usize) -> AnyElement {
        match &block.kind {
            BlockKind::Heading { level, inlines, .. } => self.heading(*level, inlines),
            BlockKind::Paragraph(inlines) => self.inline(inlines),
            BlockKind::List {
                ordered,
                start,
                tight,
                items,
            } => self.list(*ordered, *start, *tight, items, depth),
            BlockKind::BlockQuote(children) => self.quote(children),
            BlockKind::Alert { kind, children } => self.alert(*kind, children),
            BlockKind::CodeBlock { text, .. } => self.code_block(text),
            BlockKind::Table {
                alignments,
                head,
                rows,
            } => self.table(alignments, head, rows),
            BlockKind::ThematicBreak => div()
                .h(rule_height(&self.theme))
                .bg(self.theme.solid_border)
                .into_any_element(),
            BlockKind::Html(raw) | BlockKind::Math(raw) => self.code_block(raw),
            BlockKind::Frontmatter(_) => div().into_any_element(),
            BlockKind::FootnoteDefinition { children, .. } => div()
                .text_color(self.theme.muted)
                .text_size(footnote_size(&self.theme))
                .children(self.owned_stack(children, true))
                .into_any_element(),
        }
    }

    fn heading(&self, level: u8, inlines: &[Inline]) -> AnyElement {
        let theme = &self.theme;
        let style = heading_style(level, theme);
        let mut heading = div()
            .text_size(style.size)
            .font_weight(FontWeight::SEMIBOLD)
            .line_height(relative(theme.heading_line_height))
            .child(self.inline(inlines));
        if style.muted {
            heading = heading.text_color(theme.muted);
        }
        if let Some(width) = style.rule {
            heading = heading
                .pb(style.padding_bottom)
                .border_b(width)
                .border_color(theme.border);
        }
        heading.into_any_element()
    }

    fn list(
        &self,
        ordered: bool,
        start: u64,
        tight: bool,
        items: &[ListItem],
        depth: usize,
    ) -> AnyElement {
        let indent = list_indent(&self.theme);
        div()
            .flex()
            .flex_col()
            .children(items.iter().enumerate().map(|(index, item)| {
                let marker = self.marker(ordered, start + index as u64, item, depth);
                let content = div()
                    .flex_1()
                    .min_w_0()
                    .children(self.list_item_children(item, tight, depth));
                let content = if item.task == Some(true) {
                    content.line_through().text_color(self.theme.muted)
                } else {
                    content
                };
                div()
                    .flex()
                    .flex_row()
                    .mt(item_gap(index, &self.theme))
                    .child(
                        div()
                            .w(indent)
                            .flex_none()
                            .flex()
                            .justify_end()
                            .pr(px(6.0))
                            .child(marker),
                    )
                    .child(content)
            }))
            .into_any_element()
    }

    fn list_item_children(&self, item: &ListItem, tight: bool, depth: usize) -> Vec<AnyElement> {
        item.children
            .iter()
            .enumerate()
            .map(|(index, child)| {
                let gap = child_gap(index, tight, &self.theme);
                div()
                    .mt(gap)
                    .child(self.block(child, depth + 1))
                    .into_any_element()
            })
            .collect()
    }

    fn marker(&self, ordered: bool, number: u64, item: &ListItem, depth: usize) -> AnyElement {
        if let Some(checked) = item.task {
            return self.checkbox(checked, item.source_lines.0);
        }
        let line = item.source_lines.0;
        div()
            .debug_selector(move || format!("marker-{depth}-{line}"))
            .child(list_marker(ordered, number, depth))
            .into_any_element()
    }

    fn checkbox(&self, checked: bool, source_line: usize) -> AnyElement {
        let size = checkbox_size(&self.theme);
        let view = self.view.clone();
        let mut checkbox = div()
            .id(("task", source_line))
            .debug_selector(move || format!("task-line-{source_line}"))
            .mt(checkbox_top_margin(&self.theme))
            .size(size)
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(3.0))
            .border_1()
            .border_color(self.theme.solid_border)
            .cursor(CursorStyle::PointingHand)
            .on_click(move |_, _, cx| {
                view.update(cx, |_, cx| {
                    cx.emit(PreviewEvent::ToggleTask { source_line });
                });
            });
        if checked {
            checkbox = checkbox
                .bg(self.theme.link)
                .text_color(self.theme.background)
                .text_size(checkbox_tick_size(&self.theme))
                .child("\u{2713}");
        }
        checkbox.into_any_element()
    }

    fn quote(&self, children: &[Block]) -> AnyElement {
        div()
            .pl(self.theme.font_size)
            .border_l(quote_bar_width(&self.theme))
            .border_color(self.theme.solid_border)
            .text_color(self.theme.muted)
            .children(self.owned_stack(children, true))
            .into_any_element()
    }

    fn alert(&self, kind: AlertKind, children: &[Block]) -> AnyElement {
        let color = self.theme.alert_color(kind);
        div()
            .px(self.theme.font_size)
            .py(alert_vertical_padding(&self.theme))
            .border_l(quote_bar_width(&self.theme))
            .border_color(color)
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(color)
                    .child(alert_title(kind)),
            )
            .children(self.owned_stack(children, true))
            .into_any_element()
    }

    fn code_block(&self, text: &str) -> AnyElement {
        let theme = &self.theme;
        div()
            .p(px(16.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(theme.code_border)
            .bg(theme.code_background)
            .overflow_hidden()
            .font_family(theme.mono_family.clone())
            .text_size(theme.code_font_size)
            .line_height(relative(theme.code_line_height))
            .whitespace_nowrap()
            .child(text.trim_end_matches('\n').to_owned())
            .into_any_element()
    }

    fn table(
        &self,
        alignments: &[Alignment],
        head: &[Vec<Inline>],
        rows: &[Vec<Vec<Inline>>],
    ) -> AnyElement {
        let theme = &self.theme;
        let header = self.table_row(alignments, head, true, false);
        let body = rows
            .iter()
            .enumerate()
            .map(|(index, row)| self.table_row(alignments, row, false, row_is_striped(index)));
        div()
            .flex()
            .flex_col()
            .border_1()
            .border_color(theme.solid_border)
            .child(header)
            .children(body)
            .into_any_element()
    }

    fn table_row(
        &self,
        alignments: &[Alignment],
        cells: &[Vec<Inline>],
        header: bool,
        striped: bool,
    ) -> AnyElement {
        let theme = &self.theme;
        let mut row = div().flex().flex_row();
        if header {
            row = row
                .bg(theme.table_header_background)
                .font_weight(FontWeight::SEMIBOLD);
        } else if striped {
            row = row.bg(theme.table_stripe_background);
        }
        row.children(cells.iter().enumerate().map(|(index, cell)| {
            div()
                .flex_1()
                .min_w_0()
                .px(px(13.0))
                .py(px(6.0))
                .border_1()
                .border_color(theme.solid_border)
                .text_align(cell_alignment(alignments.get(index)))
                .child(self.inline(cell))
        }))
        .into_any_element()
    }

    fn inline(&self, inlines: &[Inline]) -> AnyElement {
        let flat = flatten_inlines(inlines);
        let index = self.next_text.get();
        self.next_text.set(index + 1);
        let mut offset = 0;
        let mut highlights = Vec::new();
        for (len, flags) in flat.flag_runs() {
            if flags != 0 {
                highlights.push((offset..offset + len, highlight_style(flags, &self.theme)));
            }
            offset += len;
        }
        let styled = StyledText::new(flat.text.clone()).with_highlights(highlights);
        let text: AnyElement = if flat.links.is_empty() {
            styled.into_any_element()
        } else {
            let ranges = flat.links.iter().map(|(range, _)| range.clone()).collect();
            let targets: Vec<LinkTarget> = flat
                .links
                .iter()
                .map(|(_, target)| target.clone())
                .collect();
            let view = self.view.clone();
            InteractiveText::new(("inline-text", index), styled)
                .on_click(ranges, move |range_index, _, cx| {
                    let event = match &targets[range_index] {
                        LinkTarget::Url(url) => PreviewEvent::OpenLink(url.clone()),
                        LinkTarget::Note(id) => PreviewEvent::OpenNote(id.clone()),
                    };
                    view.update(cx, |_, cx| cx.emit(event));
                })
                .into_any_element()
        };
        div()
            .debug_selector(move || format!("inline-text-{index}"))
            .child(text)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, size};

    fn kind(source: &str) -> BlockKind {
        PreviewDoc::parse(source).blocks.remove(0).kind
    }

    #[test]
    fn paragraphs_are_one_block_gap_apart() {
        let theme = PreviewTheme::light();
        assert_eq!(gap_between(&kind("a"), &kind("b"), &theme), theme.block_gap);
    }

    #[test]
    fn headings_open_with_the_larger_top_gap() {
        let theme = PreviewTheme::light();
        assert_eq!(
            gap_between(&kind("a"), &kind("# b"), &theme),
            theme.heading_top_gap
        );
        assert_eq!(
            gap_between(&kind("# a"), &kind("b"), &theme),
            theme.block_gap
        );
    }

    #[test]
    fn rules_get_the_rule_gap_on_both_sides() {
        let theme = PreviewTheme::light();
        assert_eq!(
            gap_between(&kind("a"), &kind("---"), &theme),
            theme.rule_gap
        );
        assert_eq!(
            gap_between(&kind("---"), &kind("a"), &theme),
            theme.rule_gap
        );
    }

    fn bounds(top: f32, height: f32) -> Bounds<Pixels> {
        Bounds::new(point(px(10.0), px(top)), size(px(100.0), px(height)))
    }

    #[test]
    fn layouts_are_relative_to_the_first_block() {
        let layouts = block_layouts(&[(0, 0), (2, 3)], &[bounds(50.0, 30.0), bounds(96.0, 40.0)]);
        assert_eq!(
            layouts,
            vec![
                BlockLayout {
                    source_lines: (0, 0),
                    top: px(0.0),
                    height: px(30.0)
                },
                BlockLayout {
                    source_lines: (2, 3),
                    top: px(46.0),
                    height: px(40.0)
                },
            ]
        );
    }

    #[test]
    fn no_blocks_means_no_layouts() {
        assert_eq!(block_layouts(&[], &[]), vec![]);
    }
}
