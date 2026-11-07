# Inkdrop 6.1.5: Markdown editor and preview, reverse-engineered

Written 2026-11-07. Research only. Goal: give Bourbon (Rust, GPUI, no pre-made editor or markdown components) enough structure, values and behaviour to rebuild both from scratch.

Clean-room rule: this file records identifiers, class names, CSS variable names, numeric values and behaviour. It contains no Inkdrop source or CSS text. Bourbon is GPL-3.0.

## 0. Sources and how to read the evidence

Extracted app.asar at `/home/lowh/.claude/jobs/6d257de5/tmp/rea/inkdrop-app`. Shorthand used below:

| Tag | File |
|---|---|
| `NP` | `renderer/assets/notebook-picker-Bo2H_0--.js` (despite the name it holds the editor, the markdown parser extensions and the preview pipeline) |
| `APP` | `renderer/assets/app-BUTU7go0.js` (config defaults, keymap, commands, paste, scroll sync, preview component) |
| `APPCSS` | `renderer/assets/app-ClbQ0QMw.css` (layout rules) |
| `CSS-MD` | `node_modules/@inkdropapp/css/markdown.css` (preview theme) |
| `CSS-SYN` | `node_modules/@inkdropapp/css/syntax.css` (editor and code-token theme) |
| `CSS-UI` | `node_modules/@inkdropapp/css/ui.css`, `tokens.css` (fonts, h1-h6 scale, base colours) |
| `view` | `dist-HRoI40E2.js` (CodeMirror view, commands, language, search chunk) |
| `mdlang` | `dist-BKzBtnmQ.js` (lang-markdown and the GFM parser) |

`packages/default-light` and `packages/default-dark` contain only empty layer stubs (`theme.preview`, `theme.syntax`, `theme.ui`). All real values come from the base package `@inkdropapp/css`. Light and dark share one set of variables resolved with the CSS `light-dark()` function, switched by `color-scheme` on `body` (dark under `body.dark-mode`).

Line numbers inside a minified file are meaningless, so search terms are given instead. Unresolved items are listed in section 7.

---

## 1. Editor engine

### 1.1 Engine

CodeMirror 6, bundled by Vite/Rolldown. Evidence: `NP` holds the whole `EditorView.theme` definition and `cm-*` classes (search `.cm-gutters`, `.cm-activeLine`, `.cm-selectionBackground`, `.cm-tooltip-autocomplete`, `.cm-foldPlaceholder`); the Lezer runtime is in `dist-HRoI40E2.js` and `dist-CITzfOwb.js` (state). No CodeMirror 5 (`.CodeMirror` is absent from the CSS; a `highlighterCompatMode` option only adds a `CodeMirror` class plus legacy `cm-*` tag classes for old themes). Exact @codemirror version was not recorded in the bundle; the API surface used (`transactionExtender`, `scrollHandler`, `contentAttributes.compute`, `Facet.define({static})`) matches the 6.3x line.

React 19 wraps it (`refCM`, `resetState`, and per-option `Compartment` reconfigure effects in `NP`, search `Lt(` and the `new T` compartments).

### 1.2 Base extension list (always on)

The static `_h.extensions` array in `NP` (search `allowMultipleSelections.of(!0)`) resolves to:

1. `highlightSpecialChars`
2. `history` (undo stack, `Mod-z`, `Mod-y`, `Mod-u` undo selection)
3. `dropCursor`
4. `EditorState.allowMultipleSelections = true`
5. `rectangularSelection` (Alt-drag column selection; the `crosshairCursor` is not added)
6. `indentOnInput`
7. `bracketMatching` (style `.cm-matchingBracket`, `.cm-nonmatchingBracket`)
8. `closeBrackets`
9. `highlightSelectionMatches`
10. Class highlighter `tokenHighlighter` (class set `tok-*`), plus a second one for Markdown marks (class set `md-*`)
11. Heading style: bold plus h1 2em, h2 1.5em, h3 1.17em, h4 1em, h5 .83em, h6 .67em
12. Custom view plugins (section 1.4)
13. Selection listener (`updateListener`, `selectionSet`)
14. The big `EditorView.theme` (section 2)
15. The table editor extension (section 3.7)

Then, per setting, reconfigurable compartments:

| Setting (config key) | Default | Effect |
|---|---|---|
| `editor.lineNumbers` | false | `lineNumbers()` gutter on; else a 0.4em left padding theme on `.cm-content` and `.cm-line` |
| `editor.styleActiveLine` | false | `highlightActiveLine` and `highlightActiveLineGutter` |
| `editor.lineWrapping` | true | `EditorView.lineWrapping` |
| `editor.tabSize` / `indentUnit` | 2 / 2 | `EditorState.tabSize` |
| `editor.lineHeight` | 1.5 | `.cm-scroller` line-height |
| `editor.fontFamily` | SFMono-Regular, Consolas, Liberation Mono, Menlo, Courier, monospace | editor font |
| `editor.fontSize` / `fontWeight` | 13 / normal | |
| `editor.scrollPastEnd` | true | `scrollPastEnd()` |
| `editor.cursorScrollMargin` | 0 | custom transaction extender keeps N px between caret and viewport edge |
| `editor.inlineImageWidgets` | true | image widget decoration (section 2.5) |
| `editor.showInvisibles` | false | `highlightWhitespace`/`highlightTab` plus `.cm-newline` marks |
| `editor.highlightTrailingWhitespace` | false | `highlightTrailingWhitespace` (`.cm-trailingSpace`) |
| `editor.unorderedListBullet` | `*` | facet used when toggling bullet lists |
| `editor.unwrapBlocks` | false | code blocks and tables never soft-wrap (section 2.4) |
| `editor.limitMaxWidth` | false | "readable line length", content column 740px centred |
| `editor.pasteUrlAsLink` | true | section 3.5 |
| `editor.spellcheck` | true | native `spellcheck` content attribute; `autocorrect` forced off |
| `editor.readOnly` | per note | `EditorState.readOnly` |

Evidence: `APP` search `cursorScrollMargin:0,fontFamily:` (defaults) and `unwrapBlockAtCursor` (config mapping); `NP` search `highlightTrailingWhitespace`.

Not found in the extension list: `foldGutter`, `foldKeymap`, `autocompletion` default keymap (completion has its own commands, `editor:completion-selection-*`), `crosshairCursor`. The screenshot shows fold chevrons beside headings and fenced code, so folding exists; it is most likely contributed by the app-level extension list (reducer default `extensions: [...]` in `APP`, search `Type '/' for commands`). Folding is driven by the markdown language `foldNodeProp` (headings and Table, search `Table:(e,t)=>({from:t.doc.lineAt(e.from).to` in `mdlang`). Flagged unresolved in section 7.

Default keymap commands are exposed as app commands (`editor:go-*`, `editor:select-*`, `editor:delete-*`, `editor:move-line-up/down`, `editor:copy-line-up/down`, `editor:indent-more/less`, `editor:go-matching-bracket`, `editor:select-lines-upward/downward`, `editor:undo-selection`, `editor:redo-selection`, `editor:jump-to-line`, plus Emacs-style Ctrl-F/B/N/P/A/E/V/K). Search `APP`: `editor:go-char-left`.

### 1.3 Markdown language

Language: `@codemirror/lang-markdown` over `@lezer/markdown`, base parser `commonmark + GFM` (`mdlang` search `commonmarkLanguage`, `markdownLanguage`; `Table`, `TaskList` with node `TaskMarker`, `Strikethrough`, autolinks; `Emoji` node also present). Created with `addKeymap:false`: the stock markdown keymap is not installed; Inkdrop supplies its own (section 3). `codeLanguages` provides nested highlighting.

Inkdrop-specific Lezer extensions added in `NP` (search `Cp=e=>[cp,gp,qd,sl,Fd,e]`):

| Extension | Nodes | Purpose |
|---|---|---|
| Frontmatter | `Frontmatter`, nested YAML (legacy yaml mode) | `---` block at doc start |
| Fenced math | `FencedMath`, `MathText`, nested sTeX | `$$` blocks |
| Inline math | `InlineMath` (`$...$`, after `Emphasis`) | |
| Mark-token props | `HeaderMark`, `HardBreak`, `QuoteMark`, `ListMark`, `LinkMark`, `EmphasisMark`, `CodeMark`, `TableDelimiter`, `StrikethroughMark`, `InlineCode` mapped to `md-header-mark`, `md-hard-break`, `md-quote-mark`, `md-list-mark`, `md-link-mark`, `md-emphasis-mark`, `md-code-mark`, `md-table-delimiter`, `md-strikethrough-mark`, `md-inline-code` | colour/typography hooks |
| GitHub alerts | `Alert`, `AlertNOTE/TIP/IMPORTANT/WARNING/CAUTION`, `AlertQuoteMark<T>`, `AlertHeading<T>` | block parser inserted before `Blockquote` for `> [!TYPE]` |

Embedded code languages (the `codeLanguages` list in `NP`, search `C.of({name:`): about 150 entries. First-class Lezer packages: C, C++, CSS, Go, HTML, Java, JavaScript/JSX/TS/TSX, Jinja, JSON, LESS, Liquid, Markdown, PHP, Python, Rust, Sass/SCSS, SQL dialects (Cassandra, MariaDB, MSSQL, MySQL, PLSQL, PostgreSQL, SQLite), WebAssembly, XML, YAML, Vue, Angular, Elixir, GraphQL, Svelte, Julia, Nix, Mermaid. Everything else comes from `@codemirror/legacy-modes` (one chunk per language: shell, ruby, haskell, swift, lua, perl, r, toml, dockerfile, nginx, ...). Aliases such as `js`, `ts`, `py`, `sh/bash/zsh`, `cpp`, `md`, `math` (aliased to LaTeX) resolve fence info strings. Languages load lazily on first use. A completion source offers the fence info strings after "```" (search `matchBefore(/^```.*/)`).

Inline decorations on top of the tree (all `ViewPlugin` + `Decoration`, rebuilt only for `visibleRanges`, see 1.4):
- `md-inline-mark` for text between `<mark>` and `</mark>` HTML tags (3000-char look-behind).
- InlineCode ranges get `spellcheck=false`.

### 1.4 View plugins (decoration passes)

All in `NP` around `Ou`, `ju`, `Pu`, `yd`, `sd`, `Dd`, `Gu`, `zu`, `Ad`:

1. Code block lines: every line of `FencedCode`/`CodeBlock` (not nested in a blockquote or alert) gets line class `md-codeblock` plus `md-codeblock-begin` on the first and `md-codeblock-end` on the last; `spellcheck=false`.
2. Table lines: same pattern with `md-table`, `md-table-begin`, `md-table-end`.
3. Blockquote lines: `md-blockquote`, and `md-alert-<type>` for alerts.
4. `TaskMarker` gets mark `md-task-marker`; when the marker text is `[x]`/`[X]` the rest of the line gets `md-task-done-text` (strike-through at 40% transparency).
5. List mark depth colouring: the nest level cycles through 3 values; classes `md-list-mark` (level 1), `md-list-mark-2`, `md-list-mark-3` (depth minus 1 modulo 3, plus 1).
6. Hanging indent for wrapped list/quote lines (class `md-content-indent`): the width of the prefix up to the content column (list mark plus spaces, or quote marks) is measured (canvas `measureText` for proportional, a hidden `pre` with the tab size for monospace or tab content) and applied as negative `text-indent` plus equal left padding on that line. Wrapped continuation lines therefore align under the item text, not under the bullet.
7. Heading newline marks scale with the heading font-size when invisibles are shown.
8. `md-unwrap` mode (when `unwrapBlocks` is on): block lines get `white-space: pre`, widths of the widest block are measured with a ResizeObserver so each block scrolls horizontally independently and a `--md-block-overflow` variable extends the scroller.
9. Cursor entering a code block or table (when unwrapBlockAtCursor) toggles `md-nowrap` on the content and restores the scroll anchor.
10. `cm-gutters-hscrolled` class on the gutter when the scroller is scrolled horizontally (makes the gutter background solid).
11. Selection listener `gh` calls `ml` (token under caret) to feed the floating toolbar and context.
12. `moveToLineBoundary` is patched so Home/End on wrapped lines jump to the visual line boundary first, then the logical one.

---

## 2. Editor visual design

Theme sources: `NP` `EditorView.theme` blocks (structure, which variable goes where) and `CSS-SYN` (variable values). Light and dark values below are `light / dark`.

### 2.1 Typography and metrics

| Item | Value | Source |
|---|---|---|
| Font | `editor.fontFamily` (monospace stack above) | `APP` defaults |
| Size / line-height / weight | 13px / 1.5 / normal | `APP` defaults |
| Line horizontal padding | `--editor-line-padding-inline: 8px` | `NP` theme root |
| Block horizontal padding (code, tables) | `--md-block-padding-inline: 10px` | `NP` theme root |
| Caret | 2px wide, 1px radius, colour `--editor-caret-color`, blink via `cm-blink` keyframes (opaque 40%-90%, transparent 60%-70%) | `NP` |
| Outline | none, also none when focused | `NP` |
| Heading sizes in editor | h1 2em, h2 1.5em, h3 1.17em, h4 1em, h5 0.83em, h6 0.67em, all bold | `NP` heading style |
| Spelling error | wavy underline, colour = negative colour, offset 0.2em, not in tag names | `NP` |

### 2.2 Chrome colours

| Variable | Light | Dark |
|---|---|---|
| `--editor-foreground-color` | gray-800 hsl(215 28% 17%) | gray-300 hsl(216 12% 84%) |
| `--editor-background-color` | white | neutral-950 hsl(0 0% 4%) |
| `--editor-caret-color` | violet-600 hsl(262 83% 58%) | violet-400 hsl(255 92% 76%) |
| `--editor-selection-background` (unfocused) | violet-600 at 13% | violet-400 at 13% |
| `--editor-focused-selection-background` | violet-600 at 30% | violet-400 at 30% |
| `--editor-active-line-background-color` | blue-300 at 20% | gray-600 at 20% |
| `--editor-active-line-gutter-background-color` | same as active line | same |
| `--editor-special-char-color` | slate-400 | gray-600 |
| `--editor-gutter-color` | slate-300 | gray-500 |
| `--editor-gutter-background-color` | transparent | transparent |
| `--editor-gutter-border-right` | 1px, slate-200 | 1px, neutral-800 |
| `--editor-gutter-background-solid-color` (when h-scrolled) | white | neutral-900 |
| `--editor-selection-match-background-color` | slate-100 | gray-700 |
| `--editor-search-match-background-color` | amber-500 at 25% | same |
| `--editor-search-match-selected-background-color` | amber-500 at 50%, outline 1px amber-500 | same |
| `--editor-matching-bracket-outline` | 1px slate-300 | 1px gray-600 |
| `--editor-nonmatching-bracket-outline` | 1px slate-600 | 1px gray-600 |
| `--editor-placeholder-color` | slate-900 at 20% | gray-100 at 20% |
| `--editor-trailing-space-background-color` | red-200 | red-600 |
| `--editor-highlight-space-color` | slate-100 | gray-700 |
| Tab marker | an inline SVG arrow, opacity 0.3 | |

Search and selection-match decorations have 0.4em radius.

### 2.3 Gutter

- Line numbers: `font-size 0.7em`, `font-family sans-serif`, `line-height 2em`, `min-width 3em`, right padding 1em, colour `--editor-gutter-color`. Default off (`editor.lineNumbers`).
- Fold gutter element: right padding 0.4em. Fold placeholder: pill with 0.2em radius, 1px border slate-400/neutral-400, background slate-400/neutral-400, text colour = editor background (inverted pill).
- Gutter has a 0.2s background transition (solid when horizontally scrolled).
- Panels and tooltips: panel background slate-50 / neutral-900, colour slate-600 / neutral-400; tooltip 1px border slate-200 / gray-600, radius 0.8em, shadow `--floating-shadow`; autocomplete list max-height 16em, items radius 0.2em, padding 0 0.8em, selected item background violet-100 / violet-900; fade-in animation 140ms ease-out from scale 0.98.

### 2.4 Markdown token styling in the editor

Marks (`#`, `**`, backticks, `[`, `]`, `(`, `)`, `>`, list marks) stay visible in the source; there is no live-preview hiding. They take `tok-processing-instruction` colour: blue-600 / blue-400 (the screenshot shows blue `##` and blue backticks). Token to style table (class names come from `tokenHighlighter`; colours from `CSS-SYN`):

| Token (class) | Light | Dark | Extras |
|---|---|---|---|
| heading `tok-heading`, `tok-heading-1..6` | pink-600 hsl(333 71% 51%) | pink-400 hsl(329 86% 70%) | bold, size per 2.1 |
| content separator (hr) `tok-content-separator` | pink-600 | pink-400 | bold |
| list `tok-list` | foreground | foreground | |
| quote `tok-quote` | slate-600 | gray-400 | italic |
| emphasis `tok-emphasis` | foreground | foreground | italic |
| strong `tok-strong` | foreground | foreground | bold |
| link `tok-link` | teal-600 hsl(175 84% 32%) | teal-500 hsl(173 80% 40%) | underline, `text-underline-position: under` |
| url `tok-url` | teal-600 | teal-500 | `word-break: break-all` |
| monospace / inline code `tok-monospace`, `md-inline-code` | foreground | foreground | padding 1px, 1px border, radius 0.4em, line-height calc(140% + 2px) |
| strikethrough `tok-strikethrough` | | | line-through |
| comment | slate-400 | gray-400 | italic |
| string | green-600 | emerald-400 | |
| number, bool, atom, null, modifier | blue-600 | blue-400 | |
| keyword, operator-keyword, control, module, operator | violet-600 | violet-400 | |
| definition-keyword | blue-600 | blue-400 | |
| function-ish variable/property | cyan-600 | cyan-300 | |
| standard variable/property, class name | pink-600 | pink-400 | |
| tag name | blue-600 | blue-400 | |
| invalid | red-500 | red-500 | 1px dotted bottom border |

Markdown-specific variables:

| Variable | Light | Dark |
|---|---|---|
| `--md-list-mark-color` (level 1) | indigo-500 | indigo-500 |
| `--md-list-mark-2-color` | orange-500 | orange-500 |
| `--md-list-mark-3-color` | emerald-500 | emerald-500 |
| `--md-task-marker-font-weight` | bold | bold |
| `--md-task-done-text-decoration` | line-through, colour = currentColor at 60% | same |
| `--md-inline-code-background-color` | slate-200 at 10% | neutral-600 at 20% |
| `--md-inline-code-border` | 1px, slate-300 at 50% | 1px, blue-200 at 10% |
| `--md-codeblock-background-color` | slate-200 at 10% | neutral-700 at 20% |
| `--md-codeblock-border` | 1px, slate-300 at 50% | 1px, blue-200 at 10% |
| `--md-table-*` | same as code block | same |
| `--md-blockquote-border-color` | slate-400 | neutral-500 |
| `--md-inline-mark-*` (the `<mark>` tag) | yellow-500 at 30% background, 3px yellow underline offset 0.2em | same |

HSL source tokens (`tokens.css`): violet-600 262 83% 58%; violet-400 255 92% 76%; pink-600 333 71% 51%; pink-400 329 86% 70%; blue-600 221 83% 53%; blue-400 213 94% 68%; blue-500 217 91% 60%; teal-600 175 84% 32%; teal-500 173 80% 40%; green-600 142 76% 36%; emerald-400 158 64% 52%; cyan-600 192 91% 36%; cyan-300 187 92% 69%; indigo-500 239 84% 67%; yellow-500 45 93% 47%; amber-500 38 92% 50%; gray-300 216 12% 84%; gray-400 218 11% 65%; gray-500 220 9% 46%; gray-800 215 28% 17%; slate-100 210 40% 96%; slate-200 214 32% 91%; slate-300 213 27% 84%; slate-400 215 20% 65%; neutral-800 0 0% 15%; neutral-900 0 0% 9%; neutral-950 0 0% 4%.

### 2.5 Block decorations and widgets

- Code block and table "card": the lines carry padding-inline 10px; a `::after` layer sits behind the text, inset 3px left and right, with 1px side borders and the card background. The first line adds 4px top padding and a top border with 0.4em top radius; the last adds 4px bottom padding, bottom border and 0.4em bottom radius. Visually one rounded box per block, spanning the full line box, while the text keeps the editor font. See the screenshot (fenced block with a pale box and radius).
- Blockquote: inset 0.3em left bar (box-shadow) in `--md-blockquote-border-color`. Alert blocks use the same bar in the alert colour (`--gfm-alert-note` blue-500, `-tip` green-500, `-important` purple-500, `-warning` orange-500, `-caution` red-500); the `> [!TYPE]` heading and quote marks take that colour.
- Image widget (setting `inlineImageWidgets`): for each `Image` node (URL from the `URL` child, alt from `![...]`) a widget (side 1, block false) is placed at the end of the line containing the image. DOM: `.cm-image-widget` (block padding 0.5rem) > `.cm-image-widget-backdrop` (flex centre, margin-inline 3px, overflow hidden, max-width 100%, 0.4em radius, 1px border, backdrop colour slate-100 / neutral-600 at 20%) > `figure.cm-image-widget-figure` > `img.cm-image-widget-img` (width 100%, max-height 100%). Widgets are cached per (url, alt). Click on the figure calls the overridable `onFigureClick` (the app shows an image resize toolbar: Auto, 30%, 50%, 100% via `editor:resize-image-*`). Attachment URLs use the `inkdrop://file:...` scheme.
- Link tooltip: when the caret is inside a `URL`/`URI` node (not in an image) whose text starts with `https://` or `inkdrop://note`, a tooltip with class `cm-link-tooltip` shows "Open Note" or "Open URL" plus the shortcut label (command `editor:open-link`).
- Link pending preview: spinner `cm-link-format-preview-loading` while a pasted URL is resolved to a title (8-dot spinner keyframes `mulShdSpin`, 1.1s).
- Placeholder: "Just start typing, or" plus a "choose template" button on an empty note; "Type '/' for commands..." slash-menu hint.
- AI ghost text (`cm-nes-ghost`, `cm-nes-inline-tooltip`) exists but is out of scope for Bourbon.

---

## 3. Editor behaviours

### 3.1 Keymap (defaults, `APP` search `.cm-editor .cm-scroller`)

Written with the app's own command system (`editor:*`, `core:*`), `cmd-` means Cmd on macOS and Ctrl elsewhere.

| Keys | Command | Notes |
|---|---|---|
| Enter | `editor:new-line-continue-markup` | list/quote/task continuation, then fallback |
| Shift-Enter | `editor:new-line` | plain newline |
| Cmd-Enter / Cmd-Shift-Enter | `editor:insert-blank-line` / `editor:insert-line-before` | open line below/above |
| Tab / Shift-Tab | `editor:indent` / `editor:unindent` | list-aware, section 3.2 |
| Cmd-[ / Cmd-] | `editor:indent-less` / `editor:indent-more` | generic indent |
| Alt-Up/Down | `editor:move-line-up/down` | |
| Shift-Alt-Up/Down | `editor:copy-line-up/down` | |
| Cmd-Shift-K style delete | `editor:delete-line` | |
| Cmd-L | `editor:select-line` | |
| Ctrl-Shift-Up/Down | `editor:select-lines-upward/downward` | multi-cursor add |
| Escape | `editor:single-selection` | collapse multi-cursor |
| Cmd-G / Cmd-Shift-G | find next / prev | |
| Cmd-F / Cmd-Alt-F | find / replace panel | custom UI, not the stock CM panel |
| Cmd-Shift-L | `core:toggle-bulleted-list` | |
| Cmd-Shift-O | `core:toggle-numbered-list` | |
| Cmd-Shift-T | `core:toggle-task-list` | |
| Cmd-' | `core:toggle-blockquote` | |
| Cmd-1..4 | `core:toggle-heading-1..4` | |
| Cmd-B / Cmd-I / Cmd-U | strong / emphasize / strikethrough | |
| Backtick | inline code when there is a selection | empty selection types a backtick (closeBrackets pairs it) |
| Cmd-Shift-H | insert horizontal rule | |
| Cmd-Alt-K | insert link (uses clipboard URL if valid) | |
| Cmd-Shift-C | toggle fenced code block | |
| Cmd-E / Cmd-P | toggle preview / toggle side-by-side | |
| `/` at line start | slash command menu | |
| `[[` | note-link autocomplete | |

Menu items also expose: Insert Image, Insert Alert Note/Tip/Important/Warning/Caution.

### 3.2 Enter, Tab and list handling

- Enter: first runs the lang-markdown `insertNewlineContinueMarkup` (continues bullets, ordered numbers, task lists, and quotes; on an empty item it removes the marker), then a fallback. The Backspace-over-marker helper (`deleteMarkupBackward`) is available in `mdlang`; whether it is bound was not confirmed.
- Tab on a list item calls `indentListItem`; Shift-Tab calls `outdentListItem` (custom code, `NP` search `indentListItem`). Algorithm: for each selection range, find the list-item line (regex: leading spaces, marker `-`/`*`/`+` or 1-9 digits plus `.`/`)`, then spaces; skipping lines inside Code, Math or Frontmatter nodes). Indent target is the previous list item's content column when this item's indent equals the previous item's indent or is smaller than its content column; outdent target is the nearest shallower item's indent. The block of lines under the selection moves together; after moving, numbered siblings at the source and target depth are renumbered consecutively (equal numbers like "1. 1. 1." are preserved). If the line is not a list item, Tab falls back to generic indent.
- Toggle list (`Sl`): operates on all selected lines; if the first line already has the list/quote marker (checked through the syntax tree) it removes the markers, otherwise it prefixes each line with the bullet (config `unorderedListBullet`), `1.` (ordered; every line gets `1.`, relying on renumber) or `> `. Blockquote toggle keeps the deepest indent shared by the block. Selection ending at column 0 of the next line excludes that line.
- Toggle task: if the line is `ListItem > Task` with a `TaskMarker`, remove `[ ] `; otherwise insert `[ ] ` after the existing list mark, or `<bullet> [ ] ` if no mark. Multi-line aware.
- Task checkbox in the editor: a `mousedown` and `touchstart` capture handler on the content resolves the position under the pointer; when the syntax node there is `TaskMarker`, the marker text flips `[ ]` to `[x]` (or back), the event is cancelled, and the caret does not move. The marker has `cursor: pointer` and bold weight.
- Heading toggle: Cmd-1..4 sets exactly that many `#` (pressing it again removes them); bigger/smaller variants cycle h1..h6 and clear after h6.
- Wrap toggles (bold, italic, strike, inline code): if the selection is inside the matching node, the first and last child marks are removed and the node is selected; otherwise marks are inserted at both ends and the selection stays on the inner text; an empty selection between doubled marks removes them. Markers: `*`, `**`, `~~`, a backtick.
- Link (`core:insert-link`): wraps the selection as `[text]()` and puts the caret in the parentheses; pressing again on an empty `[]()` removes it. Command `editor:insert-link` accepts title and URL.
- `<mark>` toggle, horizontal rule (`* * *` with blank-line handling), fenced code (wraps the lines in three backticks; toggling inside a block unwraps to the code text), alerts (`> [!TYPE]` header then quoted body).

### 3.3 Auto-pairing

`closeBrackets` is on, using the language data of the active (including nested) language. For Markdown the pairs follow the lang-markdown data (parentheses, brackets, braces, quotes, backtick); this exact list was not read out of the bundle. Typing a closing character over an auto-inserted one skips it; Backspace between a pair deletes both.

### 3.4 Slash menu and autocomplete

- `/` opens a completion list ("Type '/' for commands...") with sections: first group (Link, Image File, Remote Image, Blockquote), "GitHub Alerts" (five alerts, icon coloured with the alert variable), "Other" (Note Link). Items run an action: delete the typed `/query` then dispatch a command, or insert a snippet with `${alt}`/`${url}` placeholders.
- `[[` note-link autocomplete: regex `[[` followed by non-`|`, non-`]` characters. Fewer than 3 characters: the 50 most recently updated notes. Otherwise a full-text title search ranked by relevance. Each option applies the text `[Title](inkdrop://note/<id>)`, has an info popup showing the note body (first 5000 chars) rendered in markdown (`cm-note-link-preview`), and a final option "Create new note" (creates a note, inserts the link, opens it). So the stored syntax is a normal Markdown link, never `[[wikilink]]` text.
- A code-fence info-string completion fires after three backticks.
- Completion UI: `editor:completion-selection-*` commands, matched text in bold blue (`--editor-completion-matched-text-*`), details right aligned.

### 3.5 Paste, drop, links

Paste order (`APP`, search `Attempting to paste content from clipboard`):
1. Image data in clipboard: save as an attachment and insert `![clipboard](inkdrop://file:...)`.
2. HTML with rich tags (a, b, strong, i, em, ul, ol, li, code, table, tr, td, h1-h6, hr, img, del, s): convert HTML to Markdown and insert (sanitised first: remove conditional comments, collapse whitespace). Blocked if the app flag says plain-text paste.
3. Plain text: tabs expanded to `tabSize` spaces; if the text (trimmed) is a URL and `pasteUrlAsLink` is true and the caret is not in a code block, not between `[` `]`, and not preceded by `[`, `<` or a backtick, the command `editor:paste-url` inserts a link whose title is fetched (spinner while loading).
4. `core:paste-as-plain-text` (Cmd-Shift-Alt-V) bypasses all of that.

Copy: images in the HTML clipboard that use `inkdrop-file://` are rewritten to data URIs so they paste elsewhere.

Drop: a file drop calls `editor:insert-images` at the drop position (`posAtCoords`), inserting image Markdown.

### 3.6 Search and replace

Own panel UI (find field, replace field, case, regex toggles, next/prev, replace, replace all) driven by the stock CM `SearchQuery` state effect; matches shown by `cm-searchMatch`. Opening search switches view mode to edit.

### 3.7 Table editor

A bundled table-editor kernel (the `mte-kernel` style: `formatAll`, `nextCell`, `previousCell`, `nextRow`, `escape`, `moveFocus`, `alignColumn` left/center/right/none, `insertRow`, `insertColumn`, `deleteRow`, `deleteColumn`, `moveRow`, `moveColumn`, `selectCell`; `NP` search `alignCenter:U`). Active only when the caret is in a `Table` node (adds `table-editor-active` class on the editor). Commands are `table-editor:*`. Width calculation uses a string-width table for East Asian wide characters (the big `[0,31,"N"],[32,126,"Na"]...` table in `NP`). Table keymap specifics were not read; see section 7.

### 3.8 Misc

- Frontmatter YAML is parsed and highlighted; the preview also parses it for metadata.
- Cursor scroll margin transaction extender (default 0, i.e. off).
- Undo and redo go through the app (`core:undo`, `core:redo`).

---

## 4. Preview pipeline

### 4.1 Pipeline (the class `bi` in `NP`, search `getRemarkParser`)

```
markdown string
  -> unified()                         (micromark / mdast)
     remark-parse (options)
     remark-gfm                        (tables, task lists, strikethrough, autolink literals, footnotes)
     remark-math                       ($..$ inline, $$ blocks; node type math, class language-math math-display)
     inkdrop:// protocol autolink      (micromark extension + transform that auto-links "inkdrop://..." text)
     remark-frontmatter (yaml)         (parsed into data, kept as metadata)
     github alerts plugin              (blockquote beginning with [!NOTE|TIP|IMPORTANT|WARNING|CAUTION])
     mark-fenced-code                  (sets fenced attribute on code nodes)
     code-meta                         (info string "lang:title:line" becomes lang + meta{title,line})
     image-size                        (alt text tokens w:100 h:50 become inline width/height style, alt cleaned)
     [plugin-registered remark plugins]
  -> remark-rehype (allowDangerousHtml, custom handlers for listItem and code)
     [rehype-raw only when the doc contains html nodes]   (processor is rebuilt in raw mode on demand)
     line-position pass                (adds line-start / line-end attributes to every element)
     rehype-slug                       (heading ids, github-slugger)
     metadata-section pass             (inserts a metadata element after a page-title h1 or at the top)
     [plugin-registered rehype plugins]
  -> hast-util-to-jsx-runtime          (React components map)
```

Libraries: unified, remark-parse, remark-gfm, remark-math, remark-rehype, rehype-raw, rehype-format and rehype-stringify (only for export), micromark-extension-gfm, hast-util-to-jsx-runtime. No markdown-it, no marked. Evidence: chunk names `unified-C5X28yLX`, `remark-rehype-krE97rsr`, `rehype-raw-DecDah-2`, `micromark-extension-gfm-CgOphYfI`, `remark-react-components-CQGfGCpc`; chain in `NP` search `.use(wt,this.remarkOptions)`.

Handler details:
- List items: a task item gets a leading `input` (type checkbox, class `task-list-item-checkbox`) and class `task-list-item`, plus `checked` when ticked; loose-vs-tight paragraph unwrapping follows the mdast `spread` rule.
- Fenced code is emitted as `div.codeblock.lang-<x>` > `pre` > `code.language-<x>`; the info meta is kept.
- Render result keeps `mdast` and `hast` in context for plugins and `KaTeX required` detection (a code element with class `language-math` triggers lazy KaTeX loading; the KaTeX files were not found in the bundle directory, so they are loaded from the `katex` package at runtime or from another asset).
- Math: block `$$` and inline `$`; the same grammar exists in the editor so highlighting matches.
- Mermaid: a `mermaid` fence language exists in the editor; `@inkdropapp/css/mermaid.css` styles diagrams in the preview (rendered through a code component keyed by language, `remarkCodeComponents[lang]`).
- Emoji: no emoji plugin in the remark chain. CSS for `g-emoji` exists (sizing, Apple/Segoe emoji fonts) for compatibility only. Treat as not supported beyond Unicode.
- Footnotes: from remark-gfm; CSS in section 5.
- Syntax highlighting of fenced code in preview: the `codeHighlighter` component receives `lang` and text; the theme rules in `CSS-SYN` scope the same `tok-*` classes to `.mde-preview .codeblock`, so the preview uses the same Lezer-based highlighter and token classes as the editor, not highlight.js or Prism. Line numbers: an info string such as `js:title:10` shows a numbered gutter (`mde-preview-code`, class `with-line-numbers`, starting at the given line; ranges `a-b` become "Lines a to b" in the header).
- Code blocks with meta (title, filename, line, branch/commit, url) render a header bar `.codeblock-meta` with a language icon, title (600 weight), filename link, and "Line at N" or "Lines a to b"; a GitHub URL shows the GitHub icon and the repo path.
- Inline code that is a colour literal (`#rgb`, `#rrggbb`, `hsl()`, `rgb()`) renders a colour swatch component.
- Copy button component on `pre` blocks (hidden in print mode).

### 4.2 Sanitisation and safety

There is no `rehype-sanitize`. The three "sanitize" hits in `APP` are file-name sanitising for export. Safety comes from the component map: `iframe` allowed only for `https://` sources and sandboxed (`allow-forms allow-presentation allow-same-origin allow-scripts allow-modals`, default height clamp(320px, 50vh, 640px), width 100%); `webview` renders an error box ("webview is not allowed"); `object` renders an empty span; `style` blocks are scoped under `.mde-preview` (or `.editor` in print mode); raw HTML only goes through when `rehype-raw` is enabled. Link schemes follow the config `allowedLinkSchemes` (default empty = built-in list). `javascript:` handling is not visible in the bundle; Bourbon should deny it explicitly.

### 4.3 Link and checkbox handling in the preview

- Anchor component emits an app event `a:click`; listeners can consume it. Built-in handlers in `APP` (search `a:cl`): `href` empty (the `[text]()` pattern) creates a new note titled by the link text, rewrites that source line to `[text](inkdrop://note/<id>)`, saves and opens it (Cmd/Ctrl opens a new window); `href` starting with `#` scrolls to the element with that id (`scrollIntoView`). `inkdrop://note/...` links and external links are routed by the app's link handler (opens the note in the same editor with navigation history, or the system browser); embed providers (`getProviderForURL`) can replace an anchor with an embedded component (video, cards).
- Task checkbox: a controlled input carrying the node's `line-start`. On change it emits `checkbox:change {checked, numLine}`; the handler dispatches `editor:toggle-task-list {line, checked}` to the editor (which flips the `TaskMarker` on that source line), re-renders, then saves after 100ms. The editor stays the single source of truth; the preview never edits the string itself.
- `line-start` / `line-end` are attached to every hast element from `position.start.line` / `position.end.line` (1-based source lines). They are also what scroll sync uses.

### 4.4 Render scheduling

`preview.render()` is dispatched on note load, on content change (a normal and a "slow" debounced variant, `debouncedRender`, `debouncedRenderSlow`), and when the preview becomes visible. Scroll position resets on note change. A render error shows an error component and keeps the old dom.

### 4.5 View modes and layout

- Three modes in a redux enum `viewMode`: `EDIT`, `SIDEBYSIDE`, `PREVIEW`. Toolbar: segmented control with pen, split-columns and eye icons (see the screenshot). Shortcuts: Cmd-E toggles preview, Cmd-P toggles side-by-side.
- Layout (`APPCSS`): `.editor-layout` is a flex column with `--editor-background`; `.mde-layout` is a flex row (flex auto, centred, min-height 0); `.mde` (editor) is flex 1 with min-width 0; `.mde-preview-container` is flex 1, overflow auto, row, centred; inside it `.mde-preview` has padding 10px and flex 100%. Both panes are `flex: 1`, so side-by-side is an even 50/50 split, no draggable divider was found. In `PREVIEW` mode `.mde` is `display:none`; in `EDIT` mode the preview container is hidden. A trailing `.mde-preview-scroll-past-end` block (height 100%) gives scroll-past-end in the preview.
- `limitMaxWidth` ("readable line length"): the editor content column or the preview column gets `flex: 0 1 740px` (`--readable-line-length-width`), centred.
- The preview is mounted, hidden with a `hidden` prop, rather than destroyed when not visible; the editor `CodeMirror` view is kept alive.
- Preview font size is a separate setting `preview.fontSize` (default 15), changed by "increase/decrease font size" shortcuts (Cmd-+, Cmd--); `syncScroll` default true; `expandBacklinks` true.

### 4.6 Scroll synchronisation (algorithm, `APP` search `getLinePosOnPreview`)

Units are fractional source line numbers. Only active in side-by-side mode (`syncScrollEnabled`).

Preview to editor (user scrolls preview): `getLinePosOnPreview`:
1. Collect all preview elements that have `line-start`.
2. Find the last element whose top is above the container top (line `L0`, offset `d0 <= 0`) and the first element at or below it (line `L1`, offset `d1 >= 0`). If there is no element below, use the last element's bottom and `line-end + 1`.
3. `progress = -d0 / (d1 - d0)`; `line = L0 + progress * (L1 - L0)`.
4. Store this line as the shared value and tell the editor (`editor:sync-editor-scroll`) to set its `scrollTop` to the pixel position of that fractional line (`an(view, line)` uses the line block height map).

Editor to preview (user scrolls editor): the top visible fractional line is read with the editor's helper (`kn`), stored, then `scrollPosLineAt` resolves a preview `scrollTop`:
1. `up` search: starting at `floor(line)`, walk upward line by line until an element with `line-start == n` exists and is not inline-only (tag not in `STRONG`, `EM`, `I`, `BR`).
2. `down` search: starting at `floor(line)+1`, walk downward the same way, up to the last element's `line-end`.
3. Interpolate: `t = (line - L_up) / (L_down - L_up)` between the two elements' tops; `scrollTop = top_up + t * (top_down - top_up) - containerTop + scrollTop - 10px`.

Feedback-loop guard: a three-state flag (`''`, `'editor'`, `'preview'`, `'disabled'`) records which pane initiated the scroll; programmatic scrolls set the flag and a debounced reset (100ms) clears it; events from the pane that is currently being driven are ignored. `disabled` is set while a note is switching. Editor state saved in navigation history includes `scrollLineNumber`, restored with two `requestAnimationFrame` passes.

---

## 5. Preview visual design (`CSS-MD`, `CSS-UI`)

Base: `.mde-preview` margin 0, colour `--text-color` (black 87% / neutral-50 87%), font-size 16px in the stylesheet (overridden by `preview.fontSize`, default 15), line-height `--mde-preview-line-height: 1.5`, `word-wrap: break-word`. Font family is inherited from the page: `-apple-system, BlinkMacSystemFont, Segoe UI, Lato` then Helvetica Neue, Arial, Helvetica, sans-serif and emoji faces (`--page-font`). First child has margin-top 0, last child margin-bottom 0.

### 5.1 Headings

| Level | Size | Weight | Extra |
|---|---|---|---|
| h1 | `--h1: 2em` | 600 (`--font-weights-bold`) | margin 0.67em 0; padding-bottom 0.3em; bottom border 2px (`--mde-preview-h1-border-width`) in `--mde-preview-heading-border-color` |
| h2 | `--h2: 1.5em` | 600 | padding-bottom 0.3em; 1px bottom border |
| h3 | `--h3: 1.25em` | 600 | |
| h4 | `--h4: 1em` | 600 | |
| h5 | `--h5: 0.875em` | 600 | |
| h6 | `--h6: 0.85em` | 600 | colour muted (`--mde-preview-heading-muted-text-color` = `--muted-text-color`) |

All headings: margin-top 24px, margin-bottom 16px, line-height 1.25, colour `--mde-preview-heading-color` (= text colour). Heading border colour is `--border-color` (neutral-900 at 15% / neutral-100 at 10%). `.metadata + h1..h6` have no top margin. Anchor links (`.anchor`) float left at margin-left -20px. Code inside headings inherits the font size with 0 0.2em padding.

### 5.2 Block elements

| Element | Values |
|---|---|
| Paragraph | margin-top 0, margin-bottom 10px at the base rule, 16px in the later shared rule (p, blockquote, ul, ol, dl, table, pre, details share margin-bottom 16px) |
| Blockquote | margin 0, padding 0 1em, colour `--muted-text-color` (black 60% / neutral-50 60%), left border 0.25em solid `--solid-border-color` (zinc-300 / zinc-700); first child top margin 0, last child bottom margin 0 |
| Lists | `ul, ol` padding-left 2em, margin 0; nested lists margin 0; `li + li` margin-top 0.25em; `li > p` margin-top 16px; bullets disc, nested ordered `lower-roman`, deeper `lower-alpha`; marker colour `--mde-preview-list-marker-color` (levels cycle, all = text colour by default) |
| Task list | `.task-list-item` list-style none; checkbox margin 0 0.2em 0.25em -1.4em, vertical-align middle, 4px end margin; consecutive items margin-top 4px; label weight normal; checked item line-through in `--mde-preview-task-list-checked-color` (muted) |
| hr | height 0.25em, margin 24px 0, no border, background `--solid-border-color` |
| Code block (`pre`) | padding 16px, font-size 85%, line-height 1.45, radius 6px, 1px border, background slate-200 at 10% / neutral-700 at 20%, border slate-300 at 50% / blue-200 at 10%, overflow auto, margin-bottom 16px; text colour = text colour; `word-break: normal` |
| Code fonts | `ui-monospace, SFMono-Regular, SF Mono, Menlo, Consolas, Liberation Mono, monospace`, 12px (pre and code); `pre code` 100% |
| Code meta bar | padding 8px 16px, gap 10px, font-size 80%, line-height 1.3, muted colour, background transparent-black (5% / 8% white), 1px border, radius 6px 6px 0 0, no bottom border; following `pre` has 0 top radius |
| Table | `display:block`, `width: max-content`, `max-width: 100%`, overflow auto, border-collapse; th and td padding 6px 13px, 1px border `--solid-border-color`; th weight 600; header row background strong-transparent (black 10% / white 15%); rows have background strong-transparent and a 1px top border `--border-color`; odd tbody rows use the lighter stripe (black 5% / white 8%) |
| Footnotes | `.footnotes` 12px, muted colour, top border; list padding-left 16px; target item gets a 2px outline box radius 6px in `--primary-color-active`; refs render as `[1]` via pseudo content |
| GFM alert | `.gfm-alert` padding 0.5rem 1rem, left border 0.25em in the type colour, title 600 weight coloured with the type colour plus a 16px icon with 0.5em right margin; colours: note blue-500, tip green-500, important purple-500, warning orange-500, caution red-500 |
| Definition list | dt italic 600, margin-top 16px; dd padding 0 16px |
| details/summary | summary display list-item; closed details hide non-summary children |
| Figure | margin 1em 40px |
| `mark` | background yellow-500 at 30%, 3px solid yellow underline, underline offset 3px |
| `kbd` | inline-block, padding 4px 5px, 11px mono, line-height 1, radius 6px, 1px border, inset bottom shadow |

### 5.3 Inline

| Element | Values |
|---|---|
| Link | colour `--link-color` (blue-500 hsl(217 91% 60%)), no underline, underline on hover; a without href inherits colour |
| Strong | weight 600 |
| Emphasis | colour = text colour |
| Inline code | font-size 85%, padding 0.2em 0.4em, radius 6px, background black 5% (light) / white 8% (dark), no border by default, `white-space: break-spaces`; in links inherits colour |
| Strikethrough | standard `del` |
| Image | max-width 100%, background `--strong-transparent-*`, `align` attributes get 20px padding; width/height from the alt-size syntax |
| Sup / sub | 75% size, line-height 0, offsets -0.5em / -0.25em |
| Small | 90% |
| Emoji `g-emoji` | 1em, line-height 1, vertical-align -0.075em |

Print rules (`@media print`): tables overflow hidden with word-break; `pre > code` wraps (`white-space: pre-wrap`).

Light/dark colour resolution summary: text black 87% / neutral-50 87%; muted text black 60% / neutral-50 60%; `--border-color` neutral-900 15% / neutral-100 10%; `--solid-border-color` zinc-300 / zinc-700; link blue-500 for both.

---

## 6. Proposed Rust architecture for Bourbon

Status: PROPOSAL, nothing is decided. It describes how to get the same structure and feel, not a copy of Inkdrop's code.

### 6.1 Principles taken from Inkdrop

1. Source text is the only truth; the editor shows raw markdown with marks visible (no hidden syntax). The preview is a pure function of the text (plus note-link resolution) and never edits it, except through editor commands (checkbox).
2. Everything visual is token-to-style: the parse tree yields tokens (class names), a theme maps tokens to colour/weight/size. Light and dark are two token tables.
3. Block cards (code block, table, quote bar) are decorations behind lines, not different layout.
4. Editing is a set of small pure commands over (text, selections) producing a change set; the UI only dispatches.
5. Scroll sync works on fractional source lines using per-block `line-start`/`line-end`.

### 6.2 Crate and module split (GPUI-free core, GPUI view on top)

Match the existing Bourbon layering (verify names against the workspace before creating anything).

| Crate | GPUI? | Contents | Inkdrop counterpart |
|---|---|---|---|
| `bourbon-text` | no | `Rope` buffer (ropey), `Change`/`ChangeSet` (insert/delete ranges, mapping positions), `Selection` set (anchor/head, multi-range), `Transaction` (changes + selection + user event kind), `History` (grouped undo/redo with selection restore, "isolate history" boundaries), line and column helpers, grapheme and word segmentation (unicode-segmentation, unicode-width) | CM state, history |
| `bourbon-markdown-syntax` | no | `MarkdownSyntax` trait: `parse(&Rope) -> SyntaxTree`, `reparse(edit)`; the `SyntaxTree` node kinds mirror the Lezer set (Heading, HeaderMark, ListItem, ListMark, TaskMarker, Task, Blockquote, QuoteMark, FencedCode, CodeInfo, CodeText, InlineCode, CodeMark, Emphasis, Strong, Strikethrough, Link, LinkMark, URL, Image, Table, TableDelimiter, HorizontalRule, Frontmatter, MathBlock, InlineMath, Alert*, HTMLTag); `TokenClass` enum (the `tok-*` and `md-*` names above); `highlight(range) -> Vec<(Range, TokenClass)>`; nested code highlighting through a `CodeHighlighter` trait | lang-markdown + tokenHighlighter |
| `bourbon-editor-core` | no | commands as pure functions `fn(&EditorState) -> Option<Transaction>`: continue markup on Enter, indent/outdent list item with renumber, toggle bullet/ordered/task/quote/heading, wrap toggles, link, code fence, hr, alerts, move/copy/delete line, auto-pair, smart paste (URL to link, html-to-markdown, tab expansion), table format and cell navigation, note-link query model, slash menu items; keymap data (`Keymap` of chord to command id) | Inkdrop commands and keymap |
| `bourbon-editor-layout` | no | Display model: soft-wrap and line layout traits, `LineBlock` height map (Fenwick tree or SumTree keyed by line), decorations (`Mark`, `LineClass`, `BlockWidget`, `Replace`), hanging indent computation, fold ranges, visual-line movement (needs a `TextMeasure` trait implemented by the view) | CM view heightmap and decorations |
| `bourbon-markdown-render` | no | Markdown to `PreviewDoc` IR: block and inline tree, each node with `source_lines: (start, end)`; GFM tables, task lists, footnotes, math blocks, alerts, frontmatter, code meta (`lang:title:line`), image size syntax, note-link resolution (`inkdrop://note/` equivalent for Bourbon), heading slugs, safe-URL policy; no GPUI types | remark/rehype chain |
| `bourbon-scroll-sync` | no | pure fractional-line mapping both ways over `Vec<(line_start, line_end, top, height)>` plus the origin guard state machine | `getLinePosOnPreview`, `scrollPosLineAt` |
| `bourbon-theme-markdown` | no | token tables: editor tokens (section 2) and preview typography (section 5), light and dark; HSL tokens above as constants | CSS packages |
| `bourbon-ui-editor` | yes | GPUI `EditorView` element, input handler (IME), focus, mouse, scroll, gutter, widgets, search panel, completion popup | CM view plus React wrapper |
| `bourbon-ui-preview` | yes | `PreviewView`: `PreviewDoc` to GPUI element tree, link and checkbox events, virtualised list | MDEPreview and components |
| `bourbon-ui-mde` | yes | `MdeLayout`: view modes (edit / side-by-side / preview), shared scroll controller, 50/50 split, readable width | `MDELayout` |

Dependency direction: ui crates depend on core crates, never the reverse. `bourbon-editor-core` and `bourbon-markdown-render` share only `bourbon-text` and `bourbon-markdown-syntax`. All logic crates are testable without a window (project rule: TDD). Doctrine reminder: no comments in code; the command and token names carry the meaning.

### 6.3 Parser choice: tree-sitter-md vs pulldown-cmark vs comrak

| Criterion | tree-sitter-md | pulldown-cmark | comrak |
|---|---|---|---|
| Incremental parse | yes (edit, reuse old tree) | no, full reparse | no |
| Error tolerance while typing | good (tree always exists) | good in practice (CommonMark never fails) | same |
| Marker nodes (`ListMark`, `HeaderMark`, `TaskMarker`, `QuoteMark`, `LinkMark`) | yes, as nodes | no, derive from byte ranges of the container event (scan the first bytes of the range) | no, sourcepos only per node |
| Byte offsets for inline | yes | yes (`into_offset_iter`) | line:col only, inline offsets weaker |
| GFM tables, task lists, strikethrough, footnotes, alerts | tables, task markers, strikethrough yes; alerts, math, wikilinks are not in the base grammar | all via options, plus math, alerts (`BlockQuoteKind`), YAML metadata block, wikilinks, definition lists | all via extensions, also math, alerts, wikilinks, front-matter |
| Speed | fast, incremental | very fast (a 1 MB note in a few ms) | slower (arena AST) |
| Fit for editor | closest to Lezer (node tree plus ranges) | needs a thin marker scanner | poor fit |
| Fit for preview | needs a second pass to AST | direct events to IR | direct AST |
| Build | C dependency (cc), two grammars (block and inline) with injection | pure Rust | pure Rust |

Recommendation: use pulldown-cmark behind the `MarkdownSyntax` trait for both the editor and the preview (one parser, pure Rust, ranges available), with a small marker scanner that turns container ranges into the mark tokens. Reparse the whole note on change for the MVP, optimise by reparsing the dirty block span (split on blank lines outside fenced code, expand to the container) once profiling says so. Keep a `TreeSitterMarkdownSyntax` implementation as an option if incremental parsing is needed for very large notes; the trait is the seam. Add Inkdrop-specific syntax that pulldown does not know (YAML frontmatter is built in; `$$` math and inline math are built in; custom `<mark>` tag and `inkdrop://` autolink are post-passes over the event stream and token ranges). Use `tree-sitter-highlight` (or tree-sitter language crates directly) for fenced-code tokens with a curated set of grammars (start with about 25: rust, js/ts/tsx, python, json, yaml, toml, html, css, shell, go, java, c/cpp, sql, ruby, markdown, diff), mapped onto the same `TokenClass` names so editor and preview colour code identically. Do not try to match Inkdrop's 150 legacy modes.

### 6.4 Editor view in GPUI

- Text model: `ropey::Rope`; positions as byte offsets with line/column derived from rope; selections as non-empty `Vec<Range>` plus main index; changes are applied through `Transaction`, producing a position-mapping function used by decorations, selections, folds and the syntax tree.
- Undo: group typing into runs (time and kind based), store inverse `ChangeSet` plus the previous selection; separate undo for selection only (Inkdrop has `undo-selection`).
- Layout: one custom GPUI `Element` implementing `request_layout`, `prepaint`, `paint`. Only lines in the viewport are shaped. Shape each logical line with GPUI's text system (`shape_text`/`WrappedLine` with a wrap width, `TextRun` per token class with font weight, style, colour, and per-line font-size for headings). Cache the shaped lines in an LRU keyed by (line id, revision, wrap width, theme revision). Line heights come from the shaped result; keep a prefix-sum height tree for all lines (unshaped lines use the estimated height from the font size and the last measured average), corrected when real shapes arrive; scroll position is stored as (line, pixel offset within line) to avoid jumps.
- Soft wrap: provided by the wrapped-line API. Hanging indent for list and quote continuation: shape the content part (after the marker prefix) with `wrap_width - indent_px`, paint the prefix at x0 and the content at x0 + indent_px (indent_px is the measured prefix width, the same trick as the Inkdrop measure). Wrapped code-block lines and tables in `unwrapBlocks` mode: shape without wrapping and add a per-block horizontal scroll later (phase 3).
- Gutter: separate paint pass left of the text: line numbers (font 0.7 of editor size, line-height 2x of the text line box, right padding 1em, min width 3em), fold chevrons, active-line background; border-right 1px; solid background when horizontally scrolled.
- Decorations: a `DecorationSet` of ranges (marks with token class or style), line classes, block widgets (images), replace widgets (none needed at first). Painted back to front: block card backgrounds and quote bars and active line quad, selection quads, text, caret. Card geometry: inset 3px on both sides, 1px border, radius 0.4em at first and last line, 4px extra padding top and bottom, computed from the decoration line classes.
- Selection and caret: selection quads per visual line from `WrappedLine::position_for_index`; caret as a 2px quad with 1px radius, blink timer via `cx.spawn`/`Timer` reset on input; multi-cursor draws all carets. Hit testing uses `index_for_position` on the cached shaped line; task-marker click is a hit test on the `TaskMarker` token range on mouse-down (do not move the caret).
- Input: implement GPUI's `EntityInputHandler` (text for range, selected range, marked range, replace text in range, replace and mark text in range, bounds for range, character index for point) so IME composition, dead keys and OS text services work; this is the single most error-prone part, do it first and test with a CJK IME and a dead-key layout. Key bindings go through GPUI actions (`actions!`), one action per command id from `bourbon-editor-core`, bound in a keymap table with the Inkdrop defaults above (translating Cmd to the platform primary modifier).
- Clipboard: `cx.read_from_clipboard()` returns text and images; HTML clipboard content is not exposed by GPUI on all platforms. Rich-paste (HTML to Markdown) therefore needs a platform clipboard helper (for example arboard or wl-clipboard-rs) behind a trait in `bourbon-ui-editor`; until then paste plain text and images only. Smart URL paste (title fetch) needs an async task and a placeholder decoration while loading.
- Search: panel is a small view over `bourbon-editor-core::search` (query, case, regex toggles, next/prev, replace, replace all) with match decorations (`search-match`, selected variant).
- Completion popup: floating list (slash menu and note-link) driven by pure completion sources in `bourbon-editor-core`; selection keys: up/down, page up/down, enter or tab accept, escape closes.
- Folding: heading sections and fenced code and tables via the syntax tree; fold state as replace decorations (placeholder pill).
- Performance budget: parse plus highlight under a frame (16 ms) for notes up to roughly 200 KB; above that debounce the reparse and highlight only the visible range plus a margin, reuse the previous token set for unchanged lines (map through the change set).

### 6.5 Preview view in GPUI

- `bourbon-markdown-render` yields `PreviewDoc`: blocks (Heading{level, id}, Paragraph, List{ordered, start, tight}, ListItem{task: Option<bool>}, BlockQuote, Alert{kind}, CodeBlock{lang, meta, text}, Table{align, head, rows}, ThematicBreak, Html (rejected or shown as text), Footnotes, Math, Image) and inlines (Text, Emph, Strong, Strike, Code, Link{href}, Image, Break, FootnoteRef, Math), each node carrying `source_lines`.
- `bourbon-ui-preview` maps nodes to GPUI `div` trees with the typography of section 5 (use rem-like units from the preview font size; the h1..h6 scale 2/1.5/1.25/1/0.875/0.85 em; margins 24/16 px; 16 px block gap; 2em list indent; 0.25em quote bar; table padding 6x13; code padding 16 and radius 6). Inline runs use `StyledText` with highlights for code spans, links (hover underline, click handler), strike; inline code needs a background quad, which `StyledText` does not paint, so render inline code as `TextRun` background colours (GPUI `TextRun.background_color`) plus padding approximated with thin spaces, or as inline element children when exact padding is needed.
- Virtualisation: use GPUI's list state (`list`/`ListState`) over top-level blocks so very long notes stay cheap; heights are measured and cached. Record each block's top and height for scroll sync.
- Code blocks: tokens from the same highlighter as the editor, header bar for meta, copy button (hover), optional line numbers from the info string `lang:title:startLine`.
- Images: async load, max-width 100%, placeholder background; attachment scheme resolved by the app.
- Links: `inkdrop://note/<id>` equivalent is a Bourbon note link (`bourbon://note/...` is a decision for the app shell doc); empty-href links create a note; `#id` scrolls to the heading with that slug; allow list for external schemes (https, http, mailto) and deny `javascript:`.
- Checkbox: clicking emits a `ToggleTask { source_line }` event to the editor entity, which applies the same `toggle task at line` command used by the editor click; the preview re-renders from the new text. Never mutate the preview doc.
- Math and mermaid: out of scope until later; render math blocks as monospace boxes first, and treat mermaid as a code block. A later path is a Rust LaTeX renderer or a rasterised SVG pipeline (decide separately).

### 6.6 Scroll sync

- Editor reports the top visible fractional line (line + pixel fraction of that line's height); preview reports the same from block geometry. `bourbon-scroll-sync` implements both mappings exactly as section 4.6 describes, with a guard state (`None`, `Editor`, `Preview`, `Disabled`) that suppresses echo events and resets 100 ms after the last programmatic scroll. Preview gets a 10 px lead. Skip inline-only anchors (strong, emphasis, break) when picking anchors, in Bourbon this is automatic because anchors are blocks and list items.
- Side-by-side mode owns one `ScrollSyncController` entity shared by both views; edit-only and preview-only modes detach it.

### 6.7 Phased plan

| Phase | Scope | Exit criteria |
|---|---|---|
| 0 | `bourbon-text` rope, transactions, history; property tests (apply then invert is identity; mapping stays in bounds) | tests green, no UI |
| 1 MVP editor | `bourbon-markdown-syntax` with pulldown-cmark plus marker scanner; plain GPUI element: shaped lines, caret, selection, typing, IME handler, copy/cut/paste text, undo/redo, soft wrap, token colours for headings, emphasis, links, code, lists, quotes; light and dark tables | type and edit a note, correct highlighting, IME works |
| 1 MVP preview | `bourbon-markdown-render` plus view: headings, paragraphs, lists (tight/loose), quotes, code blocks (no colours), inline styles, links, hr, images, tables | preview matches section 5 on a fixture note |
| 2 Behaviour | Enter continuation, Tab list indent with renumber, toggles (bold, italic, strike, code, heading, list, task, quote, link, fence, hr), auto-pair, task click in editor and preview, gutter and active line, find and replace, mode switcher and side-by-side with scroll sync | behavioural tests from section 3 as unit tests in `bourbon-editor-core` |
| 3 Parity | code-block and table cards and quote bar decorations, hanging indent, folding, image widgets, slash menu, note-link autocomplete and tooltip, smart paste, table editor, alerts, frontmatter, math highlight, fenced-code highlighting with tree-sitter grammars, footnotes, readable line length, unwrap blocks | visual diff against the Inkdrop reference screenshot at identical sizes |
| 4 Polish | incremental reparse, rich HTML paste, math and mermaid rendering, performance work for large notes, multi-cursor and column select | budgets in 6.4 met |

---

## 7. What could not be determined

- Exact @codemirror and @lezer versions (no version strings in the bundle).
- Where the fold gutter is installed (visible in the screenshot, absent from the extension list read in `NP`); the markdown fold ranges are known.
- Exact auto-close pair list for markdown (comes from lang-markdown language data, not duplicated in the bundle).
- Table editor key bindings and the fallback command after Enter (`Wt` in the command list); whether `deleteMarkupBackward` is bound to Backspace.
- How the editor computes the top fractional line (`kn`) and sets scrollTop (`an`); only the preview side was fully read. The description above is the logical equivalent.
- KaTeX asset location, how mermaid diagrams are rendered in the preview (only the `mermaid.css` theme and the language entry were found), and the handler for clicks on `inkdrop://note/...` links in the preview (routing was seen only for empty href and `#anchor`).
- Whether any HTML sanitiser runs elsewhere; none exists in the renderer chain.
- Spacing of the side-by-side divider (none found; both panes `flex: 1`).
- Preview font-size application point (setting `preview.fontSize`, default 15, vs the stylesheet's 16px).
