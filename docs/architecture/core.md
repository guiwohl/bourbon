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
| `Note` | id, title, body, created_at, updated_at, notebook, status, pinned, is_template, tags (`Vec<Tag>`: id, name, colour), trashed_at |
| `NoteSummary` | id, title, excerpt, updated_at, created_at, pinned, status, tags (`Vec<Tag>`: id, name, colour). No body |
| `NoteQuery` | scope + sort; `Default` is `AllNotes` + `UpdatedNewestFirst` |
| `NoteScope` | `AllNotes`, `Notebook(NotebookId, NotebookDepth)`, `Status(NoteStatus)`, `Tags(Vec<TagId>)`, `Templates`, `Trash` |
| `NotebookDepth` | `DirectOnly`, `WithDescendants` |
| `NoteSort` | `UpdatedNewestFirst` (default), `CreatedNewestFirst`, `TitleAlphabetical` |
| `NoteStore::open(path)` | Creates parent dirs and file, enables WAL, migrates |
| `NoteStore::open_in_memory()` | Same, in memory |
| `default_notebook()` | Id of the Inbox, read from settings |
| `create_notebook(name, parent, now)` | Names need not be unique; unknown parent gives `StoreError::NotebookNotFound` |
| `rename_notebook(id, name)` | Unknown id gives `NotebookNotFound` |
| `notebook_tree()` | Roots and children ordered by name, case-insensitive |
| `create_note(notebook, now)` | New empty note, created_at = updated_at = now; unknown notebook gives `NotebookNotFound`. Delegates to `create_note_with` with the default seed |
| `create_note_with(notebook, seed, now)` | Like `create_note`, with `NoteSeed { tags: Vec<TagId>, status }` (default: no tags, `NoteStatus::None`) applied in one transaction. Unknown notebook gives `NotebookNotFound`, unknown tag `TagNotFound`; nothing is written on failure |
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
- `tags(id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE COLLATE NOCASE, color TEXT NOT NULL, created_at INTEGER NOT NULL)`
- `note_tags(note_id TEXT NOT NULL, tag_id TEXT NOT NULL, PRIMARY KEY (note_id, tag_id))`
- indexes on `notes(notebook_id)`, `notes(status)`, `notes(deleted_at)`, `note_tags(tag_id)`

v3 (templates): `ALTER TABLE notes ADD COLUMN is_template INTEGER NOT NULL DEFAULT 0`; existing notes stay ordinary.

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
| `Templates` | Non-trashed templates, any status |
| `Trash` | Only trashed notes, any status, templates included |

Every scope except `Templates` and `Trash` excludes templates.

Order: pinned notes first in every scope, then the sort. `UpdatedNewestFirst`: updated desc, ties created desc. `CreatedNewestFirst`: created desc, ties updated desc. `TitleAlphabetical`: title ascending, ties updated desc; case folding is SQLite `NOCASE`, so it covers ASCII letters only.

`excerpt` is the first non-blank line of the body, trimmed, cut to 100 characters.

`NotebookNode.note_count` counts non-trashed notes in the notebook and all descendants, every status included.

## Save rule

If title and body both equal what is stored, `save_note` is a no-op and `updated_at` is unchanged. Otherwise both are stored and `updated_at = now`.

## Database path

`$XDG_DATA_HOME/bourbon/bourbon.db`; if unset or empty, `$HOME/.local/share/bourbon/bourbon.db`.

## Notebooks

Notebook names are trimmed on create and rename; an empty name is `EmptyNotebookName`. Names need not be unique.

## Tags

Tag names are trimmed; an empty name is `EmptyTagName`. Uniqueness is case-insensitive and enforced by the schema (`UNIQUE COLLATE NOCASE`); the store pre-checks for a friendly error and also maps a unique-constraint violation to `TagNameTaken`. NOCASE folds ASCII letters only, for tag uniqueness and for title sort alike; a tag may be renamed to its own name in another case. Unknown ids give `TagNotFound`.

`list_tags` returns `TagSummary { tag, note_count }` ordered by name ignoring case; `note_count` counts non-trashed notes carrying the tag, any status. `delete_tag` also removes its `note_tags` rows.

`add_new_tag_to_note(note, name, color, now)` creates a tag (same validation as `create_tag`) and attaches it in one transaction, setting the note's `updated_at = now`; any failure writes nothing. Returns the new `Tag`.

`add_tag_to_note` / `remove_tag_from_note` set the note's `updated_at = now` only when they change something; adding a present tag or removing an absent one is a no-op. Unknown note gives `NoteNotFound`.

## Status, pin and duplicate

- `set_status(note, status, now)`: a different status is stored and sets `updated_at = now`; the same status is a no-op.
- `pin_note` / `unpin_note`: pinning is not an edit, `updated_at` never changes; repeating either is harmless.
- `toggle_pin(note) -> bool`: flips the pinned flag in the store and returns the new state; `updated_at` never changes; unknown note gives `NoteNotFound`.
- `duplicate_note(note, now)`: a new note with a new id in the same notebook with the same title, body, status, tags and template flag; not pinned, not trashed, `created_at = updated_at = now`. A trashed source is allowed and yields a live copy. The source is untouched. The note and tag rows are inserted in one transaction.
- All four fail with `NoteNotFound` for an unknown note.

## Trash

`TRASH_RETENTION` is 7 days. Trashing sets `deleted_at` and leaves `updated_at` alone; trashing a trashed note keeps the first `deleted_at`.

| Operation | Behaviour |
|---|---|
| `trash_note(id, now)` | Sets `deleted_at = now`; no-op if already trashed; unknown id is `NoteNotFound` |
| `restore_note(id)` | Clears `deleted_at`; if the note's notebook was deleted it moves to the default notebook; no-op if not trashed |
| `empty_trash()` | Permanently deletes every trashed note and its `note_tags` rows; returns the count |
| `purge_expired_trash(now)` | Same, only for notes trashed at least `TRASH_RETENTION` before `now` (exactly 7 days is purged); returns the count |
| `delete_notebook(id, now)` | One transaction: deletes the notebook and its whole subtree, trashes every non-trashed note inside with `deleted_at = now` (already-trashed notes keep theirs) |

The default notebook cannot be deleted (`DefaultNotebookUndeletable`). The core never purges by itself; the app calls `purge_expired_trash`.

## Templates

A template is a note with `is_template` set. It is listed only by `NoteScope::Templates` (non-trashed, any status, pinned first, then the sort). `AllNotes`, `Notebook`, `Status` and `Tags` never show templates; `Trash` shows trashed templates like any trashed note. `notebook_tree` counts and `list_tags` counts exclude templates. Templates keep a notebook, status, pin and tags like any note; `save_note`, trash and restore work on them unchanged.

| Operation | Behaviour |
|---|---|
| `create_template(notebook, now)` | New empty note flagged as template; unknown notebook gives `NotebookNotFound` |
| `mark_as_template(note)` / `unmark_template(note)` | Sets or clears the flag; not an edit, `updated_at` unchanged; repeating is harmless; unknown note gives `NoteNotFound` |
| `create_note_from_template(template, notebook, now, zone)` | New non-template note in `notebook` with the template's expanded title and body and the same tags; status `None`, not pinned, `created_at = updated_at = now`. One transaction. The template is untouched and may be trashed. Unknown template gives `NoteNotFound`, unknown notebook `NotebookNotFound`, a note that is not a template `NotATemplate(id)` |

The caller chooses the notebook. Variables are expanded in title and body only, with `now` shown in `zone`:

| Variable | Expands to |
|---|---|
| `{{date}}` | `YYYY-MM-DD` |
| `{{time}}` | `HH:MM`, 24 hour |

Matching is exact and case-sensitive; every occurrence is replaced; anything else (`{{Date}}`, `{{ date }}`, `{{name}}`) is left as written. A timestamp outside the supported calendar range leaves the text unexpanded.

`TemplateTimeZone` wraps a `jiff` time zone: `system()` for the app, `utc()` for deterministic callers. The `jiff` dependency is pure Rust and pulls no GPUI crate; its bundled tz database is enabled for tests only, so tests that name a zone do not depend on the host.
