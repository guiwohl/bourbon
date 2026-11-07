# bourbon-markdown

GPUI-free logic for the editor and preview ([decision](../decisions/2026-11-07-ui-rebuild.md), research [sections 1-6](../research/inkdrop-markdown-editor-preview.md)). Depends only on `pulldown-cmark`; `just architecture` fails if it pulls any `gpui*` crate. No dependency on `bourbon-core`: the `bourbon://note/<id>` scheme is parsed locally (`url::note_id`, non-empty ASCII alphanumeric id).

Parser options: tables, task lists, strikethrough, footnotes, math, GFM alerts, YAML metadata blocks. Heading attributes off.

## Public API

| Item | Role |
|---|---|
| `highlight(source) -> Vec<HighlightSpan>` | Editor tokens: `{ range: Range<usize>, class: TokenClass }`, byte ranges, sorted, disjoint, on char boundaries, inner construct wins |
| `TokenClass` | Heading(1-6), header/emphasis/strong/strikethrough/code/link marks, emphasis, strong, strikethrough, inline code, code block info/text, link text, URL, image, note link, list mark, task marker (checked), quote mark/text, alert marker, rule, table header/delimiter, frontmatter, math, HTML tag |
| `PreviewDoc::parse(source)` | `PreviewDoc { blocks }`; `Block { source_lines, kind }`; `ListItem { source_lines, task, children }` |
| `BlockKind` | Heading{level, slug, inlines}, Paragraph, List{ordered, start, tight, items}, BlockQuote, Alert{kind}, CodeBlock{lang, meta, text}, Table{alignments, head, rows}, ThematicBreak, Html, Math, Frontmatter, FootnoteDefinition |
| `Inline` | Text, Emphasis, Strong, Strikethrough, Code, Link, NoteLink, Image, SoftBreak, HardBreak, FootnoteRef, InlineMath |
| `task_lines(source) -> Vec<(line, checked)>` | Source line of every task marker, for preview checkboxes |
| `Anchor`, `preview_offset_for_line`, `line_for_preview_offset` | Scroll-sync math (research 4.6 / 6.6) |
| `SyncGuard`, `Pane`, `SyncState` | Echo-suppression state machine |

## Rules

- `source_lines` are 0-based inclusive, computed from byte offsets; trailing blank space of a block is excluded. Tight list items get an implicit `Paragraph` child with its own lines.
- Slugs are GitHub style (lowercase, spaces to `-`, punctuation dropped) and deduplicated with `-1`, `-2`; a suffix never collides with an existing slug.
- Safe URLs: schemes `http`, `https`, `mailto`, `bourbon` and scheme-less URLs. Whitespace and control characters are stripped before the scheme check. An unsafe link keeps only its children; an unsafe image becomes its alt text.
- `Frontmatter` is a block the renderer must skip. Block `Html` and inline HTML are raw text, never interpreted. Inline HTML is a `Text`.
- Marks pulldown-cmark does not emit (`#`, list markers, `>`, emphasis/code/strike delimiters, link brackets and parens, fences) are recovered by a scanner over the event ranges and painted last. The editor keeps marks visible.
- Highlighting is a per-byte canvas painted in event order, so inner constructs win and spans cannot overlap. Adjacent equal classes merge (`](` is one `LinkMark`).
- Fenced code: `CodeMark` fences, `CodeBlockInfo`, `CodeBlockText`. No language highlighting yet.

## Scroll sync

Anchors are blocks and list items (`line_start`, `line_end`, `top`, `height`), sorted by `top`.

- `line_for_preview_offset`: last anchor with `top <= offset` and the next one, linear interpolation of `line_start`; past the last anchor it uses `top + height` and `line_end + 1`; before the first it starts from line 0 at top 0.
- `preview_offset_for_line`: nearest anchor at or above `floor(line)` and the first below, interpolation of `top`, minus a 10 px lead, never negative.
- `SyncGuard::accept(pane, now)`: the first pane to scroll drives; the other pane's echo is rejected; the same pane keeps driving and refreshes the timer; the state returns to `None` 100 ms after the last accepted event. `disable()` (note switching) rejects everything until `enable()`.

## Tests

Unit tests per module with exact spans and trees, a fixture note ("App hangs when pasting a 5,000-row table") covering every construct, and `tests/properties.rs` (proptest): spans sorted, disjoint and on char boundaries for arbitrary input; parsing never panics and block lines stay in bounds; both scroll mappings are monotonic.
