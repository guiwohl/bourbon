# Inkdrop 6.1.5 UI shell: structure, tokens, metrics, interaction

Written 2026-11-07. Research only; no Bourbon code changed. Clean-room notes: this document records structure, names and numeric values observed in the locally installed Inkdrop 6.1.5. No Inkdrop source, CSS blocks or assets are reproduced. Bourbon is GPL-3.0 and rebuilds everything natively in Rust/GPUI from these facts.

Related docs: [ui-analysis.md](ui-analysis.md) (Bourbon vs Inkdrop critique, estimated by eye from a screenshot), [inkdrop-theming.md](inkdrop-theming.md) (theme packaging). This document supersedes the by-eye numbers in `ui-analysis.md` with values read from the app. The Markdown editor and preview are out of scope (see `inkdrop-markdown-editor-preview.md`).

## Method and evidence conventions

Sources, all under the extracted app (`<APP>` = `/home/lowh/.claude/jobs/6d257de5/tmp/rea/inkdrop-app`, extracted app.asar):

| Tag | File | What it is |
|---|---|---|
| `[bundle]` | `<APP>/renderer/assets/app-BUTU7go0.js` | Main React 19 bundle: layout registry, sidebar, note list, editor header/meta, telescope, keymaps, context menus |
| `[ui-lib]` | `<APP>/renderer/assets/debounce-C-SokFGr.js` | Small chunk with the primitive components (modal frame, mde toolbar, title input) |
| `[icons]` | `<APP>/renderer/assets/dist-DbC_24yQ.js` | Icon registry (150 inline SVGs) |
| `[css]` | `<APP>/renderer/assets/app-ClbQ0QMw.css` | Compiled stylesheet (338 KB); component rules at the end of the file |
| `[tokens]` | `<APP>/node_modules/@inkdropapp/css/ui.css` | The UI token set (about 960 custom properties, light and dark in one place via `light-dark()`) |
| `[pal]` | `<APP>/node_modules/@inkdropapp/css/tokens.css` | Tailwind-style colour ramps as HSL triples |
| `[prefs]` | `<APP>/renderer/assets/preferences-layout-*.js/.css` | Settings window |

Evidence is given as file tag plus a searchable term, for example `[bundle] "main:full"`. Pixel values: the stylesheet is authored in a 14 px root (`--font-size: 14px`, 1 rem = 14 px); where the CSS uses rem fractions this document converts to px.

Important fact for the theme work: the shipped `default-light` and `default-dark` packages have an empty `ui.css` (an `@layer theme.ui {}` shell with a comment). They ship only `palette.json`, `syntax.css` and `preview.css` (editor and preview colours). **All shell colours come from `@inkdropapp/css/ui.css`, which uses `light-dark()` so one token set serves both appearances.** The `<body>` gets `color-scheme: dark` when it has class `dark-mode` or a class matching `dark-ui` (`<APP>/renderer/index.html`, inline `global-base-styles`). `themeAppearance` is `light` / `dark` in the two package.json files.

## 1. Component tree of the main window

### 1.1 Layout registry (the real tree)

Inkdrop does not hard-code the tree. It keeps a registry mapping a *layout name* to an ordered list of *component names*; a layout renders its children by looking each name up in a component registry (that is how plugins inject UI). The default registry is in `[bundle]` search `"main:full"` (second hit, the object that also holds `"sidebar-menu"`).

| Layout name | Ordered children (component names) |
|---|---|
| `main:full` | SideBarLayout, NoteListBarLayout, EditorLayout, ModalLayout |
| `main:slim` (sidebar hidden) | NoteListBarLayout, EditorLayout, ModalLayout |
| `main:distraction-free` | EditorLayout, ModalLayout |
| `sidebar` | AppControlView, SideBarNavigation, SideBarFooter, SideBarPluginUpdatesNotificationView, SideBarAutoUpdateNotificationView |
| `sidebar-menu` | SideBarMenuItemAllNotes, SideBarMenuItemTemplates, SideBarMenuSectionNotebooks, SideBarMenuBookListRoot, SideBarMenuSectionStatus, SideBarMenuStatusList, SideBarMenuSectionTags, SideBarMenuTagList, SideBarMenuItemTrash |
| `sidebar-workspace-menu` (drilled into one notebook) | SidebarWorkspaceMenuTitle, SideBarWorkspaceMenuAllNotes, SideBarMenuSectionNotebooks, SideBarWorkspaceMenuBookList, SideBarMenuSectionStatus, SidebarWorkspaceMenuStatusList, SideBarMenuSectionTags, SidebarWorkspaceMenuTagList |
| `note-list-bar` | NoteListHeader, NoteListSearchBar, NoteListHeaderOptions, NoteListHeaderSeparator, NoteListBarInviteFriendsPanel, NoteListBarContainer |
| `note-list-bar:trash` | NoteListHeader, NoteListBarContainer |
| `editor` | EditorHeaderLayout, EditorTitleBarLayout, EditorMetaLayout, MDELayout, MDESearchBar, PreviewFindBar, EditorStatusBarLayout, EditorAIPopover |
| `editor-header` | AppMenuButton, EditorHeaderToggleDistractionFree, EditorHeaderNavigateBack, EditorHeaderNavigateForward, EditorHeaderSpacer, EditorHeaderViewMode, EditorHeaderSpacer2, EditorHeaderButtonSpacer, EditorHeaderMore, EditorWindowBar |
| `editor-title-bar` | EditorTitleBarConflicted, EditorTitleBarRevision, EditorTitleBarTemplateIcon, EditorTitleBarInput |
| `editor-meta` | EditorMetaNotebook, EditorMetaStatusPicker, EditorMetaTags |
| `modal` | NewNotebook, DeleteNotebook, RenameNotebook, ChangeNotebookIcon, TagSettings, DeleteTag, AssistiveError, MoveNotes, MoveNotebook, EmptyTrash, ShareNote, NoteRevisions, UpdateNotification, NewNoteFromSelection, ChooseTemplate, EditorInsertLink, ImageViewer, TelescopeBar, NetworkDiagnosis (all `...Dialog` except TelescopeBar) |
| `editor-floating-actions` | EditorFloatingActionTogglePreview, EditorFloatingActionToggleSideBySide |
| `editor-toolbar`, `editor-status-bar` | empty by default (plugin slots) |

Selection logic: `[bundle]` search `main:distraction-free`: distraction-free flag wins, else sidebar visible gives `main:full`, else `main:slim`. State flags: `mainLayout.distractionFreeEnabled`, `mainLayout.sidebarVisible`, persisted as `core.mainWindow.distractionFreeEnabled` and `core.mainWindow.sideBar.visible`. After toggling either, the app dispatches `editor:refresh` after 100 ms. Toggling the sidebar also clears distraction-free. Mouse buttons 4 and 5 (back/forward) dispatch `core:navigate` on Linux and Windows.

Root: `#app-container` (full window, background `--page-background`, 0.4 s background transition) containing `.main-layout` (flex row, full height, 0.4 s opacity transition); the layout class also carries `main-layout-full|slim|distraction-free`. `<html>`/`<body>`: no scroll, `user-select: none`, text rendering "optimizelegibility". The window is frameless: the header strips are `-webkit-app-region: drag` zones, with the interactive controls opted out.

### 1.2 Visual hierarchy (full layout)

```
.main-layout (row)
|- .sidebar-layout (flex none, width 200 default)      <- SideBarLayout, resizable right edge
|  `- .sidebar-layout-content (column, bg --sidebar-background, border-right)
|     |- .app-control-view          (min-height 40: app icon 24, title "Inkdrop", spacer, app-menu + settings buttons)
|     |- .sidebar-navigation-container (flex auto, scroll)
|     |    `- .sidebar-menu | .sidebar-workspace-menu
|     |         |- All Notes, Templates            (items)
|     |         |- Notebooks  [search][add]        (section header + accessory buttons)
|     |         |    `- notebook tree rows
|     |         |- Status  [collapse toggle]       (section header)
|     |         |- Tags    [search][collapse]      (section header)
|     |         `- Trash
|     `- sync status footer (40 px) + update notification strips
|- .note-list-bar-layout (flex 0 auto, width 260 default) <- NoteListBarLayout, resizable right edge
|  `- .note-list-bar-layout-content (column, bg --note-list-bar-background, border-right)
|     |- .note-list-header (min-height 40, drag region): title bar (notebook icon + name, 1.2em, weight 500, centred)
|     |- .note-list-search-bar (34 px)
|     |- .note-list-header-options: sort dropdown pill
|     `- .note-list-bar (scroller) > .note-list-holder (virtual height) > rows (sticky section headers + note items)
`- .editor-layout (column, flex 1, bg --editor-background, min-width 0)
   |- .editor-header-layout (min-height 40, padding-left 4): [menu*] [distraction-free] [back] [forward] <spacer> [view-mode segmented x3] <spacer2> [more "..."]
   |- .editor-title-bar-layout (row): [conflict badge] [revision badge] [template icon] title <input> (1.5em)
   |- .editor-meta-layout (wrap, padding 0 12): notebook picker, status picker, tags bar (+ dropdown)
   |- .mde-layout > .mde (CodeMirror) | .mde-preview-container   (+ find bars, floating actions bottom-right)
   `- .editor-status-bar-layout (empty slot; children margin .4rem)
```
`*` the app-menu button is shown only when the sidebar is hidden or on a slim layout (CSS rules for `.app-menu-button` visibility; hidden on macOS).

### 1.3 Resizable and collapsible

| Pane | Resize | Default | Min | Max | Persist key | Evidence |
|---|---|---|---|---|---|---|
| Sidebar | right edge only | 200 | 170 | 500 | `core.mainWindow.sideBar.w` (saved on resize stop, from client width) | `[bundle]` "SideBarLayout" ; `[css]` root var `--sidebar-width:200px` |
| Note list | right edge only | 260 | 220 | 500 | `core.mainWindow.noteListBar.w` | `[bundle]` "NoteListBarLayout"; `--note-list-bar-width:260px` |
| Editor | takes the rest (`flex: 1`, `min-width: 0`) | - | - | - | - | `[css]` `.editor-layout` |

- Resizer: a generic resizable-box library (re-resizable pattern). The invisible handle is 10 px wide, positioned -5 px outside the pane's right edge, `col-resize` cursor, mouse-down and touch-start start the drag. There is **no visible splitter**; the pane's 1 px right border (`--sidebar-border-right`, `--note-list-bar-border-right`) is the only visual divider. No hover highlight on the handle was found. `[bundle]` search `col-resize`.
- Width is applied live while dragging and written to config on release.
- Collapse: the whole sidebar toggles (`view:toggle-sidebar`, Ctrl+/). No partial collapse of the note list. Distraction-free hides both.
- Inside the sidebar, sections collapse individually: Status and Tags sections have a chevron accessory button (arrow-up-1 / arrow-down-1 icons); hidden state is stored in `core.mainWindow.hiddenComponents[layoutName]`. Notebook folders expand and collapse with a chevron toggle that rotates 90 degrees over 0.2 s ease-out; collapsed book ids persist in `core.mainWindow.sideBar.collapsedBooks`.
- Drill-down "workspace" mode: the sidebar can swap its content to a single notebook's subtree (breadcrumb title row, back button, left-right slide transition keyed on direction). Entered via "Show Detail.." / the hover pill; exited with Backspace or the back button.

### 1.4 Other windows

- Preferences: separate window type; `.preferences-layout` = row of a 160 px vertical menu (`.preferences-sidebar`) and a scrolling content pane (padding 12). Captions: Account, Backup, Editing, General, Integrations, Keybindings, Local Server, Plugins, Preview, Sync, Themes, Updates, Install (`[prefs]` search `caption:`).
- Login view, loading view (centred spinner plus progress bar while the full-text index rebuilds), error boundary page.
- Notification bar (lazy chunk `notification-bar-*`), toasts at the window level.

## 2. Design tokens

All values read from `[tokens]` unless stated. `light | dark` means the two arguments of the `light-dark()` call. Colour ramps are Tailwind v3-compatible HSL triples in `[pal]` (names `--hsl-<ramp>-<50..950>`, ramps: rose, pink, fuchsia, purple, violet, indigo, blue, sky, cyan, teal, emerald, green, lime, yellow, amber, orange, red, neutral, stone, zinc, gray, slate). Hex values below are converted from those HSL values by me.

### 2.1 Scale tokens

| Group | Token | Value |
|---|---|---|
| Root | `--font-size` | 14px (1rem = 14px) |
| Type scale (x 14px) | `--mini-size` 11/14, `--tiny-size` 12/14, `--small-size` 13/14, `--medium-size` 14/14, `--large-size` 16/14, `--big-size` 18/14, `--huge-size` 20/14, `--massive-size` 24/14 | 11, 12, 13, 14, 16, 18, 20, 24 px |
| Headings | `--h1..--h6` | 2em, 1.5em, 1.25em, 1em, 0.875em, 0.85em |
| Heading weight/leading | `--header-font-weight`, `--header-line-height` | 600, 1.2857em |
| Body leading | `--line-height` | 1.4285em (20 px at 14 px) |
| Font weights | `--font-weights-*` | thin 100 ... normal 400, medium 500, semibold 600, bold 700 ... black 900 |
| Px helpers | `--1px --2px --4px --8px --12px --15px --24px` | 0.0714rem (=1px), 2, 4, 8, 12, 15, 24 px |
| Input padding | `--input-vertical-padding`, `--input-horizontal-padding` | 11px, 14px (as em fractions) |
| Radii | `--absolute-border-radius`, `--default-border-radius` | 7px |
|  | `--default-squircle-radius` | 7px by default; **15px** when the engine supports `corner-shape: squircle` (Electron 6.1.5 does); `--large-squircle-radius` 12px, **24px** with squircle |
|  | `--circular-radius` | 500rem (pill) |
|  | tag label radius | 12px; segmented control inner radius = squircle radius - 1px; checkbox radius 8px |
| Motion | `--default-duration`, `--default-easing` | 0.1s, `ease` |
| Loader | `--loader-speed`, `--loader-line-width` | 0.6s, 0.2em |
| Opacity | `--disabled-opacity` | 0.45 |
| Scrollbar | `--scrollbar-width` | 6px (hover widens to 14px on Linux/Windows) |
| Z-index | popup 10, tooltip 20, dropdown menu 11, modal container 8, `.ui.modal` 1001, tags dropdown 11, editor floating actions 3, window bar 3 | |
| Icon | `--icon-width` | 1.18em; `--glyph-width` 1.1em |

### 2.2 Fonts

`--font-name`: `-apple-system, BlinkMacSystemFont, Segoe UI, Lato`; `--page-font` and `--header-font` append `Helvetica Neue, Arial, Helvetica, sans-serif` plus the three emoji faces. `--font-smoothing: antialiased`. Code font and editor font are separate (other doc). On Linux this effectively falls through to the system sans (Lato if installed). Bourbon should pick a good neutral sans and treat Lato as the closest named reference.

### 2.3 Shell colours: light and dark

Hex = my conversion of the Tailwind-style HSL triple; alpha is kept as written. "black/87%" means hsl(black) with 87 percent alpha.

| Token | Light | Dark |
|---|---|---|
| `--page-background` | white `#ffffff` | neutral-950 `#0a0a0a` (initial pre-render value in index.html: neutral-300 `#d4d4d4` light / neutral-950 dark) |
| `--text-color` | black 87% | neutral-50 `#fafafa` 87% |
| `--muted-text-color` | black 60% | neutral-50 60% |
| `--light-text-color` | black 40% | neutral-50 85% (note: not monotonic, kept as shipped) |
| `--dark-text-color` | black 85% | neutral-50 40% (inverted naming, as shipped) |
| `--unselected-text-color` | black 40% | neutral-50 40% |
| `--hovered-text-color` / `--pressed-text-color` / `--selected-text-color` | black 80% / 90% / 95% | neutral-50 80% / 90% / 95% |
| `--disabled-text-color` | neutral-800 30% | neutral-50 30% |
| `--inverted-text-color` | white 90% | neutral-950 90% |
| `--primary-color` | blue-500 `#3c83f6` | blue-400 `#61a6fa` at 80% alpha (index.html pre-render fallback `#4f8eff`) |
| `--primary-color-hover/focus` | blue-600 `#2463eb` | same |
| `--primary-color-down/active` | blue-700 | same |
| `--link-color` | blue-500 | blue-500 (hover: blue-600 light, blue-400 dark) |
| `--highlight-background` (selection) | blue-200 | sky-600 `#0284c5`; highlight text: text color / white |
| `--border-color` | neutral-900 15% | neutral-100 10% |
| `--strong-border-color` | neutral-900 22% | neutral-100 22% |
| `--internal-border-color` | neutral-900 10% | neutral-100 10% |
| `--selected-border-color` | neutral-900 35% | neutral-100 35% |
| `--strong-selected-border-color` | neutral-900 50% | neutral-100 50% |
| `--solid-border-color` | zinc-300 `#d4d4d8` | zinc-700 `#3f3f46` |
| `--solid-internal-border-color` | neutral-50 | neutral-800 `#262626` |
| `--subtle-transparent-black` / `--transparent-black` / `--strong-...` / `--very-strong-...` | 3% / 5% / 10% / 15% black | 6% / 10% / 20% / 30% black |
| `--subtle-transparent-white` / `--transparent-white` / `--strong-transparent-white` | 2% / 8% / 15% white (both modes) | same |
| Focus ring colour `--focused-outline-color` | sky-500 `#0da2e7` 40% | sky-300 `#7ed4fc` 30% |
| Focused input border `--focused-form-border-color` | sky-600 50% | sky-300 60% |
| Focus ring geometry | width 2px, solid, offset 0 when focused, offset 0.4rem (5.6px) when unfocused and transparent | same |
| `--input-background` | white | neutral-900 `#171717` |
| `--input-color` | text colour | neutral-50 85% |
| `--input-placeholder-color` / focus | rgb(191 191 191 / 87%) / rgb(115 115 115 / 87%) | neutral-200 25% / neutral-300 35% |
| `--input-border` | 1px solid `--border-color` | same |
| Semantic positive/negative/info/warning | green-500 / red-500 / sky-500 / amber-400 | green-500 / red-500 / cyan-400 / amber-400 |
| Message backgrounds | colour at 10% | pale ramp step (200) at 20% |
| `--mark-*` (search highlight) | yellow-400 20% background, 40% border, weight bold | same; text gray-950 / gray-50 |

Panel surfaces:

| Token | Light | Dark |
|---|---|---|
| `--sidebar-background` | gray-100 `#f3f4f6` | neutral-900 `#171717` |
| `--sidebar-border-right` | 1px `--border-color` | 1px neutral-800 `#262626` |
| `--note-list-bar-background` | stone-50 `#fafaf9` | neutral-900 at 50% over the page colour |
| `--note-list-bar-border-right` | 1px gray-200 `#e5e7eb` (`--white-down`) | 1px neutral-800 at 75% |
| `--editor-background` | white | neutral-950 `#0a0a0a` |
| Section header background (note list) | neutral-200 `#e6e6e6` | neutral-900 `#171717` |
| `--editor-drawer-background` | white 70% | neutral-900 70% (+ 1px left border) |
| `--editor-floating-actions-background` | white 60% | neutral-800 60% |
| `--preferences-sidebar-background` | gray-100 | neutral-900 |
| `--preferences-view-background` | white | neutral-950 |
| `--notification-item-background` | white 80% | neutral-900 80% |
| Menus / popups / dropdowns / modal body background | white | neutral-900 |
| Modal actions strip background | white-ish (off-white `#f9fafb`) | neutral-800 |
| Segmented/segment background | white | neutral-900 |

Selection states (sidebar and note list share the same pair):

| Token | Light | Dark |
|---|---|---|
| Item selected, panel not focused (`--sidebar-menu-item-inactive-background`, `--note-list-view-item-selected-background`) | sky-800 `#075783` at 20% | blue-300 `#91c3fd` at 20% |
| Item selected, panel focused (the "active" token, used for the 5 px left accent bar and for drop highlight) | `--primary-color` | blue-300 at 30% (sidebar) / 40% (note list) |
| Sidebar item text | dark-text (85%) | muted (60%); active item: dark-text / full text colour |
| Sidebar section header text | unselected (40%) | unselected (40%) |
| Sidebar section separator | internal border | white 10% |
| Sidebar action icon (idle / hover) | light-text / light-text hover | disabled-text / light-text |
| Note list date text | `--primary-color` | `--primary-color` |
| Note list title / excerpt text | text colour / muted | same |
| Selection-list (dropdown/command palette) hover | subtle-transparent-black | subtle-transparent-white |
| Selection-list active | transparent-black (5%) | transparent-white (8%) |
| Menu hover / active item | subtle (3%) / transparent-black | subtle white / transparent white |
| Dropdown hovered item | transparent-black | transparent-white |
| Sync status footer background | subtle-transparent-white | neutral-800 50% |
| Notification strip (update available) | warning-color background, warning-text colour | amber-900 text on warning background |

Buttons (default grey button): background gray-200 / neutral-800 70%; hover gray-300 / neutral-700 70%; down gray-400 / neutral-800 80%; active gray-400 / neutral-700 70%; text black 60% / neutral-50 60%; weight 600; inset 1px ring `--strong-transparent-black` / white; radius squircle 15px. Basic (outline) button: transparent, inset 1px `--border-color`, hover white / neutral-800 50% with a stronger ring. Positive/negative buttons: solid semantic colour, inverted text.

Shadows:

| Token | Value (summary) |
|---|---|
| `--subtle-shadow` | 0 1px 2px 0 in border colour (light) / black (dark) |
| `--floating-shadow` | two layers: 0 2 4 0 at 12% / 20% and 0 2 10 0 at 15% / 30% (dark: black) |
| `--floating-shadow-hover` | 0 2 4 15% and 0 2 10 25% |
| `--raised-shadow` | three layers: 0 1.5 3 -2 at 40% black, 0 4 6 at 20%, 0 8 12 -1 at 10% (dropdown menus, tags dropdown) |
| Modal shadow | six stacked layers growing from 2.8 px to 100 px blur, 6.7% to 24% black |
| Command palette shadow | 0 10 30 at 30% black |

Note status colours (`status.css`, shell and preview): active slate-400 `#94a3b8`, on-hold amber-500 `#f59f0a`, completed emerald-500 `#10b77f`, dropped rose-400 `#fb6f84` (same in light and dark). None status has no icon.

Task-progress bar (`task-progress.css`): border gray-950 30% light / gray-50 10% dark; track gray-950 30%; fill white 60% light / gray-50 60% dark; completed green-500 `#21c45d`. In the active (selected) note row the track/fill switch to the foreground variants. The view uses `mix-blend-mode: multiply` in light and `screen` in dark.

Tag chip palette (`tags.css`): per colour (default, red, orange, yellow, olive, green, teal, blue, violet, purple, pink, brown, grey, black) eight tokens: background, border, text, active-background, active-border, active-text, circular-background, circular-border. Pattern: light variant background colour-400 at 20%, border 40%, text colour-500; active background 40%, border 60%; dark variant background 10%, border 20%, text colour-400. Default (no colour) chip: slate-400 border 40%, gray-400 20% fill, gray-500 text (light); slate-400 20% / gray-400 10% / gray-400 text (dark). Sidebar tags use a "circular" dot (20 px wide slot, coloured fill plus ring).

### 2.4 Acrylic variant (translucent window)

When `body.acrylic-window` is set (macOS vibrancy, Windows acrylic): page background transparent; sidebar becomes 2% white / black 10% (20% on Windows dark with page black 40%); note list stone-100 80% / neutral-900 50%; editor white 70% / neutral-950 60%; menus and drawers go opaque. `[tokens]` search `acrylic-window`. Bourbon can ignore this initially.

## 3. Component specifications

Conventions: values in px at the default 14 px root. "State: none" means no rule exists in the stylesheet (checked), not that I could not find it.

### 3.1 App control strip (top of sidebar)

- `.app-control-view`: min-height 40, padding-right 2. Title block: app icon 24x24 + label "Inkdrop" at 60% opacity, gap 6, margin 3 0 3 11. A flexible spacer (drag region; double-click toggles maximize). Buttons right-aligned: app menu (icon `navigation-menu`) and settings (`cog`). Both are "app header buttons". macOS hides the title and menu button (traffic lights live there).
- Header button (`.app-header-button`): borderless, transparent, centred icon; hover background `--button-hover-background-color`; `.active` background `--button-focus-background-color`; disabled (back/forward at history ends) uses the generic disabled opacity. Pointer cursor. Hit size comes from the generic icon-only button (font-size 1.2857rem = 18 px icon, padding from `--button-padding`); the segmented-control buttons override to width 44, padding 6 8.
- Each header button is a "command button": tooltip text is the command title plus a `kbd` chip with the current keybinding (resolved from the keymap). This is how every toolbar tooltip shows its shortcut.

### 3.2 Sidebar

Container: `box-sizing: border-box`, column, full height, background and border transitions 0.4 s ease-in-out, `user-select: none`.

Scroller `.sidebar-menu`: full size, overflow auto; custom 6 px scrollbar on Linux/Windows (track `--scrollbar-track-background` 5% black / 2% white; thumb 20% black / 20% white; thumb transparent when the window is inactive; widens to 14 px on hover).

Row `.sidebar-menu-item` (All Notes, Templates, notebook, status, tag, Trash):

| Property | Value |
|---|---|
| Min height | 30 |
| Padding | 0 6 0 14 (left indent grows per tree depth; `indentLevel` prop) |
| Gap between icon, label, accessories | 0.3em (~4 px) |
| Font | 14 px, weight normal; label nowrap, ellipsis |
| Icon slot | 16 px fixed, margin 1 (notebook icon also 16); tag dot slot 20 px |
| Background | none; shape: rectangle, radius 0, full width (no inset, no rounding) |
| Left accent | inset box-shadow 5 px, transparent normally; becomes the active colour only while the sidebar has focus-within and the row is active |
| Transition | background-color, box-shadow, color: 0.1 s ease |
| Hover | **no background change**. Hovering reveals (a) the "disclosure" pill if the row has one and hides the count |
| Selected, sidebar unfocused | background = inactive tint (sky-800 20% / blue-300 20%), text = active-item colour |
| Selected, sidebar focused | same tint plus the 5 px left bar in `--sidebar-menu-item-active-background` (primary colour in light) |
| Count (`accessory-count`) | 10 px, weight 200, opacity 0.6, margins 4; hidden on hover when a disclosure exists |
| Accessory button | 16 px icon, weight 400, colour = action colour, hover = action-hover colour, last one margin-right 0.2em |
| Disclosure pill | border 1px in action colour, radius 16, padding 4 0 4 6, label 8-9 px (xx-small) weight 300; hidden until row hover |
| Drop-over-notes highlight | the row content background becomes the active colour (dragging notes onto a tag/notebook/status) |

Section header (`.sidebar-menu-section-header`, Notebooks, Status, Tags): same row, label 12 px, colour `--sidebar-menu-section-color` (40% text). Accessories at the right: Notebooks gets [search] [add-circle]; Tags gets [search] [collapse chevron]; Status gets [collapse chevron]. Sections are not uppercase and are not bold.

Notebook rows: expand toggle (chevron icon, 12 px, stroke 3 px, opacity 0.5, hover 0.8, rotates 90 degrees when open over 0.2 s ease-out), then 16 px notebook icon (user-selectable icon), name, count. Nested books indent per depth.

Drag and drop of notebooks (reorder / nest):
- Dragged row: opacity 0.5.
- Drop before / after: 2 px line in `--primary-color`, positioned -1 px outside the row top/bottom, full row width.
- Drop into: 2 px dashed outline in `--primary-color`, offset -2, radius default.
- Custom drag preview: floating chip with input-background, 7 px radius, hover floating shadow, 1 px solid-border outline, padding 8 12, notebook icon + name (max 300 wide), offset 12/12 from the cursor.
- Dragging notes: preview chip padding 8, icon plus title ellipsised, same chrome.

Workspace title row (when drilled in): min-height 42, padding-left 15 / right 8, bottom border (section separator colour), margin-bottom 6, padding-bottom 6; back button (16 px icon), breadcrumb items 12 px with 4 px padding and 8x8 chevron-right separators at 60% opacity.

Blank slate (no tags yet): small 11 px message in an info message box, horizontal margin 1.5em.

Footer:
- Sync status: 40 px tall, row, padding 6, top border 1px, background tint; status text 12 px weight 400 in muted colour (message ellipsised), detail line 10 px (unselected colour) with a trailing dropdown chevron; "syncing" shows an ellipsis animated 0.9 s in 4 steps. Error state: solid negative-colour background, padding 14 10, bold 14 px message.
- Update strips (plugin updates, app update): warning background, 11 px text, padding 4 4 4 14, icon plus text, clickable.

### 3.3 Note list

Header `.note-list-header`: min-height 40, padding 0 2, drag region. Centre title block: font 1.2em (~16.8 px), weight 500, line-height 130%, centred, ellipsis, gap 0.3em, leading notebook/tag icon at 0.8 opacity, 1 em. The title reads the current context: All Notes, notebook name, tag, status, Trash, Templates. Right-click on it gives the notebook/tag context menu.

Search bar `.note-list-search-bar`: fixed 34 px (flex 0 0 34). Input wrapper margin 2 12; input padding 4 8; border 1px internal-border when unfocused, none when focused (the focus ring replaces it: 2 px outline animating from a 5.6 px offset to 0 over 0.1 s ease). Left magnifier icon container 2.67em wide at 30% opacity (transition 0.3 s), clear "close-circle" icon on the right. Enter moves focus to the list; Ctrl+Enter focuses the editor; Esc clears.

Options row `.note-list-header-options`: min-height 40, padding 0 0.8em. Contains the sort dropdown: pill with 1px border, radius 1.5 x default (about 10.5 px), padding 0 0.5em, font 0.8em (~11 px), label = sort key + up/down thin arrow icon. Menu header "Sort by" (0.7rem), entries with a 1.4em check slot showing a filled dot for the current choice: Best match (search only), Title A to Z / Z to A, Date Created new to old / old to new, Date Updated new to old / old to new, divider, "Set as default".

Scroller `.note-list-bar`: focusable (tabindex 0), outline none, flex auto, class `single-selection` / `multiple-selection`.

**Virtualization**: TanStack Virtual-style row virtualizer. Absolute-positioned rows moved with `translateY`, container height = total size, `will-change: transform`. Estimated sizes: 32 for section headers, 87 for note rows; rows are measured after render, so real heights vary (title up to 2 lines). Overscan: library default (1). Section headers are sticky: the header for the current section is always included in the rendered range and positioned sticky at the top. When the last virtual row is reached and not loading, `loadMore(50)` fetches 50 more notes (paged data, not all in memory). On open note change, the list scrolls the selected item to the centre if it is out of view. Insert/remove uses a height+opacity animation (layout animation) only when an "animation scheduled" flag is set (note change events); otherwise rows render plain. Evidence: `[bundle]` search `note-list-holder`, `rV=32,iV=87`, `loadMore(50)`.

Section header `.note-list-section-header` (Pinned and similar): uppercase, letter-spacing 0.02em, 11 px, weight 600, padding 6 12, background neutral-200 light / neutral-900 dark, text colour full. Pinned sections end with a 4 px transparent bottom spacing.

Note row `.note-list-item-view` (the whole row is also `.note-list-bar-item`, with `.pinned` and task-status classes):

| Part | Spec |
|---|---|
| Row padding | top 2, right 0, bottom 10, left 14; 1px bottom separator (`--note-list-view-item-separator-border`: gray-200 light, neutral-800 at 75% dark) |
| Title | weight 600, colour text, max 2 lines (line clamp), max-height 2.7em, margin 8 8 8 0, word-break; font size = base 14 px |
| Title leading icons (inline, in order) | conflict warning (colour warning), pin (colour primary), status icon, share/visibility icon; then the title text. Untitled notes show "Untitled" |
| Dropped-task title | strike-through |
| Meta line (`.meta`) | inline; children spaced 0.4em; 12 px, line-height 1.35em. Contents in order: date (colour primary, weight 400, nowrap; not shown for templates), task progress bar, tag chips |
| Date format | relative ("fromNow", short form) unless config `showAbsoluteDate` is on or the note is older than 37 days, then `YYYY-MM-DD`; which timestamp: created or updated, following the active sort key |
| Task progress | pill 32 wide x 6 high, 1px border, radius 12, margins 2 / 4; fill proportion = checked / total tasks; only if the note has at least one task. Turns green when complete |
| Tag chip (list) | radius 12, padding 3 4, 8-10 px text (x-small), weight 400, 1px inset ring |
| Excerpt | 14 px, muted colour, margin-right 8, padding-top 3-4; single line with ellipsis (nowrap); source is the body with front-matter stripped, first 128 characters. In search mode it shows up to 10 matching lines with `mark` highlights (yellow 20% fill, 1px border 40%, radius 4, bold) |
| Selected (not focused list) | background `--note-list-view-item-selected-background` (sky-800 20% / blue-300 20%) |
| Focused item in a focused list | additional 5 px inset left bar in the active colour (primary / blue-300 40%) |
| Hover | **none defined** |
| Multi-select | click = open; Shift+click = range from the open note; Meta/Cmd+click toggles; Ctrl+A selects all (list scoped command) |
| Double-click / middle-click / Shift+Enter | open in separate window |
| Right-click | if the clicked note is in the selection, the action targets the whole selection, else only that note; then the native context menu |
| Drag | note rows are drag sources (type "notes"); drop targets are sidebar notebook, tag and status rows |

Empty and error states (centred column, whole block at opacity 0.5, bottom padding 80): large icon 50 px at 0.3 opacity or an illustration, message 20 px light weight, small tips with a help link. Variants: "Input search query" (search icon), "No Notes Found" plus how-to-search link, "No <status> notes" plus explanation, trash illustration (100 px), generic error (message 20 px, icon 50 px at 0.2).

Invite-friends panel (promo card) is shown in the list above the notes: 4 px padding, blue 5080d7 card. Not relevant for Bourbon.

### 3.4 Editor header (top bar of the editor column)

- `.editor-header-layout`: row, min-height 40, padding-left 4, transparent, centre aligned; opacity transition 1.2 s ease-in-out (fades with distraction-free). On macOS a 76 px spacer (`.editor-header-button-spacer`) reserves traffic-light room; elsewhere a window-controls bar sits on the right.
- Order left to right: [app menu button, only when sidebar hidden], [distraction-free toggle: icon `expand` / `shrink`], [navigate back: `arrow-left`], [navigate forward: `arrow-right`], flexible drag spacer, **view-mode segmented control**, second spacer, [more menu: `navigation-menu-vertical`], window bar. Back/forward are disabled when history offset is at either end.
- View-mode segmented control `.editor-header-view-mode`: container with 1px `--border-color`, radius 15 (squircle), `flex: none`. Three buttons, each 44 wide, padding 6 8, radius 0 except outer corners (radius = container radius - 1). Order: Show Editor (icon `pen-1`), View Side by Side (`layout-two-colums`), Show Preview (`view-1`). The selected one swaps to the `-bold` icon variant and carries `.active` (filled background). No sliding thumb, no animation.
- Floating actions (bottom-right of the editor, `.editor-floating-actions`): vertical pill, padding 0.3em, radius 1em, 1px solid border, `backdrop-filter: blur(10px)`, translucent background, 6 px from the bottom-right corner; hidden (opacity 0, display none) until visible; fade-in 0.4 s; buttons at 50% opacity, 100% when enabled. Holds quick toggles for preview and side-by-side.
- More menu: opens a native menu placed under the button (4 px below). Same template as the note-row context menu (see 3.10) with disabled states for trash/read-only/template.

### 3.5 Title bar and metadata

- `.editor-title-bar-layout`: row, padding-right 8, optional badges, flexible input.
- Title input: transparent background, no border, no shadow, **font-size 1.5em (21 px)**, height 1.7em (~36 px), padding-left 0.5em (10.5 px), text colour = input colour. Placeholder "Untitled". Key behaviours: Enter moves focus into the editor body (`editor:move-focus-to-next`), Down arrow focuses and jumps to document start, Ctrl+W disabled in this field.
- Conflict badge: pill height 26, radius 15, warning background/border/text colours, "small" font, icon plus label; revision badge: info tint, max-width 120, icon `history` plus short label, radius 15, padding 0 6.
- Meta row `.editor-meta-layout`: wrap, horizontal padding 12, font 0.9rem (12.6 px). Items:
  - Notebook picker: min-height 28, margin-right 4; inline dropdown showing notebook icon, name, small chevron; menu min-width 220; the menu is a searchable list (see 3.8). Hidden for unsaved template notes. Right-click gives Open Notebook / Go to Workspace / Copy Notebook ID.
  - Status picker: dropdown whose label is the status icon plus "Status" or the status name plus chevron. Menu items show icon, label and a shortcut hint (x-small, 40% opacity). Shortcuts Shift+Alt+1..5 for None, Active, On hold, Completed, Dropped.
  - Tags bar `.note-tags-bar`: wrap, min-height 32, padding-right 2. Each tag chip: label padding 4 6, margin 6 2; active (keyboard-selected) chip gets the 2 px focus outline. Input: transparent, no border, height 24, padding 4, min-width 6em, font 0.9rem. Autocomplete dropdown below: width 40% (min 200, max 300), max-height max(40vh, 200px), raised shadow, 2 px below the bar, z-index 11; items show a coloured dot (8 px) plus the name; "create tag" item in muted colour with the new name in selected colour; empty placeholder in placeholder colour.

### 3.6 Editor body frame (shell only)

`.editor` row, centred; `.mde` flex 1; `.mde-layout` and `.mde-preview-container` both flex 1 with min-width 0; preview container scrolls, padding 10. "Limit max width" mode centres the text column at a readable width (CSS var `--readable-line-length-width`, 740 px default) in both edit and preview. Background transitions 0.4 s. The in-editor formatting toolbar (`.mde-toolbar`: padding 4 6, buttons 2.2em = ~31 px circles-ish, radius 1.4em, transparent 1px border, hover shows `--strong-selected-border-color` ring plus basic-button hover fill; disabled opacity 0.5; separators 1px, margin 0 6) is only displayed while the CodeMirror editor has focus. Find bars (`.mde-search-bar`, `.preview-find-bar`): padding 8, find field plus regex and case-sensitive toggle buttons (padding 8), match-count label min 5em, 140 px wide button column.

### 3.7 Blank and multi-select editor states

- No note open: `.editor-blank-slate`, centred column, text colour disabled-text, an illustration at 30% opacity (max 300 high, status illustration max 160 wide), message 24 px light weight, guidance 20 px light, tips max 640 wide with 1.4 line-height.
- Multiple notes selected: placeholder heading (h2/h3 at 300 weight, light-text colour) and a centred control panel segment (max 500 wide, padding 2em 1em, wrap, gap 0.2em) with batch actions (move, status, delete).

### 3.8 Dropdowns, selection lists and menus (in-app, not native)

Dropdown (`.ui.dropdown`): transition box-shadow/width/outline 0.1 s. Menu: background white / neutral-900, border 1px `--border-color`, radius squircle 15, raised shadow, z-index 11, `min-width: max-content`, no padding. Item: padding 11 16 (0.7857rem 1.1429rem), font 14 px, line-height 1, hover background transparent-black / transparent-white, active item font weight bold (inline dropdown) with no background. Header: uppercase, 11 px (0.7857em), weight 600, margin 14 0 10, padding 0 16. Divider: 1px internal-border, margin 0.5em 0. Chevron: dropdown-chevron icon at 0.7em (small). A searchable dropdown puts a borderless full-width input (min-height 2.5rem) at the top of the menu. Menu max-height max(50vh, 200px) for inline dropdowns.

Keyboard in lists (selection list `.list-bar`): Up/Down move, Home/End first/last, Enter submits; the selected row is highlighted with the hover tint, and tinted + 2 px focus outline (offset -2) when the list owns focus. Each dropdown/list gets its own key handler only while open.

Popup/tooltip: background popup (white / neutral-900), 1px solid border, radius 15 squircle, floating shadow, padding 11.7 x 14 (0.833em x 1em); tooltip font 11 px (mini), muted text, line-height base, with a 5 px-border arrow drawn by rotated box. Command tooltips lay out title and `kbd` chip in a column with 0.3em gap; `kbd`: muted colour, off-white / 2% white background, 1px border.

### 3.9 Dialogs (modals)

- Backdrop: container fills the window (`position: absolute; inset: 0`, z-index 8), centred content; background animates from transparent to black at 30% (the code sets the container transition to 2 s, the exit to 0.4 s). Click on the backdrop closes; Esc closes (`core:cancel`). Focus is moved into the dialog on open (optional autofocus) and restored to the previously focused element on close.
- Content enter: opacity 0 to 1, blur 10 px to 0, scale 80% to 100%, 0.2 s; exit: opacity to 0 and blur back to 10 px over 0.2 s; `filter: none` is applied after the enter finishes (avoids blur-compositing cost).
- Frame (`.ui.modal.dialog-frame`): column, min-width 240, max-height 90vh, radius 24 (large squircle), six-layer shadow, no border. "Large" variant: width 60vw, min-width 520, min-height 360. Fixed breakpoints for generic modals: 95% below 768, 88% to 991, then 850, 900, 950 px.
- Parts: header (padding 16 x 17, bottom border, font 14 px weight 600, line-height 1.2857, icon plus text), content (padding 14, line-height 1.4; flex variant; no-padding variant), actions strip (flex, background neutral/off-white, top border, buttons min-width 10em, margin-left 0.75em; `space-between` variant; destructive button = negative colour fill).
- Position variants: `dialog-move-notes` aligns top with 12vh margin; command palette aligns top with 4em margin.
- Dialogs present: new/rename/delete/move notebook, change notebook icon (grid with empty-state slot), tag settings (colour), delete tag, move notes (searchable notebook list), empty trash, share note, note revisions (timeline diff list with a vertical line, 2 rem round badges), new note from selection, choose template (30/70 split: grouped list left with uppercase 11 px section headers, preview pane right), insert link, image viewer (full-window, 48 px translucent toolbar with blur 6, zoom cycle fit/2x/1x/0.5x), update notification (progress bar 10 px high, radius 10, 0.3 s width ease), network diagnosis.

### 3.10 Context menus

Context menus are **native OS menus**, not custom-drawn. The app keeps a selector-to-template map (`[bundle]` search `"context-menu"`): on right-click it finds the matching selectors for the event target and its ancestors, merges the templates, evaluates `visible` / `enabled` per item with the current state (read-only, trash, template), and shows an OS menu at the pointer. The editor "more" button shows the same menu anchored under the button. Bourbon must draw its own, since GPUI has no native menu path here.

Templates by target (labels summarised):

| Target | Items |
|---|---|
| Notebook row | Show Detail, Copy Notebook ID, ---, New Sub Notebook, Rename, Change Icon, Move, Delete |
| Note row / editor more menu | Move to Notebook, Duplicate, Pin to Top / Unpin from Top (one visible), ---, Copy Note Link / URI / ID, ---, Open in Separate Window, Revision History, Share on Web, ---, Move to Trash (becomes Delete Permanently in trash) |
| Tag row | Tag Settings, Filter by Tag, Copy Tag ID, ---, Delete Tag |
| Trash row | Empty Trash |
| Tag chip in note | Tag Settings, Remove Tag, ---, Filter by Tag, Copy Tag ID |
| Note-list header title | Rename notebook / Change icon / Copy id (notebook mode); Tag Settings / Copy id (tag mode) |
| Editor | Undo, Redo, ---, Cut, Copy, Paste, Paste as Plain Text, Select All; with selection: Look Up, Edit with AI, New Note from Selection; on URL: Open Link, Copy Link Address |
| Preview | Look Up, Copy, Select All; on link: Copy Link Address; on image: Copy Image, Save Image As |
| Text inputs | Undo, Redo, Cut, Copy, Paste, Select All |
| Notebook name in meta | Open Notebook, Go to Workspace, Copy Notebook ID |

Right-click on an unselected note row re-targets the action to that row; on a selected row it keeps the multi-selection.

### 3.11 Command palette ("telescope", Ctrl+K)

- A modal with `animate: false` (instant), aligned to the top with a 56 px (4em) top margin; max-height = viewport - 56 - 28.
- Container: base background, 1px border, column, overflow hidden, shadow 0 10 30 at 30% black. Input row: 3.4em (~48 px) tall, padding 0.4em 0.6em 0.4em 0.8em, bottom 1px border, input font 1.3em (~18 px), left icon `telescope` (1.4em) or a back arrow button (tiny, padding 6 8) in sub-mode, placeholder "Search commands, notebooks, tags, etc...". A "Loading..." label (12 px) is pinned right-centre while sources load; "No results found" centred with 40 20 padding.
- Option list: virtualized (estimated 36 px rows), padding 4 0. Row: margin 2 8, padding 8 10, radius 0.5em (7 px), icon slot 20x20 with 12 px right margin, label (weight 500, ellipsis) with optional prefix (light colour) and detail line (12 px, muted, margin-top 2), accessory area on the right (12 px muted: shortcut or kind). Hover and selected states use the selection-list tints; selected adds a 3 px info-colour bar on the left edge. Hover updates the selected index on mouse move.
- Footer actions bar: padding 8 20, 12 px muted hints, column gap 16. The shown action changes live when modifier keys are held (the component listens to key down/up for Alt, Shift, Cmd, Ctrl and picks the action whose modifier set matches). Enter, Ctrl+Enter, Shift+Enter, Alt+Enter and combinations all submit.
- Sources: commands (palette lists every registered command with its description and bound key), notebooks, tags, notes (`core:telescope-commands`, `-notebooks`, `-tags`). Commands marked hidden are omitted.

### 3.12 Settings window

Window type "preferences". Row: left menu 160 px wide (secondary vertical pointing menu; items other than the active one at 60% opacity; active item has a left primary-colour border and a tinted background, weight 400; icon at 20 px with 1 px strokes aligned right; optional round badge for update counts in warning colour, 11 px bold) and a content pane with 12 px padding on the preferences background. Section headings `h2/h3.ui.section.header` at h4 size (1em); field descriptions at 60% opacity; checkbox descriptions indented 26 px; action buttons right-aligned with 0.5rem gap. Pages: General, Editing, Preview, Themes, Keybindings, Plugins (installed, install, updates sub-pages), Sync, Backup, Local Server, Integrations (AI providers as expandable segments with a chevron that flips 180 degrees in 0.2 s, MCP), Account (activity dot chart: 8 px dots, 2 px gaps, per-week columns; stats numbers in h2 bold).

### 3.13 Scrollbars, focus ring, checkbox, toggle

- Sidebar scrollbar: 6 px, hover 14 px (Linux/Windows only; macOS uses overlay scrollbars). Other scrollers use the engine default.
- Focus ring (`.focus-outline`): applies to inputs, dropdowns, buttons, toolbar items. Resting: 2 px solid transparent outline offset 5.6 px. Focused: outline colour = `--focused-outline-color`, offset 0, both transitioned 0.1 s ease, giving a ring that tightens in. List rows use an inset 2 px ring (offset -2).
- Checkbox: box radius 8, 1px solid border, checked fills with the primary colour and an inverted check at 90% opacity; indeterminate shows a dash; disabled opacity 0.5. Toggle: pill lane (transparent-black), white round handle with subtle shadow and 1px inner border, on state lane = primary.
- Loader: spinning ring, 0.6 s, line 0.2em; three-dot text loader for AI (1.2 s).

### 3.14 Motion summary

| Where | Property | Duration / easing |
|---|---|---|
| Sidebar rows | bg, box-shadow, color | 0.1 s ease |
| Panel / editor background and border colour, layout opacity | background-color, border-color, opacity | 0.4 s ease-in-out |
| Editor header opacity | opacity | 1.2 s ease-in-out |
| Notebook chevron | transform rotate 90 | 0.2 s ease-out |
| Focus ring | outline-offset, outline-color | 0.1 s ease |
| Input icon opacity | opacity | 0.3 s |
| Modal content | opacity, blur 10 to 0, scale 80 to 100 | 0.2 s; backdrop to 30% black |
| Sidebar tag list | opacity | 0.4 s spring, bounce 0 (popLayout) |
| Note insertion | height, opacity (layout animation) | library default, only when scheduled |
| Code-block copy button | opacity + scale .9 to 1 | 0.2 s ease-in-out |
| Floating actions | opacity | 0.4 s |
| Sync ellipsis | width in 4 steps | 0.9 s infinite |
| Progress bar | width | 0.3 s ease-in-out |
| Mac inset spacer | width | 0.3 s ease-out |
| Sidebar workspace switch | slide in direction of navigation | library spring |

Utility keyframes for attention (flash, shake, bounce, tada, pulse 0.5 s, jiggle, glow 2 s with cubic-bezier(0.19, 1, 0.22, 1)) exist but are for rare cases (for example a new-note glow).

## 4. Icon system and fonts

- **Icon set**: Streamline "Ultimate" regular and bold line icons, embedded as 150 inline SVG components in one registry (`[icons]` search `Streamline`). Each SVG: viewBox 0 0 24 24, `fill: none`, `stroke: currentColor`, stroke-width **1.5**, round line caps and joins. Default render size `1em x 1em` (so 14 px in text, 16 px in the sidebar slots, 18 px in header buttons via `font-size`, 20 px for dialog headers, 50 px for empty states). Chevrons used as disclosure toggles override the stroke to 3 px; the section collapse arrows to 2.5 px; preferences menu icons to 1 px at 20 px.
- Selected/active states swap to a `-bold` variant (`pen-1-bold`, `layout-two-colums-bold`, `view-1-bold`, `pin-bold`, `warning-bold`, `hyperlink-2-bold`, `task-list-check-1-bold`, `navigation-menu-*-bold`) rather than colouring.
- Icon names in use (grouped): navigation `arrow-left/right/up/down` (+ `-1`, `-thin`), `dropdown-chevron`, `sidebar`, `expand`, `shrink`, `navigation-menu`, `navigation-menu-vertical`, `navigation-menu-horizontal`; entities `common-file-text`, `repo-template`, `book-close-2`, `book-library-shelf`, `book-open-bookmark`, `bin-1`, `bin-2-alternate`, `tags`, `hash`, `pin`; status `status-active`, `status-onhold`, `status-completed`, `status-dropped`; actions `search`, `add-circle`, `close`, `close-circle`, `check`, `copy-paste`, `share`, `history`, `telescope`, `cog`, `filter-1`, `settings-*`; editor `text-bold`, `text-italic`, `text-strike-through`, `list-bullets`, `list-numbers`, `task-list-check`, `code-block`, `hyperlink-2`, `image-size-*`, `regex`, `case-sensitivity`; misc `warning`, `information-circle`, `question-circle`, `lock-password`, `love-it`, `star`, plus a few playful ones (`dog`, `border-collie`).
- Missing icon renders a literal "Icon Not Found" text span (a dev affordance).
- Notebook icons are user-selectable from the same set (Change Notebook Icon dialog); tags use a coloured dot instead of an icon.
- **Bourbon implication**: do not copy these assets. Draw an own 24-grid line icon set (stroke 1.5 px, round joins) as GPUI paths, or use a permissively licensed icon font/SVG set whose license is compatible with GPL-3.0, and reproduce the *behaviour* (1em sizing, stroke width, bold variant for selected). The mapping table above is a checklist of icons the shell needs.
- **UI font stack**: see 2.2. UI text sizes in use: 10 (counts), 11 (section headers in lists, tooltips, dropdown headers), 12 (sidebar section, note meta, footer), 13 (small), 14 (base: sidebar, menu items, excerpt), 16.8 (list header title), 18 (palette input), 21 (note title), 24 (blank slate). Weights in use: 200 (counts), 300 (blank-slate), 400 (base), 500 (list header, palette labels), 600 (note titles, section headers, dropdown headers, button text), 700 (stats).

## 5. Interaction model

### 5.1 Command system

Everything is a *command* (`namespace:name`) dispatched on a DOM target; keymaps are selector-scoped (the key binding is only active when the event target matches the selector). A command may be registered on an element (list, input) or on `body`. Buttons dispatch the same command (and show its binding in the tooltip). The palette lists all registered commands. Bourbon equivalent: an action registry with focus-context scoping (GPUI key contexts).

### 5.2 Default keymap (Linux/Windows column; macOS replaces Ctrl with Cmd and a few Alt/Ctrl combos)

Source: `[bundle]` search `"ctrl-,":` (keymap object `body`, then per-selector objects). The native-looking text editing keys inside `.native-key-bindings` fields are left to the OS.

Global (`body`):

| Keys | Command |
|---|---|
| Ctrl+, | Open preferences |
| Ctrl+N / Ctrl+Shift+N / Ctrl+Alt+N | New note / new note in separate window / new window |
| Ctrl+S | Save note |
| Ctrl+Alt+M | Move to notebook |
| Ctrl+K | Command palette (telescope) |
| Ctrl+Shift+F / Ctrl+Shift+Alt+F | Find (note list filter) / global find |
| Ctrl+/ | Toggle sidebar |
| Ctrl+Shift+D | Toggle distraction-free |
| Ctrl+E | Toggle preview |
| Ctrl+P | Toggle side-by-side |
| F11 | Fullscreen |
| Ctrl+= / Ctrl+- (and + / _) | Increase / decrease UI font size |
| Alt+Left / Alt+Right | Navigate back / forward |
| Ctrl+Tab / Ctrl+Shift+Tab | Open next / previous note |
| Ctrl+Shift+J / Ctrl+Shift+K | Sidebar select next / previous item |
| Ctrl+Alt+L | Focus the note list |
| Ctrl+J | Show editor action menu |
| Ctrl+Enter | Focus the editor |
| Ctrl+T | Choose template (in editor layout) |
| Shift+Alt+1..5 | Note status None / Active / On hold / Completed / Dropped |
| Ctrl+B / I / U | Strong / emphasis / strikethrough |
| Ctrl+1..4 | Heading 1..4 |
| Ctrl+Shift+L / O / T | Bulleted / numbered / task list |
| Ctrl+' | Blockquote |
| Ctrl+Shift+H, Ctrl+Alt+K, Ctrl+Shift+C | Horizontal rule, insert link, insert code block |
| Ctrl+F / Ctrl+G / Ctrl+Shift+G / Ctrl+Alt+F | Editor find / next / previous / replace |
| Ctrl+W, Ctrl+F4 | Close window |
| Ctrl+Alt+R / Ctrl+Alt+I | Reload / dev tools |
| Ctrl+Z, Ctrl+Shift+Z or Ctrl+Y, Ctrl+X/C/V, Ctrl+A | Standard edit commands |

Pane-scoped:

| Selector (focus context) | Keys | Command |
|---|---|---|
| `.sidebar-menu` | Up / Down | Select previous / next item |
|  | Enter | Disclose item (drill into workspace) |
|  | Right / Left | Expand / collapse item |
|  | Tab | Focus note list |
| `.sidebar-workspace-menu` | same, plus Backspace | Go up one level |
| `.note-list-bar` | Up / Down | Open previous / next note (opening is immediate: arrow keys *open* notes, not just move a cursor) |
|  | Enter or Tab | Focus title field |
|  | Ctrl+Enter | Focus editor body |
|  | Shift+Enter | Open in separate window |
|  | Shift+Tab | Focus sidebar |
|  | Delete | Delete note |
|  | P / M | Pin to top / move to notebook (single-letter keys, list context only; disabled in template mode) |
|  | Ctrl+C / Ctrl+Shift+C | Copy note link / URI |
|  | Ctrl+D | Duplicate note |
| `.note-list-search-bar input` | Enter / Ctrl+Enter / Esc | Focus list / focus editor / clear find |
| `.editor-title-bar-input input` | Enter / Down | Move to editor / go to document start |
| `.telescope-bar input`, `.notebook-list-bar input`, template picker | Up, Down, Home, End, Enter, Esc | select prev/next/first/last, submit, cancel |
| `.mde-search-bar` | Esc, Alt+R, Alt+C; Enter / Shift+Enter in the find field; Ctrl+Enter replaces all | clear, regex, case, next/prev |
| Autocomplete active | Up/Down/PageUp/PageDown, Enter, Esc | move, accept, close |
| Table editor active | Tab, Shift+Tab, Enter, Ctrl+arrows, Ctrl+Alt+arrows | cell navigation and alignment |
| Editor (CodeMirror) | the usual Emacs-ish and Sublime-ish line commands (Alt+Up/Down move line, Ctrl+D delete line, Ctrl+L select line, Ctrl+[ ] indent, F9 sort lines, Ctrl+Shift+J join, Ctrl+Enter insert blank line) | (editor engine doc) |

### 5.3 Focus handling between panes

- Panes are focusable containers (`tabindex 0`, `outline: none`); the visible cue is the *left accent bar* on the selected row, which only appears when the pane has focus-within, plus the stronger selection tint.
- Flow: sidebar (Tab) to note list (Tab or Enter) to title field (Enter) to editor body; Shift+Tab goes back one pane; Ctrl+Enter jumps straight to the body; Ctrl+Alt+L jumps to the list; on launch focus goes to the sidebar unless a note opens. Search input: Enter hands focus to the list, Esc clears.
- Opening a note from the list does not steal focus from the list (arrow keys keep working); explicit Enter/Tab move on.
- Dialogs trap focus (backdrop owns the keyboard), then restore it to the element focused before opening.
- Mouse buttons 4/5 navigate history; double-clicking any header drag area toggles maximize.

### 5.4 Drag and drop

Uses a pointer-based DnD library with typed draggables/droppables (types "notes", "book", "tag", "status") and a custom drag layer (floating preview chip, see 3.2). Sources: note rows, notebook rows. Targets: notebook rows (move note; before/after/into for notebook reordering with the 2 px line and dashed outline described above), tag rows (add tag), status rows (set status), Trash. The hovered target takes the active background (`is-over-notes`).

### 5.5 History

A navigation stack (offset plus history array) records opened notes; Alt+Left/Right and the header back/forward buttons move through it; buttons disable at the ends.

### 5.6 Selection model summary

Single-click opens (and selects); Shift extends a range from the open note; Cmd/Ctrl-click toggles (Cmd on macOS, with `Ctrl` on Windows/Linux handled by a platform check in tag clicks); Ctrl+A selects all; `selectedNoteIds` plus an `actionTargetNoteIds` list decides what batch actions apply to. More than one selected note replaces the editor with the batch panel.

## 6. Ideas worth bringing to Bourbon

Small, specific, cheap to build, and visible in the shipped app:

1. **Tooltips with the live shortcut.** Every icon button has a tooltip made of the command title plus a `kbd` chip resolved from the current keymap. Bourbon: one `command_button` helper that looks up the binding.
2. **Left accent bar instead of a rounded selection pill, and it only appears when the pane has focus.** The selected row is always tinted, but the strong 5 px bar tells you which pane owns the keyboard. Cheap and very readable.
3. **No hover backgrounds on list rows.** Hover is reserved for revealing controls (count swaps for a disclosure pill, accessory icons). This keeps dense lists calm. Bourbon should decide deliberately; it is a recognisable part of the look.
4. **Arrow keys in the list open notes** (live preview in the editor) while focus stays in the list; Enter/Tab hand focus on. Single-letter list shortcuts (P pin, M move).
5. **Sticky section headers in a virtualized list** (the active header is always injected into the rendered range). Rows of unknown height are measured; the estimate (87 px) only seeds the scrollbar.
6. **Paged loading (50 at a time)** when the last virtual row appears.
7. **Smart relative dates** with a cutoff (37 days) and an option for absolute dates; the timestamp follows the sort key (created vs updated).
8. **Task progress pill** in the list row and **excerpt that switches to matching lines with highlights** when searching.
9. **Status icons plus 4 status colours** reused everywhere (list, picker, palette, activity chart); shortcuts Shift+Alt+1..5 and the shortcut shown inside the picker menu.
10. **Palette footer that changes with held modifier keys** (the action label updates as Shift/Alt/Ctrl are held). Also: the palette is not animated (instant open) while normal dialogs animate (scale 80 to 100, blur 10 to 0, 0.2 s).
11. **Drop indicators**: 2 px line for reorder, dashed outline for nest-into, 50% opacity on the dragged row, custom floating preview chip.
12. **Right-click on an unselected row retargets** the action; on a selected row it keeps the multi-selection. Menu items hide or disable by context (trash, template, read-only) instead of being absent in code paths.
13. **Drill-down sidebar ("workspace")** with breadcrumb and a directional slide: focus on one notebook subtree without losing the global tree.
14. **Sidebar sync/status footer** with a three-step animated ellipsis, error turning the strip solid red (Bourbon: git or file-watch status).
15. **Section collapse persisted per layout** and notebook folder expansion persisted.
16. **Quiet chrome on focus loss**: the thumb of the scrollbar and the selection accent follow window/pane focus.
17. **Distraction-free that fades the header over 1.2 s** instead of cutting it.
18. **Focus ring that tightens in** (outline offset 5.6 px to 0 over 0.1 s) on inputs and buttons, instead of a hard ring.
19. **Back/forward history** in the header, with mouse-4/5 support.
20. **Everything is a command** with a description; the palette, the menus, tooltips and the keybinding preferences all read from one registry.
21. **Layout is a registry of named slots** (layout name to ordered list of components): a good model for Bourbon's own plugin or panel extensibility later, even if it is just a Rust trait object list.

### Notes for the GPUI rebuild (my inference, not from Inkdrop)

- Corner radius: Inkdrop uses a 15 px *squircle* on menus, popups, inputs and segmented controls; GPUI has no squircle. A plain rounded rectangle at 10 to 12 px looks closest; the modal at 24 px squircle maps to about 16 to 18 px round.
- `light-dark()` pairs map to one `Theme` struct with two palettes; alpha-over-surface colours (text at 87/60/40 %, borders at 10-35 %) are worth keeping as alpha values on a base colour rather than precomputed solids, so they work on tinted surfaces.
- Virtualization: GPUI `uniform_list` needs equal heights; Inkdrop's rows vary (1 or 2 title lines), so Bourbon needs the variable-height `list` element with measured rows, or force a fixed height (for example 2-line title slot) for a simpler first version.
- All the layout widths are plain numbers: sidebar 200 (170-500), list 260 (220-500), header strips 40, search 34, sidebar row 30, footer 40; 4 px grid with 14 px as the dominant side inset.

## Not determined

- Exact hit height of icon-only header buttons: the stylesheet defers to the generic button padding (11 x 14 em-fractions) with an 18 px icon; a rendered measurement was not taken. Screenshots in `/home/lowh/.claude/jobs/6d257de5/tmp/` show roughly 28 to 32 px squares.
- The hover/drag visual of the 10 px resize handle: no rule found, so I report none.
- Exact tooltip show delay (comes from a library default, not set in the bundle).
- The slide animation parameters of the sidebar workspace switch (spring parameters live in the animation library call, only direction was readable).
- Native OS context-menu look is platform-dependent and not specified by the app.
- The `ui.css` token section for "Components" (about 400 further lines from line 702 on) was summarised, not exhaustively tabulated; button hover/active and dropdown tokens above are complete for the shell, segment/table/label/message tokens are only listed where the shell uses them.
