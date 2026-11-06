# Spec status

Audit of [feature scope](../decisions/2026-11-06-feature-scope.md), [project foundation](../decisions/2026-11-05-project-foundation.md) and every `2026-11-06-m*.md` decision against the code. Status: done, partially, missing. "Tests" means the behaviour is covered by tests in the same crate.

## Feature scope

| Item | Status | Where |
|---|---|---|
| Markdown preview, three modes (Edit, Side-by-side, Preview), persisted | done | `editor/toolbar.rs`, `workspace/writing_intents.rs`, `bourbon-core/src/settings.rs` (`ViewMode`) |
| Checklists (clickable) and tables | done | `editor/tasks.rs`, `editor/preview_blocks.rs`, `editor/preview.rs` |
| Note links `[[ ]]`, suggestions from `search_notes`, click opens | done | `editor/link_picker.rs`, `editor/link_trigger.rs`, `bourbon-core/src/note_link.rs`, `workspace/writing_intents.rs` |
| Find and replace in the note | done | `workspace/writing_intents.rs`, `workspace/tests/find.rs` |
| Focus mode | done | `workspace/tests/focus.rs`, `editor/mod.rs` |
| Word count | done | `editor/word_count.rs` |
| Control pane (`Ctrl+O` notes, `Ctrl+P` actions) | done | `control_pane/`, `workspace/control_pane_intents.rs` |
| Full-text search (FTS5, titles and bodies) | done | `bourbon-core/src/store/search.rs` |
| Pin note | done | `bourbon-core/src/store/note_state.rs`, `note_list/row.rs` |
| Note status (5 values, `Shift+Alt+1..5`) | done | `bourbon-core/src/status.rs`, `window.rs` |
| Move and duplicate | done | notebook selector, `Ctrl+Alt+M`, drag (`dragged_note.rs`), control pane "Duplicate note" |
| Sorting (updated, created, title) | done | `bourbon-core/src/query.rs`, `note_list/header.rs` |
| Light and dark themes, Bourbon palette and follow Omarchy | done | `theme/`, `bourbon-core/src/omarchy.rs` |
| Resizable panes | done (no real mouse test) | `panes.rs`, `workspace/settings_intents.rs` |
| Collapsible sidebar | done | `sidebar/mod.rs`, `Ctrl+/` |
| Settings screen | done | `settings_view.rs` |
| Restore backup | done | `backup_runner.rs`, `workspace/data_intents.rs`, `bourbon-core/src/backup.rs` |
| `.desktop` file and icon | done | `packaging/bourbon.desktop`, `assets/icons/hicolor` |
| AUR package | partially | `packaging/aur/PKGBUILD`, `.SRCINFO`; checksum is a placeholder until the first release |
| Single instance | done | `single_instance.rs`, `window.rs` |
| Release process | missing (to discuss) | proposal in [release.md](release.md), `.github/workflows/release.yml` |
| English-only app | done | all UI strings |

## Milestones

| Milestone | Status | Notes |
|---|---|---|
| M1 Minimal core | done | create, edit, list, autosave |
| M2 Organization | done | notebooks, tags with colours, trash (7-day purge at startup), pin, status, templates with `{{date}}`/`{{time}}`, history `Alt+Left/Right`, default notebook changeable from the notebook menu |
| M3 Navigation | done | see M3 decision rows 1-10; row 11 (search box in the note list) is intentionally out |
| M4 Editor | done | see feature rows above |
| M5 Visual | partially | functional part done; animations deliberately left to the UI pass |
| M6 Data | done | GitHub backup (manual, 5 min after edits, on close), restore with safety copy, Markdown export |
| M7 Distribution | partially | see AUR and release rows |
| M8 CLI | done | `bourbon get <id>`, `bourbon` launches the app, "Copy CLI command" in the editor menu and control pane |

## Foundation decisions

| # | Topic | Status | Where |
|---|---|---|---|
| 1-5 | Name, Linux, Rust + GPUI, components | done | workspace layout |
| 5 | Animations through `with_animation`/`with_spring` | missing | none yet; UI pass |
| 6 | SQLite source of truth | done | `bourbon-core/src/store.rs` |
| 7, 19 | Markdown export with frontmatter and numbering | done | `bourbon-core/src/export/` |
| 8 | Autosave 0.5 s, no manual save | done | `autosave.rs` |
| 9 | Soft delete, purge after 7 days | done | `store/trash.rs` |
| 10-13 | Backup: single forced commit, `VACUUM INTO` plus integrity check, triggers, `gh` token or PAT | done | `bourbon-core/src/backup/`, `backup_runner.rs` |
| 14-17 | CLI reads one note to stdout, ID only, 12 chars, binary `bourbon` | done | `bourbon-cli/`, `bourbon-core/src/identifier.rs` |
| 18 | Nested notebooks and tags | done | `store/notebooks.rs`, `store/tags.rs` |
| 20 | GPL-3.0-only | done | workspace `Cargo.toml` |

## Out of scope (not built, as decided)

Images in notes, Vim mode, importing a Markdown folder or from InkDrop, cross-machine sync, search box in the note list.

## Known gaps for the UI pass

- Pane dividers are tested by calling the drag intents, not with a real mouse drag.
- The preview link click and the 720 px focus column are verified only indirectly (action and layout tests, no pixel check).
- Key hints in the control pane are static text, not read from the key bindings.
- Animations (M5, foundation 5) are not started.
- Untitled notes appear in `[[` suggestions only while nothing is typed after `[[`, because search matches text.
- Release process (versioning, tags, binaries, AUR checksum) waits for the user's decision.
