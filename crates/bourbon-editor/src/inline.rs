use std::ops::Range;

use bourbon_markdown::Inline;

pub const BOLD: u8 = 1;
pub const ITALIC: u8 = 2;
pub const STRIKE: u8 = 4;
pub const CODE: u8 = 8;
pub const LINK: u8 = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkTarget {
    Url(String),
    Note(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FlatText {
    pub text: String,
    pub flags: Vec<u8>,
    pub links: Vec<(Range<usize>, LinkTarget)>,
}

impl FlatText {
    fn push(&mut self, text: &str, flags: u8) {
        self.text.push_str(text);
        self.flags.extend(std::iter::repeat_n(flags, text.len()));
    }

    pub fn flag_runs(&self) -> Vec<(usize, u8)> {
        let mut runs: Vec<(usize, u8)> = Vec::new();
        for flags in &self.flags {
            match runs.last_mut() {
                Some((len, last)) if last == flags => *len += 1,
                _ => runs.push((1, *flags)),
            }
        }
        runs
    }
}

pub fn flatten_inlines(inlines: &[Inline]) -> FlatText {
    let mut flat = FlatText::default();
    append(&mut flat, inlines, 0);
    flat
}

fn append(flat: &mut FlatText, inlines: &[Inline], flags: u8) {
    for inline in inlines {
        match inline {
            Inline::Text(text) => flat.push(text, flags),
            Inline::Emphasis(children) => append(flat, children, with(flags, ITALIC)),
            Inline::Strong(children) => append(flat, children, with(flags, BOLD)),
            Inline::Strikethrough(children) => append(flat, children, with(flags, STRIKE)),
            Inline::Code(code) | Inline::InlineMath(code) => flat.push(code, with(flags, CODE)),
            Inline::Link { href, children } => {
                append_link(flat, children, flags, LinkTarget::Url(href.clone()));
            }
            Inline::NoteLink { note_id, children } => {
                append_link(flat, children, flags, LinkTarget::Note(note_id.clone()));
            }
            Inline::Image { alt, .. } => flat.push(alt, flags),
            Inline::SoftBreak => flat.push(" ", flags),
            Inline::HardBreak => flat.push("\n", flags),
            Inline::FootnoteRef(label) => flat.push(&format!("[{label}]"), flags),
        }
    }
}

fn with(flags: u8, flag: u8) -> u8 {
    flags | flag
}

fn append_link(flat: &mut FlatText, children: &[Inline], flags: u8, target: LinkTarget) {
    let start = flat.text.len();
    append(flat, children, with(flags, LINK));
    let end = flat.text.len();
    if end > start {
        flat.links.push((start..end, target));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(value: &str) -> Inline {
        Inline::Text(value.to_owned())
    }

    #[test]
    fn plain_text_has_no_flags() {
        let flat = flatten_inlines(&[text("abc")]);
        assert_eq!(flat.text, "abc");
        assert_eq!(flat.flag_runs(), vec![(3, 0)]);
    }

    #[test]
    fn nested_styles_combine_their_flags() {
        let flat = flatten_inlines(&[
            text("a"),
            Inline::Strong(vec![text("b"), Inline::Emphasis(vec![text("c")])]),
        ]);
        assert_eq!(flat.text, "abc");
        assert_eq!(
            flat.flag_runs(),
            vec![(1, 0), (1, BOLD), (1, BOLD | ITALIC)]
        );
    }

    #[test]
    fn strikethrough_and_code_are_flagged() {
        let flat = flatten_inlines(&[
            Inline::Strikethrough(vec![text("x")]),
            Inline::Code("y".to_owned()),
        ]);
        assert_eq!(flat.flag_runs(), vec![(1, STRIKE), (1, CODE)]);
    }

    #[test]
    fn links_record_their_byte_range_and_target() {
        let flat = flatten_inlines(&[
            text("see "),
            Inline::Link {
                href: "https://example.com".to_owned(),
                children: vec![Inline::Strong(vec![text("site")])],
            },
            text("!"),
        ]);
        assert_eq!(flat.text, "see site!");
        assert_eq!(
            flat.links,
            vec![(4..8, LinkTarget::Url("https://example.com".to_owned()))]
        );
        assert_eq!(flat.flag_runs(), vec![(4, 0), (4, LINK | BOLD), (1, 0)]);
    }

    #[test]
    fn note_links_target_the_note() {
        let flat = flatten_inlines(&[Inline::NoteLink {
            note_id: "abc".to_owned(),
            children: vec![text("budget")],
        }]);
        assert_eq!(flat.links, vec![(0..6, LinkTarget::Note("abc".to_owned()))]);
    }

    #[test]
    fn empty_links_record_nothing() {
        let flat = flatten_inlines(&[Inline::Link {
            href: "x".to_owned(),
            children: vec![],
        }]);
        assert_eq!(flat.links, vec![]);
    }

    #[test]
    fn breaks_images_and_footnotes_become_text() {
        let flat = flatten_inlines(&[
            text("a"),
            Inline::SoftBreak,
            text("b"),
            Inline::HardBreak,
            Inline::Image {
                src: "s".to_owned(),
                alt: "alt".to_owned(),
            },
            Inline::FootnoteRef("1".to_owned()),
        ]);
        assert_eq!(flat.text, "a b\nalt[1]");
    }

    #[test]
    fn flags_cover_every_byte_of_multibyte_text() {
        let flat = flatten_inlines(&[Inline::Strong(vec![text("é🙂")])]);
        assert_eq!(flat.flags.len(), flat.text.len());
        assert_eq!(flat.flag_runs(), vec![(6, BOLD)]);
    }

    #[test]
    fn repeating_a_style_does_not_cancel_it() {
        let flat = flatten_inlines(&[Inline::Strong(vec![Inline::Strong(vec![text("a")])])]);
        assert_eq!(flat.flag_runs(), vec![(1, BOLD)]);
        let flat = flatten_inlines(&[Inline::Emphasis(vec![Inline::Emphasis(vec![text("a")])])]);
        assert_eq!(flat.flag_runs(), vec![(1, ITALIC)]);
        let flat = flatten_inlines(&[Inline::Strikethrough(vec![Inline::Strikethrough(vec![
            text("a"),
        ])])]);
        assert_eq!(flat.flag_runs(), vec![(1, STRIKE)]);
    }

    #[test]
    fn a_link_inside_a_link_keeps_the_link_flag() {
        let inner = Inline::Link {
            href: "b".to_owned(),
            children: vec![text("a")],
        };
        let flat = flatten_inlines(&[Inline::Link {
            href: "a".to_owned(),
            children: vec![inner],
        }]);
        assert_eq!(flat.flag_runs(), vec![(1, LINK)]);
    }
}
