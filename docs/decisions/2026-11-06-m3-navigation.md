# M3 navigation: control pane and full-text search

Decided by Claude during the autonomous run; review at the UI pass.

| # | Decision |
|---|---|
| 1 | Full-text search is SQLite FTS5 (bundled with `rusqlite` `bundled`, no extra feature). Schema v4 adds `notes_fts(title, body)` as an external-content table over `notes`, joined by `rowid`, kept in sync by insert, update (of title, body) and delete triggers. The migration back-fills with `rebuild` |
| 2 | `NoteStore::search_notes(text, limit) -> Vec<SearchHit>`; `SearchHit { summary, snippet }`. Each whitespace-separated token becomes an FTS5 string literal (quotes doubled) with a `*` prefix match; tokens are AND-joined, so no input can be read as FTS syntax |
| 3 | Ranking is `bm25(notes_fts, 10.0, 1.0)` (title weighs ten times the body), ties by newest update. Trashed notes and templates are never found; Completed and Dropped notes are |
| 4 | Blank text lists the `limit` most recently updated notes (not pinned first, any status), snippet = excerpt |
| 5 | The snippet comes from FTS `snippet()` on the body, markers stripped, whitespace collapsed to spaces. If the body has no match (title-only hit) the snippet is the excerpt |
| 6 | One centred overlay, the control pane. `Ctrl+O` opens it empty (Notes mode), `Ctrl+P` opens it with `>` (Actions mode). The mode is decided by the text: a leading `>` is Actions, deleting it is Notes |
| 7 | Notes mode searches all notebooks (limit 50). `Up`/`Down` move the highlight and stop at the ends (like the tag suggestions), `Enter` or a click opens, `Esc` or a click outside closes. Opening flushes the pending save; if the note is not in the current listing the navigation becomes All Notes; then the note is selected |
| 8 | Actions mode is a static list of 21 commands mapped to existing workspace intents. Filter: case-insensitive subsequence of the name (spaces in the query ignored); ranked by shortest span of the best window, then earliest start, then declared order. Actions that need an open note are hidden without one. Key bindings are shown on the right as fixed text |
| 9 | "Empty Trash" switches to Trash first, then opens the existing confirmation dialog, because the dialog counts the listed notes |
| 10 | The workspace root now takes focus at startup even when a note is selected, otherwise no shortcut reaches it before the first click |
| 11 | A search box in the note list, editor features and theming stay out of M3 |
