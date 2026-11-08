# bourbon-editor

GPUI views of the Markdown editor and preview (Phase 1, view half, of [the UI rebuild](../decisions/2026-11-07-ui-rebuild.md); research [sections 2, 5, 6.4, 6.5](../research/inkdrop-markdown-editor-preview.md)). Depends only on `gpui-pre` (lib `gpui`), `bourbon-text` and `bourbon-markdown`; no pre-made input or editor component. The app is not migrated yet. Dev deps: `gpui` with `test-support`, `gpui-pre-platform` (demo window), `proptest`.

Run the demo: `cargo run -p bourbon-editor --example editor_demo` (app id `bourbon-editor-demo`; `ctrl-shift-l` light/dark, `ctrl-shift-n` line numbers). Editor and preview side by side on the research fixture note; the preview follows the editor live and its checkboxes toggle the editor text.

## Public API

| Item | Role |
|---|---|
| `Editor::new(text, theme, cx)`, `text()`, `set_text()` (resets history and scroll, no event), `set_theme()`, `set_line_numbers()` (off by default), `set_selection(anchor, head)`, `selected_range()`, `scroll_top()`, `toggle_task_at_line(line)` | The entity. `EventEmitter<EditorEvent>`, `Focusable`, `Render`, `EntityInputHandler` |
| `EditorEvent::Changed(String)`, `TaskToggled { line }` | `Changed` fires on every text change (typing, commands, undo, task toggles), not on `set_text` |
| `EditorTheme::light()/dark()` | Plain data: font family and fallbacks, 13 px, line height 1.5, heading scales 2/1.5/1.17/1/.83/.67, colours and `token_style(TokenClass)`. Phase 2 swaps the source for `bourbon-ui` tokens |
| `bind_editor_keys(cx)`, `KEY_CONTEXT` | Key bindings for context `BourbonEditor` |
| `PreviewView::new(source, theme, cx)`, `set_source()`, `set_theme()`, `block_layouts()` | Preview entity |
| `PreviewEvent::ToggleTask { source_line }`, `OpenLink(href)`, `OpenNote(id)` | Never mutates the document; the owner applies the toggle to the editor |
| `PreviewTheme::light()/dark()` | Preview typography and colours |
| `BlockLayout { source_lines, top, height }` | Top-level block geometry after layout, relative to the first block, for scroll sync |

## Editor design

- State is `bourbon_text::EditorState` plus `History`. Every key maps to a `bourbon_text` command (`run_command`); movement uses `bourbon_text::movement` through `motion::move_range`. The element never edits text.
- Highlight spans are recomputed with `bourbon_markdown::highlight` after each change. A line is split into `Segment`s (class, marked flag) by `segments::line_segments`; the segments become `TextRun`s (colour, weight, italic, underline, strike, inline-code background). Heading lines use the heading scale for font size and line height.
- `LineLayout` shapes one logical line: an optional prefix (`segments::hanging_prefix_len`: indentation, quote marks, list marker, task box) as a `ShapedLine`, and the rest as a GPUI `WrappedLine` with width `wrap - prefix width`, painted at the prefix width, so wrapped list and quote lines hang. Row ranges, caret position, hit testing and selection rectangles are computed per row.
- Per-line heights live in `LineHeights` (prefix sums, measured flag, `replace_lines` and `apply_changes(before, changeset)` invalidate only edited lines, `reset` on width or theme change). Unmeasured lines are measured progressively (400 per frame, then another frame is requested), visible lines are always measured. Only visible lines are painted.
- The frame (`Editor::build_frame`, called from `EditorElement::prepaint`) clamps scroll, keeps the caret in view when a movement or edit asked for it (up to 3 passes), and returns the lines with origins, selection quads, caret quad and gutter numbers. Paint order: active line, gutter border, selection, text, numbers, caret (2 px, 1 px radius, blink 530 ms restarted on every edit or move).
- Gutter (opt-in): numbers at 0.7 of the font size, width max(3em, digits + 1em) in that size, 1 px border.
- Mouse: click, shift-click, drag (clamped to the viewport), double-click word, triple-click line (with its break). A click on a `TaskMarker` span toggles that line (`task_click::toggle_task_at`, selection mapped through the change) and emits `TaskToggled`, without moving the caret. The wheel scrolls vertically.
- Vertical movement keeps a goal x across wrapped rows and lines (`motion::walk_rows`); up on the first row goes to the start, down on the last row to the end.
- IME: `EntityInputHandler` with UTF-16 conversions in `utf16` (pure, proptested). `replace_and_mark_text_in_range` inserts through `insert_text` and tracks the marked byte range, drawn underlined. A single typed character goes through `type_character` (auto-pair).
- Keys: arrows (+shift, ctrl word), home/end, ctrl-home/end, pageup/down, backspace/delete (+ctrl word), enter (list continuation, plain newline fallback), tab / shift-tab (list indent / outdent, two spaces fallback), ctrl-z, ctrl-shift-z, ctrl-y, ctrl-a, ctrl-c/x/v (plain text, paste is its own undo step), ctrl-b, ctrl-i, ctrl-shift-x, ctrl-`, ctrl-enter.

## Preview design

Blocks become `div` trees; inlines are flattened (`inline::flatten_inlines`: text, per-byte flags, link ranges) into a `StyledText` with highlights, wrapped in `InteractiveText` when it holds links. Typography: 16 px, line height 1.6, headings 2/1.5/1.25/1/.875/.85 em at weight 600 with line height 1.25 (h1 2 px and h2 1 px bottom rule, h6 muted), 16 px block gap, 24 px before headings and around rules (`gap_between`), lists with 2em indent and `•`/`◦`/`▪` or `N.` markers, task checkboxes (clickable, checked items struck and muted), quotes with a 0.25em bar, alerts with the kind colour and title, code blocks with 16 px padding, 6 px radius, border and background (no syntax colours), tables with equal-width cells, 6x13 padding, bordered, header and striped backgrounds, links in the link colour. Padding is 10 px. The editor line inset is 16 px. Font is Inter when registered, otherwise the fallback list.

## Known gaps

- Editor: no code-block card, quote bar, folding, search, completion, multi-cursor, horizontal scroll, drag auto-scroll, Home/End by visual row, mouse selection by words after a double-click drag.
- Preview: inline code uses a background only (highlights cannot switch the font family), tables use equal column widths, no images, math or footnotes layout, code blocks do not scroll horizontally, list markers of nested ordered lists are numbers, no virtualisation, no scroll sync wiring (only the block geometry).
- Hit testing and geometry tests run on GPUI's test text system (fixed 0.6 em advance), not on real fonts.

## Tests

Unit tests per module (utf16 with proptest, line heights, segments, motion, task click, theme, inline flattening, preview gaps and layouts). Integration tests in `tests/editor.rs` and `tests/preview.rs` use `TestAppContext` and `VisualTestContext`: typing, auto-pair, IME compose and commit, UTF-16 reporting, arrows, selection, word and line motion, vertical motion, wrapping, deletes, enter, tab, undo and redo, clipboard, formatting shortcuts, events, mouse (click, shift, drag, double and triple), task marker click, wheel and caret scrolling, preview checkboxes and links, block geometry.
