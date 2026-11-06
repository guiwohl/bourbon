# bourbon-app

GPUI window over `bourbon-core`. Holds no business rule: it presents state and forwards intent. Built on `gpui-kit` 0.9 with `tree-sitter-markdown` (re-exports GPUI and gpui-component).

## Behaviour

| Topic | Behaviour |
|---|---|
| Window | Title "Bourbon", Wayland `app_id` "bourbon" |
| Startup | Opens `default_database_path()`; failure prints a message and exits 1. Loads the list, selects the most recent note |
| Layout | Sidebar ("New note" button, note list; empty title shows "Untitled"; selected row highlighted) and right pane (title input, Markdown editor with soft wrap, no line numbers) |
| Empty state | No notes: the right pane shows a "New note" button |
| New note | Button or `Ctrl+N`: flush, create in the store, select, focus the title; the root view tracks its own focus handle, focused at startup when no note is selected, so `Ctrl+N` works with nothing else focused |
| Autosave | Any title or body change restarts one pending 500 ms task; when it fires: `save_note`, then reload the list. No manual save |
| Flush | Selecting or creating a note first saves the current one immediately; if that save fails the switch is cancelled |
| Close | `on_window_should_close` flushes the pending save while the view is alive. Success closes the window; failure keeps it open with the banner shown |
| Quit | `on_app_quit` flushes synchronously for quits that happen while the window is open. On Linux, closing the last window quits after the window and view are dropped, so the close hook is the one that matters there |
| Errors | `save_selected` and `flush_pending_save` return `Result<(), StoreError>`; the error is reported once at the boundary (autosave timer, select, create, close, quit). The banner is rendered once by the root view, above both panes, until the next successful save |
| Save vs reload | Only a save failure blocks switching, creating or closing. A failed list reload is reported but blocks nothing |
| Loading a note | `set_value` emits no `Change` event, so loading never schedules a save |

## Modules

| File | Responsibility |
|---|---|
| `main.rs` | Opens the store, aborts on error |
| `window.rs` | GPUI application, key binding, window options, the close hook |
| `notes_view.rs` | The root view: state, selection, save, create |
| `notes_view/sidebar.rs` | Sidebar rendering |
| `notes_view/editor_pane.rs` | Title, editor, empty state, the error banner element |
| `autosave.rs` | The 500 ms delayed task |

## Tests

Headless, through `gpui-kit` `test-support` (`#[gpui_kit::test]`, `TestAppContext`): the view takes a `NoteStore` (in memory in tests), clicks and typing go through real element ids, and the clock is advanced with `advance_clock`. They cover startup selection, autosave timing (499 ms vs 500 ms, restart on keystroke), flush on switch, creation, and the empty state. Also: close flush (through `simulate_close`, which runs the real `on_window_should_close` handler registered by `window::build_notes_view`), refused close on a failed save, quit flush (`App::shutdown`, which runs the `on_app_quit` handlers), pending autosave cancelled on switch, banner shown and cleared, failed save blocking a switch, and `Ctrl+N` through the real key binding. A save is made to fail honestly by selecting a note id the store does not hold.

## Dependency guard

`just architecture` fails if `cargo tree -p bourbon-core -e normal --prefix none` lists any crate starting with `gpui`, so it cannot go stale as GPUI crates are added. Proven by adding `gpui-pre-util` to `bourbon-core`: the recipe failed; reverted, it passed. `deny.toml` bans `gpui-kit`, `gpui-component`, `gpui-base` and `gpui-pre`, each with `wrappers` listing its exact direct parents (from `cargo tree -i`). Only `bourbon-app` may depend on `gpui-kit`, the entry point of the whole chain. Proven by adding `gpui-kit` to `bourbon-core`: `error[banned]`. `just deny` also prints many `duplicate` warnings from GPUI; they are expected.
