# 2026-11-05 — Project foundation

Decisions taken in the initial design session. This document is the starting point; later decisions live in their own files in this folder.

## Vision

Bourbon is a Markdown notes app in the spirit of InkDrop: opinionated, open source, clean to the bone and extremely pleasant to use. It ships with a CLI that AI agents can call to read notes.

## Decisions

| # | Topic | Decision | Reason |
|---|---|---|---|
| 1 | Name | **Bourbon** | Classic-spirit vibe; free on crates.io and AUR as of 2026-11-05 |
| 2 | Platform | **Linux only** | Minimal scope; Wayland and X11 |
| 3 | Language and UI | **Rust + GPUI** (Zed's framework) | Zed-level fluidity and GPU rendering. Same path as the official Herdr app |
| 4 | Components | **`gpui-component` (longbridge/gpui-kit) mixed with our own components** | The library provides an editor (Tree-sitter), Markdown rendering, virtual lists, docking and themes. Our own components go where the visual identity lives. All of them read the same theme |
| 5 | Animations | **GPUI's native API: `with_animation` and `with_spring`** | macOS-style spring physics. Layout or shared-element transitions, if needed, will be built by hand |
| 6 | Storage | **SQLite as the source of truth** | Users should not edit notes outside the app |
| 7 | Export | **Export as a folder of `.md` files** | Data is never locked inside the app |
| 8 | Saving | **Autosave only, 0.5s after typing stops.** There is no manual save (Ctrl+S style) | An "unsaved" state never exists |
| 9 | Trash | **Soft delete (`deleted_at`), permanently purged after 7 days** | Simple recovery; the trash travels with the backup |
| 10 | Backup | **Dedicated GitHub repository holding only the current database** (a single commit, rewritten and sent with `push --force`) | Old versions have no use, and the repo does not grow |
| 11 | Backup snapshot | **`VACUUM INTO` produces a consistent copy; `PRAGMA integrity_check` runs before the push and aborts on failure** | Copying the file with WAL active can corrupt it. The check keeps a corrupted database from overwriting the good backup |
| 12 | Backup triggers | **Automatic after 5 min without edits, plus one when the app closes; runs only if something changed.** Also manual from the "⋯" menu and the CLI | Nobody has to remember, and history stays free of noise |
| 13 | Backup authentication | **The local `gh` token or a user-provided fine-grained PAT** | The PAT only needs `Contents: read/write` on the chosen repo |
| 14 | CLI for agents | **Reads the requested note and writes its current content to stdout, and nothing else** (no clipboard, no file copy) | With the 0.5s autosave, the database reflects the state of the moment; no side effects |
| 15 | Note identifier | **ID only, fixed for the note's lifetime; no lookup by name.** Each note in the app offers a one-click copy of the full CLI command (e.g. `bourbon get <id>`) | No ambiguity for agents; the user never types the ID by hand |
| 16 | ID format | **12 random lowercase alphanumeric characters** (e.g. `k7f3q2m9x4ta`) | Clean in the UI and in commands; collisions are negligible at personal scale. UUIDv7 (36 chars) and ULID (26) were rejected as needlessly long |
| 17 | CLI binary name | **`bourbon`, with no alias** | One name to remember and package; agents copy the command from the app anyway |
| 18 | Organization | **Nested notebooks (folders) + tags**, as in InkDrop | Folders give structure; tags cut across it |
| 19 | Export format | **Each notebook becomes a folder, each note a `<title-slug>.md` file. Frontmatter carries `id`, `title`, `tags`, `created`, `updated`.** Inside the app, duplicate titles are allowed, since the ID identifies the note. On export, notes with the same title in the same folder are numbered by age: the oldest keeps `<title-slug>.md`, the next become `<title-slug>-1.md`, `<title-slug>-2.md` and so on. Notes in the trash are not exported | Mirrors the app's structure; the output is readable by any Markdown tool |

## Out of scope for now

- **Cross-machine sync.** Backup covers a single machine. When sync comes back to the table, the preferred option is an append-only operation log per device, so git never conflicts.
- Mobile, macOS and Windows.

## References

- [Inkdrop Desktop v6.0.0](https://forum.inkdrop.app/t/inkdrop-desktop-v6-0-0/5571): Electron, CodeMirror 6, SQLite
- [herdrdev/herdr-app](https://github.com/herdrdev/herdr-app): Rust + GPUI + gpui-kit
- [longbridge/gpui-kit](https://github.com/longbridge/gpui-kit)
- [GPUI animation.rs](https://github.com/zed-industries/zed/blob/main/crates/gpui/src/elements/animation.rs)
