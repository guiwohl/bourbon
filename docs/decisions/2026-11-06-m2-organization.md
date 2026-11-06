# 2026-11-06 — Milestone 2: organization

Rule: follow InkDrop exactly, unless a row says otherwise. Sources: [How to organize notes](https://docs.inkdrop.app/start-guide/how-to-organize-notes), [Organize notebooks](https://docs.inkdrop.app/manual/organize-notebooks), [Note statuses](https://docs.inkdrop.app/reference/note-statuses).

## Layout

| # | Topic | Decision |
|---|---|---|
| 1 | Columns | **Three columns: sidebar, note list, editor** |
| 2 | Sidebar sections | **All Notes, Notebooks (tree), Status, Tags, Trash**. `Ctrl+/` toggles the sidebar |
| 3 | Editor header | **Title; below it the notebook selector, the Status button and the tags field** |

## Notebooks

| # | Topic | Decision |
|---|---|---|
| 4 | Nesting | **Unlimited depth.** Each tree item shows its note count and expands/collapses |
| 5 | Membership | **Every note belongs to exactly one notebook** |
| 6 | Default notebook | **"Inbox", created on first run.** Notes created from All Notes, Status or Tags go there. Changeable from a notebook's context menu ("Use for new notes"). Notes from M1 databases move into Inbox on migration |
| 7 | Viewing | **An expanded notebook lists only its direct notes; a collapsed one also lists all its sub-notebooks' notes** |
| 8 | Moving a note | **Notebook selector, `Ctrl+Alt+M`, or dragging the note onto a notebook in the sidebar** |

## Tags

| # | Topic | Decision |
|---|---|---|
| 9 | Membership | **A note has any number of tags** |
| 10 | Assigning | **Tags field under the title, with autocomplete of existing tags; `Enter` adds** |
| 11 | Colours | **Each tag has a colour** |
| 12 | Filtering | **Clicking a tag filters the list; several selected tags narrow it to notes having all of them** |

## Status

| # | Topic | Decision |
|---|---|---|
| 13 | Values | **None (default), Active, On Hold, Completed, Dropped** |
| 14 | Visibility | **Completed and Dropped are hidden from All Notes, notebooks and tags; shown when picked in the sidebar's Status section and in keyword search** |
| 15 | Shortcuts | **`Shift+Alt+1` to `5`** map to None, Active, On Hold, Completed, Dropped |

## Note list and navigation

| # | Topic | Decision |
|---|---|---|
| 16 | Pinned | **Pinned notes stay at the top of any list they appear in** (InkDrop docs do not cover this; our rule) |
| 17 | Sorting | **By updated (default), created or title** |
| 18 | New note | **Button at the top right of the note list; `Ctrl+N`. The new note is always visible in the current list: in a notebook it goes there; in All Notes it goes to Inbox; in Tags it goes to Inbox with those tags; in a Status it goes to Inbox with that status; from Trash the view switches to All Notes first.** Changing the view keeps the open note open |
| 19 | History | **`Alt+←` / `Alt+→` move back and forward between viewed notes** |
| 20 | Duplicate | **Duplicating creates a copy in the same notebook** |

## Trash

| # | Topic | Decision |
|---|---|---|
| 21 | Deleting | **Soft delete into Trash; purged after 7 days** (foundation decision #9) |
| 22 | Trash view | **Restore a note, or empty the trash** |
| 23 | Deleting a notebook | **The notebook and all its sub-notebooks are removed; every note inside goes to Trash.** A note restored after its notebook is gone lands in Inbox |

## Templates

| # | Topic | Decision |
|---|---|---|
| 24 | What a template is | **A note flagged as template. Listed only under a "Templates" sidebar section; never in All Notes, notebooks, tags, status or trash lists** |
| 25 | Creating templates | **"New template" in the Templates section, or mark/unmark an existing note as template** |
| 26 | Using a template | **"New from template" next to "New note". The new note copies the template's title, body and tags; its notebook follows rule #18 (visible in the current list)** |
| 27 | Variables | **`{{date}}` → `YYYY-MM-DD`, `{{time}}` → `HH:MM` (24h), in the local time zone, expanded in title and body at creation** |
| 28 | Storage | **Schema v3 adds `notes.is_template`** |

