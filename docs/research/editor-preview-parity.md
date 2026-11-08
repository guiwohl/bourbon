# Editor and preview parity with the InkDrop reference

Pixel measurements of the reference screenshot (InkDrop 6 web demo, editor column crop 838x798, editor pane 419 px, preview pane 419 px, content from y = 113) and the values Bourbon uses. Clean room: only the picture was measured, no InkDrop code, CSS or assets.

## How it was measured

Grey-scale row and column ink profiles of the crop (count of pixels darker than 150 per row or column), glyph start columns for advances, baselines for line pitch, darkest pixel of a region for colours.

## Editor

| Metric | Reference | Bourbon |
|---|---|---|
| Glyph advance | 8.68 px (`a` of "Pasting" to `a` of "table": 217 px over 25 glyphs) | JetBrains Mono 14.5 px, advance 8.7 |
| Line pitch | 361 px from line 1 to line 17 = 17 rows + 3 h2 rows (1.5x) = 21.5 rows, 16.79 px | line height 1.16, 16.82 px |
| h2 | advance 12.94 px, row 25 px | scale 1.5 (21.75 px), row 25.2 px |
| Gutter | border at x = 46 (slate-200), numbers right edge at about x = 20, slate-300 | min width 4.5 em of the 0.7 em number size (47 px), number padding 2.6 em |
| Text inset | x = 55, 8 px after the gutter | `line_padding` 8 px |
| Code card | x 50 to 415 (3 px inset), 1 px slate-300 at 50% border, rounded ends, text at +2 px, first and last line 4 px taller | `block_padding` 10, `card_inset` 3, `card_padding_y` 4, radius 0.4 em, `code_background` slate-200 at 50%, `code_border` slate-300 at 50% |
| Inline code | box with the card colours around code and backticks; rows holding it 23 px, box about 20 px | same colours, 1 px outset; row `1.4 em + 2 px` (22.3 px), box 1.4 em (20.3 px), centred |
| Wrapped rows | list and quote continuation rows start at x = 55 (column 0) | no hanging indent, same x |
| Fold chevrons | x 30 to 39, 4 to 5 px tall, centred on the first row, numbers colour | stroked path 0.62 em (9 px) wide, centre 1.25 widths left of the border (x 34.75) |
| Ink | line 17 holds 139k grey levels of ink over a (235) background; Linux rendering of gray-800 gave 187k | foreground slate-600 (71,85,105): 158k. The darkest reference pixel is still gray-800, the thinner macOS stems make it read lighter |
| Card tint | card fill 5 levels and border 17 levels darker than the page | `code_background` slate-200 at 30%, `code_border` slate-300 at 40% |
| Colours | `##` and fences blue-600 (36,99,235), heading pink-600 (219,41,121), list marks indigo-500 (100,103,242), text gray-800 (31,41,55) | unchanged tokens |
| Math | `$` and `\times` blue-600, formula text gray-800 | `MathMark` (delimiters and `\commands`) in `mark`, `Math` in the foreground |
| Ligatures | `->`, `==` drawn as separate glyphs | `calt` and `liga` off on every editor and preview code font |

## Preview

| Metric | Reference | Bourbon |
|---|---|---|
| Body | cap height 12 px, x-height 9 px, pitch 23 to 24 px; a 54 character line is 381 px wide | Inter 15 px, line height 1.55 (23.25 px), same line breaks as the reference |
| h2 | cap height 17 px | 24 px (headings scale from a 16 px base, `heading_base_size`) |
| Code block | advance 8.41 px, pitch 20 px, text 17 px from the left border, lines wrap | JetBrains Mono 14 px, line height 1.45, padding 16, wrapped |
| Lists | marker right edge to text 6 px, items 28 px apart | 2 em indent, 0.3 em item gap (27.75 px) |
| Size check | body line 1 spans rows 11 to 25 (cap to descender), h2 "Repro" rows 82 to 103 | Inter 15: rows 16 to 29, h2 rows 87 to 108: same glyph heights, a 16 px body would wrap "freezes" onto the next line |
| Code card | border 14 levels darker than its fill, first text row 22 px below the top border | `code_border` slate-300 at 35%, padding 16 (also 22 px) |

## Known differences

- The reference window is translucent over a photo, so its background is (240,237,236); Bourbon paints white.
- macOS font rendering is lighter than Linux; the reference monospace is SF Mono, Bourbon bundles JetBrains Mono (its backticks sit higher, so a closing fence touches the descenders of a wrapped row above).
- The fold placeholder was not visible in the reference; its look comes from the research (inverted slate-400 pill).
- Preview inline code keeps the body size (font-size 85% cannot be set per run) and its ligature features; table cells repeat their borders (2 px between cells).

## Repeating the check

`BOURBON_DEMO_WINDOW_SIZE=838x685 cargo run -p bourbon-editor --example editor_demo` opens the demo on the reference note and lays it out in an 838x685 canvas at the top-left of the window (a tiling compositor may make the window bigger; crop the top-left 838x685). Compare with the reference crop from y = 113. `BOURBON_DEMO_FOLD_LINE=N` folds line N (1-based) at start, for the folded shot.
