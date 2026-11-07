mod inline;
mod text;

use crate::doc::{
    AlertKind, Alignment, Block, BlockKind, Inline, ListItem, PreviewDoc, SourceLines,
};
use crate::lines::LineIndex;
use crate::parse;
use crate::slug::Slugger;
use pulldown_cmark::{BlockQuoteKind, CodeBlockKind, Event, HeadingLevel, Tag, TagEnd};
use std::ops::Range;
use text::plain_text;

struct Builder<'a> {
    source: &'a str,
    events: Vec<(Event<'a>, Range<usize>)>,
    position: usize,
    lines: LineIndex,
    slugger: Slugger,
}

impl PreviewDoc {
    #[must_use]
    pub fn parse(source: &str) -> Self {
        let mut builder = Builder {
            source,
            events: parse::events(source),
            position: 0,
            lines: LineIndex::new(source),
            slugger: Slugger::default(),
        };
        Self {
            blocks: builder.blocks(),
        }
    }
}

impl<'a> Builder<'a> {
    fn peek(&self) -> Option<&Event<'a>> {
        self.events.get(self.position).map(|(event, _)| event)
    }

    fn next_event(&mut self) -> Option<Event<'a>> {
        let event = self.events.get(self.position)?.0.clone();
        self.position += 1;
        Some(event)
    }

    fn skip_end(&mut self) {
        if matches!(self.peek(), Some(Event::End(_))) {
            self.position += 1;
        }
    }

    fn lines_of(&self, range: &Range<usize>) -> SourceLines {
        let end = self.source[..range.end].trim_end().len().max(range.start);
        let last_byte = end.saturating_sub(1).max(range.start);
        (
            self.lines.line_of(range.start),
            self.lines.line_of(last_byte),
        )
    }

    fn blocks(&mut self) -> Vec<Block> {
        let mut blocks = Vec::new();
        loop {
            if self.starts_inline() {
                blocks.push(self.implicit_paragraph());
                continue;
            }
            match self.peek() {
                None => break,
                Some(Event::End(_)) => {
                    self.position += 1;
                    break;
                }
                Some(_) => blocks.extend(self.block()),
            }
        }
        blocks
    }

    fn implicit_paragraph(&mut self) -> Block {
        let first = self.position;
        let inlines = self.inline_run();
        let range = self.events[first].1.start..self.events[self.position - 1].1.end;
        Block {
            source_lines: self.lines_of(&range),
            kind: BlockKind::Paragraph(inlines),
        }
    }

    fn block(&mut self) -> Vec<Block> {
        let range = self.events[self.position].1.clone();
        let Some(event) = self.next_event() else {
            return Vec::new();
        };
        let kind = match event {
            Event::Rule => BlockKind::ThematicBreak,
            Event::Start(tag) => return self.tagged_block(tag, &range),
            _ => return Vec::new(),
        };
        vec![Block {
            source_lines: self.lines_of(&range),
            kind,
        }]
    }

    fn tagged_block(&mut self, tag: Tag<'_>, range: &Range<usize>) -> Vec<Block> {
        let kind = match tag {
            Tag::Paragraph => self.paragraph(),
            Tag::Heading { level, .. } => self.heading(level),
            Tag::BlockQuote(None) => BlockKind::BlockQuote(self.blocks()),
            Tag::BlockQuote(Some(kind)) => BlockKind::Alert {
                kind: alert_kind(kind),
                children: self.blocks(),
            },
            Tag::CodeBlock(kind) => self.code_block(&kind),
            Tag::List(start) => self.list(start),
            Tag::Table(alignments) => self.table(&alignments),
            Tag::HtmlBlock => BlockKind::Html(self.raw_text().trim_end().to_owned()),
            Tag::MetadataBlock(_) => BlockKind::Frontmatter(self.raw_text()),
            Tag::FootnoteDefinition(label) => BlockKind::FootnoteDefinition {
                label: label.to_string(),
                children: self.blocks(),
            },
            _ => return self.blocks(),
        };
        vec![Block {
            source_lines: self.lines_of(range),
            kind,
        }]
    }

    fn paragraph(&mut self) -> BlockKind {
        let display_math = match (
            self.events.get(self.position),
            self.events.get(self.position + 1),
        ) {
            (Some((Event::DisplayMath(text), _)), Some((Event::End(TagEnd::Paragraph), _))) => {
                Some(text.to_string())
            }
            _ => None,
        };
        if let Some(text) = display_math {
            self.position += 2;
            return BlockKind::Math(text);
        }
        let inlines = self.inline_run();
        self.skip_end();
        BlockKind::Paragraph(inlines)
    }

    fn heading(&mut self, level: HeadingLevel) -> BlockKind {
        let inlines = self.inline_run();
        self.skip_end();
        let slug = self.slugger.slug(&plain_text(&inlines));
        BlockKind::Heading {
            level: level as u8,
            slug,
            inlines,
        }
    }

    fn raw_text(&mut self) -> String {
        let mut text = String::new();
        while let Some(event) = self.next_event() {
            match event {
                Event::End(_) => break,
                Event::Text(chunk) | Event::Html(chunk) => text.push_str(&chunk),
                _ => {}
            }
        }
        text
    }

    fn code_block(&mut self, kind: &CodeBlockKind<'_>) -> BlockKind {
        let (lang, meta) = match kind {
            CodeBlockKind::Fenced(info) => split_info(info),
            CodeBlockKind::Indented => (None, None),
        };
        BlockKind::CodeBlock {
            lang,
            meta,
            text: self.raw_text(),
        }
    }

    fn list(&mut self, start: Option<u64>) -> BlockKind {
        let mut items = Vec::new();
        let mut loose = false;
        while let Some((Event::Start(Tag::Item), range)) = self.events.get(self.position).cloned() {
            self.position += 1;
            let task = self.task_marker();
            loose |= matches!(self.peek(), Some(Event::Start(Tag::Paragraph)));
            let children = self.blocks();
            items.push(ListItem {
                source_lines: self.lines_of(&range),
                task,
                children,
            });
        }
        self.skip_end();
        BlockKind::List {
            ordered: start.is_some(),
            start: start.unwrap_or(1),
            tight: !loose,
            items,
        }
    }

    fn task_marker(&mut self) -> Option<bool> {
        let Some(Event::TaskListMarker(checked)) = self.peek() else {
            return None;
        };
        let checked = *checked;
        self.position += 1;
        Some(checked)
    }

    fn table(&mut self, alignments: &[pulldown_cmark::Alignment]) -> BlockKind {
        let mut head = Vec::new();
        let mut rows = Vec::new();
        while let Some(event) = self.next_event() {
            match event {
                Event::Start(Tag::TableHead) => head = self.cells(),
                Event::Start(Tag::TableRow) => rows.push(self.cells()),
                _ => break,
            }
        }
        BlockKind::Table {
            alignments: alignments
                .iter()
                .map(|alignment| table_alignment(*alignment))
                .collect(),
            head,
            rows,
        }
    }

    fn cells(&mut self) -> Vec<Vec<Inline>> {
        let mut cells = Vec::new();
        while matches!(self.peek(), Some(Event::Start(Tag::TableCell))) {
            self.position += 1;
            cells.push(self.inline_run());
            self.skip_end();
        }
        self.skip_end();
        cells
    }
}

fn split_info(info: &str) -> (Option<String>, Option<String>) {
    let info = info.trim();
    match info.split_once(char::is_whitespace) {
        _ if info.is_empty() => (None, None),
        Some((lang, meta)) => (Some(lang.to_owned()), Some(meta.trim().to_owned())),
        None => (Some(info.to_owned()), None),
    }
}

fn alert_kind(kind: BlockQuoteKind) -> AlertKind {
    match kind {
        BlockQuoteKind::Note => AlertKind::Note,
        BlockQuoteKind::Tip => AlertKind::Tip,
        BlockQuoteKind::Important => AlertKind::Important,
        BlockQuoteKind::Warning => AlertKind::Warning,
        BlockQuoteKind::Caution => AlertKind::Caution,
    }
}

fn table_alignment(alignment: pulldown_cmark::Alignment) -> Alignment {
    match alignment {
        pulldown_cmark::Alignment::None => Alignment::None,
        pulldown_cmark::Alignment::Left => Alignment::Left,
        pulldown_cmark::Alignment::Center => Alignment::Center,
        pulldown_cmark::Alignment::Right => Alignment::Right,
    }
}

#[cfg(test)]
mod tests;
