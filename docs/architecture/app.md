# bourbon-app

GPUI window over `bourbon-core`. Holds no business rule: it presents state and forwards intent. Built on `gpui-kit` 0.9 (re-exports GPUI and gpui-component, still used by the control pane and settings), `bourbon-ui`, `bourbon-editor` and `bourbon-markdown`.

## Behaviour

| Topic | Behaviour |
|---|---|
| Window | Title "Bourbon", Wayland `app_id` "bourbon" |
| Startup | Opens `default_database_path()`; failure prints a message and exits 1. Purges expired trash once (errors go to the banner), loads the sidebar data and the list, selects the most recent note |
| Layout | Three columns: sidebar (200 px) \| note list (260 px) \| editor (flex), separated by `bourbon_ui::Splitter` (1 px border, invisible 10 px handle). Limits 170..500 and 220..500 (`Settings` clamps to the same range), saved when the drag ends |
| Sidebar | See the Sidebar section. Clicking a row sets the navigation and reloads the list |
| Note list | See the Note list section |
| Editor | See Editor column. Empty state: "No notes yet" with a `bourbon_ui` primary "New note" button |
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
| `control_pane/` | The search and command palette (see Control pane) |
| `test_id.rs` | `TestSupportExt::test_support()`: sets the element's id as its GPUI debug selector (recorded only in debug test builds, a no-op elsewhere) |
| `overlay.rs` | `tagged(id, element)` and `layer(id, dialog)`: wrap `bourbon-ui` overlays and buttons in test-visible ids |
| `workspace/dialogs.rs` | `DialogState` (dialog focus, empty trash pending), `Workspace::dialog` (sidebar prompt, restore, empty trash; one at a time), the generic confirmation dialog (Cancel + danger confirm, Enter confirms, Esc and backdrop cancel). Dialogs without a field take the dialog focus when shown |
| `navigation.rs` | `Navigation`, `Listing`, the pure mapping to `NoteQuery` |
| `workspace.rs` | Root view: owns `NoteStore`, listing, selection, autosave and flush, trash purge, error banner state; lays out the three columns |
| `dragged_note.rs` | `DraggedNote` drag payload and preview, built from a `NoteSummary` |
| `presentation.rs` | Labels and colours shared by the columns (pill text shade comes from `bourbon_ui::pill_text_color`): `UNTITLED`, `display_title`, `status_label`, `status_icon`, `dot_color` (tag pills and dots everywhere), `color_label`, `is_current_color` (sidebar) |
| `sidebar/mod.rs` | `render(SidebarData, cx)`: control strip, `ScrollArea` with `ListRow` / `SectionHeader` rows from `bourbon-ui`, notebook tree flattening, selection rule |
| `note_list/mod.rs` | `render(NoteListData, cx)`: header, filter, virtualised `list` of note rows, `adjacent_note` for arrow keys |
| `assets.rs` | `AppAssets`: `bourbon_ui::IconAssets` first, the component assets as fallback for the control pane and settings |
| `workspace/menus.rs` | `MenuBuilder`, `Workspace::open_menu` / `close_menu`: every popup menu of the left columns is a `bourbon_ui::Menu` in a `Popover` at the pointer; Enter/click runs the entry after closing and restoring focus |
| `editor/mod.rs` | `EditorPane`: title field, `bourbon_editor::Editor` body, `PreviewView`, tag field, header state, menu anchors, scroll sync, theme sync; event forwarding; empty state, error banner |
| `editor/{toolbar,header,body}.rs` | Toolbar, meta row (notebook/status pickers, tag pills, Add Tags, suggestions), body panes |
| `editor/{scroll_sync,themes,menu_anchor,history,link_trigger,link_picker,word_count}.rs` | Pure, unit-tested logic: sync guard wiring, `EditorTheme`/`PreviewTheme` from the `bourbon-ui` theme, menu position under a trigger, history ends, `[[` detection, suggestions, counts |
| `workspace/writing_intents.rs` | View mode, tasks, link picker and link opening, find, focus mode intents |

## Tests

Headless, through GPUI's own `test-support` (`#[gpui::test]`, `TestAppContext`, `VisualTestContext`): the workspace takes a `NoteStore` (in memory in tests), clicks and typing go through real element ids (harness below), and the clock is advanced with `advance_clock`. They cover startup selection, autosave timing (499 ms vs 500 ms, restart on keystroke), flush on switch, creation, and the empty state. Also: close flush (through `simulate_close`, which runs the real `on_window_should_close` handler registered by `window::build_workspace`), refused close on a failed save, quit flush (`App::shutdown`, which runs the `on_app_quit` handlers), pending autosave cancelled on switch, banner shown and cleared, failed save blocking a switch, and `Ctrl+N` through the real key binding. Navigation tests click real sidebar row ids (`nav-notebook-<id>`, `nav-trash`, ...) and check the listed titles, the notebook a created note lands in, and the startup trash purge. A save is made to fail honestly by selecting a note id the store does not hold.

## Dependency guard

`just architecture` fails if `cargo tree -p bourbon-core -e normal --prefix none` lists any crate starting with `gpui`, so it cannot go stale as GPUI crates are added. Proven by adding `gpui-pre-util` to `bourbon-core`: the recipe failed; reverted, it passed. `deny.toml` bans `gpui-kit`, `gpui-component`, `gpui-base` and `gpui-pre`, each with `wrappers` listing its exact direct parents (from `cargo tree -i`). Only `bourbon-app` may depend on `gpui-kit`, the entry point of the whole chain. `bourbon-app` also depends on `gpui-pre` directly (`gpui` in code), so the overlays use GPUI without the kit. Proven by adding `gpui-kit` to `bourbon-core`: `error[banned]`. `just deny` also prints many `duplicate` warnings from GPUI; they are expected.

## Sidebar

Implements decision rows 2, 4, 7, 11, 12 and 23 of [M2 organization](../decisions/2026-11-06-m2-organization.md). Code: `sidebar/` (rendering, pure helpers, state types) and `workspace/sidebar_intents.rs` (every `impl Workspace` method the sidebar calls).

| Topic | Behaviour |
|---|---|
| Sections | Control strip (min 40 px, only the gear on the right, `app-menu` opening our `Menu`: Settings Ctrl+, , Back up now, Restore from backup..., Export to Markdown...), then a `ScrollArea` with All Notes, Note Templates (`nav-templates`), Notebooks (search, `new-notebook`), the notebook tree, Status (collapse), Tags (search, collapse), Trash. `bourbon_ui::ListRow` per research 3.2: 30 px, 14 px, 16 px icon, count 10 px at 60%, no hover background, selected tint, 5 px accent bar only while the sidebar has focus (`sidebar_focus`, `contains_focused`). Status rows use the token status colours; section collapse state is `SidebarState::collapsed` (memory only; header ids `status-header`, `tags-header`). Search icons open the control pane |
| Tree | Depth-first rows indented per depth; each row shows name and `NotebookNode::note_count`. Rows with children have the animated chevron disclosure (`ListRow::disclosure`, `notebooks::disclosure_state`) that expands/collapses without navigating |
| Expansion | `SidebarState::expanded` (set of `NotebookId`), all collapsed at startup, lives in the workspace |
| Depth | `Navigation::Notebook(id, depth)`. Selecting an expanded notebook uses `DirectOnly`; a collapsed one `WithDescendants`. Toggling the disclosure of the selected notebook re-applies the depth and reloads the list |
| Notebook menu | Right click (our `Menu`, `notebooks::notebook_menu`): New sub-notebook (also expands the parent), Rename, Use for new notes (sets the default notebook), Delete. The Inbox (`default_notebook()`, read once at startup) has no Delete |
| Tag rows | Colour dot (fixed palette in `presentation.rs`, independent of the theme), name, `TagSummary::note_count`. Click selects the tag alone; `Ctrl+click` toggles it in the `Navigation::Tags` selection (notes having all of them); removing the last tag goes to All Notes |
| Tag menu | Rename, Change colour (the owner replaces the menu with the 14 `TagColor` values, current one checked, at the same position), Delete |
| Prompt | A `bourbon_ui::Dialog` over the window (`SidebarState::prompt`, rendered by `sidebar::prompt_dialog` from `Workspace::dialog`; overlay id `sidebar-prompt`, field `sidebar-prompt-name` is a `bourbon_editor::TextField`, buttons `sidebar-prompt-confirm` / `sidebar-prompt-cancel`): title New notebook / New sub-notebook / Rename notebook / Rename tag with Cancel + Save (primary); Delete notebook / Delete tag with the question and Cancel + Delete (danger). Enter confirms, Esc or a backdrop click cancels. A core error (empty name, duplicate tag) goes to the banner and the dialog stays open; success clears both |
| After a mutation | `refresh_listing` once (sidebar data and list). Deleting a notebook first flushes the pending save, and afterwards deselects the open note if it is now trashed or gone (empty state). Creating a sub-notebook re-applies the parent's depth to the navigation without reloading, so the single reload covers it. Deleting a notebook whose subtree contains the current notebook, or deleting a selected tag, adjusts the navigation first (All Notes, or the remaining tags) |
| `Ctrl+/` | `ToggleSidebar` action bound in `window::bind_keys`; `SidebarState::is_visible`, the other two columns take the space |
| Drop target | Notebook rows accept a dragged note (`on_drop::<DraggedNote>`) and the row takes the active selection colour while one is over it. The drop calls `Workspace::drop_note_on_notebook` |
| Not in this slice | Sidebar sorting |

Tests (`workspace/tests/sidebar.rs`) drive real elements: row clicks (wrapper ids `nav-*` carry `test_support`, since kit rows are not observable), the disclosure by clicking at its offset in the row, `Ctrl+click`, right click plus `down`/`enter` in our `Menu` (labels read from `Workspace::open_menu`), Escape closing the menu, focus returning to the sidebar, typing into the prompt, and `Ctrl+/`. The accent bar itself is tested in `bourbon-ui` with a debug selector; here the tests check which pane holds the focus that decides it.

## Note list

`note_list::render(NoteListData, cx)` draws the second column from data the workspace passes in; every action is an intent method on `Workspace`, defined in `workspace/note_list_intents.rs`. The column never touches `NoteStore`.

| File | Responsibility |
|---|---|
| `note_list/mod.rs` | Column layout, the "No notes" empty state, selection rule |
| `note_list/header.rs` | 40 px header: sidebar toggle `IconButton` (`toggle-sidebar`, tooltip with the live `Ctrl+/`), centred view name (`note-list-title`, 16.8 px, medium, no icon) opening our `Menu` (Sort by updated/created/title with check, then a disabled "New from template" label and one item per template), compose `IconButton` (`new-note`; creates a template in Templates; in Trash it is `empty-trash`, disabled when empty) |
| `note_list/filter.rs` | 34 px bar (4 px gap below), margins 2 / 12 around a `bourbon_editor::TextField` (placeholder "Filter", search icon; metrics in [editor.md](editor.md#text-field)) wrapped in `note-filter-input`. `narrow` intersects the listing with core `search_notes` ids (titles and bodies); empty means no filter. Field events: `Changed` reloads the list, `Submit` (Enter) focuses the list, `Cancel` (Esc) clears. Search skips templates and trash, so the filter empties those views |
| `note_list/date.rs` | `relative_age(updated, now)` (just now, N minute(s)/.../year(s) ago) and `date_label(stamp, now, zone)`: relative up to 37 days, then `YYYY-MM-DD` in the given zone (research 3.3). The stamp is the created time when sorting by created, else the updated time; templates show no date |
| `note_list/row.rs` | Row per research 3.3: padding 2/0/10/14, 1 px separator, pin and status icons inline before the bold 2-line title (strike-through for Dropped), meta line (date in the date colour, checklist icon, 32x6 progress bar, green when complete, "1 of 2", compact `bourbon_ui::Pill` per tag), muted one-line excerpt. Selected tint; 5 px bar only while the list has focus; no hover background. Right click opens the note menu |
| `note_list/title.rs` | Pure view title: "All Notes", notebook name, status label (from `presentation.rs`), tag names, "Trash" |
| `workspace/note_list_intents.rs` | `sort_by`, `select_adjacent_note`, `toggle_pin`, `duplicate`, `move_to_trash`, `restore`, `confirm_empty_trash`, `empty_trash` |

### Behaviour

- The rows live in a GPUI `list` (variable heights) driven by `Workspace::note_list_state`; `reload_list` splices it, opening a note scrolls it into view. The notes are an `Rc<[NoteSummary]>` shared with the row closure.
- The list pane has its own focus handle and key context `NoteList`: `Up`/`Down` (`SelectPreviousNote`/`SelectNextNote`) open the neighbouring note through `select` and the focus stays in the list.
- The sort selector sets `Listing::sort` and reloads; pinned notes stay on top because the core guarantees it (decision 16).
- The context menu (right click) outside Trash offers Pin or Unpin, Duplicate, Move to Trash; in Trash only Restore.
- Every change goes through one path (`change_notes`): `flush_then`, call the core, `refresh_listing`, report a store error in the banner. Pin uses the core `toggle_pin`; the app does not read its list to pick a direction. A tag chip click calls the sidebar's single-tag selection (`select_tag`).
- Duplicate selects the copy. Move to Trash on the selected note selects the next row, else the previous, else nothing (editor empty state); on another row the selection is untouched.
- "Empty trash" opens the Empty trash dialog ("Permanently delete N notes?", Cancel + Empty trash in danger red; ids `empty-trash-confirmation`, `empty-trash-confirm`, `empty-trash-cancel`); only the confirm button or Enter calls `empty_trash`, Esc and the backdrop cancel. A selected note that was purged is deselected.
- A tag chip navigates to that single tag. Its mouse-down stops propagation so the row is not selected.
- Rows outside Trash are draggable (`on_drag` with a `DraggedNote { id, title }`, defined in `dragged_note.rs`; the preview is a compact chip with the title, `UNTITLED` when empty). Dropped on a sidebar notebook row, the note moves through `move_note_to`, the same path as the notebook selector and `Ctrl+Alt+M`: `flush_then`, `move_note`, `refresh_listing`. The header is reloaded only when the dropped note is the open one; the open note stays open. Trash rows are not draggable.
- Tag colours map from `TagColor` to the component palette (`ColorName`) in `presentation.rs`, shared with the editor header chips.

### Tests

`workspace/tests/drag.rs`: real drags (`drag_to` from `note-<id>` to `nav-notebook-<id>`) check the move, the open note staying open, the list refresh, the header, the pending-edit flush and Trash rows not being draggable.

`workspace/tests/note_list.rs`: the context menu and the sort menu are driven through our `Menu` (right click or click on `note-list-title`, then `down`/`enter`; labels come from the open menu), arrow keys with the list focus, which pane holds the focus, the dialog through `enter` and `escape`, chips through `chip-<note>-<tag>`, pins through `pin-<note>`. Dates (including the 37-day switch), titles, `adjacent_note` and prompt text are pure unit tests.

## Editor column

Built on `bourbon-ui`, `bourbon-editor` and `bourbon-markdown` (research [3.4-3.6](../research/inkdrop-ui-shell.md), [4.5-4.6](../research/inkdrop-markdown-editor-preview.md)). Top to bottom:

| Part | Behaviour |
|---|---|
| Toolbar | Min height 40, padding-left 4: distraction-free toggle (`toggle-focus-mode`, maximize-2 / minimize-2, `Ctrl+Shift+F`), back / forward (`history-back`, `history-forward`, disabled at the ends of `History`), spacer, `SegmentedControl` (`view-mode`: pencil Show Editor, columns Show Side by Side, eye Show Preview), spacer, ⋮ (`note-actions`). Tooltips show the live binding |
| Title | `bourbon_editor::TextField` (`title`), `borderless()`, `text_size(21 px)`, in a 36 px row with a 10.5 px inset, no gpui-component left in the column; placeholder "Untitled"; Enter moves focus to the body |
| Meta row | Padding 0 12, 12.6 px: notebook picker (`notebook-selector`: notebook icon, bold name, chevron), status picker (`status-button`: status icon, name or "Status", chevron), one `Pill` per tag (`tag-chip-<id>`, × `remove-tag-<id>` at 40 % opacity, full on hover), inline "Add Tags" `TextField` (`tag-input`, borderless, 12.6 px, in a 24 px row with a 4 px inset, min 84 px; `up`/`down` bound in `TagInput > BourbonTextField` move the suggestion highlight) |
| Body | Edit: `Editor` (`body-editor`, line numbers on, wrapped rows unnumbered, active line across the gutter; no fold chevrons, folding is not built). Preview: `PreviewView` (`preview`, 10 px padding). Side by side: both `flex_1` (50/50) with a 1 px internal-border divider |
| Status bar | 24 px, muted 11 px word count on the right (`word-count`) |

Pickers and suggestion panels `occlude`, so a click on them never reaches the editor behind.

Menus (notebook list indented by depth with the current one checked; statuses with `Shift+Alt+N` hints and the current one checked; ⋮ = the note-row menu (Pin/Unpin, Duplicate, Move to Trash) + Mark/Unmark template + Copy CLI command) are `bourbon_ui::Menu`s opened through `Workspace::open_menu` 4 px under their trigger, whose bounds `MenuAnchor` records every prepaint, so `Ctrl+Alt+M` opens the notebook menu at the same place. The ⋮ menu reloads the note first so Pin/Unpin is current.

Themes: `themes::pane_themes` picks `EditorTheme`/`PreviewTheme` light or dark from the `bourbon-ui` appearance and paints the tokens' editor background; `EditorPane::sync_theme` reapplies them in `Workspace::render` when appearance or background change (settings, Omarchy).

Scroll sync (side by side only): `Editor` emits `EditorEvent::Scrolled`, `PreviewView` `PreviewEvent::Scrolled`. `ScrollSync` asks `SyncGuard::accept` which pane drives, then maps the editor's fractional `top_line` through `preview_offset_for_line` over `PreviewView::scroll_anchors`, or the preview `scroll_top` back through `line_for_preview_offset` into `Editor::scroll_to_line`. The echo of the driven pane is dropped for 100 ms. Loading a note resets both panes to the top instead of disabling the guard, so the first user scroll after a switch is never swallowed.

Preview events: `ToggleTask` toggles the line in the editor (autosave follows from its `Changed`), `OpenLink` opens the URL or `bourbon://note/<id>`, `OpenNote` opens the note.

| Part | Behaviour |
|---|---|
| Autocomplete | A deferred panel under the meta row (dot plus name). While typing, rows `tag-suggestion-<id>` (the highlighted one has `aria_selected`) list existing tags, not already on the note, whose name contains the typed text (case-insensitive). `Down`/`Up` move the highlight (captured before the single-line input swallows them); clicking a row adds it |
| Tag `Enter` | Highlighted suggestion if any; else the existing tag named exactly as typed (case-insensitive, trimmed); else `add_new_tag_to_note(name, TagColor::Default)` (create and attach in one transaction). Blank input does nothing. The input is cleared on success |
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

`workspace/tests/editor.rs`: moving through the real notebook menu (labels from the open `Menu`, `down`/`enter`), the menu position under its button, status through the real menu and `Shift+Alt+1`…`5`, picker texts (`test_text`), `Ctrl+Alt+M`, chips, suggestions, keyboard and click picking, create-on-Enter, chip removal, flush before each action, banner on failure, history across three notes including a trashed and a purged one, and `History` itself (forward entries dropped).

## Templates

Implements decision rows 18 and 24 to 28 of [M2 organization](../decisions/2026-11-06-m2-organization.md). Core does the work (`create_template`, `mark_as_template`, `unmark_template`, `create_note_from_template` with `TemplateTimeZone::system()`); the app only presents and forwards.

| Topic | Behaviour |
|---|---|
| Sidebar | "Note Templates" row (`nav-templates`, after All Notes, lists `NoteScope::Templates`); no "new template" row |
| New template | The compose button (`new-note`) in the Templates view, or `Ctrl+N` there (`create_note` delegates to `create_template`). Flush, navigate to Templates, create in `default_notebook()`, reload, select, focus the title |
| New from template | Items at the end of the `note-list-title` menu, one per template titled by `display_title` (`UNTITLED` for blank), absent when there are no templates. The workspace keeps the template list (`Workspace::templates`), reloaded by `refresh_listing`, sorted like the current listing |
| Placement | `Workspace::new_note_placement` is the single navigation-to-notebook rule of decision 18 (notebook view: that notebook; otherwise `default_notebook()`; tags and status seeds apply only to plain notes). Trash and Templates (`Navigation::hides_new_notes`) switch to All Notes first. Then reload, select, focus the title |
| Mark and unmark | The editor header "..." button (`note-actions`) offers "Mark as template" or "Unmark template" by `Note::is_template`. The intent runs through `edit_selected` (flush, change, refresh header and listing); the note stays open even though it leaves the current list |

Tests: `workspace/tests/templates.rs`.

## Control pane

Decisions in [M3 navigation](../decisions/2026-11-06-m3-navigation.md), look in [overlays](../decisions/2026-11-08-overlays.md). `Workspace::control_pane: Option<ControlPane>`; rendered as a `bourbon_ui::Modal` placed 56 px from the top (`pane-backdrop` > `pane-panel` with `pane-input` and `pane-row-<n>`).

Look (research 3.11): 600 px wide, max height = window - 56 - 28, page background, 1 px border, radius 12, shadow `0 10 30` at 30 %. Input row 48 px with a 20 px search icon and a borderless 18 px `TextField`; rows margin 2 8, padding 8 10, radius 7, 20 px icon slot (note: file icon, action: chevron), label weight 500, snippet 12 px muted, shortcut 12 px muted on the right; hover tint `selection_list_hover`, highlighted row `selection_list_active` plus a 3 px info bar; the pointer moving over a row highlights it. "No results found" when empty. Footer hints: `↑↓ to navigate`, `↵ to open` / `↵ to run`, `esc to close`.

| File | Responsibility |
|---|---|
| `control_pane/query.rs` | Pure `parse(query) -> ParsedQuery { mode, text }`: a leading `>` is `Mode::Actions` (text trimmed, `>` dropped), anything else `Mode::Notes` |
| `control_pane/actions.rs` | `PaneAction` (21 commands), `name`, `shortcut`, `needs_selected_note`, pure `matching(text, has_selected_note)`: filter and rank |
| `control_pane/highlight.rs` | Pure `next(current, length)` and `previous(current)`, clamped at both ends |
| `control_pane/mod.rs` | `ControlPane` state: the input, the highlighted row, `Results::{Notes(Vec<SearchHit>), Actions(Vec<PaneAction>)}`; `search(query, store, has_selected_note)` |
| `control_pane/render.rs` | Modal: input row, rows (note: title or `UNTITLED` plus snippet; action: name plus the shortcut on the right), empty state, footer hints (`footer_hints(mode)`) |
| `workspace/control_pane_intents.rs` | `open_note_search`, `open_action_search`, `close_control_pane`, query change, highlight movement, activation and `run_pane_action` |

Behaviour:

- Every query change recomputes the results and resets the highlight to the first row. `set_value` emits no change, so opening computes the first results itself.
- `Up` / `Down` are `control_pane::HighlightNext` / `HighlightPrevious`, bound in the `ControlPane` key context (the text field binds neither). `Enter` and `Esc` are the field's `Submit` and `Cancel`. A mouse down on the backdrop closes; the panel stops propagation.
- Activating a row closes the pane, restores focus to the workspace root, then runs the target. A note goes through `flush_then`: if it is not in `Workspace::notes` the navigation becomes All Notes, then `select`. An action calls the existing intent: `create_note`, `create_template`, `toggle_sidebar`, `navigate`, `sort_by`, `set_selected_status`, `open_notebook_selector`, `duplicate`, `toggle_pin`, `move_to_trash`, `history_back`, `history_forward`; Empty Trash navigates to Trash and opens `confirm_empty_trash`.
- No selected note: the actions that need one are hidden from the list.
- Startup now always focuses the workspace root, so `Ctrl+O` and `Ctrl+P` work before any click.

Tests: pure unit tests next to each file; `workspace/tests/control_pane.rs` drives the real keys (`Ctrl+O`, `Ctrl+P`, `Up`, `Down`, `Enter`, `Escape`, typing into `pane-input`) and clicks (`pane-row-<n>`, backdrop, inside the input).

## Settings, themes and panes (M5)

- `locations.rs`: `Locations` (settings file, Omarchy colors path) is injected into `Workspace::new`; tests pass temp paths.
- `theme/`: `palette.rs` holds the Bourbon palettes as data, the Omarchy mapping and `resolve`; `tokens.rs` resolves the `bourbon_ui::Tokens` (Inkdrop light/dark, or `omarchy_tokens` for Omarchy) and `apply` installs them next to gpui-component's `Theme`, which still paints what is left on gpui-kit; `watch.rs` is the mtime watch. `Workspace` polls the watch every 2 s.
- `settings_view.rs` is the Inkdrop preferences layout and replaces the three columns while open (Inkdrop uses a separate window): a 160 px page menu (`settings-page-general`, `settings-page-backup`; inactive items at 60 %, active one tinted with a 3 px primary bar) and a scrolling content pane (12 px padding, max 640 px, close button `settings-close`). General: Theme and Appearance `bourbon_ui::Select`s (ids `settings-theme`, `settings-appearance`; Appearance disabled with a hint while following Omarchy) opening our `Menu` under the trigger with the current choice checked. Backup: repository and (token login only, masked) token `TextField`s, Sign in with select (`settings-backup-auth`), "Back up automatically" `Checkbox` (`settings-backup-enabled`), Back up now / Restore buttons right-aligned. `workspace/settings_intents.rs` holds the intents (`open_settings`, `show_settings_page`, `set_theme_source`, `set_appearance`, `resize_*`, `finish_resizing`). Each change applies the theme and saves.
- Column widths come from `Settings` and flow into `SidebarData` and `NoteListData`; `bourbon_ui::Splitter` reports them through `resize_sidebar` / `resize_note_list` and `finish_resizing` saves on release.
- Keys: `Ctrl+,` opens settings; control pane action "Open settings".

## Data: backup, restore, export (M6)

- `backup_runner.rs`: `BackupRunner` trait and `GitBackupRunner`; `Locations` carries the database path, token file, work dir and runner.
- `workspace/data_intents.rs`: `DataState` (running, dirty, status, timers), `back_up_now`, the 5 minute timer (`note_data_changed`), `request_close` (bounded 20 s), `begin_restore`/`confirm_restore`/`cancel_restore`, `export_to_markdown`/`export_into`.
- `sidebar/menu.rs`: the gear menu; entries run `PaneAction`s through `run_pane_action`.
- `settings_view.rs`: Backup page and `notice_banner` (info tint at 10 %, 13 px, bottom border). The restore question is the Restore from backup dialog in `workspace/dialogs.rs` (ids `restore-confirmation`, `restore-confirm`, `restore-cancel`). Tests: `workspace/tests/data.rs` with a recording fake runner. Details in [the M6 app decision](../decisions/2026-11-06-m6-data-app.md).

## Single instance

At startup `main` calls `claim` on `$XDG_RUNTIME_DIR/<app id>.sock` (fallback `/tmp/<app id>-<uid>.sock`). A live instance gets `focus\n` and the new process exits 0 without opening the store. A refused connection is a stale socket: it is removed and rebound. The listener runs on a std thread and forwards each `focus` line through an mpsc channel; a GPUI task polls it every 100 ms and calls `activate_window()`. Other messages are ignored. Dev builds use the app id `bourbon-dev`, so they never collide with a release build.
## Writing features (M4)

Decisions in [M4 editor](../decisions/2026-11-06-m4-editor.md). Pure logic is in `editor/`, intents in `workspace/writing_intents.rs`, the link format in `bourbon-core` (`format_note_link`, `parse_note_link`).

| Feature | Keys / ids | Notes |
|---|---|---|
| View modes | `Ctrl+E`, segments of `view-mode`, panes `body-editor` and `preview` | One mode on `Workspace`, not per note, saved in `Settings::editor_view_mode` |
| Preview tasks | `task-line-<line>` (debug selector of `PreviewView`) | `PreviewEvent::ToggleTask` → `Editor::toggle_task_at_line`, then the normal autosave |
| Task shortcut | `Ctrl+Enter` in the body | `bourbon-editor` `toggle_task` (any line becomes a task, a task flips) |
| Note links | `[[`, `link-suggestions`, `link-suggestion-<id>`; `Up`/`Down`/`Enter`/`Esc` while open | `LinkPicker` lives on `EditorPane`, drawn over the body; keys are bound in `LinkPicker > BourbonEditor` after `bind_editor_keys`, so they win over the editor's own; the link goes in with `Editor::replace_range` |
| Find and replace | `Ctrl+F`, `Ctrl+H` | `bourbon-editor` has no search panel yet: both switch to the editor (from Preview) and focus the body |
| Focus mode | `Ctrl+Shift+F`, `Esc`, toolbar toggle | `Workspace::focus_mode` hides the two columns at render time (Inkdrop's distraction free); the column keeps its full width |
| Word count | `word-count` | Computed from the body on every render |

Tests: `workspace/tests/{view_modes,tasks,links,find,focus}.rs` (real key bindings, real clicks on the segments by offset and on preview checkboxes by debug bounds, `PreviewView::doc`, scroll sync both ways and per mode, reset on note switch) plus unit tests next to each pure module and in `bourbon-core/src/note_link.rs`. The preview link test clicks inside the `preview` pane at the first character of the link.

## Test harness

`workspace/tests/harness.rs`, on GPUI only: `render` (refresh + draw), `bounds(id)` (`VisualTestContext::debug_bounds` of the id set by `test_support()`), `is_listed`, `click`, `click_at(id, offset)`, `click_with(id, Press { button, modifiers, count })`, `right_click`, `hover`, `drag(from, to)` and `drag_by(id, distance)` (8 pointer steps, so GPUI starts the drag), `press(keys)` (`simulate_keystrokes`), `type_text` (`simulate_input`), `RenderFrame` for closures that need a fresh frame. State a test asserts on is marked with its own debug selector: `pane-row-<n>-highlight` on the highlighted palette row, `settings-page-<page>-active` on the active settings page.

Editor-column observation without gpui-kit: element ids through `test_id`; highlighted suggestions add a `<id>-highlight` marker child; picker texts and the word count carry `picker-label: <text>` / `word-count: <label>` debug selectors; focus is read from the entity focus handles (`EditorPane::title_is_focused`, the body's `focus_handle`); menus are read from the open `bourbon_ui::Menu`.

Still on `gpui-kit` (follow-up): the window is opened through `gpui_kit::open_window` after `gpui_kit::init` (gpui-component setup), `assets.rs` still serves gpui-component icons, `theme/` still paints gpui-component's `Theme`, and `tests/settings.rs` reads that `Theme`. The editor column itself uses no gpui-kit item. The app is started with `gpui_platform::application()` and the root background and text use the `bourbon-ui` tokens.
