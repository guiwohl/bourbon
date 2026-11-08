# bourbon-ui

Our own GPUI component kit and Inkdrop-faithful theme tokens (decision: `docs/decisions/2026-11-07-ui-rebuild.md`, values: `docs/research/inkdrop-ui-shell.md`). Depends only on `gpui-pre` (lib name `gpui`); `just architecture` fails if `gpui-kit` or `gpui-component` appear in its tree. `bourbon-app` renders its sidebar, note list, control pane, settings screen, dialogs and menus with this crate; and its editor column.

## Modules

| Module | Contents |
|---|---|
| `tokens` | `Tokens { scale, fonts, colors, shadows }`, `Tokens::light()`, `Tokens::dark()`, `Appearance`. `Scale` is shared (research 2.1 and layout widths), `Colors` has the resolved `light-dark()` pair (2.3, hex from the research conversions, alpha kept as authored), `Fonts` has `Inter` and the mono stack (`Liberation Mono` is the pick on Linux) |
| `palette` | Hex ramp constants used by the tokens and `solid`, `translucent`, `blended` |
| `theme` | `Theme { tokens, appearance }` GPUI global, `cx.theme()` via `ThemeExt`, `Theme::install`, `Theme::install_tokens` (custom tokens such as the Omarchy mapping), `Themed::new(appearance, child)` renders a subtree with the other appearance (used by the gallery) |
| `fonts` | `register_fonts(cx)`, `UI_FAMILY`; Inter TTFs and OFL text in `assets/fonts` |
| `icon` | `IconName` (34 Lucide icons), `Icon` (size, colour, `Stroke`, `bold(accent)`, `quarter_turns`), `IconAssets` (`AssetSource`) |
| `button` | `Button` (plain, subtle, primary, danger = negative fill darkened 10 / 20 % on hover / press; icon and/or label; disabled, selected) and `IconButton` (header 32 px with a 22 px icon, or accessory 22 px; `tooltip`, `tooltip_action(label, &Action)`) |
| `tooltip` | `Tooltip` entity: label plus a `kbd` chip with the live binding (`live_binding` reads the keymap), 11 px, floating shadow |
| `segmented` | `SegmentedControl` (44 x 30 segments, 20 px icons, 1 px border, selected fill plus accent icon, no thumb animation) with `on_change(index)` |
| `pill` | `Pill` tag chip: tint 15 %, border 30 %, text from `readable::pill_text_color` (WCAG 4.5 contrast search, moved from the app with its tests) |
| `list_row` | `ListRow` (30 px min height, left padding 14 + 12 per depth, icon slot 16, count, disclosure chevron rotating over 0.2 s, accessories revealed on row hover, no hover background) and `SectionHeader` (12 px label, accessories, collapse). `accent(selected, pane_focused)`: tint when selected, 5 px left bar only when the pane is focused |
| `modal` | `Modal::new(id, content)`: deferred full-window layer on the 30 % black backdrop, `Placement::Centered` or `Top(px)`, `track_focus(handle)`, `on_dismiss` (backdrop mouse down, Esc) and `on_submit` (Enter). Key context `BourbonModal`, actions `Dismiss`, `Submit`; `bourbon_ui::init` binds them |
| `dialog` | `Dialog::new(id, title)` on `Modal` (research 3.9): 420 px frame (min 240), radius 18, six-layer modal shadow, menu background; header 16/17 px padding, 14 px semibold, bottom border; content padding 14; actions strip right-aligned on `modal_actions_background`, top border, each action min 140 px, gap 10.5. `child`, `action(any element)`, `track_focus`, `on_dismiss`, `on_submit` |
| `checkbox` | `Checkbox::new(id, label, checked)`: 17 px box, radius 6, 1 px strong border; checked = primary fill and a heavy white check at 90 %; label gap 9, optional `description` indented 26 px in 13 px muted; `disabled`; `on_toggle(next_value)` |
| `select` | `Select::new(id, label)`: dropdown trigger (padding 8/14, min 180, radius 12, 1 px border, input background, chevron 10 px); `on_open(bounds)` hands the trigger bounds so the owner opens a `Menu` with `popover::below(bounds)`; `disabled` |
| `menu` | `Menu` entity (items with icon, label, shortcut, check, disabled, submenu arrow, separators). Key context `BourbonMenu`, actions `SelectNext/Previous/First/Last`, `Confirm`, `Cancel`; events `MenuEvent::Confirmed(index)` and `Dismissed`. Call `bourbon_ui::init(cx)` to bind keys. `navigation::next_enabled` is the pure up/down logic (skips disabled and separators, wraps) |
| `popover` | `Popover::at(point, content)` / `below(bounds, content)`: deferred, anchored, snapped to the window with an 8 px margin, `on_dismiss` on outside click |
| `splitter` | `Splitter` (1 px border line with an invisible 10 px handle straddling it), `WidthLimits`, `dragged_width`, `on_resize(width)`, `on_resize_end()`. Defaults live in `Scale::sidebar_width` (170, 200, 500) and `note_list_width` (220, 260, 500). The owner stores the width and persists it on drag end |
| `scroll_area` | `ScrollArea` over a `ScrollHandle` with a 6 px thumb widening to 14 px on hover (`thumb` is the pure geometry) |
| `divider` | `Divider::horizontal()` / `vertical()` in the internal border colour |

## Conventions

- Components are `RenderOnce` and read `cx.theme()` in `render`, so one global switches every component; `Themed` swaps it for a subtree during layout.
- Radii: Inkdrop squircles (15 / 24 px) map to 12 / 18 px rounded rectangles (`radius_control`, `radius_large`), the 7 px default stays.
- GPUI has no CSS transitions: the 0.1 s hover tint is instant; the chevron rotation uses `with_animation` (0.2 s, ease-out quint) and also plays when a row first mounts.
- No text input here: the single-line `TextField` lives in `bourbon-editor` (it shares the text core), styled with these tokens; see [editor.md](editor.md#text-field).
- Shadows: `Shadows::modal` (six layers 2.8 to 100 px blur, 6.7 to 24 % black) and `Shadows::palette` (`0 10 30` at 30 %).
- `ListRow::dot` draws the 10 px tag dot in the 20 px slot; `Pill::compact` is the note-list chip (10 px text, no fixed height); `Menu::entries` exposes the entries; a splitter ignores drags that another splitter started (the drag payload carries its owner id).

## Gallery

`cargo run -p bourbon-ui --example gallery` opens "Bourbon Gallery" (app id `bourbon-gallery`) with every component in light (left) and dark (right).

## Tests

Headless with `gpui` `test-support`: menu keyboard navigation, dismissal and click; popover outside click; segmented selection callback; splitter drag, clamping and drag end; list row click and disclosure; tooltip text with the live keybinding; theme scoping; token tables (light differs from dark, surfaces opaque, alpha ramps); icon asset loading; readable-shade maths. Pure logic (`next_enabled`, `dragged_width`, `accent`, `thumb`, `icon_path`) is tested without a window.

## Not matched

- Modal enter/exit animation (opacity, blur, scale) and the 2 s backdrop fade: the modal appears at once. Focus is not restored to the previous element when a dialog closes (it returns to the window root).
- Squircle corners, `backdrop-filter` blur, inset box shadows (the accent bar is a 5 px child instead), CSS transitions on hover, the focus ring tightening animation, hover-reveal "disclosure pill" and its count swap, scrollbar thumb dragging, submenu opening (the arrow is drawn, the owner opens the submenu on `Confirmed`).
