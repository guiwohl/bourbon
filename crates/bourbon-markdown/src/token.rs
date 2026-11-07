use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TokenClass {
    Heading(u8),
    HeaderMark,
    Emphasis,
    Strong,
    Strikethrough,
    EmphasisMark,
    StrongMark,
    StrikethroughMark,
    InlineCode,
    CodeMark,
    CodeBlockInfo,
    CodeBlockText,
    LinkText,
    LinkMark,
    Url,
    Image,
    ListMark,
    TaskMarker { checked: bool },
    QuoteMark,
    QuoteText,
    HorizontalRule,
    TableDelimiter,
    TableHeader,
    Frontmatter,
    Math,
    AlertMarker,
    HtmlTag,
    NoteLink,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HighlightSpan {
    pub range: Range<usize>,
    pub class: TokenClass,
}
