# 2026-11-07 — Rebuild the whole UI, Inkdrop-identical, without pre-made components

Decided with the user. Research: [inkdrop-ui-shell.md](../research/inkdrop-ui-shell.md), [inkdrop-markdown-editor-preview.md](../research/inkdrop-markdown-editor-preview.md).

| # | Topic | Decision | Why |
|---|---|---|---|
| 1 | Goal | **The UI must look and behave like Inkdrop 6** (layout, metrics, tokens, typography, interaction), above all the Markdown editor and preview, following the same component structure | The user wants Inkdrop's feel exactly |
| 2 | Components | **No pre-made UI components.** Drop `gpui-kit` / `gpui-component`; depend only on `gpui-pre` (the same GPUI fork) | Pre-made editor/preview/inputs could not reach parity |
| 3 | Clean room | **Reimplement from measured structure, values and behaviour; never copy Inkdrop code, CSS, icons or images** | Bourbon is GPL-3.0; Inkdrop is proprietary |
| 4 | Crates | **`bourbon-text`** (rope, selections, transactions, history, editing commands; no GPUI), **`bourbon-markdown`** (syntax tokens, preview document tree, scroll-sync math; no GPUI), **`bourbon-ui`** (our component kit and theme tokens; GPUI), **`bourbon-editor`** (EditorView, PreviewView, three-mode layout; GPUI), `bourbon-app` rebuilt on top | Same layering as core/app; logic stays testable without a window |
| 5 | Editor stack | **`ropey` buffer, `pulldown-cmark` + marker scanner for highlighting and preview, custom GPUI element for lines/caret/selection/gutter/IME/soft wrap** | Pure Rust, ranges available, one parser for editor and preview |
| 6 | Phases | **0** text core → **1** minimal editor + minimal preview → **2** `bourbon-ui` + Inkdrop shell → **3** editor/preview behaviours, three modes, scroll sync → **4** fine parity (cards, folding, `[[` completion), verified by screenshot diff against Inkdrop at identical sizes | Each phase is usable and testable |
| 7 | Known differences | Icons are free line icons in the same style (Inkdrop's set is licensed); text rendering is GPUI's, not Chromium's; squircle corners approximated; context menus drawn by us; math and mermaid later | Honest limits |
| 8 | Open PRs | **#29 merged** (core counts, task progress, stripped excerpts, relative dates, Inter font); **#30 closed** (editor rebuilt from scratch) | Keep the core work, drop the throwaway UI |

## Progress

- Phase 2 slice 1 (sidebar and note list on `bourbon-ui`, Inkdrop-identical, splitters, our menus): done; editor column, control pane and settings follow.
- Text field (`bourbon_editor::TextField`, in `bourbon-editor` so it reuses the text core; `bourbon-editor` depends on `bourbon-ui` for tokens and icons): done for the note-list filter and the sidebar name prompt. Text selection uses `focus_outline` (no selection token in Inkdrop inputs: the browser default applies).
