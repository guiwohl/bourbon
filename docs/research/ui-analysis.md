# UI analysis: Bourbon vs reference

Written 2026-11-06 for the UI pass. Research only, no code was changed. Hex values for the reference are estimated by eye from a screenshot, not sampled.

## Inputs

- **Bourbon**: dark Bourbon theme, demo data, 941x1150 px window.
- **Reference**: the Inkdrop 6 desktop app (the original Bourbon clones), shown in the marketing demo on inkdrop.app ("Your context, beautifully organized"), captured in a browser with a vertical tab sidebar. The app window is the centre card, about 1300x800 px, light theme, macOS traffic lights, translucent sidebar. Note: the reference is the **light** theme, Bourbon's screenshot is **dark**. Contrast figures below compare structure, not color mood.

## Part 1. Critique

### 1. Layout and proportions

| | Reference (Inkdrop) | Bourbon now |
|---|---|---|
| Sidebar | ~200 px, 15% of window | 240 px (25%) |
| Note list | ~260 px, 20% | 320 px (34%) |
| Editor | rest, ~840 px, split edit/preview | 380 px (40%) |

- Bourbon gives 60% of the width to navigation. The editor, the reason the app exists, is the smallest column and shows ~50 characters per line. Inkdrop's editor in split mode still shows ~45 chars per pane but in single-pane it has 840 px.
- Defaults should be sidebar 200, list 280, editor takes the rest. Settings already allow 160-400 and 220-560 so only defaults change (`bourbon-core::settings`).
- Reference has a **top bar spanning the list and editor columns** (36 px) carrying navigation (back/forward), the view-mode segmented control and a kebab menu. Bourbon has no top bar; its controls float in three different places (list header, editor header, status line).

### 2. Spacing and rhythm

Reference uses a 4 px grid with a 12-16 px gutter: sidebar rows 30 px tall, 10 px horizontal inset, list rows ~85-105 px with 14 px side padding and a 1 px hairline divider between rows, editor text starts 14 px from the gutter.

Bourbon:
- Sidebar rows are 38 px with 8 px selection inset; section labels ("Notebooks", "Status", "Tags") are 14 px and almost as heavy as items. Reference labels are 12 px, uppercase-free, in a muted tone with trailing action icons (search, add).
- List rows are 83 px tall with `px_2 py_1` (8/4 px): the title sits 4 px from the row edge, too tight against the selected-row background; reference gives ~10 px vertical padding.
- The editor title input, the notebook/status line, the tag input and the editor box each have different widths and left edges (580, 588, 580, 580 px). The "Inbox / None / ..." row is inset 8 px more than the title field above it. Alignment bug.
- Editor header takes ~190 px of vertical space (title, meta row, tag input, mode switch) before the first line of text. Reference compresses title + meta + tags into ~70 px and puts the mode switch in the top bar.

### 3. Typography

Reference (estimated): UI font is a neutral grotesque (Inter/SF-like), 13 px for sidebar and list titles at weight 600 for titles, 12 px for excerpts at weight 400, 11 px for metadata and counts. Editor and code: monospace ~13 px, 1.5 line height, Markdown syntax marks (`##`, list numbers) coloured, headings in accent colour, bold weight. Preview: 15 px body, headings 28/22/18 px with a bottom rule under h1/h2.

Bourbon:
- UI font is Inter-like at 16 px (sidebar) and ~16 px list titles, weight 400 for everything. There is **no weight hierarchy**: title, notebook names and tags look equally important. Titles need 600, excerpts 12-13 px.
- Editor uses a generic monospace at ~13 px with no syntax colouring and no line-height set: wrapped lines are tight. Target 1.6 line height, and a measure (max width) of ~72 ch centred or left padded 24 px, not the full pane.
- Excerpts show raw Markdown (`# Goals`, `**typography**`, `- Softer sidebar`). Strip syntax for excerpts (reference shows plain prose). This is a `bourbon-core` presentation function (`presentation.rs` in the app).
- Date is a bare `18:50` in every row (demo data, all same minute); reference uses relative time ("2 hours ago") in the accent colour, which also makes the metadata line scannable.
- Sidebar counts are right aligned but at 16 px; reference uses 11 px muted.

### 4. Colour and contrast

Reference (light), estimated:

| Role | Hex |
|---|---|
| Sidebar bg (translucent) | `#e9e9ee` to `#ececf0` |
| Note list bg | `#f4f4f6` |
| Selected sidebar row | `#d6d9e6` (cool blue-grey, ~12% blue) |
| Selected list row | `#dbe3f4` |
| Editor bg | `#f6f6f7`, active line `#dde6f5` |
| Text | `#1f2328` primary, `#6b7078` excerpt, `#8a8f98` counts |
| Accent / links / dates | `#3b6fe0` |
| Syntax heading | `#2f5bd8`, bold, with pink `#d1245a` for `##` Markdown headings in the sample and list markers `#2f5bd8` |
| Tag chips | translucent pill: work `#cfe0fb` bg/`#2f5fb8` text; bug `#fbd0d4`/`#c0392b`; research `#e0d4f7`/`#6c4bb8`; personal `#d2f0d8`/`#2e7d46` |

Bourbon (read from screenshot):

| Role | Hex |
|---|---|
| Sidebar | `#17110e` (matches palette) |
| Note list | `#1c1613` with a vertical gradient to `#131010` |
| Editor box | near black `#0a0a0a`, darker than every other surface |
| Text | `#e8dccf`; excerpt `#9c8b7c` |
| Selection | `#3d2c20` with copper tint |

Findings:
- The editor box is **darker than the sidebar**, inverted elevation. In a dark UI the working surface should be the lightest (or at least equal to the list), e.g. background `#1c1613`, sidebar `#17110e`, list `#1a1411`, editor `#201813`. Likely the editor input paints a gpui-component default (`input`/`background` colour) rather than the palette `background`.
- The background gradient (visible top-lit brown to near-black in the list and editor) is not in the palette; it appears to be a default overlay. Either make it intentional or remove it. Flat surfaces read cleaner.
- Selection contrast: `#3d2c20` vs `#1c1613` is 1.5:1. Visible, but the white-on-copper-brown selected row does not carry a clear accent. Use a 2 px left accent bar or `accent` at 18% alpha plus 1 px inner border.
- Excerpt `#9c8b7c` on `#1c1613` is 5.3:1: fine. Tag chip text at 10-11 px on tinted fills is under 4.5:1 for the blue chip (`reference`): bump text lightness.
- Accent copper `#b06a3b` is under-used: only the pin icon area and tag outline. Reference uses its accent for dates, links, headings, active tab, focus. Apply accent to relative date, editor headings, caret and links.

### 5. Surfaces, borders, radii

- Reference: borderless panes separated by a single 1 px hairline of ~8% alpha; selection pills radius 6 px; chips fully round (pill, 10 px); the window card radius 12 px with soft shadow. Inputs are white fields with 1 px border and 6 px radius.
- Bourbon: bordered, outlined inputs everywhere (title, tags, editor, "New note" buttons) at radius 6 px. The editor sits in a rounded outlined box inside the pane, a card-in-a-column look that wastes 20 px each side. Header buttons "New from template" / "New note" have outlines and **overlap the editor column**: "New note" is clipped at x~610 and sits on top of the editor title field. This is a P0 bug (list header overflow at 320 px).
- Tag chips: rectangles with 2 px radius and 1 px coloured outline. Reference: filled pill, no outline, 10 px type.
- The scroll area and dividers: Bourbon divider between sidebar and list is 1 px `#3a2d25`, fine; list-to-editor divider has a drag handle that is invisible, no hover feedback.

### 6. Iconography

- Reference: consistent 16 px line icons (Lucide-style, 1.5 px stroke) on every sidebar row (notes, templates, inbox, folder-open, status circles coloured per state, tag dots) and in the toolbar. Status entries have coloured glyphs (blue active, amber on hold, green completed, red dropped).
- Bourbon: almost no icons. Only a pin glyph and a chevron on "Work". Status rows are plain text, tags have a 6 px dot. Add 16 px icons (gpui-kit `IconName`) to All Notes, Notebooks, status items, Templates, Trash. This is the cheapest way to look closer to the reference.

### 7. List row design

Reference row anatomy (3 lines): title 600 weight, up to two lines; meta line = relative time (accent), task progress mini-bar "2 of 4", tag pills; excerpt 1-2 lines muted. Selected row: tinted fill, no border, full-bleed edge to edge with the hairline separators. Hover: lighter tint.

Bourbon anatomy: title + time on one line (time is `text_xs` muted, good alignment), excerpt, tags on a third line. Issues: time competes with title on the first line and truncates it at narrow widths; no hover state visible in the screenshot; pinned rows show a 12 px pin but unpinned have no indent so titles misalign between pinned and unpinned rows (Welcome vs Bourbon UI ideas start at x 276 vs 260). Reserve the 16 px slot for the pin on all rows or put the pin on the right.

### 8. Selection and hover

- Reference has three clear states: hover (`~6%` tint), selected (`~12%` tint of accent), active/focused pane (stronger saturation of selected). Selected rows in an unfocused pane desaturate.
- Bourbon has a single selected state (`accent` = selection brown) and the sidebar "All Notes" and list row use different tints (`#3d2c20` vs a more orange fill). Unify with `list_active`, `list_hover` and add an unfocused-pane variant.
- Focus ring is missing for keyboard navigation (the app is keyboard first, Omarchy users).

### 9. Header and toolbar density

- Reference: one 36 px bar over editor with 5 controls, one 40 px header over list with sidebar-toggle, title "All Notes" centred and compose icon, then a filter input. Sidebar header with settings gear.
- Bourbon: list header has `Updated v`, `New from template v`, `New note` as text buttons (~28 px each, 3 words wide) and the sidebar header has a lone `...` at the top right. Replace with: sort icon-button, a single `+` compose button with the template chooser in its menu, and a filter field. Saves ~200 px of header width and fixes the overflow.
- No search or filter field is visible in Bourbon; reference has "Filter" input at the top of the list. Check whether search exists (not in screenshot): if yes, surface it.

### 10. Editor area

- Reference: gutter with line numbers in 10 px muted, fold arrows, active-line highlight, `##` marks visible but headings enlarged/coloured, inline code and fenced code get a `#ebebef` background; preview pane has heading rules and a task-style list. Content measure ~560 px.
- Bourbon: plain textarea look, 12-word/60-char footer at the bottom in muted 14 px (fine, but belongs inside the editor card, not on the window background), empty editor area below text is a huge black void (no empty state, no click-to-focus affordance such as cursor on the whole area). Mode switch (`Edit | Side-by-side | Preview`) is text only, selected state is a faint grey; no icons.
- Title field uses the same outlined input as tags: it should be borderless 20-22 px/700 text, i.e. a heading, not a form field.

### 11. Empty states

Not visible in the screenshots. Needed: empty list ("No notes here" + `New note` button), empty trash, no search results, editor with no note selected (currently?). Use a muted 40 px icon, one line of text and one primary action.

### 12. Inconsistencies

1. Three different "button" styles: outlined text buttons (list header), plain text tabs (mode switch), `+` icon (Notebooks).
2. Section label "Notebooks" with `+` but "Tags" and "Templates" use different affordances (`+ New template` row vs nothing).
3. Counts: shown for notebooks and tags, absent for status and All Notes (reference shows total on All Notes, per-status counts).
4. Left edges: sidebar items at x=8/16/36/16 with different indents for "All Notes" (8) vs sections (16).
5. Date format fixed `18:50` vs expected relative date.
6. Editor box darker than neighbours (see colours).

## Prioritized backlog

Files refer to `crates/bourbon-app/src/`.

### P0 (broken or structurally wrong)

1. **Fix list header overflow.** Replace the three text buttons by sort icon, `+` button with dropdown (New note / New from template), and keep the row `h(px(40))`, `px(12)`, `flex-none` buttons, `min_w_0` title. File `note_list/header.rs`. Target: nothing clipped at list width 220.
2. **Fix elevation inversion.** Editor surface = `background` (`#1c1613` dark), list = `#1a1411`, sidebar = `#17110e`. Remove the editor's default input background and the vertical gradient. Files `editor/mod.rs`, `theme/palette.rs` (paint `input` so a borderless input is transparent).
3. **Rebalance column widths.** Defaults sidebar 220, list 300 → target 200/280; editor min width 420. `bourbon-core/src/settings.rs`.
4. **Align editor header.** One left edge (24 px) for title, meta row, tags, editor text; borderless title at 22 px/700; tag row inline with `+` chip instead of a full-width input; single 36 px top row for mode switch and word count. `editor/header.rs`.
5. **Strip Markdown from list excerpts.** Remove `#`, `-`, `**`, numbering and links; collapse whitespace; 120 chars, two lines clamp. `bourbon-core` (excerpt builder) or `presentation.rs`.

### P1 (large visual gain)

6. **Type scale and weights.** Sidebar item 13 px/400, section labels 11 px/600 muted with 0.04em tracking, list title 14 px/600, excerpt 12 px/400 line-height 1.4, meta 11 px, counts 11 px. Currently everything is ~16 px/400.
7. **Row padding.** List rows `px(14) py(10)`, gap 4, 1 px divider at 6% alpha, full-bleed selected fill, radius 0 (or 6 with 8 px inset, pick one and use it in sidebar too). `note_list/row.rs`.
8. **Reserve pin slot** (16 px) on every row, or show the pin at the right of the meta line. `row.rs`.
9. **Icons in sidebar.** 16 px icons for All Notes, Notebooks, Status (coloured per status), Templates, Trash; section headers with trailing icon actions; 30 px row height. `sidebar/mod.rs`, `sidebar/tree.rs`.
10. **Tag chips as pills.** Filled 14% alpha of tag colour, text at 100%, 10 px/500, radius 999, `px(6) py(1)`, no outline. `presentation.rs::chip_color` and `row.rs::tag_chip`.
11. **State system.** Hover `muted` @ 60%, selected `accent` @ 16% with 2 px left bar, selected-in-unfocused-pane `muted`; add focus ring on keyboard focus. Palette gets `list_hover`/`list_active` distinct from `selection` (already painted equal now).
12. **Relative time + accent colour** for date ("2h", "Yesterday", then `5 Nov`). `note_list/date.rs`.
13. **Editor typography.** Monospace 14 px/1.6, max measure 72 ch, 24 px side padding, caret in accent, selection `selection`. Syntax tinting for headings, list markers, code (see theming doc for the tokens).
14. **Top bar over list+editor** with back/forward, mode icons (pen, columns, eye) as a segmented control, kebab. Replaces text mode switch. `editor/header.rs`.

### P2 (polish)

15. Filter field at the top of the list.
16. Line-number gutter and active-line highlight in the editor.
17. Preview typography: h1/h2 rule, 15 px body, code block radius 6 and `muted` background.
18. Empty states (list, trash, search, no selection) with icon + one action.
19. Task progress bar ("2 of 4") in rows when a note has checkboxes.
20. Window chrome: 12 px radius, soft shadow (only if the compositor shows client decorations; on Omarchy tiling this is moot, skip).
21. Resize handle: 4 px hit area, 1 px visible line that becomes accent on hover, `cursor_col_resize`.
22. Sidebar translucency/blur: not feasible reliably on Wayland through GPUI, skip.

### Quick wins (under an hour each)

- Remove the editor box border and gradient.
- Hide the `...` in the sidebar header or turn it into a gear.
- Set row height 30 px in the sidebar and font 13 px.
- Add `font_semibold` to list titles.
- Pin slot reserved.
- Bump chip text lightness.
- Status line (word count) moves into the editor header or a 24 px footer inside the pane.

## Motion suggestions (GPUI `with_animation`, spring-like easing)

Keep durations short (100-200 ms); respect a `reduced_motion` setting; nothing may delay input.

| Where | Effect | Duration/easing |
|---|---|---|
| Selection change in list/sidebar | Background fade in (opacity 0 to 1) instead of a hard swap | 120 ms ease_out_quint |
| Hover | tint fade | 80 ms linear |
| Sidebar collapse (Ctrl+/) | width tween from current to 0 plus content opacity | 180 ms ease_in_out, spring-like overshoot of 2% optional |
| Notebook expand/collapse | chevron rotate 0 to 90 deg, children height reveal | 140 ms ease_out |
| Settings screen enter | 8 px slide up + fade | 160 ms ease_out |
| New note | new row slides in from top (height 0 to full, opacity) | 180 ms spring |
| Drag a note onto a notebook | target row pulses accent @ 20% | 120 ms |
| Mode switch (Edit/Split/Preview) | segmented indicator slides under active label | 140 ms ease_out |
| Theme change / Omarchy follow | cross-fade colours (interpolate Hsla over 200 ms); do not animate on startup | 200 ms |
| Toast/save indicator | "Saved" fades in, holds 1 s, fades out | 150/1000/300 ms |

Implementation note: gpui's `Animation::new(Duration).with_easing(...)` runs per element id; colour interpolation needs the two Hsla values in the closure, so keep prior and next palette in workspace state during a theme transition.

## Open questions

1. Is the target the Inkdrop **light** look, or should the dark variant follow Inkdrop's dark theme (cool slate) instead of Bourbon's warm brown?
2. Square full-bleed list rows (Inkdrop) or inset rounded rows (current)?
3. Is a top bar acceptable (Inkdrop style) or does the user want to keep per-column headers?
