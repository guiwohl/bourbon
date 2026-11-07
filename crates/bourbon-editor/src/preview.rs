use std::cell::Cell;
use std::rc::Rc;

use bourbon_markdown::{
    AlertKind, Alignment, Block, BlockKind, Inline, ListItem, PreviewDoc, SourceLines,
};
use gpui::{
    AnyElement, Bounds, Context, CursorStyle, Entity, EventEmitter, FontStyle, FontWeight,
    HighlightStyle, InteractiveText, IntoElement, ParentElement, Pixels, Render,
    StatefulInteractiveElement, StrikethroughStyle, Styled, StyledText, UnderlineStyle, Window,
    div, prelude::*, px, relative,
};

use crate::inline::{BOLD, CODE, ITALIC, LINK, LinkTarget, STRIKE, flatten_inlines};
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
                    .px(px(24.0))
                    .py(px(16.0))
                    .on_children_prepainted(move |bounds, _, cx| {
                        view.update(cx, |preview, _| preview.record_layouts(&bounds));
                    })
                    .children(children),
            )
    }
}

impl Renderer {
    fn stack(&self, blocks: &[&Block], loose: bool) -> Vec<AnyElement> {
        blocks
            .iter()
            .enumerate()
            .map(|(index, block)| {
                let gap = match (index, loose) {
                    (0, _) | (_, false) => px(0.0),
                    _ => gap_between(&blocks[index - 1].kind, &block.kind, &self.theme),
                };
                div().mt(gap).child(self.block(block)).into_any_element()
            })
            .collect()
    }

    fn owned_stack(&self, blocks: &[Block], loose: bool) -> Vec<AnyElement> {
        let references: Vec<&Block> = blocks.iter().collect();
        self.stack(&references, loose)
    }

    fn block(&self, block: &Block) -> AnyElement {
        match &block.kind {
            BlockKind::Heading { level, inlines, .. } => self.heading(*level, inlines),
            BlockKind::Paragraph(inlines) => self.inline(inlines),
            BlockKind::List {
                ordered,
                start,
                tight,
                items,
            } => self.list(*ordered, *start, *tight, items, 0),
            BlockKind::BlockQuote(children) => self.quote(children),
            BlockKind::Alert { kind, children } => self.alert(*kind, children),
            BlockKind::CodeBlock { text, .. } => self.code_block(text),
            BlockKind::Table {
                alignments,
                head,
                rows,
            } => self.table(alignments, head, rows),
            BlockKind::ThematicBreak => div()
                .h(self.theme.font_size * 0.25)
                .bg(self.theme.solid_border)
                .into_any_element(),
            BlockKind::Html(raw) | BlockKind::Math(raw) => self.code_block(raw),
            BlockKind::Frontmatter(_) => div().into_any_element(),
            BlockKind::FootnoteDefinition { children, .. } => div()
                .text_color(self.theme.muted)
                .text_size(self.theme.font_size * 0.75)
                .children(self.owned_stack(children, true))
                .into_any_element(),
        }
    }

    fn heading(&self, level: u8, inlines: &[Inline]) -> AnyElement {
        let theme = &self.theme;
        let size = theme.font_size * theme.heading_scale(level);
        let mut heading = div()
            .text_size(size)
            .font_weight(FontWeight::SEMIBOLD)
            .line_height(relative(theme.heading_line_height))
            .child(self.inline(inlines));
        if level == 6 {
            heading = heading.text_color(theme.muted);
        }
        if level <= 2 {
            let width = if level == 1 { px(2.0) } else { px(1.0) };
            heading = heading
                .pb(size * 0.3)
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
        let indent = self.theme.font_size * self.theme.list_indent_em;
        div()
            .flex()
            .flex_col()
            .children(items.iter().enumerate().map(|(index, item)| {
                let marker = self.marker(ordered, start + index as u64, item, depth);
                let content = div()
                    .flex_1()
                    .min_w_0()
                    .children(self.list_item_children(item, tight, depth));
                let content = match item.task {
                    Some(true) => content.line_through().text_color(self.theme.muted),
                    _ => content,
                };
                div()
                    .flex()
                    .flex_row()
                    .mt(if index == 0 {
                        px(0.0)
                    } else {
                        self.theme.font_size * 0.25
                    })
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
                let gap = match (index, tight) {
                    (0, _) | (_, true) => px(0.0),
                    _ => self.theme.block_gap,
                };
                let element = match &child.kind {
                    BlockKind::List {
                        ordered,
                        start,
                        tight,
                        items,
                    } => self.list(*ordered, *start, *tight, items, depth + 1),
                    _ => self.block(child),
                };
                div().mt(gap).child(element).into_any_element()
            })
            .collect()
    }

    fn marker(&self, ordered: bool, number: u64, item: &ListItem, depth: usize) -> AnyElement {
        if let Some(checked) = item.task {
            return self.checkbox(checked, item.source_lines.0);
        }
        let text = if ordered {
            format!("{number}.")
        } else {
            ["\u{2022}", "\u{25E6}", "\u{25AA}"][depth.min(2)].to_owned()
        };
        div().child(text).into_any_element()
    }

    fn checkbox(&self, checked: bool, source_line: usize) -> AnyElement {
        let size = self.theme.font_size * 0.875;
        let view = self.view.clone();
        let mut checkbox = div()
            .id(("task", source_line))
            .debug_selector(move || format!("task-line-{source_line}"))
            .mt(self.theme.font_size * 0.2)
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
                .text_size(size * 0.8)
                .child("\u{2713}");
        }
        checkbox.into_any_element()
    }

    fn quote(&self, children: &[Block]) -> AnyElement {
        div()
            .pl(self.theme.font_size)
            .border_l(self.theme.font_size * self.theme.quote_bar_em)
            .border_color(self.theme.solid_border)
            .text_color(self.theme.muted)
            .children(self.owned_stack(children, true))
            .into_any_element()
    }

    fn alert(&self, kind: AlertKind, children: &[Block]) -> AnyElement {
        let color = self.theme.alert_color(kind);
        let title = match kind {
            AlertKind::Note => "Note",
            AlertKind::Tip => "Tip",
            AlertKind::Important => "Important",
            AlertKind::Warning => "Warning",
            AlertKind::Caution => "Caution",
        };
        div()
            .px(self.theme.font_size)
            .py(self.theme.font_size * 0.5)
            .border_l(self.theme.font_size * self.theme.quote_bar_em)
            .border_color(color)
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(color)
                    .child(title),
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
            .map(|(index, row)| self.table_row(alignments, row, false, index % 2 == 0));
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
            let aligned = div()
                .flex_1()
                .min_w_0()
                .px(px(13.0))
                .py(px(6.0))
                .border_1()
                .border_color(theme.solid_border)
                .child(self.inline(cell));
            match alignments.get(index) {
                Some(Alignment::Right) => aligned.text_right(),
                Some(Alignment::Center) => aligned.text_center(),
                _ => aligned,
            }
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
                highlights.push((offset..offset + len, self.highlight(flags)));
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

    fn highlight(&self, flags: u8) -> HighlightStyle {
        let theme = &self.theme;
        HighlightStyle {
            color: (flags & LINK != 0).then_some(theme.link),
            font_weight: (flags & BOLD != 0).then_some(FontWeight::SEMIBOLD),
            font_style: (flags & ITALIC != 0).then_some(FontStyle::Italic),
            background_color: (flags & CODE != 0).then_some(theme.inline_code_background),
            strikethrough: (flags & STRIKE != 0).then_some(StrikethroughStyle {
                thickness: px(1.0),
                color: None,
            }),
            underline: None::<UnderlineStyle>,
            fade_out: None,
        }
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
