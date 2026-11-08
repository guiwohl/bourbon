# 2026-11-08 — Editor folding and parity row metrics

Follows [2026-11-07-ui-rebuild.md](2026-11-07-ui-rebuild.md). Values from [editor-preview-parity.md](../research/editor-preview-parity.md) and research [section 2](../research/inkdrop-markdown-editor-preview.md).

| # | Topic | Decision | Why |
|---|---|---|---|
| 1 | Fold ranges | `bourbon_markdown::fold_ranges`: a heading folds until the next heading of the same or a higher level (trailing blank lines stay visible); a fenced block folds from fence to fence. Indented code and tables do not fold | Matches the chevrons of the reference (headings, fences); pure, no GPUI |
| 2 | Fold state | `bourbon_text::Folds`: hidden byte ranges `header line end..last line end`, view state only (text untouched). Mapped through every `ChangeSet`; an edit touching a hidden range drops that fold. `set_text` clears them | Folds survive typing elsewhere; editing hidden text would be invisible |
| 3 | Caret | The head never rests in hidden text: moving forward lands after the fold, backward (or at the document end) on the header line end. Vertical motion counts hidden lines as zero rows | "The caret cannot enter folded text" |
| 3b | Search | Selecting a match inside a fold (`set_selection`) unfolds it to reveal the match; matches in hidden lines are not painted | CodeMirror/InkDrop reveal search results; requested by the coordinator after #43 |
| 4 | Gutter | Chevron (stroked path, 0.62 em, gutter text colour) right of the numbers, down when open, right when folded; click toggles. Only with line numbers on | Reference column right of the numbers |
| 5 | Placeholder | `…` in an inverted pill (fold colour slate-400 / neutral-400, label in the editor background, 0.2 em radius) after the header text | Research: InkDrop fold placeholder |
| 6 | Keys | `ctrl-alt-[` folds the range at the caret (the one starting on the line, else the innermost enclosing), `ctrl-alt-]` unfolds the fold of the caret line | No fold keys in the InkDrop keymap; picked here |
| 7 | Wrapped rows | No hanging indent: wrapped list and quote rows start at the left text edge | The reference starts them at column 0 |
| 8 | Inline code rows | Rows holding inline code are `1.4 em + 2 px` tall (22.3 px), the code box is 1.4 em, centred | Reference 23 px rows, `calc(140% + 2px)` |
| 9 | Preview | Inline code in the mono family (`with_font_family_overrides`); tables are a max-content grid inside a flex row, so columns take their content width | Reference |
