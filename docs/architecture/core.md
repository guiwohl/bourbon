# bourbon-core

Pure domain plus SQLite. Never depends on GPUI. No repository trait: one storage, tests use an in-memory database.

## Public API

| Item | Behaviour |
|---|---|
| `NoteId::generate()` | 12 random chars from `[a-z0-9]` |
| `NoteId::parse(&str)` | Exactly 12 chars of `[a-z0-9]`, else `InvalidNoteId` |
| `Timestamp` | Unix milliseconds (`now`, `from_millis`, `as_millis`) |
| `Note` | id, title, body, created_at, updated_at |
| `NoteSummary` | id, title, updated_at (no body) |
| `NoteStore::open(path)` | Creates parent dirs and file, enables WAL, migrates |
| `NoteStore::open_in_memory()` | Same, in memory |
| `create_note(now)` | New empty note, created_at = updated_at = now |
| `save_note(id, title, body, now)` | See save rule; unknown id gives `StoreError::NoteNotFound` |
| `find_note(id)` | `Option<Note>` |
| `list_notes()` | By updated_at desc, ties by created_at desc |
| `default_database_path()` | See database path |

## Schema

Versioned by `PRAGMA user_version`; an ordered list of migrations is applied past the stored version. A stored version above the known migrations fails the open with `StoreError::SchemaTooNew`.

v1: `notes(id TEXT PRIMARY KEY, title TEXT NOT NULL, body TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL)`

## Save rule

If title and body both equal what is stored, `save_note` is a no-op and `updated_at` is unchanged. Otherwise both are stored and `updated_at = now`.

## Database path

`$XDG_DATA_HOME/bourbon/bourbon.db`; if unset or empty, `$HOME/.local/share/bourbon/bourbon.db`.
