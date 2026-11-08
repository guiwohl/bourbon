# 2026-11-07 — Editor column on our own crates

Routine choices taken alone while rebuilding the editor column ([app](../architecture/app.md#editor-column)); to review at the UI pass.

| # | Topic | Decision | Why |
|---|---|---|---|
| 1 | First toolbar button | **Distraction-free toggle** (hides sidebar and note list, `Ctrl+Shift+F`), maximize-2 / minimize-2 icons | Inkdrop's first button; the sidebar alone already has its toggle in the note list header |
| 2 | Title and Add Tags fields | **`bourbon_editor::TextField` `borderless()` + `text_size`**, height and inset from the wrapping row | Same field as the filter; Inkdrop's title and tag inputs have no border |
| 3 | Find / replace | **`Ctrl+F` / `Ctrl+H` focus the body (switching from Preview)**; no search panel | `bourbon-editor` has none yet; recorded as a known gap |
| 4 | `Ctrl+Enter` | **`bourbon-editor`'s toggle**: a plain line becomes a task | One task rule for editor and keys; the old app-only rule is gone |
| 5 | Tag removal | **× next to each pill at 40 % opacity, full on hover** | Inkdrop has no visible ×; keeps the existing mouse path |
| 6 | Word count | **Kept, in a 24 px status bar on the right** | Existing Bourbon feature; Inkdrop's status bar slot is empty |
| 7 | Focus mode width | **Full width** (the 720 px cap is dropped) | Inkdrop's distraction free does not narrow the column |
| 8 | Scroll sync on note switch | **Reset both panes to the top**, no `SyncGuard::disable` | Disabling would swallow the first real scroll after a switch |
| 9 | ⋮ menu | **Note-row menu (Pin, Duplicate, Move to Trash) + Mark/Unmark template + Copy CLI command**, under the button | Research 3.4: same template as the row context menu |
| 10 | Dropdown position | **4 px under the trigger**, bounds recorded each prepaint | Research 3.4; lets `Ctrl+Alt+M` open at the same place |
| 11 | Gutter | **Line numbers on**, no fold chevrons | Matches the reference screenshot; folding does not exist yet, so no fake chevrons |
| 12 | Header icon size | **32 px buttons, 22 px icons, segments 44 x 30 with 20 px icons** (`bourbon-ui` tokens, all columns) | Lucide glyphs leave more inner padding than Inkdrop's Streamline set; this matches the reference's drawn size and its 32 px control height |
| 13 | Editor font | **Kept at the research value, 13 px / 1.5** | The reference screenshot measures wider glyphs and ~17 px lines, which no single research value explains |
