# bourbon-core

Pure domain plus SQLite. Never depends on GPUI. No repository trait: one storage, tests use an in-memory database.

## Public API

| Item | Behaviour |
|---|---|
| `NoteId`, `NotebookId`, `TagId` | Distinct types sharing one rule: 12 random chars of `[a-z0-9]`. `generate()`, `parse(&str)` (else `InvalidIdentifier`), `as_str()`, `Display`. Stored as TEXT |
| `Timestamp` | Unix milliseconds (`now`, `from_millis`, `as_millis`) |
| `NoteStatus` | `None` (default), `Active`, `OnHold`, `Completed`, `Dropped`; stored `none`, `active`, `on_hold`, `completed`, `dropped`. `is_hidden_by_default()` is true for Completed and Dropped. `ALL` lists every value |
| `TagColor` | `Default`, `Red`, `Orange`, `Yellow`, `Olive`, `Green`, `Teal`, `Blue`, `Violet`, `Purple`, `Pink`, `Brown`, `Grey`, `Black`; stored lowercase. `ALL` lists every value |
| `Tag` | id, name, color |
| `Notebook` | id, name, parent (`None` = top level), created_at |
| `NotebookNode` | notebook, note_count, children |
| `Note` | id, title, body, created_at, updated_at, notebook, status, pinned, tags (by name), trashed_at |
| `NoteSummary` | id, title, excerpt, updated_at, created_at, pinned, status, tags (by name). No body |
| `NoteQuery` | scope + sort; `Default` is `AllNotes` + `UpdatedNewestFirst` |
| `NoteScope` | `AllNotes`, `Notebook(NotebookId, NotebookDepth)`, `Status(NoteStatus)`, `Tags(Vec<TagId>)`, `Trash` |
| `NotebookDepth` | `DirectOnly`, `WithDescendants` |
| `NoteSort` | `UpdatedNewestFirst` (default), `CreatedNewestFirst`, `TitleAlphabetical` |
| `NoteStore::open(path)` | Creates parent dirs and file, enables WAL, migrates |
| `NoteStore::open_in_memory()` | Same, in memory |
| `default_notebook()` | Id of the Inbox, read from settings |
| `create_notebook(name, parent, now)` | Names need not be unique; unknown parent gives `StoreError::NotebookNotFound` |
| `rename_notebook(id, name)` | Unknown id gives `NotebookNotFound` |
| `notebook_tree()` | Roots and children ordered by name, case-insensitive |
| `create_note(notebook, now)` | New empty note, created_at = updated_at = now; unknown notebook gives `NotebookNotFound` |
| `save_note(id, title, body, now)` | See save rule; unknown id gives `StoreError::NoteNotFound` |
| `move_note(note, notebook, now)` | Sets notebook and `updated_at = now`; unknown note or notebook gives the matching error |
| `find_note(id)` | `Option<Note>`; also finds trashed notes |
| `list_notes(query)` | See listing rules |
| `default_database_path()` | See database path |

## Schema

Versioned by `PRAGMA user_version`; an ordered list of migrations is applied past the stored version, each in a transaction. A stored version above the known migrations fails the open with `StoreError::SchemaTooNew`. Migrations are append-only: never edit an applied one.

v1: `notes(id TEXT PRIMARY KEY, title TEXT NOT NULL, body TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL)`

v2 (milestone 2):

- `notebooks(id TEXT PRIMARY KEY, name TEXT NOT NULL, parent_id TEXT, created_at INTEGER NOT NULL)`
- `settings(key TEXT PRIMARY KEY, value TEXT NOT NULL)`; key `default_notebook_id` holds the Inbox id
- the Inbox notebook is inserted with a random id (`lower(hex(randomblob(6)))`) and the migration time
- `notes` rebuilt (create, copy, drop, rename) with `notebook_id TEXT NOT NULL` (existing rows go to Inbox), `status TEXT NOT NULL DEFAULT 'none'`, `pinned INTEGER NOT NULL DEFAULT 0`, `deleted_at INTEGER` (NULL = not in trash)
- `tags(id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE, color TEXT NOT NULL, created_at INTEGER NOT NULL)`
- `note_tags(note_id TEXT NOT NULL, tag_id TEXT NOT NULL, PRIMARY KEY (note_id, tag_id))`
- indexes on `notes(notebook_id)`, `notes(status)`, `notes(deleted_at)`, `note_tags(tag_id)`

No foreign keys: a trashed note may point at a deleted notebook, so integrity is the core's job.

## Listing rules

`list_notes` is the single listing query. Parameterised SQL only, built from fixed fragments; tags of all listed notes are loaded in one extra query.

| Scope | Notes shown |
|---|---|
| `AllNotes` | Not trashed, status not Completed/Dropped |
| `Notebook(id, DirectOnly)` | Same visibility, notes directly in the notebook |
| `Notebook(id, WithDescendants)` | Same visibility, notes in the notebook and its whole subtree (recursive CTE) |
| `Tags(ids)` | Same visibility, notes having all the given tags; repeated ids count once; an empty list adds no restriction |
| `Status(status)` | Not trashed, exactly that status (so Completed and Dropped are reachable here) |
| `Trash` | Only trashed notes, any status |

Order: pinned notes first in every scope, then the sort. `UpdatedNewestFirst`: updated desc, ties created desc. `CreatedNewestFirst`: created desc, ties updated desc. `TitleAlphabetical`: title ascending, ties updated desc; case folding is SQLite `NOCASE`, so it covers ASCII letters only.

`excerpt` is the first non-blank line of the body, trimmed, cut to 100 characters.

`NotebookNode.note_count` counts non-trashed notes in the notebook and all descendants, every status included.

## Save rule

If title and body both equal what is stored, `save_note` is a no-op and `updated_at` is unchanged. Otherwise both are stored and `updated_at = now`.

## Database path

`$XDG_DATA_HOME/bourbon/bourbon.db`; if unset or empty, `$HOME/.local/share/bourbon/bourbon.db`.

## Tags

Tag names are trimmed; an empty name is `EmptyTagName`. Uniqueness is case-insensitive (`TagNameTaken`); a tag may be renamed to its own name in another case. Unknown ids give `TagNotFound`.

`list_tags` returns `TagSummary { tag, note_count }` ordered by name ignoring case; `note_count` counts non-trashed notes carrying the tag, any status. `delete_tag` also removes its `note_tags` rows.

`add_tag_to_note` / `remove_tag_from_note` set the note's `updated_at = now` only when they change something; adding a present tag or removing an absent one is a no-op. Unknown note gives `NoteNotFound`.
||||||| parent of e336d89 (feat(core): add note status, pin and duplicate)

## Status, pin and duplicate

- `set_status(note, status, now)`: a different status is stored and sets `updated_at = now`; the same status is a no-op.
- `pin_note` / `unpin_note`: pinning is not an edit, `updated_at` never changes; repeating either is harmless.
- `duplicate_note(note, now)`: a new note with a new id in the same notebook with the same title, body, status and tags; not pinned, not trashed, `created_at = updated_at = now`. A trashed source is allowed and yields a live copy. The source is untouched. The note and tag rows are inserted in one transaction.
- All four fail with `NoteNotFound` for an unknown note.
