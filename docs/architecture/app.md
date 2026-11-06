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
| New note | Button or `Ctrl+N`: flush, then create so the note is visible in the current list: `Notebook(id)` creates in that notebook; `AllNotes` in `default_notebook()`; `Tags(ids)` in `default_notebook()` with every tag added; `Status(s)` in `default_notebook()` with status `s`; `Trash` switches to `AllNotes` first, then as `AllNotes`. Then reload the list and the sidebar, select, focus the title. The root view tracks its own focus handle, focused at startup when no note is selected, so `Ctrl+N` works with nothing else focused |
| Autosave | Any title or body change restarts one pending 500 ms task; when it fires: `save_note`, then reload the list only. No manual save |
| Flush | Selecting or creating a note first saves the current one immediately; if that save fails the switch is cancelled. Changing the navigation does not flush: the open note stays open |
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
| `Notebook(id)` | `Notebook(id, WithDescendants)` (expand/collapse depth comes with the sidebar slice) |
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
| Tag rows | Colour dot (fixed palette in `sidebar/palette.rs`, independent of the theme), name, `TagSummary::note_count`. Click selects the tag alone; `Ctrl+click` toggles it in the `Navigation::Tags` selection (notes having all of them); removing the last tag goes to All Notes |
| Tag menu | Rename, Change colour (submenu of the 14 `TagColor` values, current one checked), Delete |
| Prompt | One inline panel at the top of the sidebar (`SidebarState::prompt`): a name field with Save/Cancel (Enter also saves) for create and rename, a question with Delete/Cancel for deletions ("Delete notebook X? Its notes move to Trash."). No dialog: the dialog layer needs `Root` as the window root view, which the app does not use. A core error (empty name, duplicate tag) goes to the banner and the prompt stays open; success clears the banner and the prompt |
| After a mutation | Reload the sidebar data and the list. Deleting a notebook whose subtree contains the current notebook, or deleting a selected tag, adjusts the navigation first (All Notes, or the remaining tags) |
| `Ctrl+/` | `ToggleSidebar` action bound in `window::bind_keys`; `SidebarState::is_visible`, the other two columns take the space |
| Not in this slice | Drag and drop, sidebar sorting, templates |

Tests (`workspace/tests/sidebar.rs`) drive real elements: row and disclosure clicks, `Ctrl+click`, right click plus keyboard choice in the popup menu, typing into the prompt, and `Ctrl+/` through the key binding. Counts are checked from painted text inside each row's bounds.
