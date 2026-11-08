use super::highlight;
use crate::fixture::NOTE;
use crate::token::TokenClass::{self, *};

fn spans(source: &str) -> Vec<(&str, TokenClass)> {
    highlight(source)
        .into_iter()
        .map(|span| (&source[span.range], span.class))
        .collect()
}

#[test]
fn atx_heading_marks_and_text() {
    assert_eq!(
        spans("# Title\n"),
        [("#", HeaderMark), (" Title", Heading(1))]
    );
    assert_eq!(
        spans("###### x"),
        [("######", HeaderMark), (" x", Heading(6))]
    );
}

#[test]
fn heading_with_inline_code() {
    assert_eq!(
        spans("## A `b` c"),
        [
            ("##", HeaderMark),
            (" A ", Heading(2)),
            ("`", CodeMark),
            ("b", InlineCode),
            ("`", CodeMark),
            (" c", Heading(2)),
        ]
    );
}

#[test]
fn setext_heading_has_no_header_mark() {
    assert_eq!(spans("Title\n=====\n"), [("Title\n=====", Heading(1))]);
}

#[test]
fn emphasis_strong_and_strikethrough_with_marks() {
    assert_eq!(
        spans("*a* __b__ ~~c~~ ~d~"),
        [
            ("*", EmphasisMark),
            ("a", Emphasis),
            ("*", EmphasisMark),
            ("__", StrongMark),
            ("b", Strong),
            ("__", StrongMark),
            ("~~", StrikethroughMark),
            ("c", Strikethrough),
            ("~~", StrikethroughMark),
            ("~", StrikethroughMark),
            ("d", Strikethrough),
            ("~", StrikethroughMark),
        ]
    );
}

#[test]
fn nested_strong_inside_emphasis_wins_inside() {
    assert_eq!(
        spans("*a **b** c*"),
        [
            ("*", EmphasisMark),
            ("a ", Emphasis),
            ("**", StrongMark),
            ("b", Strong),
            ("**", StrongMark),
            (" c", Emphasis),
            ("*", EmphasisMark),
        ]
    );
}

#[test]
fn inline_code_with_long_delimiters() {
    assert_eq!(
        spans("``a`b``"),
        [("``", CodeMark), ("a`b", InlineCode), ("``", CodeMark)]
    );
}

#[test]
fn bullet_and_ordered_list_marks() {
    assert_eq!(spans("- a\n- b"), [("-", ListMark), ("-", ListMark)]);
    assert_eq!(spans("1. a\n2. b"), [("1.", ListMark), ("2.", ListMark)]);
    assert_eq!(spans("10) a"), [("10)", ListMark)]);
    assert_eq!(spans("* a\n\n+ b"), [("*", ListMark), ("+", ListMark)]);
}

#[test]
fn nested_list_marks_skip_indentation() {
    assert_eq!(spans("- a\n  - b"), [("-", ListMark), ("-", ListMark)]);
}

#[test]
fn task_markers() {
    assert_eq!(
        spans("- [ ] a\n- [x] b"),
        [
            ("-", ListMark),
            ("[ ]", TaskMarker { checked: false }),
            ("-", ListMark),
            ("[x]", TaskMarker { checked: true }),
        ]
    );
}

#[test]
fn quote_marks_and_text() {
    assert_eq!(
        spans("> a\n> b"),
        [
            (">", QuoteMark),
            (" a\n", QuoteText),
            (">", QuoteMark),
            (" b", QuoteText)
        ]
    );
}

#[test]
fn nested_quote_marks() {
    assert_eq!(
        spans("> > x"),
        [
            (">", QuoteMark),
            (" ", QuoteText),
            (">", QuoteMark),
            (" x", QuoteText)
        ]
    );
}

#[test]
fn alert_marker() {
    assert_eq!(
        spans("> [!NOTE]\n> hi"),
        [
            (">", QuoteMark),
            (" ", QuoteText),
            ("[!NOTE]", AlertMarker),
            ("\n", QuoteText),
            (">", QuoteMark),
            (" hi", QuoteText),
        ]
    );
}

#[test]
fn horizontal_rules() {
    assert_eq!(
        spans("a\n\n---\n\n***\n"),
        [("---", HorizontalRule), ("***", HorizontalRule)]
    );
}

#[test]
fn fenced_code_info_and_text() {
    assert_eq!(
        spans("```rust\nfn x() {}\n```\n"),
        [
            ("```", CodeMark),
            ("rust", CodeBlockInfo),
            ("fn x() {}", CodeBlockText),
            ("```", CodeMark),
        ]
    );
}

#[test]
fn fenced_code_without_info_or_closing() {
    assert_eq!(spans("~~~\nx\n"), [("~~~", CodeMark), ("x", CodeBlockText)]);
}

#[test]
fn code_fence_content_that_looks_like_markdown_stays_code() {
    assert_eq!(
        spans("```\n# not a heading\n- not a list\n```"),
        [
            ("```", CodeMark),
            ("# not a heading\n- not a list", CodeBlockText),
            ("```", CodeMark),
        ]
    );
}

#[test]
fn fence_info_is_trimmed_on_both_sides() {
    assert_eq!(
        spans("```   rust  \nx\n```"),
        [
            ("```", CodeMark),
            ("rust", CodeBlockInfo),
            ("x", CodeBlockText),
            ("```", CodeMark),
        ]
    );
}

#[test]
fn last_line_that_is_not_a_fence_is_code_text() {
    assert_eq!(
        spans("```\nx\n`\n"),
        [("```", CodeMark), ("x\n`", CodeBlockText)]
    );
    assert_eq!(
        spans("```\nx\n``` y\n"),
        [("```", CodeMark), ("x\n``` y", CodeBlockText)]
    );
}

#[test]
fn text_after_a_code_block_is_not_code() {
    assert_eq!(
        spans("```\nx\n```\n\nafter"),
        [("```", CodeMark), ("x", CodeBlockText), ("```", CodeMark)]
    );
}

#[test]
fn link_destination_whitespace_is_excluded_from_the_url() {
    assert_eq!(
        spans("[a](  u  )"),
        [
            ("[", LinkMark),
            ("a", LinkText),
            ("](", LinkMark),
            ("  ", LinkText),
            ("u", Url),
            ("  ", LinkText),
            (")", LinkMark),
        ]
    );
}

#[test]
fn indented_fence_inside_a_list_item() {
    assert_eq!(
        spans("- a\n\n  ```\n  x\n  ```\n"),
        [
            ("-", ListMark),
            ("```", CodeMark),
            ("x", CodeBlockText),
            ("```", CodeMark),
        ]
    );
}

#[test]
fn indented_code_text() {
    assert_eq!(spans("a\n\n    code\n"), [("code", CodeBlockText)]);
}

#[test]
fn inline_link_parts() {
    assert_eq!(
        spans("[text](https://x.y)"),
        [
            ("[", LinkMark),
            ("text", LinkText),
            ("](", LinkMark),
            ("https://x.y", Url),
            (")", LinkMark),
        ]
    );
}

#[test]
fn note_link_url_is_classified() {
    assert_eq!(
        spans("[Groceries](bourbon://note/abcdefghij12)"),
        [
            ("[", LinkMark),
            ("Groceries", LinkText),
            ("](", LinkMark),
            ("bourbon://note/abcdefghij12", NoteLink),
            (")", LinkMark),
        ]
    );
}

#[test]
fn link_with_formatted_text_and_title() {
    assert_eq!(
        spans("[**b**](u \"t\")"),
        [
            ("[", LinkMark),
            ("**", StrongMark),
            ("b", Strong),
            ("**", StrongMark),
            ("](", LinkMark),
            ("u \"t\"", Url),
            (")", LinkMark),
        ]
    );
}

#[test]
fn image_parts() {
    assert_eq!(
        spans("![alt](p.png)"),
        [
            ("![", LinkMark),
            ("alt", Image),
            ("](", LinkMark),
            ("p.png", Url),
            (")", LinkMark),
        ]
    );
}

#[test]
fn reference_link_and_autolink() {
    assert_eq!(
        spans("[a][b]\n\n[b]: http://x\n"),
        [("[", LinkMark), ("a", LinkText), ("][b]", LinkMark)]
    );
    assert_eq!(spans("<http://x.y>"), [("<http://x.y>", Url)]);
}

#[test]
fn table_header_and_delimiter() {
    assert_eq!(
        spans("| a | b |\n|:--|--:|\n| 1 | 2 |"),
        [("| a | b |", TableHeader), ("|:--|--:|", TableDelimiter)]
    );
}

#[test]
fn frontmatter_block() {
    assert_eq!(
        spans("---\ntitle: x\n---\n\n# H"),
        [
            ("---\ntitle: x\n---", Frontmatter),
            ("#", HeaderMark),
            (" H", Heading(1)),
        ]
    );
}

#[test]
fn math_and_html() {
    assert_eq!(
        spans("$a$ and $$b$$"),
        [
            ("$", MathMark),
            ("a", Math),
            ("$", MathMark),
            ("$$", MathMark),
            ("b", Math),
            ("$$", MathMark),
        ]
    );
    assert_eq!(spans("a <b>x</b>"), [("<b>", HtmlTag), ("</b>", HtmlTag)]);
    assert_eq!(spans("<div>x</div>\n"), [("<div>x</div>", HtmlTag)]);
}

#[test]
fn math_commands_are_marked_inside_the_formula() {
    assert_eq!(
        spans(r"$O(n \times m)$"),
        [
            ("$", MathMark),
            ("O(n ", Math),
            (r"\times", MathMark),
            (" m)", Math),
            ("$", MathMark),
        ]
    );
    assert_eq!(
        spans(r"$a\\b$"),
        [
            ("$", MathMark),
            ("a", Math),
            (r"\\", MathMark),
            ("b", Math),
            ("$", MathMark),
        ]
    );
    assert_eq!(
        spans(r"$a\é$"),
        [("$", MathMark), ("a", Math), (r"\é$", MathMark)]
    );
}

#[test]
fn empty_display_math_is_all_marks() {
    assert_eq!(spans("a $$$$ b"), [("$$$$", MathMark)]);
}

#[test]
fn multibyte_text_keeps_char_boundaries() {
    assert_eq!(
        spans("# héllo ✓ `ç`"),
        [
            ("#", HeaderMark),
            (" héllo ✓ ", Heading(1)),
            ("`", CodeMark),
            ("ç", InlineCode),
            ("`", CodeMark),
        ]
    );
}

#[test]
fn empty_and_plain_text_have_no_spans() {
    assert_eq!(highlight(""), Vec::new());
    assert_eq!(highlight("just words\n"), Vec::new());
}

#[test]
fn fixture_note_highlights_every_construct() {
    let found = spans(NOTE);
    let has = |text: &str, class: TokenClass| found.contains(&(text, class));
    assert!(has("#", HeaderMark));
    assert!(has(" Repro", Heading(2)));
    assert!(has("1.", ListMark));
    assert!(has("**", StrongMark));
    assert!(has("new", Strong));
    assert!(has("Ctrl+V", InlineCode));
    assert!(has("~~", StrikethroughMark));
    assert!(has("bourbon://note/abcdefghij12", NoteLink));
    assert!(has("the budget", LinkText));
    assert!(has(">", QuoteMark));
    assert!(has("rust", CodeBlockInfo));
    assert!(has("|:-----|-----:|", TableDelimiter));
    assert!(has("[ ]", TaskMarker { checked: false }));
    assert!(has("[x]", TaskMarker { checked: true }));
}
