use crate::doc::{AlertKind, Alignment, BlockKind, Inline, PreviewDoc};
use crate::fixture::NOTE;

fn text(value: &str) -> Inline {
    Inline::Text(value.to_owned())
}

fn paragraph(doc: &PreviewDoc, index: usize) -> &Vec<Inline> {
    match &doc.blocks[index].kind {
        BlockKind::Paragraph(inlines) => inlines,
        other => panic!("not a paragraph: {other:?}"),
    }
}

#[test]
fn headings_carry_level_slug_inlines_and_lines() {
    let doc = PreviewDoc::parse("# Hello *world*\n\n## Hello *world*\n");
    assert_eq!(doc.blocks.len(), 2);
    assert_eq!(doc.blocks[0].source_lines, (0, 0));
    assert_eq!(doc.blocks[1].source_lines, (2, 2));
    assert_eq!(
        doc.blocks[0].kind,
        BlockKind::Heading {
            level: 1,
            slug: "hello-world".to_owned(),
            inlines: vec![text("Hello "), Inline::Emphasis(vec![text("world")])],
        }
    );
    let BlockKind::Heading { level, slug, .. } = &doc.blocks[1].kind else {
        panic!()
    };
    assert_eq!((*level, slug.as_str()), (2, "hello-world-1"));
}

#[test]
fn paragraph_spans_its_lines_and_keeps_soft_breaks() {
    let doc = PreviewDoc::parse("a\nb\n\nc");
    assert_eq!(doc.blocks[0].source_lines, (0, 1));
    assert_eq!(
        paragraph(&doc, 0),
        &vec![text("a"), Inline::SoftBreak, text("b")]
    );
    assert_eq!(doc.blocks[1].source_lines, (3, 3));
}

#[test]
fn hard_break_and_inline_formatting() {
    let doc = PreviewDoc::parse("a  \nb **s** ~~x~~ `c`");
    assert_eq!(
        paragraph(&doc, 0),
        &vec![
            text("a"),
            Inline::HardBreak,
            text("b "),
            Inline::Strong(vec![text("s")]),
            text(" "),
            Inline::Strikethrough(vec![text("x")]),
            text(" "),
            Inline::Code("c".to_owned()),
        ]
    );
}

#[test]
fn safe_links_images_and_note_links() {
    let doc = PreviewDoc::parse("[a](https://x.y) ![pic](img/p.png) [n](bourbon://note/abc123)");
    assert_eq!(
        paragraph(&doc, 0),
        &vec![
            Inline::Link {
                href: "https://x.y".to_owned(),
                children: vec![text("a")]
            },
            text(" "),
            Inline::Image {
                src: "img/p.png".to_owned(),
                alt: "pic".to_owned()
            },
            text(" "),
            Inline::NoteLink {
                note_id: "abc123".to_owned(),
                children: vec![text("n")]
            },
        ]
    );
}

#[test]
fn unsafe_links_become_plain_text_and_images_become_alt() {
    let doc = PreviewDoc::parse("[**evil**](javascript:alert(1)) ![alt *x*](data:text/html,a)");
    assert_eq!(
        paragraph(&doc, 0),
        &vec![Inline::Strong(vec![text("evil")]), text(" "), text("alt x")]
    );
}

#[test]
fn inline_html_is_kept_as_text() {
    let doc = PreviewDoc::parse("a <b>x</b>");
    assert_eq!(paragraph(&doc, 0), &vec![text("a <b>x</b>")]);
}

#[test]
fn tight_ordered_list_with_start() {
    let doc = PreviewDoc::parse("3. a\n4. b\n");
    let BlockKind::List {
        ordered,
        start,
        tight,
        items,
    } = &doc.blocks[0].kind
    else {
        panic!()
    };
    assert!(*ordered && *tight);
    assert_eq!(*start, 3);
    assert_eq!(doc.blocks[0].source_lines, (0, 1));
    assert_eq!(items.len(), 2);
    assert_eq!(items[1].source_lines, (1, 1));
    assert_eq!(items[0].task, None);
    assert_eq!(items[0].children.len(), 1);
    assert_eq!(
        items[0].children[0].kind,
        BlockKind::Paragraph(vec![text("a")])
    );
    assert_eq!(items[0].children[0].source_lines, (0, 0));
}

#[test]
fn loose_bullet_list() {
    let doc = PreviewDoc::parse("- a\n\n- b\n");
    let BlockKind::List {
        ordered,
        start,
        tight,
        ..
    } = &doc.blocks[0].kind
    else {
        panic!()
    };
    assert!(!*ordered && !*tight);
    assert_eq!(*start, 1);
}

#[test]
fn task_items_and_nested_item_lines() {
    let doc = PreviewDoc::parse("- [ ] a\n- [x] b\n  - c\n");
    let BlockKind::List { items, .. } = &doc.blocks[0].kind else {
        panic!()
    };
    assert_eq!(items[0].task, Some(false));
    assert_eq!(items[1].task, Some(true));
    assert_eq!(items[1].source_lines, (1, 2));
    assert_eq!(items[1].children.len(), 2);
    assert_eq!(items[1].children[1].source_lines, (2, 2));
    assert!(matches!(items[1].children[1].kind, BlockKind::List { .. }));
}

#[test]
fn block_quote_and_alert() {
    let doc = PreviewDoc::parse("> quoted\n\n> [!WARNING]\n> careful\n");
    let BlockKind::BlockQuote(children) = &doc.blocks[0].kind else {
        panic!()
    };
    assert_eq!(children[0].kind, BlockKind::Paragraph(vec![text("quoted")]));
    assert_eq!(doc.blocks[1].source_lines, (2, 3));
    let BlockKind::Alert { kind, children } = &doc.blocks[1].kind else {
        panic!()
    };
    assert_eq!(*kind, AlertKind::Warning);
    assert_eq!(
        children[0].kind,
        BlockKind::Paragraph(vec![text("careful")])
    );
}

#[test]
fn fenced_and_indented_code_blocks() {
    let doc =
        PreviewDoc::parse("```rust title=x\nfn main() {}\n```\n\n    plain\n\n```\nbare\n```\n");
    assert_eq!(doc.blocks[0].source_lines, (0, 2));
    assert_eq!(
        doc.blocks[0].kind,
        BlockKind::CodeBlock {
            lang: Some("rust".to_owned()),
            meta: Some("title=x".to_owned()),
            text: "fn main() {}\n".to_owned(),
        }
    );
    assert_eq!(
        doc.blocks[1].kind,
        BlockKind::CodeBlock {
            lang: None,
            meta: None,
            text: "plain\n".to_owned()
        }
    );
    assert_eq!(
        doc.blocks[2].kind,
        BlockKind::CodeBlock {
            lang: None,
            meta: None,
            text: "bare\n".to_owned()
        }
    );
}

#[test]
fn table_with_alignments_head_and_rows() {
    let doc = PreviewDoc::parse("| a | b |\n|:--|--:|\n| 1 | **2** |\n");
    assert_eq!(doc.blocks[0].source_lines, (0, 2));
    assert_eq!(
        doc.blocks[0].kind,
        BlockKind::Table {
            alignments: vec![Alignment::Left, Alignment::Right],
            head: vec![vec![text("a")], vec![text("b")]],
            rows: vec![vec![vec![text("1")], vec![Inline::Strong(vec![text("2")])]]],
        }
    );
}

#[test]
fn thematic_break_html_block_and_frontmatter() {
    let doc = PreviewDoc::parse("---\ntitle: x\n---\n\na\n\n---\n\n<div>x</div>\n");
    assert_eq!(
        doc.blocks[0].kind,
        BlockKind::Frontmatter("title: x\n".to_owned())
    );
    assert_eq!(doc.blocks[0].source_lines, (0, 2));
    assert_eq!(doc.blocks[2].kind, BlockKind::ThematicBreak);
    assert_eq!(doc.blocks[2].source_lines, (6, 6));
    assert_eq!(
        doc.blocks[3].kind,
        BlockKind::Html("<div>x</div>".to_owned())
    );
    assert_eq!(doc.blocks[3].source_lines, (8, 8));
}

#[test]
fn math_blocks_and_inline_math() {
    let doc = PreviewDoc::parse("$$\nx^2\n$$\n\ninline $a+b$ here\n");
    assert_eq!(doc.blocks[0].kind, BlockKind::Math("\nx^2\n".to_owned()));
    assert_eq!(doc.blocks[0].source_lines, (0, 2));
    assert_eq!(
        paragraph(&doc, 1),
        &vec![
            text("inline "),
            Inline::InlineMath("a+b".to_owned()),
            text(" here")
        ]
    );
}

#[test]
fn a_lone_inline_math_stays_a_paragraph() {
    let doc = PreviewDoc::parse("$a+b$\n");
    assert_eq!(
        doc.blocks[0].kind,
        BlockKind::Paragraph(vec![Inline::InlineMath("a+b".to_owned())])
    );
}

#[test]
fn footnote_reference_and_definition() {
    let doc = PreviewDoc::parse("a[^1]\n\n[^1]: note\n");
    assert_eq!(
        paragraph(&doc, 0),
        &vec![text("a"), Inline::FootnoteRef("1".to_owned())]
    );
    let BlockKind::FootnoteDefinition { label, children } = &doc.blocks[1].kind else {
        panic!()
    };
    assert_eq!(label, "1");
    assert_eq!(children[0].kind, BlockKind::Paragraph(vec![text("note")]));
    assert_eq!(doc.blocks[1].source_lines, (2, 2));
}

#[test]
fn empty_source_has_no_blocks() {
    assert_eq!(PreviewDoc::parse("").blocks, Vec::new());
}

#[test]
fn fixture_note_structure() {
    let doc = PreviewDoc::parse(NOTE);
    let kinds: Vec<&str> = doc
        .blocks
        .iter()
        .map(|block| match &block.kind {
            BlockKind::Heading { level: 1, .. } => "h1",
            BlockKind::Heading { .. } => "h2",
            BlockKind::Paragraph(_) => "p",
            BlockKind::List { ordered: true, .. } => "ol",
            BlockKind::List { .. } => "ul",
            BlockKind::BlockQuote(_) => "quote",
            BlockKind::CodeBlock { .. } => "code",
            BlockKind::Table { .. } => "table",
            _ => "other",
        })
        .collect();
    assert_eq!(
        kinds,
        [
            "h1", "p", "h2", "ol", "h2", "ul", "h2", "quote", "code", "table", "ul"
        ]
    );
    let slugs: Vec<&str> = doc
        .blocks
        .iter()
        .filter_map(|block| match &block.kind {
            BlockKind::Heading { slug, .. } => Some(slug.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        slugs,
        [
            "app-hangs-when-pasting-a-5000-row-table",
            "repro",
            "environment",
            "what-the-profiler-says"
        ]
    );
    let lines: Vec<_> = doc.blocks.iter().map(|block| block.source_lines).collect();
    assert_eq!(
        lines,
        [
            (0, 0),
            (2, 2),
            (4, 4),
            (6, 8),
            (10, 10),
            (12, 14),
            (16, 16),
            (18, 18),
            (20, 24),
            (26, 29),
            (31, 32)
        ]
    );
    let BlockKind::Paragraph(inlines) = &doc.blocks[1].kind else {
        panic!()
    };
    assert!(inlines.contains(&Inline::NoteLink {
        note_id: "abcdefghij12".to_owned(),
        children: vec![text("the budget")]
    }));
    let BlockKind::List { items, .. } = &doc.blocks[10].kind else {
        panic!()
    };
    assert_eq!(
        items.iter().map(|item| item.task).collect::<Vec<_>>(),
        [Some(false), Some(true)]
    );
}
