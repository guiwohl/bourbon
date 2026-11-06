# bourbon-app

GPUI window over `bourbon-core`. Holds no business rule: it presents state and forwards intent. Built on `gpui-kit` 0.9 with `tree-sitter-markdown` (re-exports GPUI and gpui-component).

## Behaviour

| Topic | Behaviour |
|---|---|
| Window | Title "Bourbon", Wayland `app_id` "bourbon" |
| Startup | Opens `default_database_path()`; failure prints a message and exits 1. Purges expired trash once (errors go to the banner), loads the sidebar data and the list, selects the most recent note |
| Layout | Three columns: sidebar (240 px) \| note list (320 px) \| editor (flex). Fixed widths; resizing is M5 |
| Sidebar | Rows: All Notes, the notebook tree flattened with indentation, a Status header (Active, On Hold, Completed, Dropped), a Tags header (one row per tag), Trash. The row of the current navigation is highlighted. Clicking sets the navigation and reloads the list |
| Note list | Header with a "New note" button (top right); rows show the title (or "Untitled") and the excerpt; the selected row is highlighted; clicking selects |
| Editor | Title input and Markdown editor (soft wrap, no line numbers). Empty state: "No notes yet" with a "New note" button |
| Navigation | `Navigation { AllNotes, Notebook, Status, Tags, Trash }` plus the current sort form a `Listing`; `Listing::query()` maps it to a core `NoteQuery` (table below) |
| New note | Button or `Ctrl+N`: flush, then create so the note is visible in the current list: `Notebook(id, _)` creates in that notebook; `AllNotes` in `default_notebook()`; `Tags(ids)` in `default_notebook()` seeded with every tag; `Status(s)` in `default_notebook()` seeded with status `s` (both through `create_note_with` and a `NoteSeed`, one transaction); `Trash` switches to `AllNotes` first, then as `AllNotes`. Then reload the list and the sidebar, select, focus the title. The root view tracks its own focus handle, focused at startup when no note is selected, so `Ctrl+N` works with nothing else focused |
| Autosave | Any title or body change restarts one pending 500 ms task; when it fires: `save_note`, then reload the list only. No manual save |
| Flush | Every intent that must not lose the open edit runs through `Workspace::flush_then(action)`: flush, report on failure and stop, else run the action. After a change, `Workspace::refresh_listing()` reloads the list and the sidebar data and notifies. Selecting or creating a note first saves the current one immediately; if that save fails the switch is cancelled. Changing the navigation does not flush: the open note stays open |
| Close | `on_window_should_close` flushes the pending save while the view is alive. Success closes the window; failure keeps it open with the banner shown |
| Quit | `on_app_quit` flushes synchronously for quits that happen while the window is open. On Linux, closing the last window quits after the window and view are dropped, so the close hook is the one that matters there |
| Errors | `save_selected` and `flush_pending_save` return `Result<(), StoreError>`; the error is reported once at the boundary (autosave timer, select, create, close, quit). The banner is rendered once by the root view, above the columns, until the next successful save |
| Sidebar reload | The sidebar (notebook tree, tag counts) reloads on startup and on create only; saving does not touch it. Later slices (move, trash, tag changes) add their own reloads |
| Save vs reload | Only a save failure blocks switching, creating or closing. A failed list or sidebar reload is reported but blocks nothing |
| Loading a note | `set_value` emits no `Change` event, so loading never schedules a save |

### Navigation to query

| Navigation | `NoteScope` |
|---|---|
| `AllNotes` | `AllNotes` |
| `Notebook(id, depth)` | `Notebook(id, depth)`, `depth` is `DirectOnly` or `WithDescendants` |
| `Status(status)` | `Status(status)` |
| `Tags(ids)` | `Tags(ids)` |
| `Trash` | `Trash` |

The sort is the listing's own field, default `UpdatedNewestFirst`.

## Modules

Each column is a module owned by one concern. Columns render from data the workspace passes in and report intent through `cx.listener` callbacks on `Workspace`; no column touches `NoteStore`.

| File | Responsibility |
|---|---|
| `main.rs` | Opens the store, aborts on error |
| `window.rs` | GPUI application, key binding, window options, the close hook |
| `autosave.rs` | The 500 ms delayed task |
| `navigation.rs` | `Navigation`, `Listing`, the pure mapping to `NoteQuery` |
| `workspace.rs` | Root view: owns `NoteStore`, listing, selection, autosave and flush, trash purge, error banner state; lays out the three columns |
| `dragged_note.rs` | `DraggedNote` drag payload and preview, built from a `NoteSummary` |
| `presentation.rs` | Labels and colours shared by the columns: `UNTITLED`, `display_title`, `status_label`, `chip_color` (`TagColor` to `ColorName`, note list and editor header chips), `dot_color`, `color_label`, `is_current_color` (sidebar) |
| `sidebar/mod.rs` | `render(SidebarData, cx)`: navigation rows, notebook tree flattening, selection rule |
| `note_list/mod.rs` | `render(notes, selected, cx)`: header with the New note button, note rows |
| `editor/mod.rs` | `EditorPane` (title input, Markdown editor, load/read/focus, change subscriptions), empty state, error banner element |

## Tests

Headless, through `gpui-kit` `test-support` (`#[gpui_kit::test]`, `TestAppContext`): the workspace takes a `NoteStore` (in memory in tests), clicks and typing go through real element ids, and the clock is advanced with `advance_clock`. They cover startup selection, autosave timing (499 ms vs 500 ms, restart on keystroke), flush on switch, creation, and the empty state. Also: close flush (through `simulate_close`, which runs the real `on_window_should_close` handler registered by `window::build_workspace`), refused close on a failed save, quit flush (`App::shutdown`, which runs the `on_app_quit` handlers), pending autosave cancelled on switch, banner shown and cleared, failed save blocking a switch, and `Ctrl+N` through the real key binding. Navigation tests click real sidebar row ids (`nav-notebook-<id>`, `nav-trash`, ...) and check the listed titles, the notebook a created note lands in, and the startup trash purge. A save is made to fail honestly by selecting a note id the store does not hold.

## Dependency guard

`just architecture` fails if `cargo tree -p bourbon-core -e normal --prefix none` lists any crate starting with `gpui`, so it cannot go stale as GPUI crates are added. Proven by adding `gpui-pre-util` to `bourbon-core`: the recipe failed; reverted, it passed. `deny.toml` bans `gpui-kit`, `gpui-component`, `gpui-base` and `gpui-pre`, each with `wrappers` listing its exact direct parents (from `cargo tree -i`). Only `bourbon-app` may depend on `gpui-kit`, the entry point of the whole chain. Proven by adding `gpui-kit` to `bourbon-core`: `error[banned]`. `just deny` also prints many `duplicate` warnings from GPUI; they are expected.

## Sidebar

Implements decision rows 2, 4, 7, 11, 12 and 23 of [M2 organization](../decisions/2026-11-06-m2-organization.md). Code: `sidebar/` (rendering, pure helpers, state types) and `workspace/sidebar_intents.rs` (every `impl Workspace` method the sidebar calls).

| Topic | Behaviour |
|---|---|
| Sections | All Notes, Notebooks (header with a "+" button creating a top-level notebook), the notebook tree, Status (Active, On Hold, Completed, Dropped), Tags, Trash |
| Tree | Depth-first rows indented per depth; each row shows name and `NotebookNode::note_count`. Rows with children have a disclosure triangle (`toggle-notebook-<id>`) that expands/collapses without navigating |
| Expansion | `SidebarState::expanded` (set of `NotebookId`), all collapsed at startup, lives in the workspace |
| Depth | `Navigation::Notebook(id, depth)`. Selecting an expanded notebook uses `DirectOnly`; a collapsed one `WithDescendants`. Toggling the disclosure of the selected notebook re-applies the depth and reloads the list |
| Notebook menu | Right click: New sub-notebook (also expands the parent), Rename, Delete. The Inbox (`default_notebook()`, read once at startup) has no Delete |
| Tag rows | Colour dot (fixed palette in `presentation.rs`, independent of the theme), name, `TagSummary::note_count`. Click selects the tag alone; `Ctrl+click` toggles it in the `Navigation::Tags` selection (notes having all of them); removing the last tag goes to All Notes |
| Tag menu | Rename, Change colour (submenu of the 14 `TagColor` values, current one checked), Delete |
| Prompt | One inline panel at the top of the sidebar (`SidebarState::prompt`): a name field with Save/Cancel (Enter also saves) for create and rename, a question with Delete/Cancel for deletions ("Delete notebook X? Its notes move to Trash."). No dialog: the dialog layer needs `Root` as the window root view, which the app does not use. A core error (empty name, duplicate tag) goes to the banner and the prompt stays open; success clears the banner and the prompt |
| After a mutation | `refresh_listing` once (sidebar data and list). Deleting a notebook first flushes the pending save, and afterwards deselects the open note if it is now trashed or gone (empty state). Creating a sub-notebook re-applies the parent's depth to the navigation without reloading, so the single reload covers it. Deleting a notebook whose subtree contains the current notebook, or deleting a selected tag, adjusts the navigation first (All Notes, or the remaining tags) |
| `Ctrl+/` | `ToggleSidebar` action bound in `window::bind_keys`; `SidebarState::is_visible`, the other two columns take the space |
| Drop target | Notebook rows accept a dragged note (`on_drop::<DraggedNote>`) and highlight with the theme `drop_target` colour while one is over them. The drop calls `Workspace::drop_note_on_notebook` |
| Not in this slice | Sidebar sorting, templates |

Tests (`workspace/tests/sidebar.rs`) drive real elements: row and disclosure clicks, `Ctrl+click`, right click plus keyboard choice in the popup menu, typing into the prompt, and `Ctrl+/` through the key binding. Counts are checked from painted text inside each row's bounds.

## Note list

`note_list::render(NoteListData, cx)` draws the second column from data the workspace passes in; every action is an intent method on `Workspace`, defined in `workspace/note_list_intents.rs`. The column never touches `NoteStore`.

| File | Responsibility |
|---|---|
| `note_list/mod.rs` | Column layout, the "No notes" empty state, selection rule |
| `note_list/header.rs` | View title, sort selector (Updated, Created, Title), "New note" or, in Trash, "Empty trash" |
| `note_list/row.rs` | Row: title (or `UNTITLED`), excerpt, short date, pin icon, tag chips, context menu |
| `note_list/title.rs` | Pure view title: "All Notes", notebook name, status label (from `presentation.rs`), tag names, "Trash" |
| `note_list/date.rs` | Pure `short_date(updated, now, zone)`: `14:30` today, `Nov 6` this year, `Nov 6, 2025` otherwise; the app passes `jiff::tz::TimeZone::system()` |
| `workspace/note_list_intents.rs` | `sort_by`, `toggle_pin`, `duplicate`, `move_to_trash`, `restore`, `confirm_empty_trash`, `empty_trash` |

### Behaviour

- The sort selector sets `Listing::sort` and reloads; pinned notes stay on top because the core guarantees it (decision 16).
- The context menu (right click) outside Trash offers Pin or Unpin, Duplicate, Move to Trash; in Trash only Restore.
- Every change goes through one path (`change_notes`): `flush_then`, call the core, `refresh_listing`, report a store error in the banner. Pin uses the core `toggle_pin`; the app does not read its list to pick a direction. A tag chip click calls the sidebar's single-tag selection (`select_tag`).
- Duplicate selects the copy. Move to Trash on the selected note selects the next row, else the previous, else nothing (editor empty state); on another row the selection is untouched.
- "Empty trash" opens an alert dialog ("Permanently delete N notes?"); only OK calls `empty_trash`. A selected note that was purged is deselected.
- A tag chip navigates to that single tag. Its mouse-down stops propagation so the row is not selected.
- Rows outside Trash are draggable (`on_drag` with a `DraggedNote { id, title }`, defined in `dragged_note.rs`; the preview is a compact chip with the title, `UNTITLED` when empty). Dropped on a sidebar notebook row, the note moves through `move_note_to`, the same path as the notebook selector and `Ctrl+Alt+M`: `flush_then`, `move_note`, `refresh_listing`. The header is reloaded only when the dropped note is the open one; the open note stays open. Trash rows are not draggable.
- Tag colours map from `TagColor` to the component palette (`ColorName`) in `presentation.rs`, shared with the editor header chips.

### Tests

`workspace/tests/drag.rs`: real drags (`drag_to` from `note-<id>` to `nav-notebook-<id>`) check the move, the open note staying open, the list refresh, the header, the pending-edit flush and Trash rows not being draggable.

`workspace/tests/note_list.rs`: the context menu and the sort dropdown are driven through the real popup (right click on `note-<id>`, then `down`/`enter` keystrokes; menu labels are checked with `find_by_label`), the dialog through `enter` and `escape`, chips through `chip-<note>-<tag>`, pins through `pin-<note>`. Date, title and prompt text are pure unit tests.

## Editor header

Rows 3, 8, 10, 13, 15 and 19 of the [M2 decisions](../decisions/2026-11-06-m2-organization.md). Rendered above the Markdown editor, under the title, only while a note is open. All logic is in `workspace/editor_intents.rs`; `editor/` only draws and keeps view state.

| Part | Behaviour |
|---|---|
| Notebook selector | Button (id `notebook-selector`) labelled with the note's notebook; opens a popover listing every notebook, depth-first, indented by depth (rows `notebook-option-<id>`, the note's notebook has `aria_selected`). Choosing one calls `move_note` and closes the popover. The open state is the editor's own (`Popover::open` controlled), so `Ctrl+Alt+M` can open it |
| Status button | Button (id `status-button`) labelled with the current status; dropdown menu None, Active, On Hold, Completed, Dropped, current one checked. Each item dispatches its `SetStatus*` action, the same actions the shortcuts use, so button and keys share one path to `set_status` |
| Tags field | One removable chip per tag, coloured like the note list chips (`tag-chip-<id>`, × is `remove-tag-<id>` → `remove_tag_from_note`), then an "Add tags" input (`tag-input`) |
| Autocomplete | While typing, rows `tag-suggestion-<id>` (the highlighted one has `aria_selected`) list existing tags, not already on the note, whose name contains the typed text (case-insensitive). `Down`/`Up` move the highlight (captured before the single-line input swallows them); clicking a row adds it |
| `Enter` | Highlighted suggestion if any; else the existing tag named exactly as typed (case-insensitive, trimmed); else `add_new_tag_to_note(name, TagColor::Default)` (create and attach in one transaction). Blank input does nothing. The input is cleared on success |
| Every header action | Flush the pending save, call core, reload the note into the header, then reload the sidebar data and the list. The note may leave the current list; it stays open. A failure goes to the banner and the input keeps its text |

### Shortcuts

Bound in `window::bind_keys` with no key context, so they work from any focus.

| Keys | Action |
|---|---|
| `Shift+Alt+1` … `5` | Status None, Active, On Hold, Completed, Dropped |
| `Ctrl+Alt+M` | Open the notebook selector (no-op without an open note) |
| `Alt+Left` / `Alt+Right` | History back / forward |

On Linux `gpui-base` binds word movement to `Ctrl+Left/Right`, so `Alt+Left/Right` is free. On macOS it binds `Alt+Left/Right` inside inputs and would shadow these.

### History

`editor::History` holds the viewed note ids and a position. `Workspace::show` calls `visit`: a note equal to the current entry adds nothing; otherwise forward entries are dropped and the note is pushed. Back and forward flush, then step to the nearest entry whose note exists and is not trashed, and call `show`, which finds the current entry already in place and pushes nothing. With nothing usable in that direction nothing happens.

### Tests

`workspace/tests/editor.rs`: moving through the real selector, status through the real menu (`popup-menu` items) and `Shift+Alt+1`…`5`, `Ctrl+Alt+M`, chips, suggestions, keyboard and click picking, create-on-Enter, chip removal, flush before each action, banner on failure, history across three notes including a trashed and a purged one, and `History` itself (forward entries dropped).
