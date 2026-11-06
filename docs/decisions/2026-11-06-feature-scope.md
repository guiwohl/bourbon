# 2026-11-06 — Feature scope until "complete"

## Product-wide

| # | Topic | Decision |
|---|---|---|
| 1 | Language | **The whole app is English-only** for now |

## In scope

| Area | Feature | Notes |
|---|---|---|
| Editor | Markdown preview | **Three modes switched at the top, as in InkDrop: Edit, Side-by-side, Preview** |
| Editor | Checklists and tables | Clickable `- [ ]` tasks; rendered tables |
| Editor | Note links `[[ ]]` | Typing `[[` suggests notes; clicking opens the note |
| Editor | Find and replace in the note | |
| Editor | Focus mode | Hides the sidebar; text only, centred |
| Editor | Word count | Words and characters |
| Navigation | Control pane | **One centred finder, as in VS Code / Obsidian. `Ctrl+O` opens it to search notes. `Ctrl+P` opens the same pane prefilled with `>`, which switches it to searching actions (settings, commands, everything the app can do)** |
| Navigation | Full-text search | Titles and bodies (SQLite FTS5) |
| Organization | Pin note | Pinned notes stay on top |
| Organization | Note status | Active, on hold, completed, dropped |
| Organization | Move and duplicate | Move to another notebook; duplicate a note |
| Organization | Sorting | By updated, created or title |
| UI | Light and dark themes | **Bourbon's own theme by default, with a setting to follow the active Omarchy theme** |
| UI | Resizable panes | |
| UI | Collapsible sidebar | |
| UI | Settings screen | |
| Data | Restore backup | Pull the database from the backup repo |
| System | `.desktop` file and icon | |
| System | AUR package | |
| System | Single instance | Reopening focuses the existing window |

## Out of scope for now

- Images in notes
- Vim mode
- Importing a Markdown folder or from InkDrop

## To discuss soon

- Release process (versioning, tags, binaries, how AUR consumes them).

## Milestone order

| Milestone | Content |
|---|---|
| M1 Minimal core | Done: create, edit, list, autosave |
| M2 Organization | Notebooks, tags, trash, pin, status, move/duplicate, sorting |
| M3 Navigation | Control pane (`Ctrl+O` notes, `Ctrl+P` actions), full-text search |
| M4 Editor | Preview in three modes, checklists/tables, `[[links]]`, find/replace, focus mode, word count |
| M5 Visual | Themes (own + follow Omarchy), resizable panes, collapsible sidebar, settings, animations |
| M6 Data | GitHub backup, restore, Markdown export |
| M7 Distribution | Release process, AUR, `.desktop` and icon, single instance |
| M8 CLI | `bourbon get <id>` and the copy-command button |
