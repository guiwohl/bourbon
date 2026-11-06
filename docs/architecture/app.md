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
| Navigation | `Navigation { AllNotes, Notebook, Status, Tags, Templates, Trash }` plus the current sort form a `Listing`; `Listing::query()` maps it to a core `NoteQuery` (table below) |
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
| `Templates` | `Templates` |
| `Trash` | `Trash` |

The sort is the listing's own field, default `UpdatedNewestFirst`.

## Modules

Each column is a module owned by one concern. Columns render from data the workspace passes in and report intent through `cx.listener` callbacks on `Workspace`; no column touches `NoteStore`.

| File | Responsibility |
|---|---|
| `main.rs` | Claims the single instance, opens the store, aborts on error |
| `single_instance.rs` | GPUI-free Unix socket claim and focus listener (see Single instance) |
| `window.rs` | GPUI application, key binding, window options, the close hook |
| `autosave.rs` | The 500 ms delayed task |
| `control_pane/` | The centred search and command overlay (see Control pane) |
| `navigation.rs` | `Navigation`, `Listing`, the pure mapping to `NoteQuery` |
| `workspace.rs` | Root view: owns `NoteStore`, listing, selection, autosave and flush, trash purge, error banner state; lays out the three columns |
| `dragged_note.rs` | `DraggedNote` drag payload and preview, built from a `NoteSummary` |
| `presentation.rs` | Labels and colours shared by the columns: `UNTITLED`, `display_title`, `status_label`, `chip_color` (`TagColor` to `ColorName`, note list and editor header chips), `dot_color`, `color_label`, `is_current_color` (sidebar) |
| `sidebar/mod.rs` | `render(SidebarData, cx)`: navigation rows, notebook tree flattening, selection rule |
| `note_list/mod.rs` | `render(notes, selected, cx)`: header with the New note button, note rows |
| `editor/mod.rs` | `EditorPane` (title input, Markdown editor, load/read/focus, change subscriptions), empty state, error banner element |
| `editor/{tasks,preview_blocks,link_trigger,link_picker,word_count,view_mode}.rs` | Pure, unit-tested editor logic: task toggling, preview block splitting, `[[` detection, suggestion matching, counts, mode cycle |
| `editor/{preview,toolbar}.rs` | Preview pane and the mode switch |
| `workspace/writing_intents.rs` | View mode, tasks, link picker and link opening, find, focus mode intents |

## Tests

Headless, through `gpui-kit` `test-support` (`#[gpui_kit::test]`, `TestAppContext`): the workspace takes a `NoteStore` (in memory in tests), clicks and typing go through real element ids, and the clock is advanced with `advance_clock`. They cover startup selection, autosave timing (499 ms vs 500 ms, restart on keystroke), flush on switch, creation, and the empty state. Also: close flush (through `simulate_close`, which runs the real `on_window_should_close` handler registered by `window::build_workspace`), refused close on a failed save, quit flush (`App::shutdown`, which runs the `on_app_quit` handlers), pending autosave cancelled on switch, banner shown and cleared, failed save blocking a switch, and `Ctrl+N` through the real key binding. Navigation tests click real sidebar row ids (`nav-notebook-<id>`, `nav-trash`, ...) and check the listed titles, the notebook a created note lands in, and the startup trash purge. A save is made to fail honestly by selecting a note id the store does not hold.

## Dependency guard

`just architecture` fails if `cargo tree -p bourbon-core -e normal --prefix none` lists any crate starting with `gpui`, so it cannot go stale as GPUI crates are added. Proven by adding `gpui-pre-util` to `bourbon-core`: the recipe failed; reverted, it passed. `deny.toml` bans `gpui-kit`, `gpui-component`, `gpui-base` and `gpui-pre`, each with `wrappers` listing its exact direct parents (from `cargo tree -i`). Only `bourbon-app` may depend on `gpui-kit`, the entry point of the whole chain. Proven by adding `gpui-kit` to `bourbon-core`: `error[banned]`. `just deny` also prints many `duplicate` warnings from GPUI; they are expected.

## Sidebar

Implements decision rows 2, 4, 7, 11, 12 and 23 of [M2 organization](../decisions/2026-11-06-m2-organization.md). Code: `sidebar/` (rendering, pure helpers, state types) and `workspace/sidebar_intents.rs` (every `impl Workspace` method the sidebar calls).

| Topic | Behaviour |
|---|---|
| Sections | All Notes, Notebooks (header with a "+" button creating a top-level notebook), the notebook tree, Status (Active, On Hold, Completed, Dropped), Tags, Templates, Trash |
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
| Not in this slice | Sidebar sorting |

Tests (`workspace/tests/sidebar.rs`) drive real elements: row and disclosure clicks, `Ctrl+click`, right click plus keyboard choice in the popup menu, typing into the prompt, and `Ctrl+/` through the key binding. Counts are checked from painted text inside each row's bounds.

## Note list

`note_list::render(NoteListData, cx)` draws the second column from data the workspace passes in; every action is an intent method on `Workspace`, defined in `workspace/note_list_intents.rs`. The column never touches `NoteStore`.

| File | Responsibility |
|---|---|
| `note_list/mod.rs` | Column layout, the "No notes" empty state, selection rule |
| `note_list/header.rs` | View title, sort selector (Updated, Created, Title), "New from template" (when templates exist), then "New note", in Trash "Empty trash", in Templates "New template" |
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
| `Ctrl+O` | Open the control pane in Notes mode |
| `Ctrl+P` | Open the control pane in Actions mode (`>` prefilled) |

On Linux `gpui-base` binds word movement to `Ctrl+Left/Right`, so `Alt+Left/Right` is free. On macOS it binds `Alt+Left/Right` inside inputs and would shadow these.

### History

`editor::History` holds the viewed note ids and a position. `Workspace::show` calls `visit`: a note equal to the current entry adds nothing; otherwise forward entries are dropped and the note is pushed. Back and forward flush, then step to the nearest entry whose note exists and is not trashed, and call `show`, which finds the current entry already in place and pushes nothing. With nothing usable in that direction nothing happens.

### Tests

`workspace/tests/editor.rs`: moving through the real selector, status through the real menu (`popup-menu` items) and `Shift+Alt+1`…`5`, `Ctrl+Alt+M`, chips, suggestions, keyboard and click picking, create-on-Enter, chip removal, flush before each action, banner on failure, history across three notes including a trashed and a purged one, and `History` itself (forward entries dropped).

## Templates

Implements decision rows 18 and 24 to 28 of [M2 organization](../decisions/2026-11-06-m2-organization.md). Core does the work (`create_template`, `mark_as_template`, `unmark_template`, `create_note_from_template` with `TemplateTimeZone::system()`); the app only presents and forwards.

| Topic | Behaviour |
|---|---|
| Sidebar | After Tags and before Trash: a "Templates" row (`nav-templates`, `Navigation::Templates`, lists `NoteScope::Templates`) and a "New template" row (`new-template`) |
| New template | Sidebar row, the list header button in the Templates view (`list-new-template`, replaces "New note" there), or `Ctrl+N` in the Templates view (`create_note` delegates to `create_template`). Flush, navigate to Templates, create in `default_notebook()`, reload, select, focus the title |
| New from template | `new-from-template` dropdown left of "New note" (or "Empty trash"/"New template"), one item per template titled by `display_title` (`UNTITLED` for blank). Hidden when there are no templates. The workspace keeps the template list (`Workspace::templates`), reloaded by `refresh_listing`, sorted like the current listing |
| Placement | `Workspace::new_note_placement` is the single navigation-to-notebook rule of decision 18 (notebook view: that notebook; otherwise `default_notebook()`; tags and status seeds apply only to plain notes). Trash and Templates (`Navigation::hides_new_notes`) switch to All Notes first. Then reload, select, focus the title |
| Mark and unmark | The editor header "..." button (`note-actions`) offers "Mark as template" or "Unmark template" by `Note::is_template`. The intent runs through `edit_selected` (flush, change, refresh header and listing); the note stays open even though it leaves the current list |

Tests: `workspace/tests/templates.rs`.

## Control pane

Decisions in [M3 navigation](../decisions/2026-11-06-m3-navigation.md). `Workspace::control_pane: Option<ControlPane>`; rendered last in the root view as an absolute overlay (`pane-backdrop` > `pane-panel` with `pane-input` and `pane-row-<n>`).

| File | Responsibility |
|---|---|
| `control_pane/query.rs` | Pure `parse(query) -> ParsedQuery { mode, text }`: a leading `>` is `Mode::Actions` (text trimmed, `>` dropped), anything else `Mode::Notes` |
| `control_pane/actions.rs` | `PaneAction` (21 commands), `name`, `shortcut`, `needs_selected_note`, pure `matching(text, has_selected_note)`: filter and rank |
| `control_pane/highlight.rs` | Pure `next(current, length)` and `previous(current)`, clamped at both ends |
| `control_pane/mod.rs` | `ControlPane` state: the input, the highlighted row, `Results::{Notes(Vec<SearchHit>), Actions(Vec<PaneAction>)}`; `search(query, store, has_selected_note)` |
| `control_pane/render.rs` | Overlay: input, rows (note: title or `UNTITLED` plus snippet; action: name plus the shortcut on the right) |
| `workspace/control_pane_intents.rs` | `open_note_search`, `open_action_search`, `close_control_pane`, query change, highlight movement, activation and `run_pane_action` |

Behaviour:

- Every query change recomputes the results and resets the highlight to the first row. `set_value` emits no change, so opening computes the first results itself.
- `Up`, `Down` and `Esc` are captured at the overlay (like the tag suggestions) so the single-line input does not swallow them. `Enter` is the input's `PressEnter`. A mouse down on the backdrop closes; the panel stops propagation.
- Activating a row closes the pane, restores focus to the workspace root, then runs the target. A note goes through `flush_then`: if it is not in `Workspace::notes` the navigation becomes All Notes, then `select`. An action calls the existing intent: `create_note`, `create_template`, `toggle_sidebar`, `navigate`, `sort_by`, `set_selected_status`, `open_notebook_selector`, `duplicate`, `toggle_pin`, `move_to_trash`, `history_back`, `history_forward`; Empty Trash navigates to Trash and opens `confirm_empty_trash`.
- No selected note: the actions that need one are hidden from the list.
- Startup now always focuses the workspace root, so `Ctrl+O` and `Ctrl+P` work before any click.

Tests: pure unit tests next to each file; `workspace/tests/control_pane.rs` drives the real keys (`Ctrl+O`, `Ctrl+P`, `Up`, `Down`, `Enter`, `Escape`, typing into `pane-input`) and clicks (`pane-row-<n>`, backdrop, inside the input).

## Settings, themes and panes (M5)

- `locations.rs`: `Locations` (settings file, Omarchy colors path) is injected into `Workspace::new`; tests pass temp paths.
- `theme/`: `palette.rs` holds the Bourbon palettes as data, the Omarchy mapping and `resolve`; `watch.rs` is the mtime watch; `apply` paints gpui-component's `Theme`. `Workspace` polls the watch every 2 s.
- `settings_view.rs` replaces the editor column; `workspace/settings_intents.rs` holds the intents (`open_settings`, `set_theme_source`, `set_appearance`, `resize_*`, `finish_resizing`). Each change applies the theme and saves.
- `panes.rs`: the draggable `Divider` handles; widths come from `Settings` and flow into `SidebarData` and `NoteListData`.
- Keys: `Ctrl+,` opens settings; control pane action "Open settings".

## Single instance

At startup `main` calls `claim` on `$XDG_RUNTIME_DIR/<app id>.sock` (fallback `/tmp/<app id>-<uid>.sock`). A live instance gets `focus\n` and the new process exits 0 without opening the store. A refused connection is a stale socket: it is removed and rebound. The listener runs on a std thread and forwards each `focus` line through an mpsc channel; a GPUI task polls it every 100 ms and calls `activate_window()`. Other messages are ignored. Dev builds use the app id `bourbon-dev`, so they never collide with a release build.
## Writing features (M4)

Decisions in [M4 editor](../decisions/2026-11-06-m4-editor.md). Pure logic is in `editor/`, intents in `workspace/writing_intents.rs`, the link format in `bourbon-core` (`format_note_link`, `parse_note_link`).

| Feature | Keys / ids | Notes |
|---|---|---|
| View modes | `Ctrl+E`, `view-mode-edit`, `view-mode-side-by-side`, `view-mode-preview`, panes `body-editor` and `preview` | One mode on `Workspace`, not per note |
| Preview tasks | `preview-task-<line>` | Own checkbox rows split out of the Markdown by `preview_blocks`; a click runs `toggle_task_on_line` and the normal autosave |
| Task shortcut | `Ctrl+Enter` in the body | `toggle_task_at_cursor` |
| Note links | `[[`, `link-suggestions`, `link-suggestion-<id>`; `Up`/`Down`/`Enter`/`Esc` while open | `LinkPicker` lives on `EditorPane`; keys are scoped by the `LinkPicker` key context on the editor wrapper |
| Find and replace | `Ctrl+F`, `Ctrl+H` | The editor's built-in search panel; app actions cover other focus |
| Focus mode | `Ctrl+Shift+F`, `Esc` | `Workspace::focus_mode` hides the two columns at render time |
| Word count | `word-count` | Computed from the body on every render |

Tests: `workspace/tests/{view_modes,tasks,links,find,focus}.rs` (real key bindings, real clicks, painted text) plus unit tests next to each pure module and in `bourbon-core/src/note_link.rs`. The preview link test clicks inside the `preview` pane at the first character of the link.
