# 2026-11-08 — In-editor find and replace

Routine choices taken alone; to review at the UI pass. Research: [editor 2.2, 3.1, 3.6](../research/inkdrop-markdown-editor-preview.md), [shell keymap](../research/inkdrop-ui-shell.md).

| # | Topic | Decision | Why |
|---|---|---|---|
| 1 | Where | **Panel bar above the body, inside the editor pane** (`bourbon_editor::SearchPanel`) | Inkdrop's `.mde-search-bar` sits at the top of the editor |
| 2 | Keys | **`Ctrl+F` find; `Ctrl+H` and `Ctrl+Alt+F` replace; `Ctrl+G` / `Ctrl+Shift+G`; Enter / Shift+Enter; Esc; Alt+C / Alt+R; `Ctrl+Enter` replace all** | Research keymap is `Ctrl+Alt+F`; `Ctrl+H` was already Bourbon's and is shown in the control pane, so both stay. `Alt+W` added for whole word |
| 3 | Whole word | **Added** (toggle `ab`) | Asked for; Inkdrop shows only case and regex |
| 4 | Empty matches | **Skipped** (`^`, `x*` never highlight) | Nothing to draw or select; keeps every loop moving forward |
| 5 | Regex replace | **`$1` / `$name` expansion in regex mode only** | Plain mode must insert `$` literally |
| 6 | Search while typing | **Selects the first match at or after the selection start** | Incremental search; the editor scrolls to it |
| 7 | Count | **`n of m`, `0 of m` off a match, `No results`, `Invalid pattern`** | Inkdrop's count label, min 5 em |
| 8 | Colours | **Amber 500 at 25 %, current at 50 % with 1 px outline, same in both themes** | Research 2.2 |
