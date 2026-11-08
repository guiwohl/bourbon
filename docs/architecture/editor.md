# bourbon-editor

GPUI views of the Markdown editor, the preview and the single-line text field (Phase 1, view half, of [the UI rebuild](../decisions/2026-11-07-ui-rebuild.md); research [sections 2, 5, 6.4, 6.5](../research/inkdrop-markdown-editor-preview.md)). Depends only on `gpui-pre` (lib `gpui`), `bourbon-text`, `bourbon-markdown` and `bourbon-ui` (tokens and icons for the text field); no pre-made input or editor component. The app uses the text field for the note-list filter and the sidebar name prompt; and `bourbon-app` renders its editor column with it (see [app](app.md#editor-column)). Dev deps: `gpui` with `test-support`, `gpui-pre-platform` (demo window), `proptest`.

Run the demo: `cargo run -p bourbon-editor --example editor_demo` (app id `bourbon-editor-demo`; `ctrl-shift-l` light/dark, `ctrl-shift-n` line numbers, on at start). Editor and preview side by side on the reference note of the [parity check](../research/editor-preview-parity.md); the preview follows the editor live and its checkboxes toggle the editor text. `BOURBON_DEMO_WINDOW_SIZE=WxH` asks for that window size and lays the content out in a WxH canvas at the top-left, so screenshots match the reference even when the compositor tiles the window.

## Public API

| Item | Role |
|---|---|
| `Editor::new(text, theme, cx)`, `text()`, `set_text()` (resets history and scroll, no `Changed`), `set_theme()`, `set_line_numbers()` (off by default), `set_selection(anchor, head)`, `selected_range()`, `replace_range(range, text)` (one undo step, caret after the text), `scroll_top()`, `top_line()` (fractional line at the top), `scroll_to_line(line)` (clamped), `toggle_task_at_line(line)` | The entity. `EventEmitter<EditorEvent>`, `Focusable`, `Render`, `EntityInputHandler` |
| `EditorEvent::Changed(String)`, `TaskToggled { line }`, `Scrolled` | `Changed` fires on every text change (typing, commands, undo, task toggles), not on `set_text`. `Scrolled` fires from prepaint whenever `scroll_top` differs from the last reported value (wheel, caret, `scroll_to_line`, `set_text`) |
| `EditorTheme::light()/dark()` | Plain data: font family and fallbacks, 14.5 px, line height 1.16, heading scales 2/1.5/1.17/1/.83/.67, line padding 8, block padding 10, card inset 3 and end padding 4, quote bar 0.3 em, colours and `token_style(TokenClass)` (measured, see [parity](../research/editor-preview-parity.md)) |
| `bind_editor_keys(cx)`, `KEY_CONTEXT` | Key bindings for context `BourbonEditor` |
| `TextField::new(cx)`, `.placeholder(text)`, `.leading_icon(IconName)`, `.borderless(Borderless { font_size, height, inset })`, `text()`, `set_text()` (drops line breaks, caret at the end, resets history, no event), `selected_range()`, `focus(window, cx)` | Single-line field. `EventEmitter<TextFieldEvent>`, `Focusable`, `Render`, `EntityInputHandler` |
| `TextFieldEvent::Changed(String)`, `Submit`, `Cancel` | `Changed` on every text change; Enter emits `Submit`, Esc emits `Cancel`; neither edits the text |
| `bind_text_field_keys(cx)`, `TEXT_FIELD_CONTEXT` | Key bindings for context `BourbonTextField` |
| `TextField::select_all(cx)` | Selects the whole text (the search panel uses it on open) |
| `SearchPanel::new(editor, window, cx)`, `open(replacing)`, `close()`, `refresh()`, `find_next()`, `find_previous()`, `replace_current()`, `replace_all()`, `toggle_match_case/regex/whole_word()`, `set_options()`, `matches()`, `count_label()`, `is_open()`, `is_replacing()`, `find_field()`, `replace_field()` | Find / replace panel bound to one `Editor`. Renders nothing while closed |
| `SearchOptions`, `match_count_label(current, total, valid)` | Option flags; label `n of m`, `No results`, `Invalid pattern` |
| `bind_search_keys(cx)`, `SEARCH_CONTEXT`, `FindNext`, `FindPrevious`, `ToggleMatchCase`, `ToggleRegex`, `ToggleWholeWord`, `ReplaceAll` | Keys for context `BourbonSearch` |
| `PreviewView::new(source, theme, cx)`, `set_source()`, `set_theme()`, `doc()`, `block_layouts()`, `scroll_top()`, `set_scroll_top(px)`, `scroll_anchors()` | Preview entity; scrolling goes through its own `ScrollHandle` |
| `PreviewEvent::ToggleTask { source_line }`, `OpenLink(href)`, `OpenNote(id)`, `Scrolled` | Never mutates the document; the owner applies the toggle to the editor. `Scrolled` fires from prepaint when the scroll offset changed |
| `scroll_anchors(&[BlockLayout])` | Top-level blocks as `bourbon_markdown::Anchor`s, tops shifted by the 10 px padding so they are in scroll-content coordinates |
| `PreviewTheme::light()/dark()` | Preview typography and colours |
| `BlockLayout { source_lines, top, height }` | Top-level block geometry after layout, relative to the first block, for scroll sync |

## Editor design

- State is `bourbon_text::EditorState` plus `History`. Every key maps to a `bourbon_text` command (`run_command`); movement uses `bourbon_text::movement` through `motion::move_range`. The element never edits text.
- Highlight spans are recomputed with `bourbon_markdown::highlight` after each change. A line is split into `Segment`s (class, marked flag) by `segments::line_segments`; the segments become `TextRun`s (colour, weight, italic, underline, strike) in JetBrains Mono with `calt` and `liga` off (`fonts::without_ligatures`). Heading lines use the heading scale for font size and line height.
- Block decorations come from `bourbon_markdown::line_decorations` (recomputed with the spans). `line_decor::decor_of(line)` says whether a line is in a card (code block or table; first and last flags) or under a quote bar. Card lines get a `LineBox` (inset `block_padding - line_padding`, 4 px top padding on the first line, 4 px bottom padding on the last); `LineLayout` applies it to every position, hit test, selection rectangle and the painted text, and the wrap width shrinks by twice the inset. The frame turns decorations into a `CardShape` per line (inset 3 px from the line box, 1 px side borders, top or bottom border and 0.4 em radius only at the ends) and a 0.3 em quote bar at the line box edge. Inline code (`segments::inline_code_ranges`: code plus its backticks) gets a box per row, 1 px wider on each side, in the card colours.
- `LineLayout` shapes one logical line: an optional prefix (`segments::hanging_prefix_len`: indentation, quote marks, list marker, task box) as a `ShapedLine`, and the rest as a GPUI `WrappedLine` with width `wrap - prefix width`, painted at the prefix width, so wrapped list and quote lines hang. Row ranges, caret position, hit testing and selection rectangles are computed per row.
- Per-line heights live in `LineHeights` (prefix sums, measured flag, `line_position(y)` / `offset_of_line(line)` for fractional lines, `replace_lines` and `apply_changes(before, changeset)` invalidate only edited lines, `reset` on width or theme change). Unmeasured lines are measured progressively (400 per frame, then another frame is requested), visible lines are always measured. Only visible lines are painted.
- The frame (`Editor::build_frame`, called from `EditorElement::prepaint`) clamps scroll, keeps the caret in view when a movement or edit asked for it (up to 3 passes), and returns the lines with origins, selection quads, caret quad and gutter numbers. Paint order: active line, cards, quote bars, inline code boxes, gutter border, selection, text, numbers (on the text row of padded lines), caret (2 px, 1 px radius, blink 530 ms restarted on every edit or move).
- Gutter (opt-in): numbers at 0.7 of the font size, width max(4.5em, digits + 2.6em) in that size, 1 px border.
- Mouse: click, shift-click, drag (clamped to the viewport), double-click word, triple-click line (with its break). A click on a `TaskMarker` span toggles that line (`task_click::toggle_task_at`, selection mapped through the change) and emits `TaskToggled`, without moving the caret. The wheel scrolls vertically.
- Vertical movement keeps a goal x across wrapped rows and lines (`motion::walk_rows`); up on the first row goes to the start, down on the last row to the end.
- IME: `EntityInputHandler` with UTF-16 conversions in `utf16` (pure, proptested). `replace_and_mark_text_in_range` inserts through `insert_text` and tracks the marked byte range, drawn underlined. A single typed character goes through `type_character` (auto-pair).
- Keys: arrows (+shift, ctrl word), home/end, ctrl-home/end, pageup/down, backspace/delete (+ctrl word), enter (list continuation, plain newline fallback), tab / shift-tab (list indent / outdent, two spaces fallback), ctrl-z, ctrl-shift-z, ctrl-y, ctrl-a, ctrl-c/x/v (plain text, paste is its own undo step), ctrl-b, ctrl-i, ctrl-shift-x, ctrl-`, ctrl-enter.

## Search panel

Files: `src/search.rs`, `src/search/{panel,view,keys,highlights}.rs`. Matching is `bourbon_text::Search` (no GPUI).

- `open(replacing)`: a single-line editor selection prefills the find field; the field text is selected and focused. Every query or option change recomputes the matches and selects the first one at or after the selection start (the editor scrolls to it through its usual caret reveal).
- Find field: Enter next, Shift+Enter previous, Esc closes. Replace field: Enter replaces the current match and moves on (no current match: it only selects the next one), Esc closes. In the panel: `Ctrl+G` / `Ctrl+Shift+G`, `Alt+C` case, `Alt+R` regex, `Alt+W` whole word, `Ctrl+Enter` replace all. Close clears the highlights and focuses the editor.
- Replace and Replace all apply one `bourbon_text` transaction each (one undo step). The panel listens to `EditorEvent::Changed` and recomputes the matches (typing, undo); the owner calls `refresh()` after `set_text`.
- Highlights: the editor keeps the match ranges (`search_matches`); `build_frame` adds `MatchRect`s for visible lines (selection-rect geometry), painted under the selection: amber 25 %, the match equal to the selection amber 50 % with a 1 px amber outline, radius 0.4 em (research 2.2).
- Look (`bourbon-ui` tokens): padding 8, gap 6, `section_header_background`, bottom `internal_border`; find row = field, `Aa` / `ab` / `.*` toggles (accessory size, `primary` text on `button_active_background` when on), a 140 px column with the count (min 5 em, `muted_text`) and up / down / close icon buttons; replace row = field plus Replace and All buttons in the same 140 px column.
- Tests: `tests/search.rs` (keys, options, prefill, replace, undo, refresh), unit tests in `search/highlights.rs` (rects, current match, multi-line).

## Text field

Run the demo: `cargo run -p bourbon-editor --example text_field_demo` (app id `bourbon-text-field-demo`): filter and rename fields, light (left) and dark (right).

- Same core as the editor: `bourbon_text::EditorState` plus `History`; edits go through `bourbon_text` commands (`insert_text`, `delete_backward`, `delete_forward`, `delete_word_backward`, shared `delete_next_word` and `paste_text`), moves through `motion::move_range`. No auto-pair, no Markdown.
- Single line: `single_line` drops `\n` and `\r` from `set_text`, paste and IME input (like an HTML input); Enter and Esc are events only.
- Element (`text_field/element.rs`): one `ShapedLine` in the UI font (`tokens.fonts.ui_family`, Inter) at `scale.medium` (14 px), `scale.line_height` (20 px) tall; placeholder in `input_placeholder` when empty; text in `input_text`; marked IME text underlined; selection in `focus_outline` (only while focused); caret 1 px in `input_text`, blinking 530 ms (`blink::schedule_blink`, shared with the editor). Horizontal scroll keeps the caret visible (`geometry::revealing_scroll`, pure).
- Box (`text_field/view.rs`): 30 px tall, padding 0 8, gap 8, 7 px radius, 1 px `internal_border`; focused: the border turns transparent and a 2 px `focus_outline` ring is drawn just outside it (inset -3 px). Optional leading icon 16 px at 30% opacity in `text`. No background (the parent's shows through).
- Options: `masked(true)` shows one `•` per character (`text_field/mask.rs` maps byte offsets to bullet offsets and back, so caret, selection, clicks and IME stay on the real text; used by the backup token); `borderless()` drops the box (no height, padding, border or ring; used by the control pane); `text_size(px)` sets the font size, the line height keeps the 20/14 ratio (control pane: 18 px).
- Keys (`BourbonTextField`): arrows, `shift` and `ctrl` variants, home/end (+shift), backspace/delete (+ctrl word), enter, escape, ctrl-z / ctrl-shift-z / ctrl-y, ctrl-a, ctrl-c/x/v. Mouse: click, shift-click, drag, double-click word, triple-click all.
- Tests: `tests/text_field.rs` (typing, events, keys, clipboard, IME and UTF-16, mouse, icon offset, scroll, masked, borderless, text size), unit tests for `mask`, `single_line`, blink, scroll and the border/ring.

## Preview design

Blocks become `div` trees; inlines are flattened (`inline::flatten_inlines`: text, per-byte flags, link ranges) into a `StyledText` with highlights, wrapped in `InteractiveText` when it holds links. Typography: 15 px, line height 1.55, headings 2/1.5/1.25/1/.875/.85 of a 16 px heading base at weight 600 with line height 1.25 (h1 2 px and h2 1 px bottom rule, h6 muted), 16 px block gap, 24 px before headings and around rules (`gap_between`), lists with 2em indent, 0.3em item gap and `•`/`◦`/`▪` or `N.` markers, task checkboxes (clickable, checked items struck and muted), quotes with a 0.25em bar, alerts with the kind colour and title, code blocks in JetBrains Mono 14 px (ligatures off), line height 1.45, wrapped, with 16 px padding, 6 px radius, border and background (no syntax colours), tables with equal-width cells, 6x13 padding, bordered, header and striped backgrounds, links in the link colour. Padding is 10 px. The editor line inset is 8 px. Font is Inter when registered, otherwise the fallback list.

## Known gaps

- Editor: no folding (no fold chevrons), rows with inline code are not taller, completion, multi-cursor, horizontal scroll, drag auto-scroll, Home/End by visual row, mouse selection by words after a double-click drag.
- Preview: inline code uses a background only (highlights cannot switch the font family), tables use equal column widths, no images, math or footnotes layout, code blocks wrap instead of scrolling, list markers of nested ordered lists are numbers, no virtualisation; scroll anchors are top-level blocks only (no list items).
- Hit testing and geometry tests run on GPUI's test text system (fixed 0.6 em advance), not on real fonts.

## Tests

Unit tests per module (utf16 with proptest, line heights, segments, motion, task click, theme, inline flattening, preview gaps and layouts). Integration tests in `tests/editor.rs` and `tests/preview.rs` use `TestAppContext` and `VisualTestContext`: typing, auto-pair, IME compose and commit, UTF-16 reporting, arrows, selection, word and line motion, vertical motion, wrapping, deletes, enter, tab, undo and redo, clipboard, formatting shortcuts, events, mouse (click, shift, drag, double and triple), task marker click, wheel and caret scrolling, preview checkboxes and links, block geometry.
