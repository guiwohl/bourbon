use super::canvas::Canvas;
use super::marks;
use crate::token::{HighlightSpan, TokenClass};
use crate::url;
use pulldown_cmark::{CodeBlockKind, Event, LinkType, Tag, TagEnd};
use std::ops::Range;

struct LinkFrame {
    range: Range<usize>,
    open_length: usize,
    note: bool,
}

pub struct Walk<'a> {
    source: &'a str,
    canvas: Canvas<'a>,
    marks: Vec<(Range<usize>, TokenClass)>,
    in_code_block: bool,
    links: Vec<Option<LinkFrame>>,
    previous_end: usize,
}

impl<'a> Walk<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            canvas: Canvas::new(source),
            marks: Vec::new(),
            in_code_block: false,
            links: Vec::new(),
            previous_end: 0,
        }
    }

    pub fn finish(mut self) -> Vec<HighlightSpan> {
        for (range, class) in std::mem::take(&mut self.marks) {
            self.canvas.paint(range, class);
        }
        self.canvas.into_spans()
    }

    pub fn visit(&mut self, event: &Event<'_>, range: Range<usize>) {
        let body = marks::trimmed(self.source, &range);
        match event {
            Event::Start(tag) => self.start(tag, &body),
            Event::End(TagEnd::Link | TagEnd::Image) => self.end_link(),
            Event::End(TagEnd::CodeBlock) => self.in_code_block = false,
            Event::Text(_) if self.in_code_block => {
                self.canvas.paint(body, TokenClass::CodeBlockText);
            }
            Event::Code(_) => self.inline_code(&body),
            Event::InlineMath(_) | Event::DisplayMath(_) => {
                self.canvas.paint(body, TokenClass::Math);
            }
            Event::Html(_) | Event::InlineHtml(_) => self.canvas.paint(body, TokenClass::HtmlTag),
            Event::Rule => self.canvas.paint(body, TokenClass::HorizontalRule),
            Event::TaskListMarker(checked) => {
                self.canvas
                    .paint(body, TokenClass::TaskMarker { checked: *checked });
            }
            _ => {}
        }
        self.previous_end = range.end;
    }

    fn mark_all(&mut self, ranges: impl IntoIterator<Item = Range<usize>>, class: TokenClass) {
        self.marks
            .extend(ranges.into_iter().map(|range| (range, class)));
    }

    fn start(&mut self, tag: &Tag<'_>, body: &Range<usize>) {
        match tag {
            Tag::Heading { level, .. } => {
                self.canvas
                    .paint(body.clone(), TokenClass::Heading(*level as u8));
                let mark = marks::heading_mark(self.source, body);
                self.mark_all(mark, TokenClass::HeaderMark);
            }
            Tag::Emphasis => self.styled(body, TokenClass::Emphasis, 1, TokenClass::EmphasisMark),
            Tag::Strong => self.styled(body, TokenClass::Strong, 2, TokenClass::StrongMark),
            Tag::Strikethrough => {
                self.canvas.paint(body.clone(), TokenClass::Strikethrough);
                let delimiters = marks::strikethrough_delimiters(self.source, body);
                self.mark_all(delimiters, TokenClass::StrikethroughMark);
            }
            Tag::BlockQuote(kind) => self.block_quote(body, kind.is_some()),
            Tag::CodeBlock(kind) => self.code_block(body, matches!(kind, CodeBlockKind::Fenced(_))),
            Tag::Link {
                link_type,
                dest_url,
                ..
            } => self.link(body, *link_type, dest_url),
            Tag::Image { dest_url, .. } => {
                self.canvas.paint(body.clone(), TokenClass::Image);
                self.push_link(body, 2, dest_url);
            }
            Tag::Item => {
                let mark = marks::list_marker(self.source, body);
                self.mark_all(mark, TokenClass::ListMark);
            }
            Tag::TableHead => self.table_head(body),
            Tag::MetadataBlock(_) => self.canvas.paint(body.clone(), TokenClass::Frontmatter),
            _ => {}
        }
    }

    fn styled(&mut self, body: &Range<usize>, class: TokenClass, length: usize, mark: TokenClass) {
        self.canvas.paint(body.clone(), class);
        let delimiters = marks::delimiters(self.source, body, length);
        self.mark_all(delimiters, mark);
    }

    fn inline_code(&mut self, body: &Range<usize>) {
        self.canvas.paint(body.clone(), TokenClass::InlineCode);
        let delimiters = marks::code_delimiters(self.source, body);
        self.mark_all(delimiters, TokenClass::CodeMark);
    }

    fn block_quote(&mut self, body: &Range<usize>, alert: bool) {
        self.canvas.paint(body.clone(), TokenClass::QuoteText);
        let quote_marks = marks::quote_marks(self.source, body);
        self.mark_all(quote_marks, TokenClass::QuoteMark);
        if alert {
            let marker = marks::alert_marker(self.source, body);
            self.mark_all(marker, TokenClass::AlertMarker);
        }
    }

    fn code_block(&mut self, body: &Range<usize>, fenced: bool) {
        self.in_code_block = true;
        let Some(fence) = fenced
            .then(|| marks::code_fence(self.source, body))
            .flatten()
        else {
            return;
        };
        self.mark_all([fence.open], TokenClass::CodeMark);
        self.mark_all(fence.close, TokenClass::CodeMark);
        self.mark_all(fence.info, TokenClass::CodeBlockInfo);
    }

    fn link(&mut self, body: &Range<usize>, link_type: LinkType, dest_url: &str) {
        if matches!(link_type, LinkType::Autolink | LinkType::Email) {
            self.canvas.paint(body.clone(), TokenClass::Url);
            self.links.push(None);
        } else {
            self.canvas.paint(body.clone(), TokenClass::LinkText);
            self.push_link(body, 1, dest_url);
        }
    }

    fn push_link(&mut self, body: &Range<usize>, open_length: usize, dest_url: &str) {
        self.links.push(Some(LinkFrame {
            range: body.clone(),
            open_length,
            note: url::note_id(dest_url).is_some(),
        }));
    }

    fn end_link(&mut self) {
        let Some(Some(frame)) = self.links.pop() else {
            return;
        };
        let parts = marks::link_parts(
            self.source,
            &frame.range,
            frame.open_length,
            self.previous_end,
        );
        self.mark_all(parts.marks, TokenClass::LinkMark);
        let url_class = if frame.note {
            TokenClass::NoteLink
        } else {
            TokenClass::Url
        };
        self.mark_all(parts.url, url_class);
    }

    fn table_head(&mut self, body: &Range<usize>) {
        self.canvas.paint(body.clone(), TokenClass::TableHeader);
        let delimiter = marks::line_after(self.source, body.end);
        self.mark_all(delimiter, TokenClass::TableDelimiter);
    }
}
