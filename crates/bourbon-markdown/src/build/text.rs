use crate::doc::Inline;

pub fn plain_text(inlines: &[Inline]) -> String {
    inlines.iter().map(inline_text).collect()
}

fn inline_text(inline: &Inline) -> String {
    match inline {
        Inline::Text(text) | Inline::Code(text) | Inline::InlineMath(text) => text.clone(),
        Inline::Emphasis(children)
        | Inline::Strong(children)
        | Inline::Strikethrough(children)
        | Inline::Link { children, .. }
        | Inline::NoteLink { children, .. } => plain_text(children),
        Inline::Image { alt, .. } => alt.clone(),
        Inline::SoftBreak | Inline::HardBreak => " ".to_owned(),
        Inline::FootnoteRef(_) => String::new(),
    }
}
