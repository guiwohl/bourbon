# 2026-11-06 — Milestone 1: minimal core

## Scope

Open the GPUI window; create, edit and list notes; autosave to SQLite. Nothing else: no notebooks, tags, trash, backup, export, search or CLI.

## Decisions

| # | Topic | Decision | Reason |
|---|---|---|---|
| 1 | Note title | **A separate title field above the editor**, as in InkDrop | Explicit title; the body stays clean |
| 2 | Storage access | **`NoteStore`, a concrete type over SQLite; no repository trait** | One storage and no second adapter in sight. Tests use in-memory SQLite, not mocks |
| 3 | Unchanged save | **Saving an identical title and body is a no-op; `updated_at` does not move** | Autosave fires constantly; list order must reflect real edits only |
| 4 | Listing | **Notes listed by `updated_at` descending** | Most recently edited first |
| 5 | Database path | **`$XDG_DATA_HOME/bourbon/bourbon.db`, falling back to `~/.local/share/bourbon/bourbon.db`** | XDG standard; shared by app and future CLI |
| 6 | Time | **The caller passes `now` into every write** | Deterministic tests without a fake clock |
| 7 | SQLite build | **`rusqlite` with the `bundled` feature** | Same SQLite version everywhere, CI included |
| 8 | Doc-comment lints | **Clippy `missing_errors_doc` and `missing_panics_doc` allowed** | They demand doc comments, which the project forbids |

## Deferred

- SQLite `busy_timeout` for concurrent writers: needed only once the CLI shares the database file. Set it in the CLI milestone.
