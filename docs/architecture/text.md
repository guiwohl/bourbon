# bourbon-text

GPUI-free text core of the Markdown editor (Phase 0 of [the UI rebuild](../decisions/2026-11-07-ui-rebuild.md)). State model in the style of CodeMirror 6. Deps: `ropey` (no default features: only `\n` breaks lines), `unicode-segmentation`, `regex` (search). Guarded by `just architecture` (no `gpui*`).

## Model

| Type | Role |
|---|---|
| `Text` | `ropey::Rope` wrapper. Positions are **byte offsets on char boundaries**. `len`, `line_count`, `line_of`, `line_start`, `line_end` (without the break), `line_text`, `column` (chars), `slice`, `char_before/after`, `has_at/has_before` |
| `Change { range, text }` | Replace `range` with `text` |
| `ChangeSet` | Ordered, non-overlapping changes against **one** document version. `apply`, `invert(&before_text)`, `compose(&later)` (later applies to the result of self), `map_position(pos, Assoc)` |
| `Assoc::Before/After` | Side an insertion at exactly `pos` maps to. Inside a replaced range: `Before` goes to its start, `After` to the end of the replacement |
| `SelectionRange { anchor, head }` | `from`, `to`, `is_empty`, `is_reversed`, `map` (cursor maps `After`; ranges shrink away from insertions at their edges) |
| `Selection` | Non-empty ranges plus main index. `new` sorts and merges overlaps (also a cursor touching a range) and keeps the main index |
| `EditorState { text, selection }` | `apply(&Transaction)` |
| `Transaction` | `changes`, `selection_after` (in **new** coordinates), `kind`, `isolate` |
| `TransactionKind` | `Input`, `Delete`, `Paste`, `Command`, `Undo`, `Redo` |
| `History` | `record(&before, &tx, now)`, `undo(&state)`, `redo(&state)` return a `Transaction` for the caller to apply |

## History

- Entry = inverse `ChangeSet` + selection before. Undo restores both; redo uses the inverse of the undo.
- `now` is a `Duration` passed in (no clock inside). Default group window 500 ms, measured from the **last** edit (sliding).
- Only `Input` with `Input` and `Delete` with `Delete` group. `Paste` and `Command` never do. `isolate` breaks grouping on both sides.
- A new edit clears redo. Selection-only transactions, `Undo` and `Redo` are not recorded.
- A group's inverse is `newest_inverse.compose(older_inverse)`.

## Movement (`movement`)

`next/previous_grapheme_boundary`, `next/previous_word_boundary` on a `Text` and byte position. Words follow UAX 29: punctuation segments (`-`, `,`) are their own stops; `foo.bar` is one word. Word moves stop at line ends and cross the line break on the next move.

## Commands (`commands`)

Pure `fn(&EditorState) -> Option<Transaction>` (`None` = nothing to do, so the UI can fall back). Applied to every selection range; changes clamped so they never overlap.

| Command | Behaviour |
|---|---|
| `insert_text(state, text)` | Replaces each range |
| `type_character(state, ch)` | Auto-pair `( [ { " \`` with the caret inside; pairs only before whitespace, end or a closing bracket, and quotes not after a word character; typing a closer over the same closer skips it; with a selection, surrounds it |
| `delete_backward` / `delete_forward` | By grapheme; backspace between a pair deletes both |
| `delete_word_backward` | To the previous word boundary |
| `insert_newline_continue_markup` | Continues `-`, `*`, `+`, `N.`/`N)` (incremented), `- [ ]` (always unchecked), `>` quotes, keeping indentation and spacing; Enter on an empty item removes the marker (inside a quote keeps `> `); plain lines keep indentation; caret inside the marker only breaks the line |
| `indent_list_item` / `outdent_list_item` | Moves the selected items and their children. Indent nests under the previous sibling at its content column (`None` without a sibling, so the UI falls back to generic indent); outdent goes to the nearest shallower item or column 0. Ordered lists in the touched window are renumbered (equal numbers such as `1. 1. 1.` kept, a moved item starts a new nested list at 1). The window stops at blank lines. Quoted list items are not handled |
| `toggle_task` | Per selected line: `[ ]` to `[x]` and back; list item without box gets `[ ] `; other line gets `- [ ] ` |
| `toggle_bold` `**`, `toggle_italic` `_`, `toggle_strikethrough` `~~`, `toggle_inline_code` `` ` ``, `toggle_wrap(mark)` | Unwraps when the marks surround the selection or are inside it, otherwise wraps; selection stays on the inner text; empty selection between doubled marks removes them |

Line-oriented commands (`indent`, `outdent`, `toggle_task`) emit minimal per-line changes and map the selection through them.

Not covered yet: markup-aware Backspace, heading/list/quote toggles, links, code fences, move/copy line, smart paste. Syntax-tree awareness (skipping code blocks) arrives with `bourbon-markdown`.

## Search (`search`)

| Item | Behaviour |
|---|---|
| `SearchQuery { pattern, match_case, regex, whole_word }` | Plain patterns are escaped; case-insensitive unless `match_case`; `^`/`$` work per line |
| `Search::new(&query)` | `Err(InvalidPattern)` for a bad regex; an empty pattern finds nothing |
| `find_all(&text)` | Sorted, non-overlapping byte ranges. Empty matches are skipped; every step moves forward at least one char, so the loop ends. Whole word: no letter, digit or `_` right before or after; a rejected start retries one char later |
| `replace_all(state, replacement)` / `replace_match(state, range, replacement)` | One `Command` transaction, `isolate`d, so each is one undo step. Regex mode expands `$1`, `$name`; plain mode inserts literally. `replace_all` maps the selection; `replace_match` puts the caret after the new text and refuses a range that is not a match |
| `next_match` / `previous_match` / `current_match` | Index into the matches from a `SelectionRange`: next starts at the selection end, previous ends at its start, both wrap; current is the match equal to the selection |

## Tests

Unit tests per module (test helper `testing.rs` parses `|` cursors and `«…»` selections; `»…«` is reversed). Property tests in `crates/bourbon-text/tests/properties.rs` (proptest): apply then invert is identity; composing equals applying in sequence; mapped positions stay in bounds and keep order; undoing N random edits restores text and selection and redoing restores the edited state.
