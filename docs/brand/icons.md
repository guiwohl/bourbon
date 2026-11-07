# Icons

Free line icons from Lucide (https://lucide.dev), ISC licence (some derived from Feather, MIT). The licence text is vendored next to the SVGs in `crates/bourbon-ui/assets/icons/LICENSE`. Bourbon ships these instead of Inkdrop's proprietary Streamline set (clean room, see `docs/decisions/2026-11-07-ui-rebuild.md`).

- Only the SVGs in use are vendored, unmodified (24 grid, stroke 2). To add one: copy `<name>.svg` from Lucide into `crates/bourbon-ui/assets/icons/`, add a row to the `icons!` list in `crates/bourbon-ui/src/icon.rs`; the `every_icon_name_resolves_to_a_loadable_asset` and listing-count tests guard it.
- Inkdrop draws at stroke 1.5. `IconAssets::load` rewrites `stroke-width="2"` on the fly: `icons/<name>.svg` is 1.5, `icons/<name>@3.svg` and `@2.5.svg` are the heavy and medium strokes (disclosure chevrons use 3, section collapse arrows 2.5).
- Inkdrop swaps selected icons for a `-bold` variant. Bourbon has no second set: `Icon::bold(accent)` draws the same icon in the accent colour.
- The app installs the assets with `Application::with_assets(bourbon_ui::IconAssets)`.
