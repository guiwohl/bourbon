pub type SourceLines = (usize, usize);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewDoc {
    pub blocks: Vec<Block>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub source_lines: SourceLines,
    pub kind: BlockKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockKind {
    Heading {
        level: u8,
        slug: String,
        inlines: Vec<Inline>,
    },
    Paragraph(Vec<Inline>),
    List {
        ordered: bool,
        start: u64,
        tight: bool,
        items: Vec<ListItem>,
    },
    BlockQuote(Vec<Block>),
    Alert {
        kind: AlertKind,
        children: Vec<Block>,
    },
    CodeBlock {
        lang: Option<String>,
        meta: Option<String>,
        text: String,
    },
    Table {
        alignments: Vec<Alignment>,
        head: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
    ThematicBreak,
    Html(String),
    Math(String),
    Frontmatter(String),
    FootnoteDefinition {
        label: String,
        children: Vec<Block>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListItem {
    pub source_lines: SourceLines,
    pub task: Option<bool>,
    pub children: Vec<Block>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertKind {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alignment {
    None,
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inline {
    Text(String),
    Emphasis(Vec<Inline>),
    Strong(Vec<Inline>),
    Strikethrough(Vec<Inline>),
    Code(String),
    Link {
        href: String,
        children: Vec<Inline>,
    },
    NoteLink {
        note_id: String,
        children: Vec<Inline>,
    },
    Image {
        src: String,
        alt: String,
    },
    SoftBreak,
    HardBreak,
    FootnoteRef(String),
    InlineMath(String),
}
