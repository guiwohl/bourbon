use super::Builder;
use super::text::plain_text;
use crate::doc::Inline;
use crate::url;
use pulldown_cmark::{Event, Tag};

impl Builder<'_> {
    pub(super) fn starts_inline(&self) -> bool {
        matches!(
            self.peek(),
            Some(
                Event::Text(_)
                    | Event::Code(_)
                    | Event::InlineHtml(_)
                    | Event::SoftBreak
                    | Event::HardBreak
                    | Event::FootnoteReference(_)
                    | Event::InlineMath(_)
                    | Event::DisplayMath(_)
                    | Event::Start(
                        Tag::Emphasis
                            | Tag::Strong
                            | Tag::Strikethrough
                            | Tag::Link { .. }
                            | Tag::Image { .. }
                    )
            )
        )
    }

    pub(super) fn inline_run(&mut self) -> Vec<Inline> {
        let mut inlines = Vec::new();
        while self.starts_inline() {
            let Some(event) = self.next_event() else {
                break;
            };
            match event {
                Event::Text(text) | Event::InlineHtml(text) => push_text(&mut inlines, &text),
                Event::Code(text) => inlines.push(Inline::Code(text.to_string())),
                Event::SoftBreak => inlines.push(Inline::SoftBreak),
                Event::HardBreak => inlines.push(Inline::HardBreak),
                Event::FootnoteReference(label) => {
                    inlines.push(Inline::FootnoteRef(label.to_string()));
                }
                Event::InlineMath(text) | Event::DisplayMath(text) => {
                    inlines.push(Inline::InlineMath(text.to_string()));
                }
                Event::Start(tag) => self.nested_inline(&tag, &mut inlines),
                _ => {}
            }
        }
        inlines
    }

    fn nested_inline(&mut self, tag: &Tag<'_>, inlines: &mut Vec<Inline>) {
        let children = self.inline_run();
        self.skip_end();
        match tag {
            Tag::Emphasis => inlines.push(Inline::Emphasis(children)),
            Tag::Strong => inlines.push(Inline::Strong(children)),
            Tag::Strikethrough => inlines.push(Inline::Strikethrough(children)),
            Tag::Link { dest_url, .. } => link(dest_url, children, inlines),
            Tag::Image { dest_url, .. } => image(dest_url, &children, inlines),
            _ => inlines.extend(children),
        }
    }
}

fn push_text(inlines: &mut Vec<Inline>, text: &str) {
    if let Some(Inline::Text(previous)) = inlines.last_mut() {
        previous.push_str(text);
    } else {
        inlines.push(Inline::Text(text.to_owned()));
    }
}

fn link(dest_url: &str, children: Vec<Inline>, inlines: &mut Vec<Inline>) {
    if !url::is_safe(dest_url) {
        inlines.extend(children);
    } else if let Some(note_id) = url::note_id(dest_url) {
        inlines.push(Inline::NoteLink {
            note_id: note_id.to_owned(),
            children,
        });
    } else {
        inlines.push(Inline::Link {
            href: dest_url.to_owned(),
            children,
        });
    }
}

fn image(dest_url: &str, children: &[Inline], inlines: &mut Vec<Inline>) {
    let alt = plain_text(children);
    if url::is_safe(dest_url) {
        inlines.push(Inline::Image {
            src: dest_url.to_owned(),
            alt,
        });
    } else {
        inlines.push(Inline::Text(alt));
    }
}
