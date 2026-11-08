# bourbon-ui

Our own GPUI component kit and Inkdrop-faithful theme tokens (decision: `docs/decisions/2026-11-07-ui-rebuild.md`, values: `docs/research/inkdrop-ui-shell.md`). Depends only on `gpui-pre` (lib name `gpui`); `just architecture` fails if `gpui-kit` or `gpui-component` appear in its tree. `bourbon-app` renders its sidebar and note list columns with this crate (phase 2, slice 1); the editor, control pane and settings still use `gpui-kit` until the next slices.

## Modules

| Module | Contents |
|---|---|
| `tokens` | `Tokens { scale, fonts, colors, shadows }`, `Tokens::light()`, `Tokens::dark()`, `Appearance`. `Scale` is shared (research 2.1 and layout widths), `Colors` has the resolved `light-dark()` pair (2.3, hex from the research conversions, alpha kept as authored), `Fonts` has `Inter` and the mono stack (`Liberation Mono` is the pick on Linux) |
| `palette` | Hex ramp constants used by the tokens and `solid`, `translucent`, `blended` |
| `theme` | `Theme { tokens, appearance }` GPUI global, `cx.theme()` via `ThemeExt`, `Theme::install`, `Theme::install_tokens` (custom tokens such as the Omarchy mapping), `Themed::new(appearance, child)` renders a subtree with the other appearance (used by the gallery) |
| `fonts` | `register_fonts(cx)`, `UI_FAMILY`; Inter TTFs and OFL text in `assets/fonts` |
| `icon` | `IconName` (32 Lucide icons), `Icon` (size, colour, `Stroke`, `bold(accent)`, `quarter_turns`), `IconAssets` (`AssetSource`) |
| `button` | `Button` (plain, subtle, primary; icon and/or label; disabled, selected) and `IconButton` (header 30 px or accessory 22 px; `tooltip`, `tooltip_action(label, &Action)`) |
| `tooltip` | `Tooltip` entity: label plus a `kbd` chip with the live binding (`live_binding` reads the keymap), 11 px, floating shadow |
| `segmented` | `SegmentedControl` (44 x 28 segments, 1 px border, selected fill plus accent icon, no thumb animation) with `on_change(index)` |
| `pill` | `Pill` tag chip: tint 15 %, border 30 %, text from `readable::pill_text_color` (WCAG 4.5 contrast search, moved from the app with its tests) |
| `list_row` | `ListRow` (30 px min height, left padding 14 + 12 per depth, icon slot 16, count, disclosure chevron rotating over 0.2 s, accessories revealed on row hover, no hover background) and `SectionHeader` (12 px label, accessories, collapse). `accent(selected, pane_focused)`: tint when selected, 5 px left bar only when the pane is focused |
| `menu` | `Menu` entity (items with icon, label, shortcut, check, disabled, submenu arrow, separators). Key context `BourbonMenu`, actions `SelectNext/Previous/First/Last`, `Confirm`, `Cancel`; events `MenuEvent::Confirmed(index)` and `Dismissed`. Call `bourbon_ui::init(cx)` to bind keys. `navigation::next_enabled` is the pure up/down logic (skips disabled and separators, wraps) |
| `popover` | `Popover::at(point, content)` / `below(bounds, content)`: deferred, anchored, snapped to the window with an 8 px margin, `on_dismiss` on outside click |
| `splitter` | `Splitter` (1 px border line with an invisible 10 px handle straddling it), `WidthLimits`, `dragged_width`, `on_resize(width)`, `on_resize_end()`. Defaults live in `Scale::sidebar_width` (170, 200, 500) and `note_list_width` (220, 260, 500). The owner stores the width and persists it on drag end |
| `scroll_area` | `ScrollArea` over a `ScrollHandle` with a 6 px thumb widening to 14 px on hover (`thumb` is the pure geometry) |
| `divider` | `Divider::horizontal()` / `vertical()` in the internal border colour |

## Conventions

- Components are `RenderOnce` and read `cx.theme()` in `render`, so one global switches every component; `Themed` swaps it for a subtree during layout.
- Radii: Inkdrop squircles (15 / 24 px) map to 12 / 18 px rounded rectangles (`radius_control`, `radius_large`), the 7 px default stays.
- GPUI has no CSS transitions: the 0.1 s hover tint is instant; the chevron rotation uses `with_animation` (0.2 s, ease-out quint) and also plays when a row first mounts.
- No text input yet (editor slice).
- `ListRow::dot` draws the 10 px tag dot in the 20 px slot; `Pill::compact` is the note-list chip (10 px text, no fixed height); `Menu::entries` exposes the entries; a splitter ignores drags that another splitter started (the drag payload carries its owner id).

## Gallery

`cargo run -p bourbon-ui --example gallery` opens "Bourbon Gallery" (app id `bourbon-gallery`) with every component in light (left) and dark (right).

## Tests

Headless with `gpui` `test-support`: menu keyboard navigation, dismissal and click; popover outside click; segmented selection callback; splitter drag, clamping and drag end; list row click and disclosure; tooltip text with the live keybinding; theme scoping; token tables (light differs from dark, surfaces opaque, alpha ramps); icon asset loading; readable-shade maths. Pure logic (`next_enabled`, `dragged_width`, `accent`, `thumb`, `icon_path`) is tested without a window.

## Not matched

- Squircle corners, `backdrop-filter` blur, inset box shadows (the accent bar is a 5 px child instead), CSS transitions on hover, the focus ring tightening animation, hover-reveal "disclosure pill" and its count swap, scrollbar thumb dragging, submenu opening (the arrow is drawn, the owner opens the submenu on `Confirmed`).
