# InkDrop theming and a Bourbon theme system

Written 2026-11-06. Part 1 is research on InkDrop 6; Part 2 is a design.

> **PROPOSAL: decide with the user at the UI pass.** Nothing in Part 2 is decided or implemented.

## Part 1. How InkDrop themes work

Sources:
- Creating a theme (InkDrop 6 developer guide): https://developers.inkdrop.app/guides/create-a-theme (old URL https://docs.inkdrop.app/manual/creating-a-theme redirects there)
- Forum, v6.0.0-canary.21 "The UI got more beautiful": https://forum.inkdrop.app/t/inkdrop-desktop-v6-0-0-canary-21-the-ui-got-more-beautiful/5558
- Forum, canary.20: https://forum.inkdrop.app/t/inkdrop-desktop-v6-0-0-canary-20-completed-the-roadmap/5555
- Default dark UI repo (v5 style, README says not to fork it): https://github.com/inkdropapp/inkdrop-default-dark-ui-theme (via https://awesome.ecosyste.ms/projects/github.com%2Finkdropapp%2Finkdrop-default-dark-ui-theme)
- Community example (v5 UI theme): https://github.com/peterbartha/inkdrop-octopus-dark-ui-theme
- Community auto switch plugin: https://github.com/elpnt/inkdrop-auto-switch-themes

### Evolution: three theme types merged into one

- **InkDrop 5**: three separate packages, picked with three drop-downs. UI theme (side view, note list, toolbar, dropdowns), syntax theme (editor and gutter) and preview theme (header, text, code blocks in the rendered view).
- **InkDrop 6**: one unified theme package styles UI, editor and preview. One picker. Old `"theme": "ui" | "syntax" | "preview"` packages are no longer recognised and fall back to the default theme; existing settings migrate to the matching default light or dark theme (canary.21 post).

### Structure of a v6 theme

An npm-style package scaffolded with `ipm init <name> --type theme`:

```json
{
  "name": "motif",
  "theme": true,
  "themeAppearance": "dark",
  "styleSheets": ["ui.css", "syntax.css", "preview.css"],
  "scripts": { "prepublishOnly": "generate-palette" },
  "devDependencies": { "@inkdropapp/theme-dev-helpers": "^0.6.0" },
  "engines": { "inkdrop": "^6.0.0" }
}
```

- `theme: true`, `themeAppearance: "dark" | "light"` (the app uses it for light/dark dependent behaviour; `ipm init` guesses it from "dark"/"light" in the name), `styleSheets` (ordered list; an optional shared `palette.css` first). Only the sheets needed must be shipped.
- `ui.css`: app chrome. `syntax.css`: editor. `preview.css`: Markdown preview.
- **Cascade layers**: each sheet wraps its rules in `@layer theme.ui`, `theme.syntax` or `theme.preview` (InkDrop does not add the layer). App defaults live in sub-layers `theme.ui.base`, `theme.ui.components`, `theme.syntax.base`, `theme.preview.base`, so a theme's rules win without specificity fights. A shared palette uses `@layer theme`.
- Everything is **CSS custom properties on `:root`**; a theme is essentially a set of variable overrides:

```css
@layer theme.ui {
  :root {
    --page-background: var(--color-neutral-50);
    --sidebar-background: var(--color-neutral-100);
  }
}
```

### Token families (from the developer guide)

| Area | Variables |
|---|---|
| Primitives (`@inkdropapp/css`) | `--color-<family>-<step>` (opaque) and `--hsl-<family>-<step>` (HSL triplet for alpha use, `hsl(var(--hsl-violet-600) / 13%)`); families neutral, blue, slate, violet, green; sources `tokens.css`, `ui.css`, `status.css`, `tags.css`, `task-progress.css` |
| UI | `--primary-color`, `--page-background`, `--text-color`, `--sidebar-background`, `--note-list-bar-background`, `--editor-background`, `--editor-drawer-background`, `--inline-dropdown-menu-background`, plus status (`--note-status-*`), tag (`--tag-*`) and task progress (`--task-progress-view-*`) |
| Editor | `--editor-*` (background, foreground, gutter, caret, selection, tooltips, brackets) |
| Syntax | `--syntax-<lezer-token>-color` with `-font-style`, `-font-weight`, `-text-decoration` variants (keyword, string, comment ...); un-coloured tokens fall back to `--editor-foreground-color`; modifier variants fall back to the base token |
| Markdown in editor | `--md-*` (fenced code, inline code, highlight marks, tables) |
| Preview | `--mde-preview-*` (link, heading, strong, em, inline code background, blockquote text/border); most default to UI tokens (`--text-color`, `--border-color`, `--link-color`) |
| Mermaid | `--mermaid-*` (node bg/border, line, cluster bg, primary/secondary/tertiary; others cascade) |

Notes: fallbacks are a core design idea (a theme that sets 5 variables is valid). Themes should not set editor `font-family` (it would override the user's font setting). Acrylic window background is targeted with `:root:has(body.acrylic-window)`.

### Minimal theme

From the guide's own examples, the smallest useful theme is: `package.json` with `theme: true`, `themeAppearance`, one stylesheet, and in it a handful of variables: `--page-background`, `--sidebar-background`, `--text-color`, `--primary-color`, and for the editor `--editor-foreground-color`, `--editor-caret-color`, `--editor-selection-background`, plus a few `--syntax-*-color` and `--mde-preview-*`. Everything else derives from the base layers or ramps.

### Install, switch, reload, publish

- Users: Preferences (File > Settings on Linux) > Themes > **Theme** drop-down. Themes install from the plugin registry (`ipm install <name>` or from within the app) and appear in that list. Since canary.21 the picker shows **live preview cards** rendered from each theme's `palette.json`.
- Authors: `ipm link --dev` symlinks the theme into `<USER_DATA>/inkdrop/dev/packages`; reload (`Alt+Ctrl+R` on Linux); **hot reload** via Plugins > dev-tools > "Start hot reloading themes"; `npx dev-server` opens a browser inspector at `localhost:5173` listing all variables, colour tokens and live component previews; "Development Mode" enables DevTools Inspect Element.
- Publishing: `ipm publish`; `prepublishOnly` runs `generate-palette`, which writes `palette.json` (a snapshot of resolved CSS variables) that the registry and the app use for preview cards.
- Community: auto-switch plugin changes theme with the OS appearance (inkdrop-auto-switch-themes), showing InkDrop has no built-in light/dark pairing beyond the default migration.

### What does not translate to GPUI

- CSS cascade layers, specificity, selectors and arbitrary rules (`.mde-preview ...`): GPUI has no stylesheet; styling is Rust builder calls reading a fixed set of colours.
- Free-form variables (`var()`, `calc`, `hsl(var(--x) / 13%)`): replaced by a fixed token set, with derived tokens computed in Rust (mix, alpha).
- npm/`ipm` packages, a plugin registry and `palette.json` cards: out of scope for a local-first app; plain TOML files in a folder are enough.
- Acrylic/translucent window: not reliably possible; skip.
- Per-token Lezer syntax tokens: Bourbon only needs the tokens its own Markdown highlighter emits.
- Mermaid theming: no Mermaid in Bourbon.
- Fonts via `font-family`: a font setting is a separate preference, not part of a theme (InkDrop also discourages it).

## Part 2. Bourbon theme system (PROPOSAL)

### What is themable today

`gpui-component` 0.9.0 `ThemeColor` has ~170 `Hsla` fields (`background`, `foreground`, `border`, `input`, `muted`, `muted_foreground`, `primary*`, `secondary*`, `accent*`, `popover*`, `sidebar*`, `list*`, `list_hover`, `list_active`, `list_active_border`, `selection`, `caret`, `ring`, `link*`, `danger/success/warning/info` plus hover/active, `tab*`, `scrollbar*`, `title_bar*`, `overlay`, `red/green/blue/yellow/magenta/cyan(+_light)`, ...). It also ships its own `ThemeConfig` JSON (name, mode, font sizes, radius, shadow, colours, and a `highlight` block compatible with Zed themes for the code highlighter). Bourbon currently paints ~26 of these from a 10-colour `Palette` in `theme/palette.rs` (`background, sidebar, foreground, border, muted, muted_foreground, accent, accent_foreground, selection, danger`), with Omarchy mapping in `Palette::from_omarchy`. The proposal generalises that `Palette` into a data-driven theme.

### File format

Location: `$XDG_CONFIG_HOME/bourbon/themes/<name>.toml` (fallback `~/.config/bourbon/themes/`). Built-ins (`bourbon-dark`, `bourbon-light`) are embedded in the binary and also listed as read-only. Omarchy-follow stays a theme **source** (as today in `settings.toml`) and is expressed as a generated theme (below). Selected theme: `settings.toml` `theme = "<name>"` (replaces `theme_source`; `follow_omarchy = true` keeps the current behaviour).

Three layers of tokens, mirroring InkDrop's ui / syntax / preview but in one file:

1. **`[meta]`**: `name`, `appearance = "dark" | "light"`, `extends = "bourbon-dark"` (optional), `author`, `version`.
2. **`[core]`**: the minimal set. Any theme with only these is valid; everything else derives.
3. **`[ui]`, `[editor]`, `[preview]`, `[tags]`**: optional overrides. Anything absent derives from `[core]` (like InkDrop's fallbacks).

Core tokens (all `#rrggbb` or `#rrggbbaa`):

| Token | Meaning | Maps to ThemeColor |
|---|---|---|
| `background` | editor and main surface | `background`, `table`, `list` |
| `surface` | sidebar and chrome | `sidebar`, `title_bar`, `popover`, `tab_bar` |
| `surface_alt` | note list pane (default = between the two) | `list`-ish pane background |
| `foreground` | primary text | `foreground`, `sidebar_foreground`, `popover_foreground` |
| `muted_foreground` | excerpts, counts, placeholders | `muted_foreground` |
| `border` | hairlines | `border`, `sidebar_border`, `title_bar_border`, `input` |
| `accent` | brand, links, caret, primary buttons | `primary`, `ring`, `link`, `caret` |
| `selection` | selected row and text selection | `selection`, `list_active`, `sidebar_accent`, `accent` |
| `danger`, `success`, `warning`, `info` | semantic | same-named fields (+ hover/active derived) |

Derivations (computed in Rust, no theme input required): `muted` = mix(background, foreground, 9%); `list_hover` = mix(background, foreground, 6%); `primary_hover/active` = lighten/darken 6%/12%; `accent_foreground` = black or white by luminance (as `Palette::from_omarchy` does today); `*_foreground` for semantic colours likewise; `scrollbar_thumb` = foreground at 25% alpha; `overlay` = black at 50%.

Optional groups:

```
[ui]       # any ThemeColor field name, escape hatch; e.g. list_hover, scrollbar_thumb, tab_active
           # also radius = 6 (px), row_height
[editor]   background, foreground, caret, selection, gutter, active_line,
           heading, heading_marker, emphasis, strong, code, code_background,
           link, quote, list_marker, rule, task_done
[preview]  background, foreground, heading, link, strong, em, code, code_background,
           quote_text, quote_border, rule, table_border, table_header
[tags]     palette = ["#e5a50a", "#5b8def", "#e5534b", ...]   # replaces chip_color table
           chip_alpha = 0.14
[status]   active, on_hold, completed, dropped
```

Syntax in fenced code blocks: reuse gpui-component's `highlight` block (Zed-compatible) as an optional `[syntax]` table, or default to a built-in highlight theme chosen by appearance.

### Inheritance and resolution

1. Start from the hard-coded **base** (`bourbon-dark` or `bourbon-light`, by `appearance` if `extends` is absent).
2. If `extends` is set, load that theme first (max depth 3, cycle detected, built-ins always terminate the chain).
3. Overlay `[core]`, derive missing core-dependent tokens, then overlay `[ui]/[editor]/[preview]/[tags]`.
4. Paint onto `ThemeColor` (extension of `Palette::paint`) and store the Bourbon-specific tokens in a `BourbonTheme` global (`cx.global::<BourbonTheme>()`) read by the editor, preview and tag chip code.

### Validation and fallback

- Unknown keys: warn and ignore (forward compatible). Invalid colour: that token alone falls back to its derived/base value, with a warning. Missing `appearance`: infer from `background` luminance (the rule Omarchy-follow already uses).
- Parse error or missing file: fall back to the previous good theme; if none, `bourbon-dark`. Never crash, never show a blank UI.
- A contrast check at load (foreground/background 4.5:1, muted_foreground/background 3:1): only a warning in the settings UI, not a block.
- Warnings surface in the settings screen next to the theme (no toast spam) and in the log.
- Parse in `bourbon-core` (pure, testable, no GPUI): `Theme { meta, tokens }` with `parse(text) -> Result<Theme, ThemeError>` plus `resolve(base) -> ResolvedTheme`; colours as the existing `Rgb`. `bourbon-app` converts to `Hsla` and paints. This keeps the doctrine of logic in core.

### Hot reload

Extend the existing 2 s mtime poll (`theme/watch.rs`) to watch the selected theme file and its `extends` chain (mtime of each file; no new dependency). On change: re-parse, and on success repaint all windows with an optional 200 ms colour cross-fade; on failure keep the old theme and show the warning in settings. Also rescan the `themes/` directory when the settings screen is open so new files appear in the picker.

### Omarchy follow as a generated theme

`~/.local/state/omarchy/current/theme/colors.toml` is converted by `Palette::from_omarchy` logic into a `ResolvedTheme` named `omarchy` at runtime (not written to disk by default). Mapping (already decided in `docs/decisions/2026-11-06-m5-visual.md`): background, foreground, accent as given, `muted_foreground` = color8, `danger` = color1, `success` = color2, `warning` = color3, `info` = color4, surface/border/muted as mixes of background and foreground, selection mixed 40%. `[editor]` and `[preview]` tokens: heading = accent, code = color5, link = color4, quote = color8. Provide a settings action **Export current Omarchy theme as file** that writes `themes/omarchy-<theme>.toml` so users can tweak and keep it. The selected state is "Follow Omarchy" (live) or a named theme (static).

### Settings UI

Settings screen > Theme section replaces the two segmented controls:

- A grid of **preview cards** (InkDrop 6 does the same): each card is a mini three-column mock (sidebar, list, editor) painted with that theme's tokens, name below, dark/light badge. Selected card has an accent ring.
- First entries: "Follow Omarchy" (live), then built-ins, then user themes from the folder.
- Controls: "Open themes folder", "Reload", "Duplicate to edit" (copies the selected theme to a new user file with `extends`), "Export Omarchy".
- Appearance toggle only for themes that ship both variants (see below).
- Warnings list per theme (validation).

### Light/dark variants

Two options; proposal: **one file per appearance** (`foo-dark.toml`, `foo-light.toml`), which matches InkDrop (`themeAppearance` per package) and Omarchy. A theme may optionally declare `pair = "foo-light"`, so the Appearance toggle swaps between the pair. A single file with `[dark]`/`[light]` sub-tables is more compact but doubles the validation surface; avoid at first.

### Example theme file

```toml
# ~/.config/bourbon/themes/bourbon-copper-dim.toml
[meta]
name       = "Bourbon Copper Dim"
appearance = "dark"
extends    = "bourbon-dark"   # everything not set below comes from here
pair       = "bourbon-copper-light"
author     = "you"
version    = 1

[core]
background       = "#1c1613"
surface          = "#17110e"
surface_alt      = "#1a1411"
foreground       = "#e8dccf"
muted_foreground = "#9c8b7c"
border           = "#3a2d25"
accent           = "#c9814a"
selection        = "#3d2c20"
danger           = "#d9534f"
success          = "#7bae6a"
warning          = "#e5a50a"
info             = "#5b8def"

[ui]
radius       = 6
row_height   = 30
list_hover   = "#2a211b"

[editor]
caret          = "#c9814a"
active_line    = "#ffffff0a"
heading        = "#e0a070"
heading_marker = "#7a5a44"
code_background = "#241b16"
link           = "#7fb0ff"
list_marker    = "#c9814a"

[preview]
heading        = "#f0d9c0"
quote_border   = "#5a4333"
code_background = "#241b16"

[tags]
palette    = ["#e5a50a", "#5b8def", "#e5534b", "#7bae6a", "#b07ae0", "#e08aa5"]
chip_alpha = 0.14

[status]
active    = "#5b8def"
on_hold   = "#e5a50a"
completed = "#7bae6a"
dropped   = "#d9534f"
```

### Token summary

Core 13 (background, surface, surface_alt, foreground, muted_foreground, border, accent, selection, danger, success, warning, info, plus optional accent_foreground) ; derived ~25 ThemeColor fields; escape hatch `[ui]` for any ThemeColor field; Bourbon groups `[editor]` (~16), `[preview]` (~12), `[tags]` (palette, alpha), `[status]` (4), optional `[syntax]`.

### Suggested rollout

1. `bourbon-core::theme`: parse, extends, derive, validate, tests (no GPUI).
2. Convert current palettes into embedded `bourbon-dark.toml`/`bourbon-light.toml`; Omarchy generator on top.
3. App: paint extended tokens, `BourbonTheme` global, editor/preview/tag consumers.
4. Hot reload on the selected file chain.
5. Settings picker with preview cards.

### Open questions

1. Single theme setting (`theme = "<name>"`) replacing `theme_source`+`appearance`, with a migration, OK?
2. Pairing (`pair`) vs independent light/dark files?
3. Should users be able to override fonts per theme, or keep fonts a separate setting (InkDrop keeps them separate)?
4. Ship a first set of extra built-ins (e.g. an Inkdrop-like slate dark and light) as part of the UI pass?
